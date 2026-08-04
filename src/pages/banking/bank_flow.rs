use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::components::source::bank_flow_balance_bar::BankFlowBalanceBar;
use crate::components::source::raw_record_table::{
    default_columns, RawRecordFilterState, RawRecordTableBody, RawRecordToolbar,
};
use crate::components::source::record_detail::RecordDetail;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, BankFlowFilter, ColumnPrefs, RawRecord};

const SOURCE_TYPE: &str = "bank_flow";

#[component]
pub fn BankFlow() -> impl IntoView {
    let (selected_id, set_selected_id) = signal(Option::<i64>::None);

    let initial = LocalResource::new(|| async move {
        let page = ipc::list_bank_flows(&BankFlowFilter {
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
        (page, prefs)
    });

    let detail = LocalResource::new(move || {
        let id = selected_id.get();
        async move {
            match id {
                Some(id) => ipc::get_bank_flow(id)
                    .await
                    .unwrap_or(None)
                    .map(|d| d.into()),
                None => None,
            }
        }
    });

    let refresh_records = move || {
        initial.refetch();
    };

    view! {
        <div class="page-content">
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                {move || Suspend::new(async move {
                    let (page_res, prefs) = initial.await;
                    match page_res {
                        Ok(p) => {
                            let raw_items: Vec<RawRecord> = p.items.iter().map(|b| b.clone().into()).collect();
                            let raw_page = crate::ipc::RawRecordPage {
                                items: raw_items.clone(),
                                total: p.total,
                                page: p.page,
                                page_size: p.page_size,
                            };
                            let state = build_state(&raw_page, prefs);
                            view! {
                                <div class="page-toolbar">
                                    <RawRecordToolbar state=state.clone() />
                                    <BankFlowBalanceBar
                                        items=raw_items
                                        on_changed=Callback::new(move |_| refresh_records())
                                    />
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

fn build_state(p: &crate::ipc::RawRecordPage, prefs: ColumnPrefs) -> RawRecordFilterState {
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
