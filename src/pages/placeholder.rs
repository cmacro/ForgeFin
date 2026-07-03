use leptos::prelude::*;
use lucide_leptos::LayoutPanelLeft;

#[component]
pub fn Placeholder(title: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full">
            <div class="card p-8 text-center">
                <div class="w-12 h-12 mx-auto rounded-md bg-surface-hover text-secondary flex items-center justify-center mb-3">
                    <LayoutPanelLeft size=24 />
                </div>
                <h2 class="card-title" style="font-size: 18px; margin-bottom: 4px">{title}</h2>
                <p style="font-size: 14px; color: var(--color-tertiary)">"该模块正在建设中"</p>
            </div>
        </div>
    }
}
