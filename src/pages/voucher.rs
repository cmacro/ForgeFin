use leptos::prelude::*;

use crate::components::charts::badge::{status_variant, Badge};
use crate::components::charts::kpi_card::{KpiAccent, KpiCard};
use crate::components::form::search_form::{FieldKind, SearchField, SearchForm, SelectOption};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::table::pagination::Pagination;

#[component]
pub fn VoucherManagement() -> impl IntoView {
    let rows = sample_rows();
    let (selected, set_selected) = signal(2usize);
    let rows_for_detail = rows.clone();
    let rows_for_table = rows.clone();
    let set_selected_action = set_selected.clone();

    let tabs = vec![
        TabItem {
            key: "voucher_overview",
            label: "凭证概览",
            closable: true,
        },
        TabItem {
            key: "voucher_management",
            label: "凭证管理",
            closable: false,
        },
    ];

    let fields = vec![
        SearchField {
            key: "period",
            label: "期间",
            kind: FieldKind::DateRange,
            width: None,
        },
        SearchField {
            key: "voucher_no",
            label: "凭证字号",
            kind: FieldKind::Text {
                placeholder: Some("请输入凭证字号"),
            },
            width: None,
        },
        SearchField {
            key: "voucher_type",
            label: "凭证类型",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "recording",
                        label: "记账凭证",
                    },
                    SelectOption {
                        value: "transfer",
                        label: "转账凭证",
                    },
                    SelectOption {
                        value: "payment",
                        label: "付款凭证",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
        SearchField {
            key: "operator",
            label: "制单人",
            kind: FieldKind::Select {
                options: vec![SelectOption {
                    value: "all",
                    label: "全部",
                }],
                placeholder: Some("全部"),
            },
            width: None,
        },
        SearchField {
            key: "audit_status",
            label: "审核状态",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "audited",
                        label: "已审核",
                    },
                    SelectOption {
                        value: "unaudited",
                        label: "未审核",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
    ];

    view! {
        <div class="flex flex-col flex-1 min-h-0 overflow-hidden bg-surface-alt">
            <Tabs items=tabs active_key="voucher_management" />

            <div class="flex flex-col flex-1 min-h-0 p-4 gap-4 overflow-auto">
                <SearchForm
                    fields=fields
                    on_search=std::rc::Rc::new(move || {})
                    on_reset=std::rc::Rc::new(move || {})
                    expandable=true
                />

                <SummaryStats />

                <ActionBar />

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1 min-h-0">
                    <div class="bg-surface border border-main rounded-md shadow-sm flex flex-col min-h-0 overflow-hidden">
                        <DataTable rows=rows_for_table on_select=set_selected_action />
                        <Pagination total=245 current=1 page_size=20 />
                    </div>
                    <VoucherDetail rows=rows_for_detail selected=selected />
                </div>
            </div>
        </div>
    }
}

#[component]
fn SummaryStats() -> impl IntoView {
    let stats = [
        ("凭证总数", "245", Some("张"), KpiAccent::Brand),
        ("已审核凭证", "198", Some("张"), KpiAccent::Success),
        ("未审核凭证", "47", Some("张"), KpiAccent::Warning),
        ("借方金额", "8,250,000.00", Some("元"), KpiAccent::Primary),
        ("贷方金额", "8,250,000.00", Some("元"), KpiAccent::Primary),
        ("借贷差额", "0.00", Some("元"), KpiAccent::Info),
    ];
    view! {
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
            <For each=move || stats.to_vec() key=|s| s.0 let:stat>
                <KpiCard
                    label=stat.0
                    value=stat.1.to_string()
                    unit=stat.2
                    accent=stat.3
                />
            </For>
        </div>
    }
}

#[component]
fn ActionBar() -> impl IntoView {
    view! {
        <div class="flex items-center justify-between flex-wrap gap-2">
            <div class="flex items-center gap-2 flex-wrap">
                <button class="h-8 px-3 text-sm rounded-md text-white bg-brand hover:bg-brand-hover inline-flex items-center gap-1">
                    <svg class="w-3 h-3" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.8">
                        <path d="M6 2v8M2 6h8" stroke-linecap="round" />
                    </svg>
                    "新增凭证"
                </button>
                <ActionButton label="删除" icon_path="M4 5h8M6 5V3h2v2M5 5l1 8h4l1-8" />
                <ActionButton label="审核" icon_path="M4 10l4 4 8-8" />
                <ActionButton label="反审核" icon_path="M4 6l8 8M12 6l-8 8" />
                <ActionButton label="打印" icon_path="M5 7V3h10v4M4 7h12v8H4zM7 11h6" />
                <ActionButton label="导出" icon_path="M10 12V4m0 0l-3 3m3-3l3 3M4 14h12v3H4z" />
                <ActionButton label="设置" icon_path="M10 3v3M10 14v3M3 10h3M14 10h3M5 5l2 2M13 13l2 2M15 5l-2 2M7 13l-2 2" />
            </div>
        </div>
    }
}

#[component]
fn ActionButton(label: &'static str, icon_path: &'static str) -> impl IntoView {
    view! {
        <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover inline-flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 text-secondary" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d=icon_path stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            {label}
        </button>
    }
}

#[derive(Clone)]
struct VoucherRow {
    index: String,
    no: String,
    date: String,
    summary: String,
    vtype: String,
    debit: String,
    credit: String,
    operator: String,
    auditor: String,
    status: String,
}

#[component]
fn DataTable(rows: Vec<VoucherRow>, on_select: WriteSignal<usize>) -> impl IntoView {
    let total_rows = rows.len();
    let rows_for_render = rows.clone();
    view! {
        <div class="flex-1 overflow-auto">
            <table class="w-full text-sm border-collapse">
                <thead class="bg-surface-alt sticky top-0 z-10">
                    <tr class="border-b border-main">
                        <th class="w-10 px-3 py-2 text-center">
                            <input type="checkbox" class="rounded border-main" />
                        </th>
                        <th class="px-3 py-2 text-center text-secondary font-medium w-12">"序号"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"凭证字号"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"凭证日期"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"摘要"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"凭证类型"</th>
                        <th class="px-3 py-2 text-right text-secondary font-medium">"借方金额"</th>
                        <th class="px-3 py-2 text-right text-secondary font-medium">"贷方金额"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"制单人"</th>
                        <th class="px-3 py-2 text-left text-secondary font-medium">"审核人"</th>
                        <th class="px-3 py-2 text-center text-secondary font-medium">"审核状态"</th>
                        <th class="px-3 py-2 text-center text-secondary font-medium border-l border-main">"操作"</th>
                    </tr>
                </thead>
                <tbody>
                    <For each=move || rows_for_render.clone() key=|r| r.no.clone() let:row>
                        {
                            let row_for_select = row.clone();
                            let idx = row_for_select.index.parse::<usize>().unwrap_or(1);
                            view! {
                                <RowItem
                                    row=row
                                    index=idx
                                    on_select=on_select
                                />
                            }
                        }
                    </For>
                </tbody>
            </table>
            {move || {
                if total_rows == 0 {
                    view! {
                        <div class="text-center text-secondary py-10 text-sm">"暂无数据"</div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn RowItem(row: VoucherRow, index: usize, on_select: WriteSignal<usize>) -> impl IntoView {
    let status_for_badge = row.status.clone();
    let variant = status_variant(&status_for_badge);
    let (checked, set_checked) = signal(index == 3);
    view! {
        <tr
            class="border-b border-muted hover:bg-surface-hover h-10 cursor-pointer"
            on:click=move |_| on_select.set(index.saturating_sub(1))
        >
            <td class="px-3 py-2 text-center" on:click=move |ev| ev.stop_propagation()>
                <input
                    type="checkbox"
                    class="rounded border-main"
                    prop:checked=checked
                    on:change=move |ev| set_checked.set(event_target_checked(&ev))
                />
            </td>
            <td class="px-3 py-2 text-center text-secondary tabular-nums">{row.index}</td>
            <td class="px-3 py-2 text-primary">{row.no}</td>
            <td class="px-3 py-2 text-primary tabular-nums">{row.date}</td>
            <td class="px-3 py-2 text-primary">{row.summary}</td>
            <td class="px-3 py-2 text-primary">{row.vtype}</td>
            <td class="px-3 py-2 text-right text-primary tabular-nums">{row.debit}</td>
            <td class="px-3 py-2 text-right text-primary tabular-nums">{row.credit}</td>
            <td class="px-3 py-2 text-primary">{row.operator}</td>
            <td class="px-3 py-2 text-primary">{row.auditor}</td>
            <td class="px-3 py-2 text-center">
                <Badge label=status_for_badge variant=variant />
            </td>
            <td class="px-3 py-2 border-l border-main" on:click=move |ev| ev.stop_propagation()>
                <div class="flex items-center justify-center gap-1">
                    <button class="text-xs text-brand hover:text-brand-hover px-1.5">"查看"</button>
                    <button class="text-xs text-secondary hover:text-primary px-1.5">"复制"</button>
                    <button class="text-xs text-secondary hover:text-primary px-1 inline-flex items-center gap-0.5">
                        "更多"
                        <svg class="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M3 4l2 2 2-2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn VoucherDetail(rows: Vec<VoucherRow>, selected: ReadSignal<usize>) -> impl IntoView {
    let entries = vec![
        (
            "1",
            "1122.01",
            "应收账款-客户A",
            "客户: 客户A",
            "0.00",
            "250,000.00",
        ),
        (
            "2",
            "6001.01",
            "主营业务收入",
            "项目: 主营业务",
            "0.00",
            "213,675.21",
        ),
        (
            "3",
            "2221.01",
            "应交税费-应交增值税(销项税额)",
            "—",
            "0.00",
            "36,324.79",
        ),
    ];
    let (log_tab, set_log_tab) = signal("audit");
    let rows_for_no = rows.clone();
    let rows_for_summary = rows.clone();
    let rows_for_date = rows.clone();
    let voucher_no: Signal<String> = Signal::derive(move || {
        rows_for_no
            .get(selected.get())
            .map(|r| r.no.clone())
            .unwrap_or_else(|| "—".to_string())
    });
    let voucher_summary: Signal<String> = Signal::derive(move || {
        rows_for_summary
            .get(selected.get())
            .map(|r| r.summary.clone())
            .unwrap_or_else(|| "—".to_string())
    });
    let voucher_date: Signal<String> = Signal::derive(move || {
        rows_for_date
            .get(selected.get())
            .map(|r| r.date.clone())
            .unwrap_or_else(|| "—".to_string())
    });

    view! {
        <div class="bg-surface border border-main rounded-md shadow-sm flex flex-col min-h-0 overflow-hidden">
            <div class="flex items-center justify-between p-3 border-b border-main">
                <div class="flex items-center gap-3 text-sm flex-wrap">
                    <span class="text-primary font-semibold">
                        "凭证详情（"
                        {move || voucher_no.get()}
                        "）"
                    </span>
                </div>
                <button class="h-7 px-3 text-xs border border-main rounded text-primary bg-surface hover:bg-surface-hover inline-flex items-center gap-1">
                    <svg class="w-3 h-3" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M2 9l2-6h4l2 6M4 9h4M6 3v6" stroke-linecap="round" />
                    </svg>
                    "编辑"
                </button>
            </div>

            <div class="grid grid-cols-2 gap-3 p-3 text-sm border-b border-muted">
                <DetailField label="凭证类型" value="记账凭证".to_string() />
                <DetailFieldReactive label="凭证日期" value=voucher_date />
                <DetailFieldReactive label="凭证字号" value=voucher_no />
                <DetailField label="附件" value="0".to_string() />
                <DetailFieldReactive label="摘要" value=voucher_summary />
                <DetailField label="审核状态" value="已审核".to_string() highlight=true />
            </div>

            <div class="flex items-center gap-2 px-3 pt-3 text-xs flex-wrap">
                <DetailChip label="制单人" value="张会计" />
                <DetailChip label="审核人" value="李主管" />
                <DetailChip label="审核日期" value="2024-06-03" />
            </div>

            <div class="flex-1 overflow-auto p-3">
                <table class="w-full text-sm border-collapse">
                    <thead>
                        <tr class="border-b border-main bg-surface-alt">
                            <th class="px-2 py-2 text-center text-secondary font-medium w-10">"序号"</th>
                            <th class="px-2 py-2 text-left text-secondary font-medium">"科目编码"</th>
                            <th class="px-2 py-2 text-left text-secondary font-medium">"科目名称"</th>
                            <th class="px-2 py-2 text-left text-secondary font-medium">"辅助核算"</th>
                            <th class="px-2 py-2 text-right text-secondary font-medium">"借方金额"</th>
                            <th class="px-2 py-2 text-right text-secondary font-medium">"贷方金额"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For each=move || entries.clone() key=|e| e.0.to_string() let:entry>
                            <tr class="border-b border-muted h-10">
                                <td class="px-2 text-center text-secondary tabular-nums">{entry.0}</td>
                                <td class="px-2 text-primary tabular-nums">{entry.1}</td>
                                <td class="px-2 text-primary">{entry.2}</td>
                                <td class="px-2 text-secondary">{entry.3}</td>
                                <td class="px-2 text-right text-primary tabular-nums">{entry.4}</td>
                                <td class="px-2 text-right text-primary tabular-nums">{entry.5}</td>
                            </tr>
                        </For>
                        <tr class="bg-surface-alt h-10">
                            <td class="px-2 text-secondary font-medium">"合计"</td>
                            <td></td>
                            <td></td>
                            <td></td>
                            <td class="px-2 text-right text-primary font-semibold tabular-nums">"0.00"</td>
                            <td class="px-2 text-right text-primary font-semibold tabular-nums">"250,000.00"</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="flex items-center justify-between p-3 border-t border-main text-xs text-secondary">
                <div class="flex items-center gap-4 flex-wrap">
                    <span>"制单: 张会计　2024-06-03"</span>
                    <span>"审核: 李主管　2024-06-03"</span>
                </div>
            </div>

            <div class="border-t border-main flex flex-col min-h-0">
                <div class="flex items-center justify-between px-3 border-b border-muted">
                    <div class="flex">
                        <TabButton label="附件 (0)" active=false />
                        <TabButtonReactive
                            label="审核日志"
                            active=Signal::derive(move || log_tab.get() == "audit")
                            on_click=std::rc::Rc::new(move || set_log_tab.set("audit"))
                        />
                        <TabButtonReactive
                            label="操作日志"
                            active=Signal::derive(move || log_tab.get() == "operation")
                            on_click=std::rc::Rc::new(move || set_log_tab.set("operation"))
                        />
                    </div>
                </div>
                <div class="p-4 space-y-4 text-sm flex-1 overflow-auto">
                    <LogEntry
                        dot_color="bg-success"
                        title="审核通过"
                        user="李主管"
                        timestamp="2024-06-03 10:30:15"
                        comment="审核意见：凭证完整,金额正确,予以通过。"
                    />
                    <LogEntry
                        dot_color="bg-brand"
                        title="提交审核"
                        user="张会计"
                        timestamp="2024-06-03 09:15:22"
                        comment="备注：请审核"
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn DetailField(
    label: &'static str,
    value: String,
    #[prop(default = false)] highlight: bool,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-secondary shrink-0 w-16">{label}</span>
            <span class=format!(
                "h-7 px-2 flex items-center text-sm border border-main rounded bg-surface-alt min-w-0 flex-1 truncate {}",
                if highlight { "text-success font-medium" } else { "text-primary" }
            )>{value}</span>
        </div>
    }
}

#[component]
fn DetailFieldReactive(
    label: &'static str,
    value: Signal<String>,
    #[prop(default = false)] highlight: bool,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-secondary shrink-0 w-16">{label}</span>
            <span class=format!(
                "h-7 px-2 flex items-center text-sm border border-main rounded bg-surface-alt min-w-0 flex-1 truncate {}",
                if highlight { "text-success font-medium" } else { "text-primary" }
            )>{move || value.get()}</span>
        </div>
    }
}

#[component]
fn DetailChip(label: &'static str, value: &'static str) -> impl IntoView {
    let colon = ":";
    view! {
        <span class="inline-flex items-center gap-1 px-2 h-6 rounded bg-surface-alt border border-main text-xs">
            <span class="text-secondary">{label}{colon}</span>
            <span class="text-primary font-medium">{value}</span>
        </span>
    }
}

#[component]
fn TabButton(label: &'static str, active: bool) -> impl IntoView {
    view! {
        <button
            class="px-3 py-2 text-xs cursor-pointer"
            class=("text-primary border-b-2 border-brand -mb-px font-medium", active)
            class=("text-secondary hover:text-primary border-b-2 border-transparent -mb-px", !active)
        >
            {label}
        </button>
    }
}

#[component]
fn TabButtonReactive(
    label: &'static str,
    active: Signal<bool>,
    on_click: std::rc::Rc<dyn Fn()>,
) -> impl IntoView {
    let cb = on_click.clone();
    view! {
        <button
            class="px-3 py-2 text-xs cursor-pointer"
            class=("text-primary border-b-2 border-brand -mb-px font-medium", move || active.get())
            class=("text-secondary hover:text-primary border-b-2 border-transparent -mb-px", move || !active.get())
            on:click=move |_| cb()
        >
            {label}
        </button>
    }
}

#[component]
fn LogEntry(
    dot_color: &'static str,
    title: &'static str,
    user: &'static str,
    timestamp: &'static str,
    comment: &'static str,
) -> impl IntoView {
    view! {
        <div class="relative pl-6">
            <span class=format!("absolute left-1.5 top-1.5 w-2.5 h-2.5 rounded-full ring-2 ring-surface {dot_color}")></span>
            <div class="text-sm text-primary font-medium">{title}</div>
            <div class="mt-1 flex items-center gap-2 text-xs">
                <span class="text-primary font-medium">{user}</span>
                <span class="text-secondary">{timestamp}</span>
            </div>
            <div class="mt-1 text-xs text-secondary">{comment}</div>
        </div>
    }
}

fn sample_rows() -> Vec<VoucherRow> {
    vec![
        VoucherRow {
            index: "1".into(),
            no: "记-2024-06-0001".into(),
            date: "2024-06-01".into(),
            summary: "购买办公用品".into(),
            vtype: "记账凭证".into(),
            debit: "1,250.00".into(),
            credit: "1,250.00".into(),
            operator: "张会计".into(),
            auditor: "李主管".into(),
            status: "已审核".into(),
        },
        VoucherRow {
            index: "2".into(),
            no: "记-2024-06-0002".into(),
            date: "2024-06-02".into(),
            summary: "支付供应商货款".into(),
            vtype: "记账凭证".into(),
            debit: "125,000.00".into(),
            credit: "125,000.00".into(),
            operator: "张会计".into(),
            auditor: "李主管".into(),
            status: "已审核".into(),
        },
        VoucherRow {
            index: "3".into(),
            no: "记-2024-06-0003".into(),
            date: "2024-06-03".into(),
            summary: "销售商品收入".into(),
            vtype: "记账凭证".into(),
            debit: "0.00".into(),
            credit: "250,000.00".into(),
            operator: "张会计".into(),
            auditor: "李主管".into(),
            status: "已审核".into(),
        },
        VoucherRow {
            index: "4".into(),
            no: "转-2024-06-0001".into(),
            date: "2024-06-05".into(),
            summary: "计提工资".into(),
            vtype: "转账凭证".into(),
            debit: "85,000.00".into(),
            credit: "85,000.00".into(),
            operator: "张会计".into(),
            auditor: "李主管".into(),
            status: "未审核".into(),
        },
        VoucherRow {
            index: "5".into(),
            no: "记-2024-06-0004".into(),
            date: "2024-06-06".into(),
            summary: "收取客户货款".into(),
            vtype: "记账凭证".into(),
            debit: "300,000.00".into(),
            credit: "300,000.00".into(),
            operator: "张会计".into(),
            auditor: "—".into(),
            status: "未审核".into(),
        },
    ]
}
