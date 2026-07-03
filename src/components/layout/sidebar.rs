use leptos::prelude::*;
use lucide_leptos::ChevronRight;

use crate::components::icon::Icon;
use crate::nav::{nav_tree, NavItem, NavState};

#[component]
pub fn Sidebar(nav: NavState) -> impl IntoView {
    view! {
        <aside
            class="sidebar"
            class=("collapsed", move || nav.collapsed.get())
        >
            <div class="sidebar-logo">
                <div class="sidebar-logo-icon w-8 h-8 rounded-md bg-brand flex items-center justify-center text-white text-sm font-semibold">
                    "FF"
                </div>
                <Show when=move || !nav.collapsed.get()>
                    <span>"ForgeFin"</span>
                </Show>
            </div>

            <nav class="sidebar-nav">
                <For each=move || nav_tree() key=|item| item.label let:item>
                    <NavItemRow item=item nav=nav.clone() depth=0 />
                </For>
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
    let depth_style = if depth > 0 { "padding-left: 32px" } else { "" };
    let depth_for_child = depth + 1;

    view! {
        <li>
            {if has_children {
                view! {
                    <div
                        class="sidebar-section-title"
                        on:click=move |_| nav_expand.toggle_expand(item_key)
                    >
                        <span class=("hidden", move || collapsed.get())>{label}</span>
                        <Show when=move || !collapsed.get()>
                            <span
                                class="sidebar-item-icon"
                                class=("rotate-90", is_expanded)
                            >
                                <ChevronRight size=12 />
                            </span>
                        </Show>
                    </div>
                    <Show when=move || is_expanded() && !collapsed.get()>
                        <ul style="padding: 4px 0">
                            <For each=move || children_stored.get_value().unwrap_or_default() key=|child| child.label let:child>
                                <NavItemRow item=child nav=nav_child.clone() depth=depth_for_child />
                            </For>
                        </ul>
                    </Show>
                }.into_any()
            } else {
                view! {
                    <div
                        class="sidebar-item"
                        class=("active", is_active)
                        style=depth_style
                        on:click=move |_| nav_expand.activate(item_key)
                    >
                        <span class="sidebar-item-icon">
                            <NavIcon name=icon />
                        </span>
                        <span class=("hidden", move || collapsed.get())>{label}</span>
                    </div>
                }.into_any()
            }}
        </li>
    }.into_any()
}

#[component]
fn NavIcon(name: &'static str) -> impl IntoView {
    view! { <Icon name=name size=16 /> }
}
