use leptos::prelude::*;
use lucide_leptos::{Check, X};

use crate::components::layout::modal::Modal;
use crate::ipc::{self, ReconciliationItem, VoucherSummary};

/// 差异审核弹框组件。
///
/// 支持“通过/驳回”并填写备注,通过时自动生成凭证。
#[component]
pub fn DiffReviewModal(
    open: ReadSignal<bool>,
    on_close: Callback<()>,
    item: Signal<Option<ReconciliationItem>>,
    #[prop(default = Callback::new(|_| {}))] on_reviewed: Callback<Option<VoucherSummary>>,
) -> impl IntoView {
    let (comment, set_comment) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let reset = move || {
        set_comment.set(String::new());
        set_error.set(None);
        set_loading.set(false);
    };

    let submit = move |approve: bool| {
        let Some(item) = item.get() else {
            return;
        };
        set_loading.set(true);
        set_error.set(None);
        let comment = comment.get();
        leptos::task::spawn_local(async move {
            match ipc::review_summary(item.id, approve, Some(comment)).await {
                Ok(voucher) => {
                    on_reviewed.run(voucher);
                    on_close.run(());
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <Modal
            open=open
            title="差异审核"
            size=Some("sm")
            on_close=Callback::new(move |_| {
                reset();
                on_close.run(());
            })
        >
            <div class="space-y-3">
                {move || {
                    let maybe = item.get().map(|it| view! {
                        <div class="detail-grid">
                            <div class="detail-field">
                                <span class="detail-field-label">"对账日期"</span>
                                <span class="detail-field-value">{it.summary_date}</span>
                            </div>
                            <div class="detail-field">
                                <span class="detail-field-label">"银行金额"</span>
                                <span class="detail-field-value">{it.bank_amount}</span>
                            </div>
                            <div class="detail-field">
                                <span class="detail-field-label">"订单金额"</span>
                                <span class="detail-field-value">{it.order_amount}</span>
                            </div>
                            <div class="detail-field">
                                <span class="detail-field-label">"差额"</span>
                                <span class="detail-field-value">{it.diff_amount}</span>
                            </div>
                        </div>
                    });
                    match maybe {
                        Some(v) => v.into_any(),
                        None => view! { <div class="text-tertiary">"未选择差异记录"</div> }.into_any(),
                    }
                }}

                <div class="form-field">
                    <label class="form-label">"审核备注"</label>
                    <textarea
                        class="form-textarea"
                        rows="2"
                        placeholder="可填写差异原因或处理说明"
                        prop:value=comment
                        on:input=move |ev| set_comment.set(event_target_value(&ev))
                    ></textarea>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <div class="modal-form-actions">
                    <button
                        class="btn btn-outline"
                        disabled=loading
                        on:click=move |_| submit(false)
                    >
                        <X size=12 />
                        "驳回"
                    </button>
                    <button
                        class="btn btn-primary"
                        disabled=loading
                        on:click=move |_| submit(true)
                    >
                        <Check size=12 />
                        {move || if loading.get() { "处理中…" } else { "通过并生成凭证" }}
                    </button>
                </div>
            </div>
        </Modal>
    }
}
