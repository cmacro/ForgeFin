use leptos::prelude::*;
use lucide_leptos::X;

/// 通用模态对话框。
///
/// `children` 应包含表单主体与底部按钮区。底部按钮区建议使用 `.modal-footer`
/// 容器(实际渲染由 children 内部决定,本组件不强制结构)。
#[component]
pub fn Modal(
    open: ReadSignal<bool>,
    title: &'static str,
    #[prop(default = None)] size: Option<&'static str>,
    on_close: std::rc::Rc<dyn Fn()>,
    children: ChildrenFn,
) -> impl IntoView {
    let size_class = match size {
        Some("sm") => "modal modal-sm",
        Some("lg") => "modal modal-lg",
        _ => "modal",
    };
    let close = on_close.clone();
    view! {
        <Show when=move || open.get()>
            <div class="modal-overlay" on:click=move |_| on_close()>
                <div class=size_class on:click=move |ev| ev.stop_propagation()>
                    <div class="modal-header">
                        <span class="modal-title">{title}</span>
                        <button
                            class="modal-close"
                            type="button"
                            on:click=move |_| close()
                            aria-label="关闭"
                        >
                            <X size=16 />
                        </button>
                    </div>
                    <div class="modal-body">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
