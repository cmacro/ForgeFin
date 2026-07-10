use leptos::prelude::*;
use lucide_leptos::{Building2, LogIn, Plus};

use crate::auth::Session;
use crate::components::layout::company_edit_modal::CompanyEditModal;
use crate::ipc::Company;

#[component]
pub fn CompanySelection() -> impl IntoView {
    let available = Session::available_companies();
    let (edit_open, set_edit_open) = signal(false);
    let (editing, set_editing) = signal(Option::<Company>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let on_enter = move |id: String| {
        leptos::task::spawn_local(async move {
            if let Err(e) = Session::switch_company(id).await {
                set_error.set(Some(e));
            }
        });
    };

    let on_new = move || {
        set_editing.set(None);
        set_error.set(None);
        set_edit_open.set(true);
    };

    let on_saved = Callback::new(move |_| {
        set_edit_open.set(false);
        // 刷新 available_companies — 登录后 Session::init 已加载,新建后需重新获取
        leptos::task::spawn_local(async {
            if let Ok(user) = crate::ipc::current_user().await {
                Session::set_available(user.available_companies);
            }
        });
    });

    let list = move || available.get();

    view! {
        <div class="login-layout">
            <div class="login-card card" style="max-width:560px">
                <div class="login-header">
                    <div class="login-logo">"FF"</div>
                    <div>
                        <div class="login-title">"ForgeFin"</div>
                        <div class="login-subtitle">"请选择要进入的账套"</div>
                    </div>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <div class="card-body dense" style="padding:0">
                    <Show
                        when=move || !list().is_empty()
                        fallback=|| view! {
                            <div class="empty-state" style="padding:32px 0">
                                <Building2 size=32 />
                                <p class="empty-state-desc">"尚无可用账套，请创建一个新账套开始使用。"</p>
                            </div>
                        }
                    >
                        <div style="display:flex;flex-direction:column;gap:8px;padding:8px 0">
                            <For each=list key=|c| c.id.clone() let:c>
                                <div
                                    class="card"
                                    style="display:flex;align-items:center;justify-content:space-between;padding:12px 16px;margin:0 8px;cursor:pointer"
                                    on:click={
                                        let id = c.id.clone();
                                        move |_| on_enter(id.clone())
                                    }
                                >
                                    <div style="display:flex;align-items:center;gap:12px">
                                        <div
                                            style="width:40px;height:40px;border-radius:8px;background:var(--color-primary-bg);display:flex;align-items:center;justify-content:center;color:var(--color-primary);font-weight:600;font-size:16px"
                                        >
                                            {c.name.chars().next().unwrap_or('?').to_string()}
                                        </div>
                                        <div>
                                            <div style="font-weight:500;font-size:14px">{c.name.clone()}</div>
                                        </div>
                                    </div>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        on:click={
                                            let id = c.id.clone();
                                            move |ev| {
                                                ev.stop_propagation();
                                                on_enter(id.clone());
                                            }
                                        }
                                    >
                                        <LogIn size=14 />
                                        "进入"
                                    </button>
                                </div>
                            </For>
                        </div>
                    </Show>
                </div>

                <div class="login-footer" style="justify-content:center;padding-top:12px">
                    <button class="btn btn-outline" on:click=move |_| on_new()>
                        <Plus size=14 />
                        "新建账套"
                    </button>
                </div>
            </div>
        </div>

        <CompanyEditModal
            open=edit_open
            editing=editing
            set_open=set_edit_open
            on_saved=on_saved
        />
    }
}
