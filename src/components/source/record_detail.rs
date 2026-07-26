use leptos::prelude::*;
use lucide_leptos::{FileCode, FileText, Paperclip};

use crate::ipc::{self, AttachmentInfo, AuditLogEntry, RawRecordDetail};

/// 原始记录详情面板。
///
/// 展示 raw_data JSON、来源文件、行号、附件列表与审计日志。
#[component]
pub fn RecordDetail(detail: LocalResource<Option<RawRecordDetail>>) -> impl IntoView {
    let (file_content, set_file_content) = signal(Option::<String>::None);
    let (loading_content, set_loading_content) = signal(false);
    let (content_error, set_content_error) = signal(Option::<String>::None);

    let load_file_content = move |file_path: String| {
        if file_path.is_empty() {
            set_content_error.set(Some("文件路径为空".to_string()));
            return;
        }
        set_loading_content.set(true);
        set_content_error.set(None);
        set_file_content.set(None);
        leptos::task::spawn_local(async move {
            match ipc::read_source_file(file_path).await {
                Ok(content) => set_file_content.set(Some(content)),
                Err(e) => set_content_error.set(Some(format!("读取文件失败: {e}"))),
            }
            set_loading_content.set(false);
        });
    };

    view! {
        <div class="card flex flex-col min-h-0">
            <div class="card-header">
                <span class="card-title">"原始记录详情"</span>
            </div>
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"请选择一条记录"</div> }>
                {let detail = detail.clone();
                 move || {
                    let detail = detail.clone();
                    match detail.map(|d| d.clone()) {
                        Some(Some(d)) => {
                            let r = d.record.clone();
                            let fp = r.file_path.clone();
                            view! {
                                <>
                                <div class="detail-grid">
                                    <DetailField label="ID" value={r.id.to_string()} />
                                    <DetailField label="来源类型" value=r.source_type_name />
                                    <DetailField label="来源文件" value=r.source_file_name />
                                    <DetailField label="完整路径" value=r.file_path.clone() />
                                    <DetailField label="行号" value=r.source_row_no.to_string() />
                                    <DetailField label="业务单号" value=r.record_no.unwrap_or("—".to_string()) />
                                    <DetailField label="日期" value=r.record_date.unwrap_or("—".to_string()) />
                                    <DetailField label="金额" value=r.amount_total.unwrap_or("—".to_string()) />
                                    <DetailField label="币种" value=r.currency />
                                    <DetailField label="状态" value=status_cn(&r.status) />
                                </div>
                                <div class="border-t border-border p-3">
                                    <button
                                        class="btn btn-sm btn-outline flex items-center gap-1 mb-2"
                                        on:click=move |_| load_file_content(fp.clone())
                                        disabled=loading_content
                                    >
                                        <FileCode size=14 />
                                        {move || if loading_content.get() { "加载中…" } else { "查看源文件内容" }}
                                    </button>
                                    <Show when=move || content_error.get().is_some()>
                                        <div class="login-error mt-2 text-13">{move || content_error.get().unwrap_or_default()}</div>
                                    </Show>
                                    <Show when=move || file_content.get().is_some()>
                                        <div class="mt-2">
                                            <div class="text-13 text-secondary mb-1">"源文件内容"</div>
                                            <pre class="detail-code-block bg-surface p-2 rounded text-12">
                                                {move || file_content.get().unwrap_or_default()}
                                            </pre>
                                        </div>
                                    </Show>
                                </div>
                                <div class="border-t border-border p-3">
                                    <div class="text-13 text-secondary mb-2">"原始数据 (JSON)"</div>
                                    <pre class="detail-code-block-sm bg-surface p-2 rounded text-12">
                                        {format_json(&d.raw_data)}
                                    </pre>
                                </div>
                                {attachments_view(d.attachments.clone())}
                                {audit_logs_view(d.audit_logs.clone())}
                                </>
                            }.into_any()
                        }
                        _ => view! {
                            <div class="empty-state">
                                <p class="empty-state-desc">"请从左侧选择一条原始记录查看详情。"</p>
                            </div>
                        }.into_any(),
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn DetailField(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="detail-field">
            <span class="detail-field-label">{label}</span>
            <span class="detail-field-value">{value}</span>
        </div>
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

fn attachments_view(attachments: Vec<AttachmentInfo>) -> impl IntoView {
    if attachments.is_empty() {
        return view! { <></> }.into_any();
    }
    view! {
        <div class="border-t border-border p-3">
            <div class="text-13 text-secondary mb-2 flex items-center gap-1">
                <Paperclip size=14 />
                "附件"
            </div>
            <ul class="space-y-1 text-13">
                {attachments.into_iter().map(|a| {
                    view! {
                        <li class="flex items-center gap-1">
                            <FileText size=12 />
                            <span>{a.file_name}</span>
                            <span class="text-tertiary">
                                {format!("({} bytes)", a.file_size)}
                            </span>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
    .into_any()
}

fn audit_logs_view(logs: Vec<AuditLogEntry>) -> impl IntoView {
    if logs.is_empty() {
        return view! { <></> }.into_any();
    }
    view! {
        <div class="border-t border-border p-3">
            <div class="text-13 text-secondary mb-2">"审计日志"</div>
            <ul class="space-y-2 text-13">
                {logs.into_iter().map(|log| {
                    view! {
                        <li class="log-entry">
                            <span class="log-entry-dot bg-brand"></span>
                            <div class="log-entry-title">{action_cn(&log.action)}</div>
                            <div class="log-entry-meta">
                                <span class="text-primary font-medium">
                                    {log.operator_name.unwrap_or("—".to_string())}
                                </span>
                                <span class="text-tertiary">{log.created_at}</span>
                            </div>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
    .into_any()
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

fn format_json(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| raw.to_string())
}
