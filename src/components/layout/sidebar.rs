use leptos::prelude::*;

use crate::nav::{nav_tree, NavItem, NavState};

#[component]
pub fn Sidebar(nav: NavState) -> impl IntoView {
    view! {
        <aside
            class="bg-sidebar border-r border-main flex flex-col overflow-hidden shrink-0 transition-[width] duration-200"
            class=("w-60", move || !nav.collapsed.get())
            class=("w-18", move || nav.collapsed.get())
        >
            <div
                class="h-14 flex items-center gap-2 px-4 border-b border-sidebar-hover shrink-0"
                class=("justify-start", move || !nav.collapsed.get())
                class=("justify-center", move || nav.collapsed.get())
            >
                <div class="w-8 h-8 rounded-md bg-brand text-white flex items-center justify-center text-sm font-semibold shrink-0">
                    "FF"
                </div>
                <Show when=move || !nav.collapsed.get()>
                    <div class="flex flex-col leading-tight min-w-0">
                        <span class="text-white font-semibold text-sm truncate">"ForgeFin"</span>
                        <span class="text-sidebar-muted text-xs truncate">"财务管理工具"</span>
                    </div>
                </Show>
            </div>

            <nav class="flex-1 overflow-y-auto py-2">
                <ul class="space-y-0.5 px-2">
                    <For each=move || nav_tree() key=|item| item.label let:item>
                        <NavItemRow item=item nav=nav.clone() depth=0 />
                    </For>
                </ul>
            </nav>
        </aside>
    }
}

#[component]
fn NavItemRow(item: NavItem, nav: NavState, depth: u8) -> AnyView {
    let has_children = item.children.is_some();
    let item_key = item.key;
    let nav_expand = nav.clone();
    let nav_child = nav.clone();
    let collapsed = nav.collapsed;
    let children = item.children.clone();
    let children_stored = StoredValue::new(children);
    let icon = item.icon;
    let label = item.label;
    let is_expanded = move || nav.is_expanded(item_key);
    let is_active = move || nav.is_active(item_key);
    let depth_for_child = depth + 1;

    view! {
        <li>
            <div
                class="flex items-center gap-2 rounded-md text-sm cursor-pointer select-none transition-colors"
                class=(
                    "bg-sidebar-active text-sidebar-active-text font-medium",
                    is_active
                )
                class=(
                    "text-sidebar-text hover:bg-sidebar-hover hover:text-white",
                    move || !is_active()
                )
                class=("pl-3 py-2", depth == 0)
                class=("pl-9 py-1.5", depth > 0)
                on:click=move |_| {
                    if has_children {
                        nav_expand.toggle_expand(item_key);
                    } else {
                        nav_expand.activate(item_key);
                    }
                }
            >
                <span
                    class="w-4 h-4 flex items-center justify-center shrink-0"
                    class=("text-white", is_active)
                    class=("text-sidebar-muted", move || !is_active())
                >
                    <NavIcon name=icon />
                </span>
                <span class="flex-1 truncate" class=("hidden", move || collapsed.get())>{label}</span>
                <Show when=move || has_children && !collapsed.get()>
                    <svg
                        class="w-3 h-3 text-sidebar-muted transition-transform"
                        class=("rotate-90", is_expanded)
                        viewBox="0 0 12 12"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.5"
                    >
                        <path d="M4 3l4 3-4 3" stroke-linecap="round" stroke-linejoin="round" />
                    </svg>
                </Show>
            </div>

            <Show when=move || has_children && is_expanded() && !collapsed.get()>
                <ul class="space-y-0.5 mt-0.5">
                    <For each=move || children_stored.get_value().unwrap_or_default() key=|child| child.label let:child>
                        <NavItemRow item=child nav=nav_child.clone() depth=depth_for_child />
                    </For>
                </ul>
            </Show>
        </li>
    }.into_any()
}

#[component]
fn NavIcon(name: &'static str) -> impl IntoView {
    let path = match name {
        "home" => "M3 10l7-7 7 7M5 9v9h4v-5h2v5h4V9",
        "book" => "M4 4h12v16H4zM4 4h12M8 8h4",
        "file" => "M6 3h7l3 3v14H6zM13 3v3h3M8 10h4M8 13h4",
        "file-plus" => "M6 3h7l3 3v14H6zM13 3v3h3M9 13h2M10 12v2",
        "check-square" => "M4 4h12v12H4zM4 9l3 3 5-5",
        "search" => "M9 4a5 5 0 1 0 0 10 5 5 0 0 0 0-10Zm4 9l4 4",
        "scale" => "M10 3v14M6 7h8M4 7l-2 5h4zM16 7l-2 5h4zM4 17h12",
        "list" => "M4 6h12M4 10h12M4 14h12",
        "book-open" => "M3 5h6a3 3 0 0 1 3 3v8a3 3 0 0 0-3-3H3zM21 5h-6a3 3 0 0 0-3 3v8a3 3 0 0 1 3-3h6z",
        "bar-chart" => "M4 16h3v2H4zM10 10h3v8h-3zM16 5h3v13h-3z",
        "report" => "M5 3h10l3 3v14H5zM15 3v3h3M8 10h6M8 13h6",
        "download" => "M10 4v8m0 0l-3-3m3 3l3-3M4 14h12v3H4z",
        "upload" => "M10 12V4m0 0l-3 3m3-3l3 3M4 14h12v3H4z",
        "building" => "M4 3v16h12V3zM7 7h2M7 11h2M11 7h2M11 11h2M7 15h6",
        "wallet" => "M3 6h14v10H3zM3 6V4h14v2M16 9h2v4h-2",
        "target" => "M10 3a7 7 0 1 0 0 14 7 7 0 0 0 0-14Zm0 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0 3a1 1 0 1 0 0 2 1 1 0 0 0 0-2Z",
        "receipt" => "M5 3h10v16l-2-1-2 1-2-1-2 1-2-1zM7 7h6M7 10h6M7 13h6",
        "settings" => "M10 3v3M10 14v3M3 10h3M14 10h3M5 5l2 2M13 13l2 2M15 5l-2 2M7 13l-2 2",
        _ => "M5 5h10v10H5z",
    };
    view! {
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d=path stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}
