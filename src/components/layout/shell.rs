use leptos::prelude::*;

use crate::components::layout::{header::Header, sidebar::Sidebar};
use crate::nav::NavState;

#[component]
pub fn AppShell(nav: NavState, children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="h-screen w-screen flex flex-col overflow-hidden bg-surface-alt text-primary">
            <Header nav=nav.clone() />
            <div class="flex flex-1 min-h-0">
                <Sidebar nav=nav.clone() />
                <main class="flex-1 min-w-0 flex flex-col overflow-hidden">
                    {children()}
                </main>
            </div>
        </div>
    }
}
