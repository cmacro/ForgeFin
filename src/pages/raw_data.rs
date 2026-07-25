use leptos::prelude::*;
use lucide_leptos::{FileText, FolderOpen, ListFilter, RefreshCw};

use crate::components::form::search_form::{FieldKind, SearchField, SelectOption};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::source::import_file_detail::ImportFileDetail;
use crate::components::source::import_file_list::ImportFileList;
use crate::ipc::{self, ImportBatch, ImportResult};

/// 原始数据页。
///
/// 双 Tab: 导入中心 / 原始记录库。
#[component]
pub fn RawData() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("import");

    let stored_dir = ipc::get_stored_import_dir().unwrap_or_default();
    let (path, set_path) = signal(stored_dir);
    let (files, set_files) = signal(Vec::<ipc::RawFileInfo>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (import_result, set_import_result) = signal(Option::<ipc::ImportDirResult>::None);

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

    let select_dir = move |_: ()| {
        leptos::task::spawn_local(async move {
            set_loading.set(true);
            match ipc::select_raw_directory().await {
                Ok(p) => {
                    ipc::set_stored_import_dir(&p);
                    set_path.set(p);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("选择目录失败: {e}")));
                    set_loading.set(false);
                }
            }
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
                    if let Ok(list) = ipc::scan_raw_directory(p).await {
                        set_files.set(list);
                    }
                }
                Err(e) => set_error.set(Some(format!("自动导入失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    let tabs = vec![
        TabItem {
            key: "import",
            label: "导入中心",
            closable: false,
        },
        TabItem {
            key: "records",
            label: "原始记录库",
            closable: false,
        },
    ];

    view! {
        <Tabs
            items=tabs
            active_key={move || active_tab.get()}
            on_change=Callback::new(move |key| set_active_tab.set(key))
        />

        <Show when=move || active_tab.get() == "import">
            <ImportCenter
                path=path
                set_path=set_path
                files=files
                loading=loading
                error=error
                import_result=import_result
                scan=Callback::new(scan)
                auto_import=Callback::new(auto_import)
                select_dir=Callback::new(select_dir)
            />
        </Show>

        <Show when=move || active_tab.get() == "records">
            <RecordsLibrary />
        </Show>
    }
}

#[component]
fn ImportCenter(
    path: ReadSignal<String>,
    set_path: WriteSignal<String>,
    files: ReadSignal<Vec<ipc::RawFileInfo>>,
    loading: ReadSignal<bool>,
    error: ReadSignal<Option<String>>,
    import_result: ReadSignal<Option<ipc::ImportDirResult>>,
    scan: Callback<()>,
    auto_import: Callback<()>,
    select_dir: Callback<()>,
) -> impl IntoView {
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
                                placeholder="例如 /home/user/ForgeFin/tests/sample_data/health_company"
                                prop:value=path
                                on:input=move |ev| set_path.set(event_target_value(&ev))
                            />
                            <button
                                class="btn btn-outline"
                                on:click=move |_| select_dir.run(())
                            >
                                <FolderOpen size=14 />
                                "选择"
                            </button>
                            <button
                                class="btn btn-outline"
                                on:click=move |_| set_path.set(String::new())
                            >
                                "清空"
                            </button>
                        </div>
                    </div>
                    <button
                        class="btn btn-outline"
                        type="button"
                        disabled=loading
                        on:click=move |_| scan.run(())
                    >
                        <RefreshCw size=14 />
                        {move || if loading.get() { "扫描中…" } else { "扫描" }}
                    </button>
                    <button
                        class="btn btn-primary"
                        type="button"
                        disabled=move || loading.get() || files.get().is_empty()
                        on:click=move |_| auto_import.run(())
                    >
                        <FileText size=14 />
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

            <ImportSingleFile />

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
fn ImportSingleFile() -> impl IntoView {
    let (path, set_path) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (result, set_result) = signal(Option::<ImportResult>::None);

    let import = move |_| {
        let p = path.get();
        if p.trim().is_empty() {
            set_error.set(Some("请输入文件路径".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match ipc::import_raw_file(p, None, None).await {
                Ok(r) => set_result.set(Some(r)),
                Err(e) => set_error.set(Some(format!("导入失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="card p-4 mb-4">
            <div class="text-13 text-secondary mb-2">"单文件导入"</div>
            <div class="flex items-end gap-3">
                <input
                    type="text"
                    class="form-input flex-1"
                    placeholder="例如 tests/sample_data/health_company/bank_raw.tsv"
                    prop:value=path
                    on:input=move |ev| set_path.set(event_target_value(&ev))
                />
                <button
                    class="btn btn-primary"
                    disabled=loading
                    on:click=move |_| import(())
                >
                    {move || if loading.get() { "导入中…" } else { "导入" }}
                </button>
            </div>
            <Show when=move || error.get().is_some()>
                <div class="login-error mt-2">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || result.get().is_some()>
                {move || {
                    result.get().map(|r| view! {
                        <div class="mt-2 text-13 text-success">
                            {format!("导入成功: {} ({} 行, batch_id={})", r.file_name, r.row_count, r.batch_id)}
                        </div>
                    })
                }}
            </Show>
        </div>
    }
}

#[component]
fn RecordsLibrary() -> impl IntoView {
    let (source_type_filter, set_source_type_filter) = signal(Option::<String>::None);
    let (selected_batch_id, set_selected_batch_id) = signal(Option::<i64>::None);
    let (batches, set_batches) = signal(Vec::<ImportBatch>::new());
    let (loading_batches, set_loading_batches) = signal(false);
    let (search_text, set_search_text) = signal(String::new());

    let load_batches = move || {
        set_loading_batches.set(true);
        leptos::task::spawn_local(async move {
            match ipc::list_import_batches(source_type_filter.get(), Some(3)).await {
                Ok(list) => set_batches.set(list),
                Err(_e) => {}
            }
            set_loading_batches.set(false);
        });
    };

    let filtered_batches = Signal::derive(move || {
        let query = search_text.get().to_lowercase();
        if query.is_empty() {
            batches.get()
        } else {
            batches
                .get()
                .into_iter()
                .filter(|b| b.file_name.to_lowercase().contains(&query))
                .collect()
        }
    });

    let on_batch_select = move |batch_id: i64| {
        set_selected_batch_id.set(Some(batch_id));
    };

    load_batches();

    view! {
        <div class="page-content flex flex-col flex-1 min-h-0">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"原始记录库"</h1>
            </div>

            <div class="card p-4 mb-4">
                <div class="flex items-end gap-3">
                    <div class="form-field flex-1">
                        <label class="form-label">"文件名搜索"</label>
                        <input
                            type="text"
                            class="form-input"
                            placeholder="输入文件名过滤..."
                            prop:value=search_text
                            on:input=move |ev| set_search_text.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"来源类型"</label>
                        <select
                            class="form-input"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_source_type_filter.set(if val.is_empty() { None } else { Some(val) });
                            }
                        >
                            <option value="">"全部"</option>
                            <option value="bank_flow">"银行流水"</option>
                            <option value="order_flow">"订单流水"</option>
                            <option value="pos_flow">"POS流水"</option>
                            <option value="summary_flow">"数据汇总"</option>
                        </select>
                    </div>
                    <button
                        class="btn btn-primary flex items-center gap-1"
                        on:click=move |_| load_batches()
                        disabled=loading_batches
                    >
                        <ListFilter size=14 />
                        {move || if loading_batches.get() { "加载中…" } else { "过滤" }}
                    </button>
                </div>
            </div>

            <div class="flex-1 min-h-0 grid grid-cols-2 gap-4" style="height: calc(100vh - 280px);">
                <ImportFileList
                    batches=filtered_batches
                    selected_id=selected_batch_id
                    on_select=Callback::new(on_batch_select)
                />
                <ImportFileDetail
                    batches=batches.into()
                    selected_id=selected_batch_id
                />
            </div>
        </div>
    }
}

fn record_search_fields() -> Vec<SearchField> {
    vec![
        SearchField {
            key: "source_type",
            label: "来源类型",
            kind: FieldKind::Select {
                options: vec![
                    SelectOption {
                        value: "bank_flow",
                        label: "银行流水",
                    },
                    SelectOption {
                        value: "order_flow",
                        label: "订单流水",
                    },
                    SelectOption {
                        value: "pos_flow",
                        label: "POS流水",
                    },
                    SelectOption {
                        value: "summary_flow",
                        label: "数据汇总",
                    },
                ],
                placeholder: Some("全部"),
            },
            width: None,
        },
        SearchField {
            key: "batch_id",
            label: "批次号",
            kind: FieldKind::Text {
                placeholder: Some("批次 ID"),
            },
            width: None,
        },
    ]
}

#[component]
fn FileTable(files: Vec<ipc::RawFileInfo>) -> impl IntoView {
    let (importing, set_importing) = signal(Option::<String>::None);
    let (import_error, set_import_error) = signal(Option::<String>::None);
    let (files_signal, set_files) = signal(files.clone());

    let import_one = move |file: ipc::RawFileInfo| {
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
                            <span class={format!("text-13 {}", file_status_class(&file.status))}>
                                {file_status_label(&file.status)}
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

fn file_status_label(status: &str) -> &'static str {
    match status {
        "imported" => "已导入",
        "pending" => "未导入",
        "unsupported" => "不支持的文件类型",
        _ => "未知",
    }
}

fn file_status_class(status: &str) -> &'static str {
    match status {
        "imported" => "text-success",
        "pending" => "text-warning",
        "unsupported" => "text-danger",
        _ => "text-tertiary",
    }
}
