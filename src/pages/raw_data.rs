use leptos::prelude::*;
use lucide_leptos::{Folder, RefreshCw, Upload};

use crate::ipc::{self, ImportDirResult, RawFileInfo};

/// 原始数据导入页。
///
/// 支持输入原始文件目录并扫描，检测未导入的原始文档后一键自动导入。
#[component]
pub fn RawData() -> impl IntoView {
    let (path, set_path) = signal(String::from(""));
    let (files, set_files) = signal(Vec::<RawFileInfo>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (import_result, set_import_result) = signal(Option::<ImportDirResult>::None);

    let scan = move |_| {
        let p = path.get();
        if p.trim().is_empty() {
            set_error.set(Some("请输入原始文件目录路径".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_import_result.set(None);
        leptos::task::spawn_local(async move {
            match ipc::scan_raw_directory(p).await {
                Ok(list) => {
                    let empty = list.is_empty();
                    set_files.set(list);
                    if empty {
                        set_error.set(Some("目录中未找到支持的原始凭证文件".to_string()));
                    }
                }
                Err(e) => set_error.set(Some(format!("扫描失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    let auto_import = move |_| {
        let p = path.get();
        if p.trim().is_empty() {
            set_error.set(Some("请输入原始文件目录路径".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_import_result.set(None);
        leptos::task::spawn_local(async move {
            match ipc::auto_import_raw_directory(p.clone()).await {
                Ok(result) => {
                    set_import_result.set(Some(result.clone()));
                    // 刷新扫描结果
                    if let Ok(list) = ipc::scan_raw_directory(p).await {
                        set_files.set(list);
                    }
                }
                Err(e) => set_error.set(Some(format!("自动导入失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="page-content">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"原始数据导入"</h1>
            </div>

            <div class="card p-4 mb-4">
                <div class="flex items-end gap-3">
                    <div class="form-field flex-1">
                        <label class="form-label">"原始文件目录"</label>
                        <div class="flex items-center gap-2">
                            <input
                                type="text"
                                class="form-input"
                                placeholder="例如 /home/user/ForgeFin/docs/raw"
                                prop:value=path
                                on:input=move |ev| set_path.set(event_target_value(&ev))
                            />
                            <button class="btn btn-outline" on:click=move |_| set_path.set(String::new())>
                                "清空"
                            </button>
                        </div>
                    </div>
                    <button
                        class="btn btn-outline"
                        type="button"
                        disabled=loading
                        on:click=move |_| scan(())
                    >
                        <RefreshCw size=14 />
                        {move || if loading.get() { "扫描中…" } else { "扫描" }}
                    </button>
                    <button
                        class="btn btn-primary"
                        type="button"
                        disabled=move || loading.get() || files.get().is_empty()
                        on:click=move |_| auto_import(())
                    >
                        <Upload size=14 />
                        "一键导入未导入"
                    </button>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="login-error mt-3">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <Show when=move || import_result.get().is_some()>
                    {move || {
                        let r = import_result.get().unwrap_or_default();
                        let error_count = r.errors.len();
                        let errors_for_show = r.errors.clone();
                        let errors_for_list = r.errors.clone();
                        view! {
                            <div class="mt-3 p-3 bg-surface rounded border border-border text-13">
                                <div class="flex gap-4">
                                    <span class="text-success">{format!("导入成功 {}", r.imported.len())}</span>
                                    <span class="text-tertiary">{format!("已跳过 {}", r.skipped.len())}</span>
                                    <span class="text-danger">{format!("失败 {}", error_count)}</span>
                                </div>
                                <Show when=move || !errors_for_show.is_empty()>
                                    <ul class="mt-2 text-danger space-y-1">
                                        {errors_for_list.iter().map(|err| view! { <li>{err.clone()}</li> }).collect::<Vec<_>>()}
                                    </ul>
                                </Show>
                            </div>
                        }
                    }}
                </Show>
            </div>

            <div class="card flex flex-col min-h-0">
                <div class="card-header">
                    <span class="card-title">"扫描结果"</span>
                    <span class="text-13 text-tertiary">
                        {move || format!("共 {} 个文件", files.get().len())}
                    </span>
                </div>
                <div class="flex-1 overflow-auto p-3">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || {
                            let list = files.get();
                            if list.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <Folder size=48 />
                                        <p class="empty-state-desc">"请输入目录并扫描以检测原始凭证文件。"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <FileTable files=list /> }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FileTable(files: Vec<RawFileInfo>) -> impl IntoView {
    let (importing, set_importing) = signal(Option::<String>::None);
    let (import_error, set_import_error) = signal(Option::<String>::None);
    let (files_signal, set_files) = signal(files.clone());

    let import_one = move |file: RawFileInfo| {
        let path = file.file_path.clone();
        let file_name = file.file_name.clone();
        set_importing.set(Some(file_name.clone()));
        set_import_error.set(None);
        leptos::task::spawn_local(async move {
            match ipc::import_raw_file(path, None, None).await {
                Ok(_) => {
                    set_files.update(|list| {
                        for f in list.iter_mut() {
                            if f.file_name == file_name {
                                f.status = "imported".to_string();
                            }
                        }
                    });
                }
                Err(e) => set_import_error.set(Some(format!("{file_name}: {e}"))),
            }
            set_importing.set(None);
        });
    };

    view! {
        <Show when=move || import_error.get().is_some()>
            <div class="login-error mb-3">{move || import_error.get().unwrap_or_default()}</div>
        </Show>
        <table>
            <thead>
                <tr>
                    <th>"文件名"</th>
                    <th>"来源类型"</th>
                    <th>"数据行数"</th>
                    <th>"状态"</th>
                    <th class="text-center">"操作"</th>
                </tr>
            </thead>
            <tbody>
                <For each=move || files_signal.get() key=|f| f.file_path.clone() let:file>
                    <tr>
                        <td>{file.file_name.clone()}</td>
                        <td>{file.source_type.clone()}</td>
                        <td class="data-table-num">{file.row_count}</td>
                        <td>
                            <span class={format!("text-13 {}", status_class(&file.status))}>
                                {status_label(&file.status)}
                            </span>
                        </td>
                        <td class="text-center">
                            {move || {
                                let f = file.clone();
                                if f.status == "pending" {
                                    let busy = importing.get().as_ref() == Some(&f.file_name);
                                    view! {
                                        <button
                                            class="btn btn-sm btn-primary"
                                            on:click=move |_| import_one(f.clone())
                                            disabled=busy
                                        >
                                            "导入"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span class="text-tertiary text-13">"—"</span> }.into_any()
                                }
                            }}
                        </td>
                    </tr>
                </For>
            </tbody>
        </table>
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "imported" => "已导入",
        "pending" => "未导入",
        "unsupported" => "不支持的文件类型",
        _ => "未知",
    }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "imported" => "text-success",
        "pending" => "text-warning",
        "unsupported" => "text-danger",
        _ => "text-tertiary",
    }
}
