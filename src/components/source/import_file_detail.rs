use leptos::prelude::*;

use crate::ipc::{self, ImportBatch};

#[component]
pub fn ImportFileDetail(
    batches: Signal<Vec<ImportBatch>>,
    selected_id: ReadSignal<Option<i64>>,
) -> impl IntoView {
    let (content, set_content) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let detail_signal = Signal::derive(move || {
        let id = selected_id.get()?;
        batches.get().into_iter().find(|b| b.id == id)
    });

    let load_content = move |batch: ImportBatch| {
        if batch.file_path.is_empty() {
            set_error.set(Some("文件路径为空".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_content.set(None);
        let path = batch.file_path.clone();
        leptos::task::spawn_local(async move {
            match ipc::read_source_file(path).await {
                Ok(c) => set_content.set(Some(c)),
                Err(e) => set_error.set(Some(format!("读取失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    Effect::watch(
        move || selected_id.get(),
        move |id, _, _| {
            if let Some(batch_id) = id {
                if let Some(batch) = batches.get().into_iter().find(|b| b.id == *batch_id) {
                    load_content(batch);
                }
            }
        },
        true,
    );

    let source_type_label = |source_type: &str| -> String {
        match source_type {
            "bank_flow" => "银行流水",
            "order_flow" => "订单流水",
            "pos_flow" => "POS流水",
            "summary_flow" => "数据汇总",
            _ => source_type,
        }
        .to_string()
    };

    view! {
        <div class="card flex flex-col min-h-0">
            <div class="card-header">
                <span class="card-title">"文件详细信息"</span>
            </div>
            <div class="flex-1 overflow-auto p-3">
                <Suspense fallback=|| view! { <div class="text-tertiary p-4">"请选择一个文件"</div> }>
                    {move || {
                        match detail_signal.get() {
                            Some(b) => {
                                let fn_val = b.file_name.clone();
                                let st_val = source_type_label(&b.source_type);
                                let rc_val = b.row_count;
                                let ia_val = b.imported_at.clone();
                                let cb_val = b.created_by.clone().unwrap_or_default();
                                view! {
                                    <>
                                    <div class="detail-grid">
                                        <DetailField label="文件名" value=fn_val />
                                        <DetailField label="文件类型" value=st_val />
                                        <DetailField label="完整路径" value=b.file_path.clone() />
                                        <DetailField label="数据行数" value=rc_val.to_string() />
                                        <DetailField label="导入时间" value=ia_val />
                                        <DetailField label="操作人" value=cb_val />
                                    </div>
                                    <div class="border-t border-border pt-3">
                                        <div class="flex items-center justify-between mb-2">
                                            <span class="text-13 text-secondary">"文件内容预览"</span>
                                            <Show when=move || loading.get()>
                                                <span class="text-13 text-tertiary">"加载中..."</span>
                                            </Show>
                                        </div>
                                        <Show when=move || error.get().is_some()>
                                            <div class="login-error mb-2 text-13">
                                                {move || error.get().unwrap_or_default()}
                                            </div>
                                        </Show>
                                        <Show when=move || content.get().is_some()>
                                            <pre class="bg-surface p-3 rounded text-12 overflow-auto" style="max-height: calc(100vh - 480px);">
                                                {move || content.get().unwrap_or_default()}
                                            </pre>
                                        </Show>
                                    </div>
                                    </>
                                }.into_any()
                            }
                            None => view! {
                                <div class="empty-state">
                                    <p class="empty-state-desc">"请选择一个文件查看详情"</p>
                                </div>
                            }.into_any(),
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn DetailField(label: &'static str, value: String) -> impl IntoView {
    let vc = value.clone();
    view! {
        <div class="detail-field">
            <span class="detail-field-label">{label}</span>
            <span class="detail-field-value truncate" title={vc}>{value}</span>
        </div>
    }
}
