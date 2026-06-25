use leptos::prelude::*;

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
        <div class="flex items-center gap-1 border-b border-main bg-surface px-2">
            <For each=move || items.clone() key=|t| t.key let:tab>
                {move || {
                    let key = tab.key;
                    let closable = tab.closable;
                    let is_active = key == active_key;
                    view! {
                        <div
                            class="group flex items-center gap-2 px-3 py-2 text-sm cursor-pointer border-b-2 -mb-px"
                            class=("text-primary border-brand font-medium", is_active)
                            class=("text-secondary border-transparent hover:text-primary", !is_active)
                        >
                            <span>{tab.label}</span>
                            {closable.then(|| view! {
                                <button class="w-4 h-4 flex items-center justify-center rounded text-disabled hover:bg-surface-hover hover:text-primary opacity-0 group-hover:opacity-100">
                                    <svg class="w-3 h-3" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M3 3l6 6M9 3l-6 6" stroke-linecap="round" />
                                    </svg>
                                </button>
                            })}
                        </div>
                    }
                }}
            </For>
        </div>
    }
}
