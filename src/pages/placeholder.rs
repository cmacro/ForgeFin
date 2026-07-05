use leptos::prelude::*;
use lucide_leptos::LayoutPanelLeft;

#[component]
pub fn Placeholder(title: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full">
            <div class="card p-8 text-center">
                <div class="placeholder-icon">
                    <LayoutPanelLeft size=24 />
                </div>
                <h2 class="page-title mb-4">{title}</h2>
                <p class="text-tertiary">"该模块正在建设中"</p>
            </div>
        </div>
    }
}
