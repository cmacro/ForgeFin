use leptos::prelude::*;
use lucide_leptos::{Plus, RotateCcw, ShieldPlus, Trash2};

use crate::auth::Session;
use crate::components::layout::modal::Modal;
use crate::ipc::{self, BackupEntry, Company, CompanyInput};

/// 系统设置页: 公司管理 + 备份恢复 + 用户管理(入口)。
///
/// 分三个区块卡片。每个公司一个操作行,支持新建/编辑/删除/切换/备份。
#[component]
pub fn Settings() -> impl IntoView {
    let companies = Resource::new(|| (), move |_| async { ipc::list_companies().await });
    let backups = Resource::new(|| (), move |_| async { ipc::list_backups().await });

    let (edit_open, set_edit_open) = signal(false);
    let (editing, set_editing) = signal(Option::<Company>::None);
    let (error, set_error) = signal(Option::<String>::None);
    let (info, set_info) = signal(Option::<String>::None);

    let refresh = move || {
        companies.refetch();
        backups.refetch();
    };

    let open_new = move || {
        set_editing.set(None);
        set_error.set(None);
        set_edit_open.set(true);
    };

    let open_edit = move |c: Company| {
        set_editing.set(Some(c));
        set_error.set(None);
        set_edit_open.set(true);
    };

    let on_switch = move |id: String| {
        leptos::task::spawn_local(async move {
            match Session::switch_company(id).await {
                Ok(_) => set_info.set(Some("已切换账套".to_string())),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_delete = move |c: Company| {
        leptos::task::spawn_local(async move {
            match ipc::delete_company(c.id).await {
                Ok(_) => {
                    set_info.set(Some(format!("已删除账套「{}」", c.name)));
                    refresh();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_backup = move |id: String| {
        leptos::task::spawn_local(async move {
            match ipc::backup_company(id).await {
                Ok(p) => {
                    set_info.set(Some(format!("已备份至: {p}")));
                    backups.refetch();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let on_backup_system = move || {
        leptos::task::spawn_local(async move {
            match ipc::backup_system().await {
                Ok(p) => {
                    set_info.set(Some(format!("已备份系统库至: {p}")));
                    backups.refetch();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let current_company = Session::company_id();

    view! {
        <div class="page-content">
            <Show when=move || error.get().is_some()>
                <div class="login-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || info.get().is_some()>
                <div class="tag tag-success">{move || info.get().unwrap_or_default()}</div>
            </Show>

            <div class="card">
                <div class="card-header">
                    <span class="card-title">"账套管理"</span>
                    <button class="btn btn-primary btn-sm" on:click=move |_| open_new()>
                        <Plus size=14 />
                        "新建账套"
                    </button>
                </div>
                <div class="card-body dense">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || Suspend::new(async move {
                             match companies.await {
                                 Ok(list) => view! {
                                     <>
                                         <table class="data-table" style="border:none">
                                             <thead>
                                                 <tr>
                                                     <th>"名称"</th>
                                                     <th>"税号"</th>
                                                     <th>"法人"</th>
                                                     <th>"币种"</th>
                                                     <th>"状态"</th>
                                                     <th class="text-center border-l border-border">"操作"</th>
                                                 </tr>
                                             </thead>
                                             <tbody>
                                                 <For each=move || list.clone() key=|c| c.id.clone() let:c>
                                                     <tr>
                                                         <td>
                                                             <Show when=move || current_company.get() == Some(c.id.clone())>
                                                                 <span class="tag tag-brand">"当前"</span>
                                                             </Show>
                                                             {c.name.clone()}
                                                         </td>
                                                         <td>{c.tax_id.clone().unwrap_or("—".to_string())}</td>
                                                         <td>{c.legal_person.clone().unwrap_or("—".to_string())}</td>
                                                         <td>{c.currency.clone()}</td>
                                                         <td>
                                                             <span class=format!("tag {}", if c.is_active { "tag-success" } else { "tag-draft" })>
                                                                 {if c.is_active { "启用" } else { "停用" }}
                                                             </span>
                                                         </td>
                                                         <td class="text-center border-l border-border">
                                                             <div class="flex items-center justify-center gap-4">
                                                                 <button
                                                                     class="text-xs text-brand"
                                                                     on:click=move |_| on_switch(c.id.clone())
                                                             >"切换"</button>
                                                             <button
                                                                 class="text-xs text-secondary"
                                                                 on:click=move |_| open_edit(c.clone())
                                                             >"编辑"</button>
                                                                 <button
                                                                     class="text-xs text-info"
                                                                     on:click=move |_| on_backup(c.id.clone())
                                                                 >"备份"</button>
                                                                 <button
                                                                     class="text-xs text-danger inline-flex"
                                                                     on:click=move |_| on_delete(c.clone())
                                                                 >
                                                                     <Trash2 size=12 />
                                                                 </button>
                                                             </div>
                                                         </td>
                                                     </tr>
                                                 </For>
                                             </tbody>
                                         </table>
                                         <Show when=move || list.is_empty()>
                                             <div class="empty-state">
                                                 <p class="empty-state-desc">"尚无账套,点击「新建账套」开始。"</p>
                                             </div>
                                         </Show>
                                     </>
                                 }.into_any(),
                                 Err(e) => view! { <div class="login-error">{format!("加载账套失败: {e}")}</div> }.into_any(),
                             }
                        })}
                    </Suspense>
                </div>
            </div>

            <div class="card mt-4">
                <div class="card-header">
                    <span class="card-title">"备份与恢复"</span>
                    <button class="btn btn-outline btn-sm" on:click=move |_| on_backup_system()>
                        <ShieldPlus size=14 />
                        "备份系统库"
                    </button>
                </div>
                <div class="card-body dense">
                    <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                        {move || Suspend::new(async move {
                            match backups.await {
                                Ok(list) => view! {
                                    <table class="data-table" style="border:none">
                                        <thead>
                                            <tr>
                                                <th>"备份文件"</th>
                                                <th class="data-table-num">"大小(KB)"</th>
                                                <th class="text-center">"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For each=move || list.clone() key=|b| b.name.clone() let:b>
                                                <tr>
                                                    <td>{b.name.clone()}</td>
                                                    <td class="data-table-num">{format!("{}", b.size / 1024)}</td>
                                                    <td class="text-center">
                                                        <RestoreButton backup=b on_restore_ok=move || refresh() />
                                                    </td>
                                                </tr>
                                            </For>
                                        </tbody>
                                    </table>
                                     <Show when=move || list.is_empty()>
                                         <div class="empty-state">
                                             <p class="empty-state-desc">"尚无备份文件。点击「备份系统库」或某账套的「备份」生成备份。"</p>
                                         </div>
                                     </Show>
                                 }.into_any(),
                                 Err(e) => view! { <div class="login-error">{format!("加载备份失败: {e}")}</div> }.into_any(),
                             }
                        })}
                    </Suspense>
                </div>
            </div>

            <div class="card mt-4">
                <div class="card-header">
                    <span class="card-title">"用户管理"</span>
                </div>
                <div class="card-body dense">
                    <p class="empty-state-desc">
                        "用户与权限的完整管理(创建用户、授予权限、停用)将在后续迭代完善。当前登录用户信息见顶栏右侧。"
                    </p>
                </div>
            </div>
        </div>
        <CompanyEditModal
            open=edit_open
            editing=editing
            set_open=set_edit_open
            on_saved=move || refresh()
        />
    }
}

#[component]
fn RestoreButton(backup: BackupEntry, on_restore_ok: impl Fn() + 'static) -> impl IntoView {
    let (open, set_open) = signal(false);
    let company_id = Session::company_id();
    let (target, set_target) = signal(String::new());
    let (err, set_err) = signal(Option::<String>::None);
    let (busy, set_busy) = signal(false);
    let backup_path = backup.path.clone();

    let on_submit = move || {
        let cid = target.get();
        if cid.is_empty() {
            set_err.set(Some("请输入目标账套 ID".to_string()));
            return;
        }
        set_busy.set(true);
        set_err.set(None);
        leptos::task::spawn_local(async move {
            match ipc::restore_company(cid, backup_path, true).await {
                Ok(_) => {
                    set_open.set(false);
                    on_restore_ok();
                }
                Err(e) => set_err.set(Some(e)),
            }
            set_busy.set(false);
        });
    };

    let close = move || set_open.set(false);
    let close_rc = std::rc::Rc::new(close);
    view! {
        <button class="text-xs text-warning inline-flex items-center" on:click=move |_| set_open.set(true)>
            <RotateCcw size=12 />
            "恢复"
        </button>
        <Modal open=open title="恢复账套" size=Some("sm") on_close=close_rc>
            <div class="modal-form">
                <p class="empty-state-desc">
                    "此操作将用备份覆盖目标账套数据库,且不可撤销。请确认。"
                </p>
                <div class="form-field">
                    <label class="form-label">"目标账套 ID"</label>
                    <input
                        class="form-input"
                        placeholder="公司/账套的 UUID"
                        prop:value=target
                        on:input=move |ev| set_target.set(event_target_value(&ev))
                    />
                    <Show when=move || company_id.get().is_some()>
                        <span class="form-helper">
                            "当前账套: " {move || company_id.get().unwrap_or_default()}
                        </span>
                    </Show>
                </div>
                <Show when=move || err.get().is_some()>
                    <div class="login-error">{move || err.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" on:click=move |_| close()> "取消"</button>
                <button class="btn btn-primary" disabled=busy on:click=move |_| on_submit()>
                    {move || if busy.get() { "恢复中…" } else { "确认恢复" }}
                </button>
            </div>
        </Modal>
    }
}

#[component]
fn CompanyEditModal(
    open: ReadSignal<bool>,
    editing: ReadSignal<Option<Company>>,
    set_open: WriteSignal<bool>,
    on_saved: impl Fn() + 'static,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (tax_id, set_tax_id) = signal(String::new());
    let (legal_person, set_legal_person) = signal(String::new());
    let (address, set_address) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (currency, set_currency) = signal("CNY".to_string());
    let (is_active, set_is_active) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);

    Effect::new(move |_| {
        if let Some(c) = editing.get() {
            set_name.set(c.name);
            set_tax_id.set(c.tax_id.unwrap_or_default());
            set_legal_person.set(c.legal_person.unwrap_or_default());
            set_address.set(c.address.unwrap_or_default());
            set_phone.set(c.phone.unwrap_or_default());
            set_currency.set(c.currency);
            set_is_active.set(c.is_active);
        } else if open.get() {
            set_name.set(String::new());
            set_tax_id.set(String::new());
            set_legal_person.set(String::new());
            set_address.set(String::new());
            set_phone.set(String::new());
            set_currency.set("CNY".to_string());
            set_is_active.set(true);
        }
    });

    let close = move || set_open.set(false);
    let close_rc = std::rc::Rc::new(close);

    let on_submit = move || {
        let editing_id = editing.get().map(|c| c.id);
        let input = CompanyInput {
            name: name.get(),
            tax_id: opt_str(tax_id.get()),
            legal_person: opt_str(legal_person.get()),
            address: opt_str(address.get()),
            phone: opt_str(phone.get()),
            currency: Some(currency.get()),
            is_active: Some(is_active.get()),
        };
        set_saving.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = editing_id {
                ipc::update_company(id, &input).await
            } else {
                ipc::create_company(&input).await
            };
            set_saving.set(false);
            match res {
                Ok(_) => {
                    set_open.set(false);
                    on_saved();
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <Modal open=open title="账套" size=Some("lg") on_close=close_rc>
            <div class="modal-form">
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"账套名称"</label>
                        <input
                            class="form-input"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"税号"</label>
                        <input
                            class="form-input"
                            prop:value=tax_id
                            on:input=move |ev| set_tax_id.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"法人"</label>
                        <input
                            class="form-input"
                            prop:value=legal_person
                            on:input=move |ev| set_legal_person.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"币种"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_currency.set(event_target_value(&ev))
                        >
                            <option value="CNY" selected=move || currency.get() == "CNY">"人民币 CNY"</option>
                            <option value="USD" selected=move || currency.get() == "USD">"美元 USD"</option>
                            <option value="EUR" selected=move || currency.get() == "EUR">"欧元 EUR"</option>
                        </select>
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"电话"</label>
                        <input
                            class="form-input"
                            prop:value=phone
                            on:input=move |ev| set_phone.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"地址"</label>
                        <input
                            class="form-input"
                            prop:value=address
                            on:input=move |ev| set_address.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <label class="flex items-center gap-2 text-13">
                    <input
                        type="checkbox"
                        style="width:16px;height:16px"
                        prop:checked=is_active
                        on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                    />
                    "启用账套"
                </label>
                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" on:click=move |_| close()>"取消"</button>
                <button class="btn btn-primary" disabled=saving on:click=move |_| on_submit()>
                    {move || if saving.get() { "保存中…" } else { "保存" }}
                </button>
            </div>
        </Modal>
    }
}

fn opt_str(s: String) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}
