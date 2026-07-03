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
        <div class="flex items-center gap-1"
            style="border-bottom: 1px solid var(--color-border); background: var(--color-surface); padding: 0 24px; margin: -24px -24px 0 -24px"
        >
            <For each=move || items.clone() key=|t| t.key let:tab>
                {move || {
                    let key = tab.key;
                    let closable = tab.closable;
                    let is_active = key == active_key;
                    let tab_style = if is_active {
                        "color: var(--color-primary); border-color: var(--color-brand); font-weight: 500"
                    } else {
                        "color: var(--color-secondary); border-color: transparent"
                    };
                    view! {
                        <div class="group flex items-center gap-2 px-3 py-2 text-sm cursor-pointer"
                            style=format!("border-bottom: 2px solid; margin-bottom: -1px; {}", tab_style)
                        >
                            <span>{tab.label}</span>
                            {closable.then(|| view! {
                                <button class="w-4 h-4 flex items-center justify-center rounded opacity-0 group-hover:opacity-100"
                                    style="color: var(--color-disabled)"
                                >
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
