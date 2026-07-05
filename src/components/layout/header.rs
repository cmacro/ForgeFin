use leptos::prelude::*;
use lucide_leptos::{CircleQuestionMark, Keyboard, Menu, MessageSquare};

use crate::nav::NavState;

#[component]
pub fn Header(nav: NavState) -> impl IntoView {
    let title = move || nav.active.get().title();
    let subtitle = move || nav.active.get().subtitle();

    view! {
        <header class="app-header">
            <div class="app-header-left">
                <button
                    class="header-action"
                    on:click=move |_| nav.toggle_collapse()
                    aria-label="折叠菜单"
                >
                    <Menu size=18 />
                </button>
                <div>
                    <div class="page-title">{title}</div>
                    <Show when=move || !subtitle().is_empty()>
                        <div class="page-subtitle">{subtitle}</div>
                    </Show>
                </div>
            </div>
            <div class="app-header-right">
                <span class="header-action" title="快捷键" aria-label="快捷键">
                    <Keyboard size=18 />
                </span>
                <span class="header-action" title="帮助" aria-label="帮助">
                    <CircleQuestionMark size=18 />
                </span>
                <span class="header-action" title="消息" aria-label="消息">
                    <MessageSquare size=18 />
                </span>
                <div class="w-px h-6 bg-border"></div>
                <div class="user-profile">
                    <span class="header-avatar">"张"</span>
                    <div class="user-profile-info">
                        <div class="name">"张会计"</div>
                        <div class="dept">"财务部"</div>
                    </div>
                </div>
            </div>
        </header>
    }
}
