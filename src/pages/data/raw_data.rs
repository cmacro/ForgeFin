use leptos::prelude::*;
use lucide_leptos::{FileText, FolderOpen, ListFilter, RefreshCw, X};

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
    let (selected_file, set_selected_file) = signal(Option::<ipc::RawFileInfo>::None);

    let on_file_select = move |file: ipc::RawFileInfo| {
        set_selected_file.set(Some(file));
    };

    let on_close_detail = move |_| {
        set_selected_file.set(None);
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
                        let errors_for_list = r.errors.clone();
                        let warnings: Vec<(String, String)> = r.imported.iter()
                            .filter_map(|item| item.balance_check_warning.as_ref().map(|w| (item.file_name.clone(), w.clone())))
                            .collect();
                        let has_errors = !errors_for_list.is_empty();
                        let has_warnings = !warnings.is_empty();
                        view! {
                            <div class="mt-3 p-3 bg-surface rounded border border-border text-13">
                                <div class="flex gap-4">
                                    <span class="text-success">{format!("导入成功 {}", r.imported.len())}</span>
                                    <span class="text-tertiary">{format!("已跳过 {}", r.skipped.len())}</span>
                                    <span class="text-danger">{format!("失败 {}", error_count)}</span>
                                </div>
                                <Show when=move || has_errors>
                                    <ul class="mt-2 text-danger space-y-1">
                                        {errors_for_list.iter().map(|err| view! { <li>{err.clone()}</li> }).collect::<Vec<_>>()}
                                    </ul>
                                </Show>
                                <Show when=move || has_warnings>
                                    <div class="mt-2 text-warning">
                                        <div class="font-medium">"余额校验警告"</div>
                                        <ul class="space-y-1">
                                            {warnings.iter().map(|(fname, msg)| view! {
                                                <li>{format!("{fname}: {msg}")}</li>
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    </div>
                                </Show>
                            </div>
                        }
                    }}
                </Show>
            </div>

            <ImportSingleFile />

            <div class="flex-1 min-h-0 grid grid-cols-2 gap-4" style="height: calc(100vh - 280px);">
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
                                    view! {
                                        <FileTable
                                            files=list
                                            selected_file=selected_file
                                            on_select=Callback::new(on_file_select)
                                        />
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </div>
                </div>

                <ScannedFileDetail
                    file=selected_file
                    on_close=Callback::new(on_close_detail)
                />
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
                    result.get().map(|r| {
                        let warn = r.balance_check_warning.clone();
                        let skipped = r.skipped_count;
                        view! {
                            <div class="mt-2 text-13">
                                <div class="text-success">
                                    {format!("导入成功: {} ({} 行, batch_id={})", r.file_name, r.row_count, r.batch_id)}
                                </div>
                                {(skipped > 0i32).then(|| view! {
                                    <div class="text-warning mt-1">{format!("跳过 {} 条重复行", skipped)}</div>
                                })}
                                {warn.map(|w| view! {
                                    <div class="text-warning mt-1">{format!("余额校验警告: {w}")}</div>
                                })}
                            </div>
                        }
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
fn FileTable(
    files: Vec<ipc::RawFileInfo>,
    selected_file: ReadSignal<Option<ipc::RawFileInfo>>,
    on_select: Callback<ipc::RawFileInfo>,
) -> impl IntoView {
    let (importing, set_importing) = signal(Option::<String>::None);
    let (import_error, set_import_error) = signal(Option::<String>::None);
    let (files_signal, set_files) = signal(files.clone());

    let import_one = Callback::new(move |file: ipc::RawFileInfo| {
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
    });

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
                    <FileRow
                        file=file
                        selected_file=selected_file
                        importing=importing
                        on_select=on_select
                        import_one=import_one
                    />
                </For>
            </tbody>
        </table>
    }
}

#[component]
fn FileRow(
    file: ipc::RawFileInfo,
    selected_file: ReadSignal<Option<ipc::RawFileInfo>>,
    importing: ReadSignal<Option<String>>,
    on_select: Callback<ipc::RawFileInfo>,
    import_one: Callback<ipc::RawFileInfo>,
) -> impl IntoView {
    let file_path = file.file_path.clone();
    let file_name = file.file_name.clone();
    let source_type = file.source_type.clone();
    let row_count = file.row_count;
    let status = file.status.clone();
    let is_selected = move || {
        selected_file.get().as_ref().map(|f| f.file_path.as_str()) == Some(file_path.as_str())
    };

    let file_for_click = file.clone();
    let file_name_for_busy = file_name.clone();
    let file_for_import_click = file.clone();

    view! {
        <tr
            class=move || {
                if is_selected() {
                    "cursor-pointer hover:bg-surface transition-colors bg-surface border-l-2 border-l-brand"
                } else {
                    "cursor-pointer hover:bg-surface transition-colors"
                }
            }
            on:click=move |_| on_select.run(file_for_click.clone())
        >
            <td>{file_name.clone()}</td>
            <td>{source_type.clone()}</td>
            <td class="data-table-num">{row_count}</td>
            <td>
                <span class={format!("text-13 {}", file_status_class(&status))}>
                    {file_status_label(&status)}
                </span>
            </td>
            <td class="text-center">
                {let status2 = status.clone();
                let f = file_for_import_click.clone();
                let n = file_name_for_busy.clone();
                move || {
                    if status2 == "pending" {
                        let busy = importing.get().as_ref() == Some(&n);
                        let f2 = f.clone();
                        let f3 = f2.clone();
                        let f4 = f3.clone();
                        view! {
                            <button
                                class="btn btn-sm btn-primary"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    import_one.run(f4.clone());
                                }
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

#[component]
fn ScannedFileDetail(
    file: ReadSignal<Option<ipc::RawFileInfo>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (content, set_content) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let load_content = move |f: ipc::RawFileInfo| {
        if f.file_path.is_empty() {
            set_error.set(Some("文件路径为空".to_string()));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_content.set(None);
        let path = f.file_path.clone();
        leptos::task::spawn_local(async move {
            match ipc::read_source_file(path).await {
                Ok(c) => set_content.set(Some(c)),
                Err(e) => set_error.set(Some(format!("读取失败: {e}"))),
            }
            set_loading.set(false);
        });
    };

    Effect::watch(
        move || file.get(),
        move |f, _, _| {
            if let Some(f) = f {
                load_content(f.clone());
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
                <span class="card-title">"文件详情"</span>
                <button class="btn btn-sm btn-outline" on:click=move |_| on_close.run(())>
                    <X size=14 />
                </button>
            </div>
            <div class="flex-1 overflow-auto p-3">
                <Suspense fallback=|| view! { <div class="text-tertiary p-4">"请选择一个文件"</div> }>
                    {move || {
                        match file.get() {
                            Some(f) => {
                                let fn_val = f.file_name.clone();
                                let st_val = source_type_label(&f.source_type);
                                let rc_val = f.row_count;
                                let status_label = file_status_label(&f.status);
                                view! {
                                    <>
                                    <div class="detail-grid">
                                        <DetailField label="文件名" value=fn_val />
                                        <DetailField label="来源类型" value=st_val />
                                        <DetailField label="完整路径" value=f.file_path.clone() />
                                        <DetailField label="数据行数" value=rc_val.to_string() />
                                        <DetailField label="状态" value=status_label.to_string() />
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
                                    <p class="empty-state-desc">"请点击左侧文件查看详情"</p>
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
