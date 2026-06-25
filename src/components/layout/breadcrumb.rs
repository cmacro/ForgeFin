use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub label: &'static str,
}

#[component]
pub fn Breadcrumb(crumbs: Vec<Crumb>) -> impl IntoView {
    view! {
        <nav class="flex items-center text-sm gap-1.5 text-secondary">
            <For each=move || crumbs.clone() key=|c| c.label let:c>
                <span class="text-primary">{c.label}</span>
                <span class="text-disabled">"/"</span>
            </For>
        </nav>
    }
}
