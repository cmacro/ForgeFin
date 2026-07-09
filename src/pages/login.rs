use leptos::prelude::*;
use lucide_leptos::{LogIn, ShieldCheck};

use crate::auth::Session;

/// 登录页。
///
/// 居中卡片布局,字段:用户名 / 密码。登录成功后由 App 切换到主壳。
#[component]
pub fn Login() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (submitting, set_submitting) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let u = username.get();
        let p = password.get();
        if u.trim().is_empty() || p.is_empty() {
            set_error.set(Some("请输入用户名和密码".to_string()));
            return;
        }
        set_error.set(None);
        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            match Session::login(u, p).await {
                Ok(_) => {
                    // App 会通过 Session::user() 响应式切换。
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-layout">
            <div class="login-card card">
                <div class="login-header">
                    <div class="login-logo">"FF"</div>
                    <div>
                        <div class="login-title">"ForgeFin"</div>
                        <div class="login-subtitle">"财务管理 · 登录"</div>
                    </div>
                </div>
                <form class="login-form" on:submit=on_submit>
                    <div class="form-field">
                        <label class="form-label">"用户名"</label>
                        <input
                            class="form-input"
                            type="text"
                            autocomplete="username"
                            placeholder="请输入用户名"
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"密码"</label>
                        <input
                            class="form-input"
                            type="password"
                            autocomplete="current-password"
                            placeholder="请输入密码"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>
                    <Show when=move || error.get().is_some()>
                        <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                    </Show>
                    <button class="btn btn-primary login-submit" type="submit" disabled=submitting>
                        <LogIn size=16 />
                        {move || if submitting.get() { "登录中…" } else { "登录" }}
                    </button>
                </form>
                <div class="login-footer">
                    <ShieldCheck size=14 />
                    <span>"数据本地存储,首次使用请在系统设置中创建账套"</span>
                </div>
            </div>
        </div>
    }
}
