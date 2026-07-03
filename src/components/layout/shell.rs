use leptos::prelude::*;

use crate::components::layout::{header::Header, sidebar::Sidebar};
use crate::nav::NavState;

#[component]
pub fn AppShell(nav: NavState, children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="app-layout">
            <Sidebar nav=nav.clone() />
            <main class="app-main">
                <Header nav=nav.clone() />
                <section class="app-content">
                    {children()}
                </section>
            </main>
        </div>
    }
}
