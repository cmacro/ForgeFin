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
            <div class="flex items-center gap-2 min-w-0">
                <div class="w-8 h-8 rounded-md bg-brand text-white flex items-center justify-center text-sm font-semibold shrink-0">
                    "FF"
                </div>
                <span class="text-primary font-semibold text-base truncate">"财务管理工具"</span>
            </div>
        </div>
    }
}

#[component]
fn HeaderCenter() -> impl IntoView {
    view! {
        <nav class="hidden md:flex items-center text-sm gap-1 text-secondary">
            <span>"总账"</span>
            <span class="text-disabled">"/"</span>
            <span class="text-primary">"凭证管理"</span>
        </nav>
    }
}

#[component]
fn HeaderRight() -> impl IntoView {
    view! {
        <div class="flex items-center gap-1">
            <HeaderAction label="快捷键" icon_path="M4 8h4M4 8V4m0 4l4 4M12 8h4M12 8V4m0 4l4 4M4 16h4M4 16v-4m0 4l4-4" />
            <HeaderAction label="帮助" icon_path="M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0 12c-3 0-5 1.5-5 3.5V20h10v-1.5C14 16.5 12 15 9 15Z" />
            <HeaderAction label="消息" icon_path="M3 5h18v12H3zM3 5l9 7 9-7" />
            <div class="w-px h-6 bg-main mx-2"></div>
            <div class="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-surface-hover cursor-pointer">
                <div class="w-7 h-7 rounded-full bg-brand-hover text-white flex items-center justify-center text-xs font-medium">
                    "张"
                </div>
                <div class="text-sm leading-tight">
                    <div class="text-primary">"张会计"</div>
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
