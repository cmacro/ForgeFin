use leptos::prelude::*;
use lucide_leptos::{
    ArrowLeftRight, Check, ChevronDown, Pencil, Plus, Printer, SlidersHorizontal, Trash2, Upload,
};

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
        <Tabs items=tabs active_key="voucher_management" />

        <div class="flex flex-col gap-4 flex-1">
            <SearchForm
                fields=fields
                on_search=std::rc::Rc::new(move || {})
                on_reset=std::rc::Rc::new(move || {})
                expandable=true
            />

            <SummaryStats />

            <ActionBar />

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1 min-h-0">
                <div class="data-table flex flex-col min-h-0">
                    <DataTable rows=rows_for_table on_select=set_selected_action />
                    <div style="border-top: 1px solid var(--color-border-light)">
                        <Pagination total=245 current=1 page_size=20 />
                    </div>
                </div>
                <VoucherDetail rows=rows_for_detail selected=selected />
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
                <button class="h-8 px-3 text-sm rounded-md text-white" style="background: var(--color-brand); display: inline-flex; align-items: center; gap: 4px">
                    <Plus size=14 />
                    "新增凭证"
                </button>
                <ActionButton label="删除">
                    <Trash2 size=14 />
                </ActionButton>
                <ActionButton label="审核">
                    <Check size=14 />
                </ActionButton>
                <ActionButton label="反审核">
                    <ArrowLeftRight size=14 />
                </ActionButton>
                <ActionButton label="打印">
                    <Printer size=14 />
                </ActionButton>
                <ActionButton label="导出">
                    <Upload size=14 />
                </ActionButton>
                <ActionButton label="设置">
                    <SlidersHorizontal size=14 />
                </ActionButton>
            </div>
        </div>
    }
}

#[component]
fn ActionButton(label: &'static str, children: ChildrenFn) -> impl IntoView {
    view! {
        <button
            class="h-8 px-3 text-sm border rounded-md"
            style="border-color: var(--color-border); color: var(--color-primary); background: var(--color-surface); display: inline-flex; align-items: center; gap: 6px"
        >
            <span style="color: var(--color-secondary); display: inline-flex; align-items: center; justify-content: center">
                {children()}
            </span>
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
            <table>
                <thead>
                    <tr>
                        <th style="width: 40px; text-align: center">
                            <input type="checkbox" class="form-input" style="width: 14px; height: 14px" />
                        </th>
                        <th style="width: 48px; text-align: center">"序号"</th>
                        <th>"凭证字号"</th>
                        <th>"凭证日期"</th>
                        <th>"摘要"</th>
                        <th>"凭证类型"</th>
                        <th class="data-table-num">"借方金额"</th>
                        <th class="data-table-num">"贷方金额"</th>
                        <th>"制单人"</th>
                        <th>"审核人"</th>
                        <th style="text-align: center">"审核状态"</th>
                        <th style="text-align: center; border-left: 1px solid var(--color-border)">"操作"</th>
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
                        <div style="text-align: center; color: var(--color-tertiary); padding: 40px 0; font-size: 14px">"暂无数据"</div>
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
            on:click=move |_| on_select.set(index.saturating_sub(1))
        >
            <td style="text-align: center" on:click=move |ev| ev.stop_propagation()>
                <input
                    type="checkbox"
                    class="form-input"
                    style="width: 14px; height: 14px"
                    prop:checked=checked
                    on:change=move |ev| set_checked.set(event_target_checked(&ev))
                />
            </td>
            <td class="data-table-num">{row.index}</td>
            <td>{row.no}</td>
            <td class="data-table-num">{row.date}</td>
            <td>{row.summary}</td>
            <td>{row.vtype}</td>
            <td class="data-table-num">{row.debit}</td>
            <td class="data-table-num">{row.credit}</td>
            <td>{row.operator}</td>
            <td>{row.auditor}</td>
            <td style="text-align: center">
                <Badge label=status_for_badge variant=variant />
            </td>
            <td style="text-align: center; border-left: 1px solid var(--color-border)" on:click=move |ev| ev.stop_propagation()>
                <div style="display: flex; align-items: center; justify-content: center; gap: 4px">
                    <button class="text-xs" style="color: var(--color-brand)">"查看"</button>
                    <button class="text-xs" style="color: var(--color-secondary)">"复制"</button>
                    <button class="text-xs inline-flex items-center gap-0.5" style="color: var(--color-secondary)">
                        "更多"
                        <ChevronDown size=10 />
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
        <div class="card flex flex-col min-h-0">
            <div class="card-header">
                <span class="card-title">
                    "凭证详情（"
                    {move || voucher_no.get()}
                    "）"
                </span>
                <button class="h-7 px-3 text-xs border rounded-md inline-flex items-center gap-1" style="border-color: var(--color-border); color: var(--color-primary); background: var(--color-surface)">
                    <Pencil size=12 />
                    "编辑"
                </button>
            </div>

            <div class="grid grid-cols-2 gap-3 p-3 text-sm" style="border-bottom: 1px solid var(--color-border-light)">
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
                <table class="data-table" style="border: none">
                    <thead>
                        <tr>
                            <th style="width: 40px; text-align: center">"序号"</th>
                            <th>"科目编码"</th>
                            <th>"科目名称"</th>
                            <th>"辅助核算"</th>
                            <th class="data-table-num">"借方金额"</th>
                            <th class="data-table-num">"贷方金额"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For each=move || entries.clone() key=|e| e.0.to_string() let:entry>
                            <tr>
                                <td class="data-table-num">{entry.0}</td>
                                <td class="data-table-num">{entry.1}</td>
                                <td>{entry.2}</td>
                                <td>{entry.3}</td>
                                <td class="data-table-num">{entry.4}</td>
                                <td class="data-table-num">{entry.5}</td>
                            </tr>
                        </For>
                        <tr style="background: var(--color-surface-alt)">
                            <td class="text-secondary font-medium">"合计"</td>
                            <td></td>
                            <td></td>
                            <td></td>
                            <td class="data-table-num text-primary font-semibold">"0.00"</td>
                            <td class="data-table-num text-primary font-semibold">"250,000.00"</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="card-footer justify-between text-xs" style="color: var(--color-tertiary)">
                <div class="flex items-center gap-4 flex-wrap">
                    <span>"制单: 张会计　2024-06-03"</span>
                    <span>"审核: 李主管　2024-06-03"</span>
                </div>
            </div>

            <div style="border-top: 1px solid var(--color-border)">
                <div class="flex items-center justify-between px-3" style="border-bottom: 1px solid var(--color-border-light)">
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
                        dot_color="#00b42a"
                        title="审核通过"
                        user="李主管"
                        timestamp="2024-06-03 10:30:15"
                        comment="审核意见：凭证完整,金额正确,予以通过。"
                    />
                    <LogEntry
                        dot_color="#1677ff"
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
        <div style="display: flex; align-items: center; gap: 8px; min-width: 0">
            <span style="font-size: 12px; color: var(--color-tertiary); flex-shrink: 0; width: 64px">{label}</span>
            <span style=format!(
                "height: 28px; padding: 0 8px; display: flex; align-items: center; font-size: 14px; border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-surface-alt); min-width: 0; flex: 1; {}",
                if highlight { "color: var(--color-success); font-weight: 500" } else { "color: var(--color-primary)" }
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
        <div style="display: flex; align-items: center; gap: 8px; min-width: 0">
            <span style="font-size: 12px; color: var(--color-tertiary); flex-shrink: 0; width: 64px">{label}</span>
            <span style=format!(
                "height: 28px; padding: 0 8px; display: flex; align-items: center; font-size: 14px; border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-surface-alt); min-width: 0; flex: 1; {}",
                if highlight { "color: var(--color-success); font-weight: 500" } else { "color: var(--color-primary)" }
            )>{move || value.get()}</span>
        </div>
    }
}

#[component]
fn DetailChip(label: &'static str, value: &'static str) -> impl IntoView {
    let colon = ":";
    view! {
        <span class="tag" style="background: var(--color-surface-alt)">
            <span style="color: var(--color-tertiary)">{label}{colon}</span>
            <span style="color: var(--color-primary); font-weight: 500">{value}</span>
        </span>
    }
}

#[component]
fn TabButton(label: &'static str, active: bool) -> impl IntoView {
    view! {
        <button
            class="px-3 py-2 text-xs cursor-pointer"
            style=move || {
                if active {
                    "color: var(--color-primary); border-bottom: 2px solid var(--color-brand); font-weight: 500; margin-bottom: -1px"
                } else {
                    "color: var(--color-secondary); border-bottom: 2px solid transparent; margin-bottom: -1px"
                }
            }
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
            style=move || {
                if active.get() {
                    "color: var(--color-primary); border-bottom: 2px solid var(--color-brand); font-weight: 500; margin-bottom: -1px"
                } else {
                    "color: var(--color-secondary); border-bottom: 2px solid transparent; margin-bottom: -1px"
                }
            }
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
        <div style="position: relative; padding-left: 24px">
            <span style=format!("position: absolute; left: 6px; top: 6px; width: 10px; height: 10px; border-radius: 999px; background: {dot_color}; border: 2px solid var(--color-surface)")></span>
            <div style="font-size: 14px; color: var(--color-primary); font-weight: 500">{title}</div>
            <div style="margin-top: 4px; display: flex; align-items: center; gap: 8px; font-size: 12px">
                <span style="color: var(--color-primary); font-weight: 500">{user}</span>
                <span style="color: var(--color-tertiary)">{timestamp}</span>
            </div>
            <div style="margin-top: 4px; font-size: 12px; color: var(--color-tertiary)">{comment}</div>
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
