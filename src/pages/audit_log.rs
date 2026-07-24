use leptos::prelude::*;
use lucide_leptos::{RefreshCw, ScrollText};

use crate::components::audit::log_table::LogTable;
use crate::components::form::search_form::{FieldKind, SearchField, SearchForm, SelectOption};
use crate::components::table::pagination::Pagination;
use crate::ipc::{self};

/// 审计日志页。
///
/// 全局审计日志查询,支持按实体类型过滤与分页。
#[component]
pub fn AuditLog() -> impl IntoView {
    let (page, set_page) = signal(1i32);
    let (entity_type, set_entity_type) = signal(String::new());
    let (entity_id, set_entity_id) = signal(String::new());

    let logs = LocalResource::new(move || {
        let t = entity_type.get();
        let id = entity_id.get();
        let p = page.get();
        async move {
            ipc::list_audit_logs(
                if t.is_empty() { None } else { Some(t) },
                if id.is_empty() { None } else { Some(id) },
                p,
                20,
            )
            .await
        }
    });

    let on_search = move |_| {
        set_page.set(1);
        logs.refetch();
    };
    let on_reset = move |_| {
        set_entity_type.set(String::new());
        set_entity_id.set(String::new());
        set_page.set(1);
        logs.refetch();
    };

    view! {
        <div class="page-content">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"审计日志"</h1>
                <button
                    class="btn btn-outline btn-sm"
                    on:click=move |_| logs.refetch()
                >
                    <RefreshCw size=14 />
                    "刷新"
                </button>
            </div>

            <SearchForm
                fields=audit_search_fields()
                on_search=Callback::new(on_search)
                on_reset=Callback::new(on_reset)
            />

            <div class="data-table flex flex-col min-h-0 card mt-4">
                <div class="card-header">
                    <span class="card-title flex items-center gap-2">
                        <ScrollText size=14 />
                        "操作日志"
                    </span>
                </div>
                <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                    {move || Suspend::new(async move {
                        match logs.await {
                            Ok((items, total)) => {
                                let page_size = 20;
                                let current = page.get();
                                view! {
                                    <>
                                    <LogTable rows=items />
                                    <div class="border-t border-border-light">
                                        <Pagination
                                            total=total
                                            current=current
                                            page_size=page_size
                                        />
                                    </div>
                                    </>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="login-error">{format!("加载日志失败: {e}")}</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

fn audit_search_fields() -> Vec<SearchField> {
    vec![
        SearchField {
            key: "entity_type",
            label: "实体类型",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "source_record",
                        label: "原始记录",
                    },
                    SelectOption {
                        value: "import_batch",
                        label: "导入批次",
                    },
                    SelectOption {
                        value: "transaction_summary",
                        label: "对账汇总",
                    },
                    SelectOption {
                        value: "voucher",
                        label: "凭证",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
        SearchField {
            key: "entity_id",
            label: "实体 ID",
            kind: FieldKind::Text {
                placeholder: Some("记录 ID"),
            },
            width: None,
        },
    ]
}
