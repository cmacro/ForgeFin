use leptos::prelude::*;
use lucide_leptos::{ChevronRight, Plus, Trash2};

use crate::components::layout::modal::Modal;
use crate::ipc::{self, Account, AccountInput};

/// 会计科目管理页(树形列表 + CRUD)。
#[component]
pub fn Accounts() -> impl IntoView {
    let accounts = Resource::new(|| (), move |_| async { ipc::list_accounts().await });
    let (selected, set_selected) = signal(Option::<String>::None);
    let (edit_open, set_edit_open) = signal(false);
    let (editing, set_editing) = signal(Option::<Account>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let refresh = move || accounts.refetch();

    let open_new = move || {
        set_editing.set(None);
        set_error.set(None);
        set_edit_open.set(true);
    };

    let open_edit = move |acc: Account| {
        set_editing.set(Some(acc));
        set_error.set(None);
        set_edit_open.set(true);
    };

    let on_delete = move |id: String| {
        leptos::task::spawn_local(async move {
            match ipc::delete_account(id).await {
                Ok(_) => refresh(),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="page-content">
            <div class="action-bar">
                <div class="action-bar-group">
                    <button class="btn btn-primary" on:click=move |_| open_new()>
                        <lucide_leptos::Plus size=14 />
                        "新增科目"
                    </button>
                </div>
            </div>
            <Show when=move || error.get().is_some()>
                <div class="login-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                {move || Suspend::new(async move {
                    match accounts.await {
                        Ok(list) => {
                            let tree = build_tree(&list);
                            view! {
                                <div class="card">
                                    <table class="data-table" style="border:none">
                                        <thead>
                                            <tr>
                                                <th class="text-center" style="width:48px">"序号"</th>
                                                <th>"科目编码"</th>
                                                <th>"科目名称"</th>
                                                <th>"科目类别"</th>
                                                <th>"余额方向"</th>
                                                <th class="text-center">"状态"</th>
                                                <th class="text-center border-l border-border">"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For each=move || tree.clone() key=|n| n.account.id.clone() let:node>
                                                <AccountRow
                                                    node=node
                                                    depth=0
                                                    selected=selected
                                                    set_selected=set_selected
                                                    on_edit=open_edit
                                                    on_delete=on_delete
                                                />
                                            </For>
                                        </tbody>
                                    </table>
                                    <Show when=move || list.is_empty()>
                                        <div class="empty-state">
                                            <p class="empty-state-desc">"尚未创建任何科目,点击「新增科目」开始。"</p>
                                        </div>
                                    </Show>
                                </div>
                            }
                        }
                        Err(e) => view! {
                            <div class="login-error">{format!("加载科目失败: {e}")}</div>
                        },
                    }
                })}
            </Suspense>
        </div>
        <AccountEditModal
            open=edit_open
            editing=editing
            accounts=accounts
            set_open=set_edit_open
            on_saved=move || refresh()
        />
    }
}

#[derive(Clone)]
struct AccountNode {
    account: Account,
    children: Vec<AccountNode>,
}

fn build_tree(accounts: &[Account]) -> Vec<AccountNode> {
    let mut by_parent: std::collections::HashMap<Option<String>, Vec<Account>> =
        std::collections::HashMap::new();
    for a in accounts {
        by_parent
            .entry(a.parent_id.clone())
            .or_default()
            .push(a.clone());
    }
    fn build(
        parent: Option<&str>,
        by_parent: &std::collections::HashMap<Option<String>, Vec<Account>>,
    ) -> Vec<AccountNode> {
        let mut list = by_parent
            .get(&parent.map(|s| s.to_string()))
            .cloned()
            .unwrap_or_default();
        list.sort_by(|a, b| a.code.cmp(&b.code));
        list.into_iter()
            .map(|a| {
                let children = build(Some(&a.id), by_parent);
                AccountNode {
                    account: a,
                    children,
                }
            })
            .collect()
    }
    build(None, &by_parent)
}

#[component]
fn AccountRow(
    node: AccountNode,
    depth: usize,
    selected: ReadSignal<Option<String>>,
    set_selected: WriteSignal<Option<String>>,
    on_edit: impl Fn(Account) + 'static,
    on_delete: impl Fn(String) + 'static,
) -> impl IntoView {
    let acc = node.account.clone();
    let children = node.children.clone();
    let id = acc.id.clone();
    let has_children = !children.is_empty();
    let (expanded, set_expanded) = signal(true);
    let edit_acc = acc.clone();
    let del_id = acc.id.clone();
    let row_acc = acc.clone();
    let indent = depth * 20;
    view! {
        <tr
            class=("selected", move || selected.get() == Some(id.clone()))
            on:click=move |_| set_selected.set(Some(id.clone()))
        >
            <td class="data-table-num text-tertiary">"—"</td>
            <td class="data-table-num">
                <span style=format!("display:inline-flex;align-items:center;width:{}px", 18 + indent)></span>
                <Show when=move || has_children>
                    <span
                        class="inline-flex cursor-pointer"
                        style="margin-right:4px"
                        on:click=move |ev| {
                            ev.stop_propagation();
                            set_expanded.update(|v| *v = !*v);
                        }
                    >
                        <ChevronRight size=14 />
                    </span>
                </Show>
                {acc.code.clone()}
            </td>
            <td>{acc.name.clone()}</td>
            <td>{type_label(&acc.account_type)}</td>
            <td>{direction_label(&acc.balance_direction)}</td>
            <td class="text-center">
                <span class=format!("tag {}", if row_acc.is_active { "tag-success" } else { "tag-draft" })>
                    {if row_acc.is_active { "启用" } else { "停用" }}
                </span>
            </td>
            <td class="text-center border-l border-border" on:click=move |ev| ev.stop_propagation()>
                <div class="flex items-center justify-center gap-4">
                    <button class="text-xs text-brand" on:click=move |_| on_edit(edit_acc.clone())>"编辑"</button>
                    <button class="text-xs text-danger inline-flex" on:click=move |_| on_delete(del_id.clone())>
                        <Trash2 size=12 />
                    </button>
                </div>
            </td>
        </tr>
        <Show when=move || expanded.get()>
            <For each=move || children.clone() key=|n| n.account.id.clone() let:child>
                <AccountRow
                    node=child
                    depth=depth + 1
                    selected=selected
                    set_selected=set_selected
                    on_edit=on_edit
                    on_delete=on_delete
                />
            </For>
        </Show>
    }
}

fn type_label(t: &str) -> &'static str {
    match t {
        "asset" => "资产",
        "liability" => "负债",
        "equity" => "权益",
        "cost" => "成本",
        "profit_loss" | "income" => "损益",
        _ => "—",
    }
}

fn direction_label(d: &str) -> &'static str {
    match d {
        "debit" => "借",
        "credit" => "贷",
        _ => "自动",
    }
}

#[component]
fn AccountEditModal(
    open: ReadSignal<bool>,
    editing: ReadSignal<Option<Account>>,
    accounts: Resource<(), Result<Vec<Account>, String>>,
    set_open: WriteSignal<bool>,
    on_saved: impl Fn() + 'static,
) -> impl IntoView {
    let (code, set_code) = signal(String::new());
    let (name, set_name) = signal(String::new());
    let (parent_id, set_parent_id) = signal(Option::<String>::None);
    let (account_type, set_account_type) = signal("asset".to_string());
    let (balance_direction, set_balance_direction) = signal("auto".to_string());
    let (is_active, set_is_active) = signal(true);
    let (description, set_description) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);

    Effect::new(move |_| {
        if let Some(acc) = editing.get() {
            set_code.set(acc.code);
            set_name.set(acc.name);
            set_parent_id.set(acc.parent_id);
            set_account_type.set(acc.account_type);
            set_balance_direction.set(acc.balance_direction);
            set_is_active.set(acc.is_active);
            set_description.set(acc.description.unwrap_or_default());
        } else if open.get() {
            set_code.set(String::new());
            set_name.set(String::new());
            set_parent_id.set(None);
            set_account_type.set("asset".to_string());
            set_balance_direction.set("auto".to_string());
            set_is_active.set(true);
            set_description.set(String::new());
        }
    });

    let close = move || set_open.set(false);
    let close_rc = std::rc::Rc::new(close);

    let on_submit = move || {
        let editing_id = editing.get().map(|a| a.id);
        let input = AccountInput {
            code: code.get(),
            name: name.get(),
            parent_id: parent_id.get(),
            account_type: account_type.get(),
            balance_direction: Some(balance_direction.get()),
            is_active: Some(is_active.get()),
            description: if description.get().is_empty() {
                None
            } else {
                Some(description.get())
            },
        };
        set_saving.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = editing_id {
                ipc::update_account(id, &input).await
            } else {
                ipc::create_account(&input).await
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

    let title_static: &'static str = "科目编辑";
    view! {
        <Modal open=open title=title_static on_close=close_rc>
            <div class="modal-form">
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"科目编码"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="如 1001"
                            prop:value=code
                            on:input=move |ev| set_code.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"科目名称"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="如 库存现金"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"科目类别"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_account_type.set(event_target_value(&ev))
                        >
                            <option value="asset" selected=move || account_type.get() == "asset">"资产"</option>
                            <option value="liability" selected=move || account_type.get() == "liability">"负债"</option>
                            <option value="equity" selected=move || account_type.get() == "equity">"权益"</option>
                            <option value="cost" selected=move || account_type.get() == "cost">"成本"</option>
                            <option value="profit_loss" selected=move || account_type.get() == "profit_loss">"损益"</option>
                        </select>
                    </div>
                    <div class="form-field">
                        <label class="form-label">"余额方向"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_balance_direction.set(event_target_value(&ev))
                        >
                            <option value="auto" selected=move || balance_direction.get() == "auto">"自动"</option>
                            <option value="debit" selected=move || balance_direction.get() == "debit">"借"</option>
                            <option value="credit" selected=move || balance_direction.get() == "credit">"贷"</option>
                        </select>
                    </div>
                </div>
                <div class="form-field">
                    <label class="form-label">"上级科目"</label>
                    <select
                        class="form-select"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_parent_id.set(if v.is_empty() { None } else { Some(v) });
                        }
                    >
                        <option value="">"(无,作为一级科目)"</option>
                        {move || {
                            match accounts.get().unwrap_or_default() {
                                Ok(list) => list
                                    .iter()
                                    .map(|a| {
                                        let id = a.id.clone();
                                        let label = format!("{} · {}", a.code, a.name);
                                        view! {
                                            <option value=id>{label}</option>
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                                Err(_) => Vec::new(),
                            }
                        }}
                    </select>
                </div>
                <div class="form-field">
                    <label class="form-label">"备注"</label>
                    <textarea
                        class="form-textarea"
                        rows="2"
                        prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))
                    ></textarea>
                </div>
                <label class="flex items-center gap-2 text-13">
                    <input
                        type="checkbox"
                        style="width:16px;height:16px"
                        prop:checked=is_active
                        on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                    />
                    "启用该科目"
                </label>
                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" type="button" on:click=move |_| close()>"取消"</button>
                <button class="btn btn-primary" type="button" disabled=saving on:click=move |_| on_submit()>
                    {move || if saving.get() { "保存中…" } else { "保存" }}
                </button>
            </div>
        </Modal>
    }
}
