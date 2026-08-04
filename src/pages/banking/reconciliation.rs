use leptos::prelude::*;
use lucide_leptos::{RefreshCw, Scale};

use crate::components::charts::kpi_card::{KpiAccent, KpiCard};
use crate::components::form::search_form::{FieldKind, SearchField, SearchForm, SelectOption};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::reconciliation::diff_list::DiffList;
use crate::components::reconciliation::diff_review_modal::DiffReviewModal;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, ReconciliationPage, VoucherSummary};

/// 对账中心页。
///
/// 支持选择日期自动对账,展示差异列表并打开审核弹框。
#[component]
pub fn Reconciliation() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("reconcile");

    let tabs = vec![
        TabItem {
            key: "reconcile",
            label: "对账",
            closable: false,
        },
        TabItem {
            key: "diffs",
            label: "差异列表",
            closable: false,
        },
    ];

    view! {
        <Tabs
            items=tabs
            active_key={move || active_tab.get()}
            on_change=Callback::new(move |key| set_active_tab.set(key))
        />

        <Show when=move || active_tab.get() == "reconcile">
            <ReconcilePanel on_finished=Callback::new(move |_| set_active_tab.set("diffs")) />
        </Show>

        <Show when=move || active_tab.get() == "diffs">
            <DiffListPanel />
        </Show>
    }
}

#[component]
fn ReconcilePanel(
    #[prop(default = Callback::new(|_| {}))] on_finished: Callback<()>,
) -> impl IntoView {
    let (date, set_date) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (result, set_result) = signal(Option::<ipc::ReconcileResult>::None);

    let reconcile = move |_| {
        let d = date.get();
        if d.trim().is_empty() {
            set_error.set(Some("请选择对账日期".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match ipc::reconcile(d).await {
                Ok(r) => {
                    set_result.set(Some(r));
                    on_finished.run(());
                }
                Err(e) => set_error.set(Some(format!("对账失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="page-content">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"原始凭证对账"</h1>
            </div>

            <div class="card p-4 mb-4">
                <div class="flex items-end gap-3">
                    <div class="form-field">
                        <label class="form-label">"对账日期"</label>
                        <input
                            type="text"
                            class="form-input"
                            placeholder="YYYY-MM-DD"
                            prop:value=date
                            on:input=move |ev| set_date.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        class="btn btn-primary"
                        disabled=loading
                        on:click=move |_| reconcile(())
                    >
                        <Scale size=14 />
                        {move || if loading.get() { "对账中…" } else { "开始对账" }}
                    </button>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="login-error mt-3">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <Show when=move || result.get().is_some()>
                    {move || {
                        result.get().map(|r| {
                            let matched = r.matched_dates.len();
                            let diff = r.diff_dates.len();
                            view! {
                                <div class="mt-3 p-3 bg-surface rounded border border-border text-13">
                                    <div class="flex gap-4">
                                        <span class="text-success">{format!("自动匹配 {} 天", matched)}</span>
                                        <span class={if diff > 0 { "text-warning" } else { "text-tertiary" }}>
                                            {format!("差异 {} 天", diff)}
                                        </span>
                                    </div>
                                </div>
                            }
                        })
                    }}
                </Show>
            </div>
        </div>
    }
}

#[component]
fn DiffListPanel() -> impl IntoView {
    let (page, set_page) = signal(1i32);
    let (date_filter, set_date_filter) = signal(String::new());
    let (status_filter, set_status_filter) = signal(String::new());

    let items = LocalResource::new(move || {
        let date = date_filter.get();
        let status = status_filter.get();
        let p = page.get();
        async move {
            ipc::list_reconciliation_items(
                if date.is_empty() { None } else { Some(date) },
                if status.is_empty() {
                    None
                } else {
                    Some(status)
                },
                p,
                20,
            )
            .await
        }
    });

    let (selected_id, set_selected_id) = signal(Option::<i64>::None);
    let (modal_open, set_modal_open) = signal(false);

    let selected_item = Signal::derive(move || {
        let id = selected_id.get();
        if let Some(id) = id {
            let resolved: Option<ReconciliationPage> = items.get().and_then(|r| r.ok());
            if let Some(page) = resolved {
                return page.items.into_iter().find(|it| it.id == id);
            }
        }
        None
    });

    let open_review = move |id: i64| {
        set_selected_id.set(Some(id));
        set_modal_open.set(true);
    };

    let close_modal = move |_| {
        set_modal_open.set(false);
    };

    let on_reviewed = move |_: Option<VoucherSummary>| {
        items.refetch();
    };

    let on_search = move |_| {
        set_page.set(1);
        items.refetch();
    };
    let on_reset = move |_| {
        set_date_filter.set(String::new());
        set_status_filter.set(String::new());
        set_page.set(1);
        items.refetch();
    };

    view! {
        <div class="page-content">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"差异审核"</h1>
                <button
                    class="btn btn-outline btn-sm"
                    on:click=move |_| items.refetch()
                >
                    <RefreshCw size=14 />
                    "刷新"
                </button>
            </div>

            <SearchForm
                fields=diff_search_fields()
                on_search=Callback::new(on_search)
                on_reset=Callback::new(on_reset)
            />

            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
                <KpiCard label="差异总数" value="—".to_string() unit=None accent=KpiAccent::Brand />
                <KpiCard label="待审核" value="—".to_string() unit=None accent=KpiAccent::Warning />
                <KpiCard label="已通过" value="—".to_string() unit=None accent=KpiAccent::Success />
                <KpiCard label="已驳回" value="—".to_string() unit=None accent=KpiAccent::Danger />
            </div>

            <div class="data-table flex flex-col min-h-0 card">
                <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                    {move || Suspend::new(async move {
                        match items.await {
                            Ok(p) => view! {
                                <>
                                <DiffList
                                    rows=p.items.clone()
                                    selected_id=selected_id
                                    set_selected_id=set_selected_id
                                    on_review=Callback::new(open_review)
                                />
                                <div class="border-t border-border-light">
                                    <Pagination
                                        total=p.total
                                        current=p.page
                                        page_size=p.page_size
                                    />
                                </div>
                                </>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="login-error">{format!("加载差异失败: {e}")}</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            <DiffReviewModal
                open=modal_open
                on_close=Callback::new(close_modal)
                item=selected_item
                on_reviewed=Callback::new(on_reviewed)
            />
        </div>
    }
}

fn diff_search_fields() -> Vec<SearchField> {
    vec![
        SearchField {
            key: "date",
            label: "对账日期",
            kind: FieldKind::Text {
                placeholder: Some("YYYY-MM-DD"),
            },
            width: None,
        },
        SearchField {
            key: "status",
            label: "状态",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "pending",
                        label: "待审核",
                    },
                    SelectOption {
                        value: "auto_matched",
                        label: "自动匹配",
                    },
                    SelectOption {
                        value: "approved",
                        label: "已通过",
                    },
                    SelectOption {
                        value: "rejected",
                        label: "已驳回",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
    ]
}
