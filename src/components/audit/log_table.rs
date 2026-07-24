use leptos::prelude::*;

use crate::ipc::AuditLogEntry;

/// 审计日志表格组件。
///
/// 显示操作类型、实体、动作、操作人与时间。
#[component]
pub fn LogTable(rows: Vec<AuditLogEntry>) -> impl IntoView {
    let total_rows = rows.len();
    view! {
        <div class="flex-1 overflow-auto">
            <table>
                <thead>
                    <tr>
                        <th>"实体类型"</th>
                        <th>"实体 ID"</th>
                        <th>"动作"</th>
                        <th>"操作人"</th>
                        <th>"备注"</th>
                        <th class="data-table-num">"时间"</th>
                    </tr>
                </thead>
                <tbody>
                    <For each=move || rows.clone() key=|r| r.id let:row>
                        <tr>
                            <td>{row.entity_type}</td>
                            <td>{row.entity_id.unwrap_or("—".to_string())}</td>
                            <td>
                                <span class={format!("text-13 {}", action_class(&row.action))}>
                                    {action_cn(&row.action)}
                                </span>
                            </td>
                            <td>{row.operator_name.unwrap_or("—".to_string())}</td>
                            <td>{row.comment.unwrap_or("—".to_string())}</td>
                            <td class="data-table-num">{row.created_at}</td>
                        </tr>
                    </For>
                </tbody>
            </table>
            {move || {
                if total_rows == 0 {
                    view! {
                        <div class="text-center py-40 text-tertiary">"暂无审计日志"</div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
        </div>
    }
}

fn action_cn(action: &str) -> String {
    match action {
        "import_raw_file" => "导入原始文件",
        "reconcile" => "自动对账",
        "approve_review" => "审核通过",
        "reject_review" => "审核驳回",
        "generate_voucher" => "生成凭证",
        _ => action,
    }
    .to_string()
}

fn action_class(action: &str) -> &'static str {
    match action {
        "import_raw_file" => "text-info",
        "reconcile" => "text-brand",
        "approve_review" | "generate_voucher" => "text-success",
        "reject_review" => "text-danger",
        _ => "text-tertiary",
    }
}
