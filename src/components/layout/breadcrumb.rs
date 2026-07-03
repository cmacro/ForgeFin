use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub label: &'static str,
}

#[component]
pub fn Breadcrumb(_crumbs: Vec<Crumb>) -> impl IntoView {
    view! {
        <nav style="display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-tertiary)">
            <For each=move || _crumbs.clone() key=|c| c.label let:crumb>
                <span style="color: var(--color-primary)">{crumb.label}</span>
            </For>
        </nav>
    }
}
