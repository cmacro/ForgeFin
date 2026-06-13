use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::project_api;

#[component]
pub fn ProjectRecordForm() -> impl IntoView {
    let (project_id, set_project_id) = create_signal(0i64);
    let (amount, set_amount) = create_signal(0.0f64);
    let (is_income, set_is_income) = create_signal(false);
    let (account_id, set_account_id) = create_signal(0i64);
    let (description, set_description) = create_signal(String::new());
    let (status, set_status) = create_signal(String::from("Idle"));

    let submit = move |_| {
        let req = project_api::ProjectRecordRequest {
            project_id: project_id.get(),
            amount: amount.get(),
            is_income: is_income.get(),
            account_id: account_id.get(),
            description: description.get(),
        };

        spawn_local(async move {
            match project_api::add_project_record(req).await {
                Ok(res) => set_status(format!("Success! Voucher ID: {}", res.voucher_id)),
                Err(e) => set_status(format!("Error: {}", e)),
            }
        });
    };

    view! {
        <div class="p-4 bg-gray-100 rounded-lg">
            <h2 class="text-xl font-bold mb-4">"Record Project Transaction"</h2>
            <div class="flex flex-col gap-4">
                <div class="flex flex-col">
                    <label>"Project ID"</label>
                    <input type="number" 
                        on:input=move |ev| set_project_id(event_target_value(&ev))
                        prop:value=project_id 
                    />
                </div>
                <div class="flex flex-col">
                    <label>"Amount"</label>
                    <input type="number" 
                        on:input=move |ev| set_amount(event_target_value(&ev))
                        prop:value=amount 
                    />
                </div>
                <div class="flex flex-col">
                    <label>"Income? (Check if yes)"</label>
                    <input type="checkbox" 
                        on:change=move |ev| set_is_income(event_target_value(&ev))
                        prop:checked=is_income 
                    />
                </div>
                <div class="flex flex-col">
                    <label>"Account ID"</label>
                    <input type="number" 
                        on:input=move |ev| set_account_id(event_target_value(&ev))
                        prop:value=account_id 
                    />
                </div>
                <div class="flex flex-col">
                    <label>"Description"</label>
                    <input type="text" 
                        on:input=move |ev| set_description(event_target_value(&ev))
                        prop:value=description 
                    />
                </div>
                <button class="bg-blue-500 text-white px-4 py-2 rounded" on:click=submit>
                    "Submit"
                </button>
                <p class="mt-4">"Status: " {move || status.get()}</p>
            </div>
        </div>
    }
}
