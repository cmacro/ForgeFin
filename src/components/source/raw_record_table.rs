use leptos::prelude::*;

use crate::ipc::RawRecord;

/// 原始记录表格组件。
///
/// 显示来源类型、文件、日期、金额、对方信息等,支持行选择与分页。
#[component]
pub fn RawRecordTable(
    rows: Vec<RawRecord>,
    selected_id: ReadSignal<Option<i64>>,
    set_selected_id: WriteSignal<Option<i64>>,
) -> impl IntoView {
    let total_rows = rows.len();
    view! {
        <div class="flex-1 overflow-auto">
            <table>
                <thead>
                    <tr>
                        <th>"来源类型"</th>
                        <th>"来源文件"</th>
                        <th class="data-table-num">"行号"</th>
                        <th>"业务单号"</th>
                        <th class="data-table-num">"日期"</th>
                        <th class="data-table-num">"金额"</th>
                        <th>"对方信息"</th>
                        <th>"摘要"</th>
                        <th>"状态"</th>
                    </tr>
                </thead>
                <tbody>
                    <For each=move || rows.clone() key=|r| r.id let:row>
                        <RowItem
                            row=row
                            selected=selected_id
                            set_selected=set_selected_id
                        />
                    </For>
                </tbody>
            </table>
            {move || {
                if total_rows == 0 {
                    view! {
                        <div class="text-center py-40 text-tertiary">"暂无原始记录"</div>
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
    row: RawRecord,
    selected: ReadSignal<Option<i64>>,
    set_selected: WriteSignal<Option<i64>>,
) -> impl IntoView {
    let id = row.id;
    let is_active = move || selected.get() == Some(id);
    let status_label = status_cn(&row.status);
    view! {
        <tr
            class=("selected", is_active)
            on:click=move |_| set_selected.set(Some(id))
        >
            <td>{row.source_type_name}</td>
            <td>{row.source_file_name}</td>
            <td class="data-table-num">{row.source_row_no}</td>
            <td>{row.record_no.unwrap_or("—".to_string())}</td>
            <td class="data-table-num">{row.record_date.unwrap_or("—".to_string())}</td>
            <td class="data-table-num">{row.amount_total.unwrap_or("—".to_string())}</td>
            <td>{row.counterpart_info.unwrap_or("—".to_string())}</td>
            <td>{row.summary.unwrap_or("—".to_string())}</td>
            <td>
                <span class={format!("text-13 {}", status_class(&row.status))}>
                    {status_label}
                </span>
            </td>
        </tr>
    }
}

fn status_cn(status: &str) -> String {
    match status {
        "pending" => "待处理",
        "matched" => "已匹配",
        "approved" => "已审核",
        "rejected" => "已驳回",
        _ => status,
    }
    .to_string()
}

fn status_class(status: &str) -> &'static str {
    match status {
        "pending" => "text-warning",
        "matched" => "text-success",
        "approved" => "text-success",
        "rejected" => "text-danger",
        _ => "text-tertiary",
    }
}
