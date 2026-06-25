use leptos::prelude::*;

use crate::nav::NavState;

#[component]
pub fn Header(nav: NavState) -> impl IntoView {
    view! {
        <header class="h-14 flex items-center justify-between px-4 bg-surface border-b border-main shrink-0">
            <HeaderLeft nav=nav />
            <HeaderCenter />
            <HeaderRight />
        </header>
    }
}

#[component]
fn HeaderLeft(nav: NavState) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 min-w-0">
            <button
                class="w-8 h-8 flex items-center justify-center rounded-md text-secondary hover:bg-surface-hover"
                on:click=move |_| nav.toggle_collapse()
                aria-label="折叠菜单"
            >
                <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M2 4h12M2 8h12M2 12h12" stroke-linecap="round" />
                </svg>
            </button>
        </div>
    }
}

#[component]
fn HeaderCenter() -> impl IntoView {
    view! {
        <nav class="hidden md:flex items-center text-sm gap-1 text-secondary">
            <span class="text-primary">"总账"</span>
            <span class="text-disabled">"/"</span>
            <span class="text-primary font-medium">"凭证管理"</span>
        </nav>
    }
}

#[component]
fn HeaderRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-1">
            <HeaderAction
                label="快捷键"
                icon_path="M5 4h3v8H5zM12 4h3v8h-3zM5 14h3v2H5zM12 14h3v2h-3z"
            />
            <HeaderAction
                label="帮助"
                icon_path="M10 4a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0 9c-3 0-5 1.5-5 3.5V18h10v-1.5C15 14.5 13 13 10 13Z"
            />
            <HeaderAction
                label="消息"
                icon_path="M5 5h10v9H8l-3 3z"
            />
            <div class="w-px h-6 bg-main mx-2"></div>
            <div class="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-surface-hover cursor-pointer">
                <div class="w-8 h-8 rounded-full bg-brand text-white flex items-center justify-center text-xs font-medium">
                    "张"
                </div>
                <div class="text-sm leading-tight">
                    <div class="text-primary font-medium">"张会计"</div>
                    <div class="text-xs text-secondary">"财务部"</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn HeaderAction(label: &'static str, icon_path: &'static str) -> impl IntoView {
    view! {
        <button
            class="w-8 h-8 flex items-center justify-center rounded-md text-secondary hover:bg-surface-hover hover:text-primary"
            title=label
            aria-label=label
        >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d=icon_path stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>
    }
}
