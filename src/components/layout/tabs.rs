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
    #[prop(into)] active_key: Signal<&'static str>,
    #[prop(default = Callback::new(|_| {}))] on_change: Callback<&'static str>,
) -> impl IntoView {
    view! {
        <div class="tab-bar">
            <For each=move || items.clone() key=|t| t.key let:tab>
                {move || {
                    let key = tab.key;
                    let closable = tab.closable;
                    let is_active = key == active_key.get();
                    let change = on_change.clone();
                    view! {
                        <div
                            class="tab-bar-item"
                            class=("tab-bar-item-active", is_active)
                            on:click=move |_| change.run(key)
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
