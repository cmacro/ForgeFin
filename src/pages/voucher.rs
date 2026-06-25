use leptos::prelude::*;

use crate::components::form::search_form::{SearchField, SearchForm};
use crate::components::layout::breadcrumb::{Breadcrumb, Crumb};
use crate::components::table::data_table::{Align, ColumnConfig, DataTable, RowActionsFn};

#[component]
pub fn VoucherManagement() -> impl IntoView {
    let crumbs = vec![
        Crumb { label: "总账" },
        Crumb {
            label: "凭证管理"
        },
    ];

    let fields = vec![
        SearchField {
            key: "period",
            label: "期间",
            placeholder: Some("2026-06"),
        },
        SearchField {
            key: "voucher_no",
            label: "凭证号",
            placeholder: Some("记-001"),
        },
        SearchField {
            key: "voucher_type",
            label: "凭证类别",
            placeholder: Some("记账凭证"),
        },
        SearchField {
            key: "operator",
            label: "制单人",
            placeholder: None,
        },
        SearchField {
            key: "audit_status",
            label: "审核状态",
            placeholder: None,
        },
        SearchField {
            key: "summary",
            label: "摘要",
            placeholder: None,
        },
    ];

    let columns = vec![
        ColumnConfig {
            key: "index",
            label: "序号",
            width: Some("56px"),
            align: Align::Center,
        },
        ColumnConfig {
            key: "no",
            label: "凭证号",
            width: None,
            align: Align::Left,
        },
        ColumnConfig {
            key: "date",
            label: "日期",
            width: None,
            align: Align::Left,
        },
        ColumnConfig {
            key: "summary",
            label: "摘要",
            width: None,
            align: Align::Left,
        },
        ColumnConfig {
            key: "vtype",
            label: "类别",
            width: None,
            align: Align::Center,
        },
        ColumnConfig {
            key: "debit",
            label: "借方金额",
            width: None,
            align: Align::Right,
        },
        ColumnConfig {
            key: "credit",
            label: "贷方金额",
            width: None,
            align: Align::Right,
        },
        ColumnConfig {
            key: "operator",
            label: "制单人",
            width: None,
            align: Align::Left,
        },
        ColumnConfig {
            key: "auditor",
            label: "审核人",
            width: None,
            align: Align::Left,
        },
        ColumnConfig {
            key: "status",
            label: "状态",
            width: None,
            align: Align::Center,
        },
    ];

    let rows = sample_rows();
    let (selected, set_selected) = signal(0usize);
    let rows_for_detail = rows.clone();
    let rows_for_table = rows.clone();
    let set_selected_action = set_selected.clone();

    view! {
        <div class="flex flex-col flex-1 min-h-0 p-4 gap-4 overflow-auto">
            <Breadcrumb crumbs=crumbs />
            <Tabs />

            <SearchForm
                fields=fields
                on_search=std::rc::Rc::new(move || {})
                on_reset=std::rc::Rc::new(move || {})
            />

            <SummaryStats />

            <ActionBar />

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1 min-h-0">
                <div class="flex flex-col min-h-0">
                    <DataTable
                        columns=columns
                        rows=rows_for_table
                        row_actions=RowActionsFn::new(move |row| {
                            let idx = row.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                            set_selected_action.set(idx);
                            vec![
                                view! { <button class="text-xs text-brand hover:text-brand-hover px-1.5">"查看"</button> }.into_any(),
                                view! { <button class="text-xs text-secondary hover:text-primary px-1.5">"复制"</button> }.into_any(),
                                view! { <button class="text-xs text-secondary hover:text-primary px-1.5">"更多"</button> }.into_any(),
                            ]
                        })
                    />
                    <Pagination />
                </div>
                <VoucherDetail rows=rows_for_detail selected=selected />
            </div>
        </div>
    }
}

#[component]
fn Tabs() -> impl IntoView {
    view! {
        <div class="flex items-center gap-1 border-b border-main">
            <button class="px-3 py-2 text-sm text-secondary hover:text-primary">"凭证概览"</button>
            <button class="px-3 py-2 text-sm text-primary border-b-2 border-brand -mb-px">"凭证管理"</button>
        </div>
    }
}

#[component]
fn SummaryStats() -> impl IntoView {
    let stats = [
        ("凭证总数", "128", "text-primary"),
        ("已审核", "96", "text-success"),
        ("未审核", "32", "text-warning"),
        ("借方合计", "¥ 1,256,800.00", "text-primary"),
        ("贷方合计", "¥ 1,256,800.00", "text-primary"),
        ("借贷差额", "¥ 0.00", "text-success"),
    ];
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
            <For each=move || stats.to_vec() key=|s| s.0 let:stat>
                <div class="bg-surface border border-main rounded-md p-3 shadow-sm">
                    <div class="text-xs text-secondary">{stat.0}</div>
                    <div class=format!("text-sm font-semibold mt-1 {}", stat.2)>{stat.1}</div>
                </div>
            </For>
        </div>
    }
}

#[component]
fn ActionBar() -> impl IntoView {
    view! {
        <div class="flex items-center justify-between flex-wrap gap-2">
            <div class="flex items-center gap-2">
                <button class="h-8 px-3 text-sm rounded-md text-white bg-brand hover:bg-brand-hover">"+ 新增凭证"</button>
                <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover">"筛选"</button>
                <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover">"审核"</button>
                <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover">"反审核"</button>
            </div>
            <div class="flex items-center gap-2">
                <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover">"打印"</button>
                <button class="h-8 px-3 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover">"导出"</button>
            </div>
        </div>
    }
}

#[component]
fn Pagination() -> impl IntoView {
    view! {
        <div class="flex items-center justify-between text-xs text-secondary mt-2">
            <span>"共 128 条 · 每页 20 条"</span>
            <div class="flex items-center gap-1">
                <button class="w-7 h-7 border border-main rounded text-primary bg-surface hover:bg-surface-hover">"<"</button>
                <button class="w-7 h-7 border border-brand rounded text-white bg-brand">"1"</button>
                <button class="w-7 h-7 border border-main rounded text-primary bg-surface hover:bg-surface-hover">"2"</button>
                <button class="w-7 h-7 border border-main rounded text-primary bg-surface hover:bg-surface-hover">">"</button>
            </div>
        </div>
    }
}

#[component]
#[allow(unused_variables)]
fn VoucherDetail(rows: Vec<Vec<String>>, selected: ReadSignal<usize>) -> impl IntoView {
    let entries = vec![
        (
            "1001".to_string(),
            "库存现金".to_string(),
            "8,500.00".to_string(),
            "0.00".to_string(),
        ),
        (
            "2202".to_string(),
            "应付账款".to_string(),
            "0.00".to_string(),
            "8,500.00".to_string(),
        ),
    ];
    view! {
        <div class="bg-surface border border-main rounded-md shadow-sm flex flex-col min-h-0 overflow-hidden">
            <div class="flex items-center justify-between p-3 border-b border-main">
                <div class="flex items-center gap-3 text-sm">
                    <span class="text-primary font-medium">"记账凭证"</span>
                    <span class="text-secondary">"2026-06-01"</span>
                    <span class="text-secondary">"记-2026-06-001"</span>
                    <span class="text-secondary">"附件 2 张"</span>
                </div>
                <button class="h-7 px-3 text-xs border border-main rounded text-primary bg-surface hover:bg-surface-hover">"编辑"</button>
            </div>

            <div class="flex-1 overflow-auto p-3">
                <table class="w-full text-sm border-collapse">
                    <thead class="bg-surface-alt">
                        <tr class="border-b border-main">
                            <th class="px-2 py-2 text-left text-secondary font-medium w-10">"序号"</th>
                            <th class="px-2 py-2 text-left text-secondary font-medium">"科目编码"</th>
                            <th class="px-2 py-2 text-left text-secondary font-medium">"科目名称"</th>
                            <th class="px-2 py-2 text-right text-secondary font-medium">"借方"</th>
                            <th class="px-2 py-2 text-right text-secondary font-medium">"贷方"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For each=move || entries.clone() key=|e| e.0.clone() let:entry>
                            <tr class="border-b border-muted h-10">
                                <td class="px-2 text-secondary">"1"</td>
                                <td class="px-2 text-primary">{entry.0}</td>
                                <td class="px-2 text-primary">{entry.1}</td>
                                <td class="px-2 text-right text-primary tabular-nums">{entry.2}</td>
                                <td class="px-2 text-right text-primary tabular-nums">{entry.3}</td>
                            </tr>
                        </For>
                        <tr class="bg-surface-alt h-10">
                            <td class="px-2 text-secondary font-medium">"合计"</td>
                            <td></td>
                            <td></td>
                            <td class="px-2 text-right text-primary font-semibold tabular-nums">"8,500.00"</td>
                            <td class="px-2 text-right text-primary font-semibold tabular-nums">"8,500.00"</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="flex items-center justify-between p-3 border-t border-main text-xs text-secondary">
                <div class="flex gap-4">
                    <span>"制单: 张会计"</span>
                    <span>"审核: 李审核"</span>
                    <span>"审核日期: 2026-06-02"</span>
                    <span class="text-success">"已审核"</span>
                </div>
                <button class="h-7 px-3 text-xs rounded text-white bg-brand hover:bg-brand-hover">"提交"</button>
            </div>

            <div class="border-t border-main p-3">
                <div class="flex border-b border-muted">
                    <button class="px-3 py-1.5 text-xs text-primary border-b-2 border-brand -mb-px">"附件"</button>
                    <button class="px-3 py-1.5 text-xs text-secondary hover:text-primary">"审核日志"</button>
                    <button class="px-3 py-1.5 text-xs text-secondary hover:text-primary">"操作日志"</button>
                </div>
                <div class="text-xs text-secondary py-2">
                    {move || format!("当前查看第 {} 条", selected.get() + 1)}
                </div>
            </div>
        </div>
    }
}

fn sample_rows() -> Vec<Vec<String>> {
    vec![
        vec![
            "1".into(),
            "记-2026-06-001".into(),
            "2026-06-01".into(),
            "支付供应商货款".into(),
            "记账凭证".into(),
            "8,500.00".into(),
            "0.00".into(),
            "张会计".into(),
            "李审核".into(),
            "已审核".into(),
        ],
        vec![
            "2".into(),
            "记-2026-06-002".into(),
            "2026-06-02".into(),
            "收到客户回款".into(),
            "记账凭证".into(),
            "0.00".into(),
            "12,300.00".into(),
            "张会计".into(),
            "—".into(),
            "未审核".into(),
        ],
        vec![
            "3".into(),
            "付-2026-06-003".into(),
            "2026-06-03".into(),
            "差旅费用报销".into(),
            "付款凭证".into(),
            "3,200.00".into(),
            "0.00".into(),
            "王出纳".into(),
            "李审核".into(),
            "已审核".into(),
        ],
    ]
}
