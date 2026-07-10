use leptos::prelude::*;
use lucide_leptos::{Plus, Trash2};

use crate::components::layout::modal::Modal;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, Contact, ContactInput};

/// 客户/供应商管理页(按 contact_type 区分,由导航来源决定)。
///
/// 默认列出全部;支持按类型过滤。CRUD 通过模态框。
#[component]
pub fn Contacts() -> impl IntoView {
    let contacts = LocalResource::new(move || async { ipc::list_contacts(None).await });
    let (filter_type, set_filter_type) = signal(Option::<String>::None);
    let (keyword, set_keyword) = signal(String::new());
    let (edit_open, set_edit_open) = signal(false);
    let (editing, set_editing) = signal(Option::<Contact>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let refresh = move || contacts.refetch();

    let open_new = move || {
        set_editing.set(None);
        set_error.set(None);
        set_edit_open.set(true);
    };

    let open_edit = move |c: Contact| {
        set_editing.set(Some(c));
        set_error.set(None);
        set_edit_open.set(true);
    };

    let on_delete = move |id: String| {
        leptos::task::spawn_local(async move {
            match ipc::delete_contact(id).await {
                Ok(_) => refresh(),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let filtered = move || {
        let list = contacts.get().and_then(|r| r.ok()).unwrap_or_default();
        list.into_iter()
            .filter(|c| filter_type.get().is_none() || c.contact_type == filter_type.get().unwrap())
            .filter(|c| {
                let kw = keyword.get();
                kw.is_empty()
                    || c.name.contains(&kw)
                    || c.code.contains(&kw)
                    || c.tax_id.as_deref().is_some_and(|t| t.contains(&kw))
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="page-content">
            <div class="card p-4 shadow-xs">
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                    <div class="form-field">
                        <label class="form-label">"类型"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                set_filter_type.set(if v.is_empty() { None } else { Some(v) });
                            }
                        >
                            <option value="">"全部"</option>
                            <option value="customer">"客户"</option>
                            <option value="vendor">"供应商"</option>
                        </select>
                    </div>
                    <div class="form-field">
                        <label class="form-label">"关键字"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="编码 / 名称 / 税号"
                            prop:value=keyword
                            on:input=move |ev| set_keyword.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            </div>
            <div class="action-bar">
                <div class="action-bar-group">
                    <button class="btn btn-primary" on:click=move |_| open_new()>
                        <Plus size=14 />
                        "新增"
                    </button>
                </div>
            </div>
            <Show when=move || error.get().is_some()>
                <div class="login-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                {move || Suspend::new(async move {
                    let _ = contacts.await;
                    let rows = filtered();
                    let rows_empty = rows.clone();
                    let rows_len = rows.len() as i32;
                    let rows_for = rows.clone();
                    let rows_enumerated = move || rows_for.iter().cloned().enumerate().collect::<Vec<_>>();
                    view! {
                        <div class="data-table flex flex-col min-h-0">
                            <div class="flex-1 overflow-auto">
                                <table>
                                    <thead>
                                        <tr>
                                            <th class="w-48 text-center">"序号"</th>
                                            <th>"编码"</th>
                                            <th>"名称"</th>
                                            <th>"类型"</th>
                                            <th>"税号"</th>
                                            <th>"电话"</th>
                                            <th class="text-center">"状态"</th>
                                            <th class="text-center border-l border-border">"操作"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For each=rows_enumerated key=|(_, c)| c.id.clone() let:item>
                                            <tr>
                                                <td class="data-table-num">{item.0 + 1}</td>
                                                <td class="data-table-num">{item.1.code.clone()}</td>
                                                <td>{item.1.name.clone()}</td>
                                                <td>{type_label(&item.1.contact_type)}</td>
                                                <td>{item.1.tax_id.clone().unwrap_or("—".to_string())}</td>
                                                <td>{item.1.phone.clone().unwrap_or("—".to_string())}</td>
                                                <td class="text-center">
                                                    <span class=format!("tag {}", if item.1.is_active { "tag-success" } else { "tag-draft" })>
                                                        {if item.1.is_active { "启用" } else { "停用" }}
                                                    </span>
                                                </td>
                                                <td class="text-center border-l border-border">
                                                    <div class="flex items-center justify-center gap-4">
                                                        {let c = item.1.clone();
                                                        let c1 = c.clone();
                                                        let c2 = c.clone();
                                                        view! {
                                                        <button class="text-xs text-brand" on:click=move |_| open_edit(c1)>"编辑"</button>
                                                        <button class="text-xs text-danger inline-flex" on:click=move |_| on_delete(c2.id.clone())>
                                                            <Trash2 size=12 />
                                                        </button>
                                                        }}
                                    </tbody>
                                </table>
                                <Show when=move || rows_empty.is_empty()>
                                    <div class="empty-state">
                                        <p class="empty-state-desc">"尚无客户/供应商,点击「新增」开始。"</p>
                                    </div>
                                </Show>
                            </div>
                            <div class="border-t border-border-light">
                                <Pagination total=rows_len current=1 page_size=20 />
                            </div>
                        </div>
                    }
                })}
            </Suspense>
        </div>
        <ContactEditModal
            open=edit_open
            editing=editing
            set_open=set_edit_open
            on_saved=Callback::new(move |_| refresh())
        />
    }
}

fn type_label(t: &str) -> &'static str {
    match t {
        "customer" => "客户",
        "vendor" => "供应商",
        _ => "—",
    }
}

#[component]
fn ContactEditModal(
    open: ReadSignal<bool>,
    editing: ReadSignal<Option<Contact>>,
    set_open: WriteSignal<bool>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let (code, set_code) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (contact_type, set_contact_type) = signal("customer".to_string());
    let (tax_id, set_tax_id) = signal(String::new());
    let (bank_account, set_bank_account) = signal(String::new());
    let (bank_name, set_bank_name) = signal(String::new());
    let (address, set_address) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (remark, set_remark) = signal(String::new());
    let (is_active, set_is_active) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);

    Effect::new(move |_| {
        if let Some(c) = editing.get() {
            set_code.set(c.code);
            set_name.set(c.name);
            set_contact_type.set(c.contact_type);
            set_tax_id.set(c.tax_id.unwrap_or_default());
            set_bank_account.set(c.bank_account.unwrap_or_default());
            set_bank_name.set(c.bank_name.unwrap_or_default());
            set_address.set(c.address.unwrap_or_default());
            set_phone.set(c.phone.unwrap_or_default());
            set_email.set(c.email.unwrap_or_default());
            set_remark.set(c.remark.unwrap_or_default());
            set_is_active.set(c.is_active);
        } else if open.get() {
            set_code.set(String::new());
            set_name.set(String::new());
            set_contact_type.set("customer".to_string());
            set_tax_id.set(String::new());
            set_bank_account.set(String::new());
            set_bank_name.set(String::new());
            set_address.set(String::new());
            set_phone.set(String::new());
            set_email.set(String::new());
            set_remark.set(String::new());
            set_is_active.set(true);
        }
    });

    let close = move |_| set_open.set(false);

    let on_submit = move || {
        let editing_id = editing.get().map(|c| c.id);
        let input = ContactInput {
            code: code.get(),
            name: name.get(),
            contact_type: contact_type.get(),
            tax_id: opt_str(tax_id.get()),
            bank_account: opt_str(bank_account.get()),
            bank_name: opt_str(bank_name.get()),
            address: opt_str(address.get()),
            phone: opt_str(phone.get()),
            email: opt_str(email.get()),
            remark: opt_str(remark.get()),
            is_active: Some(is_active.get()),
        };
        set_saving.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = editing_id {
                ipc::update_contact(id, &input).await
            } else {
                ipc::create_contact(&input).await
            };
            set_saving.set(false);
            match res {
                Ok(_) => {
                    set_open.set(false);
                    on_saved.run(());
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <Modal open=open title="客户/供应商" size=Some("lg") on_close=Callback::new(close)>
            <div class="modal-form">
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"编码"</label>
                        <input
                            class="form-input"
                            prop:value=code
                            on:input=move |ev| set_code.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"名称"</label>
                        <input
                            class="form-input"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"类型"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_contact_type.set(event_target_value(&ev))
                        >
                            <option value="customer" selected=move || contact_type.get() == "customer">"客户"</option>
                            <option value="vendor" selected=move || contact_type.get() == "vendor">"供应商"</option>
                        </select>
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
                        <label class="form-label">"开户银行"</label>
                        <input
                            class="form-input"
                            prop:value=bank_name
                            on:input=move |ev| set_bank_name.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"银行账号"</label>
                        <input
                            class="form-input"
                            prop:value=bank_account
                            on:input=move |ev| set_bank_account.set(event_target_value(&ev))
                        />
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
                        <label class="form-label">"邮箱"</label>
                        <input
                            class="form-input"
                            type="email"
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="form-field">
                    <label class="form-label">"地址"</label>
                    <input
                        class="form-input"
                        prop:value=address
                        on:input=move |ev| set_address.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label class="form-label">"备注"</label>
                    <textarea
                        class="form-textarea"
                        rows="2"
                        prop:value=remark
                        on:input=move |ev| set_remark.set(event_target_value(&ev))
                    ></textarea>
                </div>
                <label class="flex items-center gap-2 text-13">
                    <input
                        type="checkbox"
                        style="width:16px;height:16px"
                        prop:checked=is_active
                        on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                    />
                    "启用"
                </label>
                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" type="button" on:click=move |_| close(())>"取消"</button>
                <button class="btn btn-primary" type="button" disabled=saving on:click=move |_| on_submit()>
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
