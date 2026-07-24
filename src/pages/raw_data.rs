use leptos::prelude::*;
use lucide_leptos::{FileText, RefreshCw};

use crate::components::charts::kpi_card::{KpiAccent, KpiCard};
use crate::components::form::search_form::{FieldKind, SearchField, SearchForm, SelectOption};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::source::import_uploader::ImportUploader;
use crate::components::source::raw_record_table::RawRecordTable;
use crate::components::source::record_detail::RecordDetail;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, ImportResult, RawRecordFilter};

/// 原始数据页。
///
/// 双 Tab: 导入中心 / 原始记录库。
#[component]
pub fn RawData() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("import");

    let (path, set_path) = signal(String::from(""));
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
    let (page, set_page) = signal(1i32);
    let (filter, set_filter) = signal(RawRecordFilter {
        source_type: None,
        batch_id: None,
        page: 1,
        page_size: 20,
    });

    let records = LocalResource::new(move || {
        let mut f = filter.get();
        f.page = page.get();
        async move { ipc::list_raw_records(&f).await }
    });

    let (selected_id, set_selected_id) = signal(Option::<i64>::None);
    let detail = LocalResource::new(move || {
        let id = selected_id.get();
        async move {
            match id {
                Some(id) => ipc::get_raw_record(id).await.unwrap_or(None),
                None => None,
            }
        }
    });

    let (error, _set_error) = signal(Option::<String>::None);

    let on_search = move |_| {
        set_page.set(1);
        records.refetch();
    };
    let on_reset = move |_| {
        set_filter.set(RawRecordFilter {
            source_type: None,
            batch_id: None,
            page: 1,
            page_size: 20,
        });
        set_page.set(1);
        records.refetch();
    };

    let on_imported = move |_: ImportResult| {
        records.refetch();
    };

    view! {
        <div class="page-content">
            <div class="flex items-center justify-between mb-4">
                <h1 class="text-lg font-semibold text-primary">"原始记录库"</h1>
            </div>

            <SearchForm
                fields=record_search_fields()
                on_search=Callback::new(on_search)
                on_reset=Callback::new(on_reset)
            />

            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
                <KpiCard label="总记录数" value="—".to_string() unit=None accent=KpiAccent::Brand />
                <KpiCard label="待处理" value="—".to_string() unit=None accent=KpiAccent::Warning />
                <KpiCard label="已匹配" value="—".to_string() unit=None accent=KpiAccent::Success />
                <KpiCard label="已审核" value="—".to_string() unit=None accent=KpiAccent::Info />
            </div>

            <div class="card p-3 mb-4">
                <ImportUploader on_imported=Callback::new(on_imported) />
            </div>

            <Show when=move || error.get().is_some()>
                <div class="login-error mb-3">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="page-grid">
                <div class="data-table flex flex-col min-h-0">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || Suspend::new(async move {
                            match records.await {
                                Ok(p) => view! {
                                    <>
                                    <RawRecordTable
                                        rows=p.items.clone()
                                        selected_id=selected_id
                                        set_selected_id=set_selected_id
                                    />
                                    <div class="border-t border-border-light">
                                        <Pagination
                                            total=p.total
                                            current=p.page
                                            page_size=p.page_size
                                        />
                                    </div>
                                    </>
                                }.into_any(),
                                Err(e) => view! {
                                    <div class="login-error">{format!("加载记录失败: {e}")}</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
                <RecordDetail detail=detail />
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
