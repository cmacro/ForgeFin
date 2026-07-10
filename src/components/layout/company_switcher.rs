use leptos::prelude::*;
use lucide_leptos::ChevronDown;

use crate::auth::Session;

/// 顶栏公司切换器。
///
/// 显示当前公司名称,点击在弹层中切换。仅在公司列表 > 1 时显示。
#[component]
pub fn CompanySwitcher() -> impl IntoView {
    let company_id = Session::company_id();
    let available = Session::available_companies();
    let (open, set_open) = signal(false);

    let current_name = move || {
        let cid = company_id.get();
        available
            .get()
            .iter()
            .find(|c| Some(c.id.clone()) == cid)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "未选择".to_string())
    };

    view! {
        <Show when=move || { available.get().len() > 1 }>
            <div class="company-switcher" on:click=move |_| set_open.update(|v| *v = !*v)>
                <span class="company-switcher-label">"账套"</span>
                <span>{current_name}</span>
                <ChevronDown size=14 />
            </div>
            <Show when=move || open.get()>
                <div
                    class="modal-overlay"
                    style="z-index:1100;background:rgba(0,0,0,0)"
                    on:click=move |_| set_open.set(false)
                >
                    <div
                        class="card"
                        style="position:absolute;top:60px;left:240px;min-width:240px;padding:4px"
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <For each=move || available.get() key=|c| c.id.clone() let:c>
                            <div
                                class="sidebar-item"
                                class=("active", {
                                    let cid = c.id.clone();
                                    move || company_id.get() == Some(cid.clone())
                                })
                                style="color:var(--color-primary);border-radius:var(--radius-md)"
                                on:click={let id = c.id.clone();
                                move |_| {
                                    let id = id.clone();
                                    leptos::task::spawn_local(async move {
                                        if Session::switch_company(id).await.is_ok() {
                                            set_open.set(false);
                                        }
                                    });
                                }}
                            >
                                <span>{c.name.clone()}</span>
                            </div>
                        </For>
                    </div>
                </div>
            </Show>
        </Show>
    }
}
