use leptos::prelude::*;
use lucide_leptos::{
    ArrowLeftRight, Check, ChevronDown, Pencil, Plus, Printer, SlidersHorizontal, Trash2, Upload,
};

use crate::components::charts::badge::{status_variant, Badge};
use crate::components::charts::kpi_card::{KpiAccent, KpiCard};
use crate::components::form::search_form::{FieldKind, SearchField, SearchForm, SelectOption};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, Voucher, VoucherFilter, VoucherPage};

/// 凭证管理页(查询 + 表格 + 详情 + 审计)。
///
/// `audit_mode`: 来自「凭证审核」导航时为 true,默认筛选未审核。
#[component]
pub fn VoucherManagement(#[prop(default = false)] audit_mode: bool) -> impl IntoView {
    let (page, set_page) = signal(1i32);
    let (filter, set_filter) = signal(VoucherFilter {
        status: if audit_mode {
            Some("unaudited".to_string())
        } else {
            None
        },
        page_size: Some(20),
        ..Default::default()
    });
    let vouchers = Resource::new(
        move || (filter.get(), page.get()),
        move |(filter, page)| async move {
            let mut f = filter;
            f.page = Some(page);
            ipc::list_vouchers(&f).await
        },
    );

    let (selected_id, set_selected_id) = signal(Option::<String>::None);
    let detail = Resource::new(
        move || selected_id.get(),
        move |id| async move {
            if let Some(id) = id {
                ipc::get_voucher(id).await.ok()
            } else {
                None
            }
        },
    );

    let (error, set_error) = signal(Option::<String>::None);

    let do_audit = move |id: String, comment: Option<String>| {
        leptos::task::spawn_local(async move {
            match ipc::audit_voucher(id, comment).await {
                Ok(_) => {
                    vouchers.refetch();
                    detail.refetch();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let do_delete = move |id: String| {
        leptos::task::spawn_local(async move {
            match ipc::delete_voucher(id).await {
                Ok(_) => {
                    vouchers.refetch();
                    detail.refetch();
                    set_selected_id.set(None);
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let tabs = vec![
        TabItem {
            key: "voucher_overview",
            label: "凭证概览",
            closable: true,
        },
        TabItem {
            key: "voucher_management",
            label: if audit_mode {
                "凭证审核"
            } else {
                "凭证管理"
            },
            closable: false,
        },
    ];

    let on_search = move || {
        set_page.set(1);
        vouchers.refetch();
    };
    let on_reset = move || {
        set_filter.set(VoucherFilter {
            status: if audit_mode {
                Some("unaudited".to_string())
            } else {
                None
            },
            page_size: Some(20),
            ..Default::default()
        });
        set_page.set(1);
        vouchers.refetch();
    };

    view! {
        <Tabs items=tabs active_key="voucher_management" />

        <div class="page-content">
            <SearchForm
                fields=search_fields()
                on_search=std::rc::Rc::new(on_search)
                on_reset=std::rc::Rc::new(on_reset)
                expandable=true
            />

            <SummaryStats vouchers=vouchers />

            <ActionBar />

            <Show when=move || error.get().is_some()>
                <div class="login-error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="page-grid">
                <div class="data-table flex flex-col min-h-0">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || Suspend::new(async move {
                            match vouchers.await {
                                Ok(p) => view! {
                                    <DataTable
                                        rows=p.items.clone()
                                        selected_id=selected_id
                                        set_selected_id=set_selected_id
                                    />
                                    <div class="border-t border-border-light">
                                        <Pagination
                                            total=p.total
                                            current=p.page
                                            page_size=p.page_size
                                        />
                                    </div>
                                },
                                Err(e) => view! {
                                    <div class="login-error">{format!("加载凭证失败: {e}")}</div>
                                },
                            }
                        })}
                    </Suspense>
                </div>
                <VoucherDetail
                    detail=detail
                    on_audit=do_audit
                    on_delete=do_delete
                />
            </div>
        </div>
    }
}

fn search_fields() -> Vec<SearchField> {
    vec![
        SearchField {
            key: "date_from",
            label: "开始日期",
            kind: FieldKind::Text {
                placeholder: Some("2024-06-01"),
            },
            width: None,
        },
        SearchField {
            key: "date_to",
            label: "结束日期",
            kind: FieldKind::Text {
                placeholder: Some("2024-06-30"),
            },
            width: None,
        },
        SearchField {
            key: "voucher_no",
            label: "凭证字号",
            kind: FieldKind::Text {
                placeholder: Some("凭证字号"),
            },
            width: None,
        },
        SearchField {
            key: "voucher_type",
            label: "凭证类型",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "记账",
                        label: "记账凭证",
                    },
                    SelectOption {
                        value: "付款",
                        label: "付款凭证",
                    },
                    SelectOption {
                        value: "收款",
                        label: "收款凭证",
                    },
                    SelectOption {
                        value: "转账",
                        label: "转账凭证",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
        SearchField {
            key: "status",
            label: "审核状态",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "draft",
                        label: "草稿",
                    },
                    SelectOption {
                        value: "unaudited",
                        label: "未审核",
                    },
                    SelectOption {
                        value: "audited",
                        label: "已审核",
                    },
                    SelectOption {
                        value: "posted",
                        label: "已过账",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
    ]
}

#[component]
fn SummaryStats(
    vouchers: Resource<(VoucherFilter, i32), Result<VoucherPage, String>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
            <KpiCard label="凭证总数" value="—".to_string() unit=None accent=KpiAccent::Brand />
            <KpiCard label="已审核" value="—".to_string() unit=None accent=KpiAccent::Success />
            <KpiCard label="未审核" value="—".to_string() unit=None accent=KpiAccent::Warning />
            <KpiCard label="借方合计" value="—".to_string() unit=None accent=KpiAccent::Primary />
            <KpiCard label="贷方合计" value="—".to_string() unit=None accent=KpiAccent::Primary />
            <KpiCard label="借贷差额" value="—".to_string() unit=None accent=KpiAccent::Info />
        </div>
        {move || Suspend::new(async move {
            if let Ok(p) = vouchers.await {
                let audited = p.items.iter().filter(|v| v.status == "audited").count();
                let unaudited = p.items.iter().filter(|v| v.status != "audited").count();
                let debit: i64 = p.items.iter().map(|v| v.debit_total.parse::<i64>().unwrap_or(0)).sum();
                let credit: i64 = p.items.iter().map(|v| v.credit_total.parse::<i64>().unwrap_or(0)).sum();
                view! {
                    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3 mt-3 text-xs text-tertiary">
                        <span>{format!("本页 {} 条", p.items.len())}</span>
                        <span>{format!("已审核 {}", audited)}</span>
                        <span>{format!("未审核 {}", unaudited)}</span>
                        <span>{format!("借方 {} 分", debit)}</span>
                        <span>{format!("贷方 {} 分", credit)}</span>
                        <span>{format!("差额 {} 分", debit - credit)}</span>
                    </div>
                }
            } else {
                view! { <span></span> }
            }
        })}
    }
}

#[component]
fn ActionBar() -> impl IntoView {
    view! {
        <div class="action-bar">
            <div class="action-bar-group">
                <button class="btn btn-primary">
                    <Plus size=14 />
                    "新增凭证"
                </button>
                <ActionButton label="审核"><Check size=14 /></ActionButton>
                <ActionButton label="反审核"><ArrowLeftRight size=14 /></ActionButton>
                <ActionButton label="打印"><Printer size=14 /></ActionButton>
                <ActionButton label="导出"><Upload size=14 /></ActionButton>
                <ActionButton label="设置"><SlidersHorizontal size=14 /></ActionButton>
            </div>
        </div>
    }
}

#[component]
fn ActionButton(label: &'static str, children: ChildrenFn) -> impl IntoView {
    view! {
        <button class="btn btn-outline gap-6">
            {children()}
            {label}
        </button>
    }
}

#[component]
fn DataTable(
    rows: Vec<Voucher>,
    selected_id: ReadSignal<Option<String>>,
    set_selected_id: WriteSignal<Option<String>>,
) -> impl IntoView {
    let total_rows = rows.len();
    view! {
        <div class="flex-1 overflow-auto">
            <table>
                <thead>
                    <tr>
                        <th class="w-40 text-center">
                            <input type="checkbox" style="width:14px;height:14px" />
                        </th>
                        <th class="w-48 text-center">"序号"</th>
                        <th>"凭证字号"</th>
                        <th>"凭证日期"</th>
                        <th>"摘要"</th>
                        <th>"凭证类型"</th>
                        <th class="data-table-num">"借方(分)"</th>
                        <th class="data-table-num">"贷方(分)"</th>
                        <th>"制单人"</th>
                        <th>"审核人"</th>
                        <th class="text-center">"审核状态"</th>
                        <th class="text-center border-l border-border">"操作"</th>
                    </tr>
                </thead>
                <tbody>
                    <For each=move || rows.clone() key=|r| r.id.clone() let:(idx, row)>
                        <RowItem
                            row=row
                            idx=idx + 1
                            selected=selected_id
                            set_selected=set_selected_id
                        />
                    </For>
                </tbody>
            </table>
            {move || {
                if total_rows == 0 {
                    view! {
                        <div class="text-center py-40 text-tertiary">"暂无数据"</div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn RowItem(
    row: Voucher,
    idx: usize,
    selected: ReadSignal<Option<String>>,
    set_selected: WriteSignal<Option<String>>,
) -> impl IntoView {
    let status_label = match row.status.as_str() {
        "draft" => "草稿",
        "unaudited" => "未审核",
        "audited" => "已审核",
        "posted" => "已过账",
        _ => "—",
    }
    .to_string();
    let variant = status_variant(&status_label);
    let is_active = move || selected.get() == Some(row.id.clone());
    let id = row.id.clone();
    view! {
        <tr
            class=("selected", is_active)
            on:click=move |_| set_selected.set(Some(row.id.clone()))
        >
            <td class="text-center" on:click=move |ev| ev.stop_propagation()>
                <input type="checkbox" style="width:14px;height:14px" />
            </td>
            <td class="data-table-num">{idx}</td>
            <td>{row.voucher_no.clone()}</td>
            <td class="data-table-num">{row.voucher_date.clone()}</td>
            <td>{row.summary.clone()}</td>
            <td>{row.voucher_type.clone()}</td>
            <td class="data-table-num">{row.debit_total.clone()}</td>
            <td class="data-table-num">{row.credit_total.clone()}</td>
            <td>{row.operator_name.clone().unwrap_or("—".to_string())}</td>
            <td>{row.auditor_name.clone().unwrap_or("—".to_string())}</td>
            <td class="text-center">
                <Badge label=status_label.clone() variant=variant />
            </td>
            <td class="text-center border-l border-border" on:click=move |ev| ev.stop_propagation()>
                <div class="flex items-center justify-center gap-4">
                    <button class="text-xs text-brand" on:click=move |_| set_selected.set(Some(id.clone()))>"查看"</button>
                    <button class="text-xs inline-flex items-center gap-0.5 text-secondary">
                        "更多"
                        <ChevronDown size=10 />
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn VoucherDetail(
    detail: Resource<Option<String>, Option<crate::ipc::VoucherDetail>>,
    on_audit: impl Fn(String, Option<String>) + 'static,
    on_delete: impl Fn(String) + 'static,
) -> impl IntoView {
    let (audit_comment, set_audit_comment) = signal(String::new());
    view! {
        <div class="card flex flex-col min-h-0">
            <div class="card-header">
                <span class="card-title">"凭证详情"</span>
                <button class="btn btn-outline btn-sm">
                    <Pencil size=12 />
                    "编辑"
                </button>
            </div>
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"请选择一条凭证查看详情"</div> }>
                {move || {
                    detail.get()
                        .flatten()
                        .map(|d| {
                            let vid = d.voucher.id.clone();
                            let status = d.voucher.status.clone();
                            let logs = d.audit_logs.clone();
                            let entries = d.entries.clone();
                            view! {
                                <div class="detail-grid">
                                    <DetailField label="凭证字号" value=d.voucher.voucher_no.clone() />
                                    <DetailField label="凭证日期" value=d.voucher.voucher_date.clone() />
                                    <DetailField label="凭证类型" value=d.voucher.voucher_type.clone() />
                                    <DetailField label="附件" value=d.voucher.attachments.to_string() />
                                    <DetailField label="摘要" value=d.voucher.summary.clone() />
                                    <DetailField label="审核状态" value=status_cn(&d.voucher.status) highlight=true />
                                </div>
                                <div class="flex-1 overflow-auto p-3">
                                    <table class="data-table" style="border:none">
                                        <thead>
                                            <tr>
                                                <th class="w-40 text-center">"序号"</th>
                                                <th>"科目编码"</th>
                                                <th>"科目名称"</th>
                                                <th>"摘要"</th>
                                                <th class="data-table-num">"借方(分)"</th>
                                                <th class="data-table-num">"贷方(分)"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For each=move || entries.clone() key=|e| e.id.clone() let:entry>
                                                <tr>
                                                    <td class="data-table-num">{entry.line_no}</td>
                                                    <td class="data-table-num">{entry.account_code.clone()}</td>
                                                    <td>{entry.account_name.clone()}</td>
                                                    <td>{entry.summary.clone().unwrap_or("—".to_string())}</td>
                                                    <td class="data-table-num">{entry.debit.clone()}</td>
                                                    <td class="data-table-num">{entry.credit.clone()}</td>
                                                </tr>
                                            </For>
                                        </tbody>
                                    </table>
                                </div>
                                <div class="border-t border-border p-3">
                                    <div class="form-field">
                                        <label class="form-label">"审核意见"</label>
                                        <textarea
                                            class="form-textarea"
                                            rows="2"
                                            prop:value=audit_comment
                                            on:input=move |ev| set_audit_comment.set(event_target_value(&ev))
                                        ></textarea>
                                    </div>
                                    <div class="modal-form-actions mt-2">
                                        <button
                                            class="btn btn-outline"
                                            on:click=move |_| do_delete(vid.clone())
                                        >
                                            <Trash2 size=12 />
                                            "删除"
                                        </button>
                                        <button
                                            class="btn btn-primary"
                                            on:click=move |_| {
                                                let c = audit_comment.get();
                                                do_audit(vid.clone(), if c.is_empty() { None } else { Some(c) })
                                            }
                                        >
                                            <Check size=12 />
                                            {move || if status == "audited" { "反审核" } else { "审核" }}
                                        </button>
                                        <button class="btn btn-outline" on:click=move |_| {
                                            let _ = window().print();
                                        }>
                                            <Printer size=12 />
                                            "打印"
                                        </button>
                                    </div>
                                </div>
                                <Show when=move || !logs.is_empty()>
                                    <div class="border-t border-border p-3">
                                        <div class="text-13 text-secondary mb-2">"审核日志"</div>
                                        {logs.iter().map(|log| {
                                            let title = match log.action.as_str() {
                                                "audit" => "审核通过",
                                                "unaudit" => "反审核",
                                                _ => "操作",
                                            };
                                            view! {
                                                <div class="log-entry">
                                                    <span class="log-entry-dot" style="background: var(--color-brand)"></span>
                                                    <div class="log-entry-title">{title}</div>
                                                    <div class="log-entry-meta">
                                                        <span class="text-primary font-medium">
                                                            {log.operator_name.clone().unwrap_or("—".to_string())}
                                                        </span>
                                                        <span class="text-tertiary">{log.created_at.clone()}</span>
                                                    </div>
                                                    <Show when=move || log.comment.is_some()>
                                                        <div class="log-entry-comment">
                                                            {log.comment.clone().unwrap_or_default()}
                                                        </div>
                                                    </Show>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </Show>
                            }
                        })
                        .unwrap_or_else(|| {
                            view! {
                                <div class="empty-state">
                                    <p class="empty-state-desc">"请从左侧选择一条凭证查看详情。"</p>
                                </div>
                            }
                        })
                        }
                }
            </Suspense>
        </div>
    }
}

fn status_cn(s: &str) -> String {
    match s {
        "draft" => "草稿".to_string(),
        "unaudited" => "未审核".to_string(),
        "audited" => "已审核".to_string(),
        "posted" => "已过账".to_string(),
        _ => s.to_string(),
    }
}

#[component]
fn DetailField(
    label: &'static str,
    value: String,
    #[prop(default = false)] highlight: bool,
) -> impl IntoView {
    view! {
        <div class="detail-field">
            <span class="detail-field-label">{label}</span>
            <span class="detail-field-value" class=("highlight", highlight)>{value}</span>
        </div>
    }
}
