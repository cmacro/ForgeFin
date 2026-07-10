use leptos::prelude::*;
use lucide_leptos::{CircleQuestionMark, Keyboard, LogOut, Menu, MessageSquare};

use crate::auth::Session;
use crate::nav::NavState;

#[component]
pub fn Header(nav: NavState) -> impl IntoView {
    let title = move || nav.active.get().title();
    let subtitle = move || nav.active.get().subtitle();
    let user = Session::user();
    let company_id = Session::company_id();
    let available = Session::available_companies();
    let company_name = move || {
        let cid = company_id.get();
        available
            .get()
            .iter()
            .find(|c| Some(c.id.clone()) == cid)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    };
    let display_name = move || {
        user.get()
            .map(|u| u.display_name)
            .unwrap_or_else(|| "未登录".to_string())
    };
    let dept = move || {
        user.get()
            .and_then(|u| u.department)
            .unwrap_or_else(|| "".to_string())
    };

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
                <Show when=move || !company_name().is_empty()>
                    <div class="text-13 text-tertiary px-2">{company_name}</div>
                </Show>
                <div class="w-px h-6 bg-border"></div>
                <div class="user-profile">
                    <span class="header-avatar">
                        {move || display_name().chars().next().unwrap_or('?').to_string()}
                    </span>
                    <div class="user-profile-info">
                        <div class="name">{display_name}</div>
                        <div class="dept">{dept}</div>
                    </div>
                </div>
                <button
                    class="header-action"
                    title="退出登录"
                    aria-label="退出登录"
                    on:click=move |_| {
                        leptos::task::spawn_local(async move {
                            let _ = Session::logout().await;
                        });
                    }
                >
                    <LogOut size=18 />
                </button>
            </div>
        </header>
    }
}
