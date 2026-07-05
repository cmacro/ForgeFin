use leptos::prelude::*;
use lucide_leptos::X;

#[derive(Clone)]
pub struct TabItem {
    pub key: &'static str,
    pub label: &'static str,
    pub closable: bool,
}

#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    #[prop(default = "voucher_overview")] active_key: &'static str,
) -> impl IntoView {
    view! {
        <div class="tab-bar">
            <For each=move || items.clone() key=|t| t.key let:tab>
                {move || {
                    let key = tab.key;
                    let closable = tab.closable;
                    let is_active = key == active_key;
                    view! {
                        <div class="tab-bar-item"
                            class=("tab-bar-item-active", is_active)
                        >
                            <span>{tab.label}</span>
                            {closable.then(|| view! {
                                <button class="tab-btn-close">
                                    <X size=12 />
                                </button>
                            })}
                        </div>
                    }
                }}
            </For>
        </div>
    }
}
