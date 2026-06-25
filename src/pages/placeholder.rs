use leptos::prelude::*;

#[component]
pub fn Placeholder(title: &'static str) -> impl IntoView {
    view! {
        <section class="flex-1 flex items-center justify-center p-8">
            <div class="text-center">
                <div class="w-12 h-12 mx-auto rounded-md bg-surface-hover text-secondary flex items-center justify-center mb-3">
                    <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M4 4h16v16H4zM4 9h16" stroke-linecap="round" />
                    </svg>
                </div>
                <h2 class="text-lg font-semibold text-primary mb-1">{title}</h2>
                <p class="text-sm text-secondary">"该模块正在建设中"</p>
            </div>
        </section>
    }
}
