use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::components::source::raw_record_table::{
    default_columns, RawRecordFilterState, RawRecordTableBody, RawRecordToolbar,
};
use crate::components::source::record_detail::RecordDetail;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, ColumnPrefs, GenerateSummaryResult, RawRecordFilter, RawRecordPage};

const SOURCE_TYPE: &str = "summary_flow";

#[component]
pub fn DataSummary() -> impl IntoView {
    let (selected_id, set_selected_id) = signal(Option::<i64>::None);

    let (date_from, set_date_from) = signal(default_date_from());
    let (date_to, set_date_to) = signal(default_date_to());
    let (generating, set_generating) = signal(false);
    let (generate_error, set_generate_error) = signal(Option::<String>::None);
    let (generate_result, set_generate_result) = signal(Option::<GenerateSummaryResult>::None);

    let initial = LocalResource::new(|| async move {
        let records = ipc::list_raw_records(&RawRecordFilter {
            source_type: Some(SOURCE_TYPE.to_string()),
            batch_id: None,
            page: 1,
            page_size: 50,
        })
        .await;
        let prefs = ipc::get_column_prefs(SOURCE_TYPE.to_string())
            .await
            .unwrap_or_else(|_| ColumnPrefs {
                source_type: SOURCE_TYPE.to_string(),
                columns: default_columns(),
            });
        (records, prefs)
    });

    let refresh_records = move || {
        initial.refetch();
    };

    let detail = LocalResource::new(move || {
        let id = selected_id.get();
        async move {
            match id {
                Some(id) => ipc::get_raw_record(id).await.unwrap_or(None),
                None => None,
            }
        }
    });

    let generate = move |_| {
        let from = date_from.get();
        let to = date_to.get();
        if from.trim().is_empty() || to.trim().is_empty() {
            set_generate_error.set(Some("请填写起始与结束日期".to_string()));
            return;
        }
        if from > to {
            set_generate_error.set(Some(format!("起始日期 {from} 晚于结束日期 {to}")));
            return;
        }
        set_generating.set(true);
        set_generate_error.set(None);
        set_generate_result.set(None);
        leptos::task::spawn_local(async move {
            match ipc::generate_summary(from, to).await {
                Ok(r) => {
                    set_generate_result.set(Some(r));
                    refresh_records();
                }
                Err(e) => set_generate_error.set(Some(format!("生成失败: {e}"))),
            }
            set_generating.set(false);
        });
    };

    view! {
        <div class="page-content">
            <div class="card p-4 mb-4">
                <div class="text-13 text-secondary mb-2">"自动生成数据汇总"</div>
                <div class="text-13 text-tertiary mb-3">
                    "数据汇总是由系统按以下三类源数据自动派生,无需手工导入:"
                    <ul class="list-disc list-inside mt-1 space-y-1">
                        <li>"银行流水 (bank_flow)"</li>
                        <li>"工商商户 POS 流水 (pos_flow)"</li>
                        <li>"微信聊天记录的说明信息(待后续落地)"</li>
                    </ul>
                </div>
                <div class="flex items-end gap-3">
                    <div class="form-field">
                        <label class="form-label">"起始日期"</label>
                        <input
                            type="date"
                            class="form-input"
                            prop:value=date_from
                            on:input=move |ev| set_date_from.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"结束日期"</label>
                        <input
                            type="date"
                            class="form-input"
                            prop:value=date_to
                            on:input=move |ev| set_date_to.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        class="btn btn-primary"
                        disabled=generating
                        on:click=move |_| generate(())
                    >
                        {move || if generating.get() { "生成中…" } else { "生成汇总" }}
                    </button>
                </div>
                <Show when=move || generate_error.get().is_some()>
                    <div class="login-error mt-2">{move || generate_error.get().unwrap_or_default()}</div>
                </Show>
                <Show when=move || generate_result.get().is_some()>
                    {move || {
                        generate_result.get().map(|r| view! {
                            <div class="mt-2 text-13 text-success">
                                {format!(
                                    "已生成:{} (区间 {} → {})",
                                    r.date_from, r.date_from, r.date_to
                                )}
                            </div>
                        })
                    }}
                </Show>
            </div>

            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                {move || Suspend::new(async move {
                    let (records_res, prefs) = initial.await;
                    match records_res {
                        Ok(p) => {
                            let state = build_state(&p, prefs);
                            view! {
                                <div class="page-toolbar">
                                    <RawRecordToolbar state=state.clone() />
                                </div>
                                <div class="page-grid-detail">
                                    <RawRecordTableBody
                                        state=state
                                        selected_id=selected_id
                                        set_selected_id=set_selected_id
                                    />
                                    <div class="detail-panel-responsive">
                                        <RecordDetail detail=detail />
                                    </div>
                                </div>
                                <div class="page-status-bar">
                                    <Pagination total=p.total current=p.page page_size=p.page_size />
                                </div>
                            }
                                .into_any()
                        }
                        Err(e) => view! {
                            <div class="login-error">{format!("加载失败: {e}")}</div>
                        }
                            .into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

fn default_date_from() -> String {
    let js_date = js_sys::Date::new_0();
    let year = js_date.get_full_year();
    let month = js_date.get_month() + 1; // JS: 0-11 -> 1-12
    format!("{year:04}-{month:02}-01")
}

fn default_date_to() -> String {
    let js_date = js_sys::Date::new_0();
    let year = js_date.get_full_year();
    let month = js_date.get_month() + 1;
    let day = js_date.get_date();
    format!("{year:04}-{month:02}-{day:02}")
}

fn build_state(p: &RawRecordPage, prefs: ColumnPrefs) -> RawRecordFilterState {
    let source_type = SOURCE_TYPE.to_string();
    let on_change = Callback::new(move |cols: BTreeMap<String, bool>| {
        let st = source_type.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = ipc::save_column_prefs(st, cols).await {
                web_sys::console::warn_1(&format!("保存列显示偏好失败: {e}").into());
            }
        });
    });
    RawRecordFilterState::with_columns(&p.items, SOURCE_TYPE, prefs.columns, Some(on_change))
}
