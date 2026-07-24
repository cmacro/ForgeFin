use leptos::prelude::*;
use lucide_leptos::{FileUp, Upload};

use crate::ipc::{self, ImportResult};

/// 文件导入上传组件。
///
/// 支持选择文件或输入目录,提供导入进度与结果反馈。
#[component]
pub fn ImportUploader(
    #[prop(default = Callback::new(|_| {}))] on_imported: Callback<ImportResult>,
) -> impl IntoView {
    let (path, set_path) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let import = move |_| {
        let p = path.get();
        if p.trim().is_empty() {
            set_error.set(Some("请选择或输入文件路径".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match ipc::import_raw_file(p, None, None).await {
                Ok(result) => {
                    on_imported.run(result.clone());
                }
                Err(e) => set_error.set(Some(format!("导入失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="card p-4 mb-4">
            <div class="flex items-end gap-3">
                <div class="form-field flex-1">
                    <label class="form-label">"原始文件路径"</label>
                    <div class="flex items-center gap-2">
                        <input
                            type="text"
                            class="form-input"
                            placeholder="例如 /home/user/ForgeFin/tests/sample_data/health_company/bank_raw.tsv"
                            prop:value=path
                            on:input=move |ev| set_path.set(event_target_value(&ev))
                        />
                        <button
                            class="btn btn-outline"
                            on:click=move |_| set_path.set(String::new())
                        >
                            "清空"
                        </button>
                    </div>
                </div>
                <button
                    class="btn btn-primary"
                    type="button"
                    disabled=loading
                    on:click=move |_| import(())
                >
                    <Upload size=14 />
                    {move || if loading.get() { "导入中…" } else { "导入文件" }}
                </button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="login-error mt-3">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="mt-4 p-3 bg-surface rounded border border-border">
                <div class="flex items-center gap-2 text-13 text-secondary">
                    <FileUp size=14 />
                    <span>"支持 .tsv / .csv / .xlsx,文件名包含 bank/order/pos/summary 等关键字以识别来源类型。"</span>
                </div>
            </div>
        </div>
    }
}
