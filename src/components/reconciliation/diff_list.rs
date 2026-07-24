use leptos::prelude::*;

use crate::ipc::ReconciliationItem;

/// 差异列表表格组件。
///
/// 显示对账日期、银行金额、订单金额、差额、状态、凭证字号及操作按钮。
#[component]
pub fn DiffList(
    rows: Vec<ReconciliationItem>,
    selected_id: ReadSignal<Option<i64>>,
    set_selected_id: WriteSignal<Option<i64>>,
    #[prop(default = Callback::new(|_| {}))] on_review: Callback<i64>,
) -> impl IntoView {
    let total_rows = rows.len();
    view! {
        <div class="flex-1 overflow-auto">
            <table>
                <thead>
                    <tr>
                        <th class="data-table-num">"对账日期"</th>
                        <th class="data-table-num">"银行金额"</th>
                        <th class="data-table-num">"订单金额"</th>
                        <th class="data-table-num">"差额"</th>
                        <th class="text-center">"状态"</th>
                        <th>"凭证字号"</th>
                        <th class="text-center">"操作"</th>
                    </tr>
                </thead>
                <tbody>
                    <For each=move || rows.clone() key=|r| r.id let:row>
                        <RowItem
                            row=row
                            selected=selected_id
                            set_selected=set_selected_id
                            on_review=on_review
                        />
                    </For>
                </tbody>
            </table>
            {move || {
                if total_rows == 0 {
                    view! {
                        <div class="text-center py-40 text-tertiary">"暂无差异记录"</div>
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
    row: ReconciliationItem,
    selected: ReadSignal<Option<i64>>,
    set_selected: WriteSignal<Option<i64>>,
    on_review: Callback<i64>,
) -> impl IntoView {
    let id = row.id;
    let is_active = move || selected.get() == Some(id);
    let status_label = status_cn(&row.review_status);
    let status_class = status_class(&row.review_status);
    let can_review = row.review_status == "pending";
    view! {
        <tr
            class=("selected", is_active)
            on:click=move |_| set_selected.set(Some(id))
        >
            <td class="data-table-num">{row.summary_date}</td>
            <td class="data-table-num">{row.bank_amount}</td>
            <td class="data-table-num">{row.order_amount}</td>
            <td class="data-table-num">{row.diff_amount}</td>
            <td class="text-center">
                <span class={format!("text-13 {status_class}")}>{status_label}</span>
            </td>
            <td>{row.voucher_no.unwrap_or("—".to_string())}</td>
            <td class="text-center" on:click=move |ev| ev.stop_propagation()>
                {move || {
                    if can_review {
                        view! {
                            <button
                                class="btn btn-sm btn-primary"
                                on:click=move |_| on_review.run(id)
                            >
                                "审核"
                            </button>
                        }.into_any()
                    } else {
                        view! { <span class="text-tertiary text-13">"—"</span> }.into_any()
                    }
                }}
            </td>
        </tr>
    }
}

fn status_cn(status: &str) -> String {
    match status {
        "pending" => "待审核",
        "auto_matched" => "自动匹配",
        "approved" => "已通过",
        "rejected" => "已驳回",
        _ => status,
    }
    .to_string()
}

fn status_class(status: &str) -> &'static str {
    match status {
        "pending" => "text-warning",
        "auto_matched" => "text-success",
        "approved" => "text-success",
        "rejected" => "text-danger",
        _ => "text-tertiary",
    }
}
