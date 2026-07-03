use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub label: &'static str,
}

#[component]
pub fn Breadcrumb(_crumbs: Vec<Crumb>) -> impl IntoView {
    view! {
        <nav class="flex items-center gap-6 text-13 text-tertiary">
            <For each=move || _crumbs.clone() key=|c| c.label let:crumb>
                <span class="text-primary">{crumb.label}</span>
            </For>
        </nav>
    }
}
