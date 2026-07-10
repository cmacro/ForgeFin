use leptos::prelude::*;

use crate::components::layout::modal::Modal;
use crate::ipc::{Company, CompanyInput};

#[component]
pub fn CompanyEditModal(
    open: ReadSignal<bool>,
    editing: ReadSignal<Option<Company>>,
    set_open: WriteSignal<bool>,
    on_saved: Callback<()>,
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

    let close = move |_| set_open.set(false);

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
                crate::ipc::update_company(id, &input).await
            } else {
                crate::ipc::create_company(&input).await
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
        <Modal open=open title="账套" size=Some("lg") on_close=Callback::new(close)>
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
                <button class="btn btn-outline" on:click=move |_| close(())>"取消"</button>
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
