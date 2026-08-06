use leptos::prelude::*;
use lucide_leptos::{Pen, Plus, Settings2, Trash2};

use crate::components::layout::modal::Modal;
use crate::components::table::pagination::Pagination;
use crate::ipc::{self, DataSummaryInput, DataSummaryPage, DataSummaryRecord, FeeRate};

const PAGE_SIZE: i32 = 50;

fn source_label(info: &Option<String>) -> (&'static str, &'static str) {
    match info
        .as_ref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
    {
        Some(v) if v["type"] == "manual" => ("手工录入", "tag-draft"),
        Some(v) if v["type"] == "order_flow" => ("订单流水", "tag-brand"),
        Some(v) if v["type"] == "bank_flow" => ("银行流水", "tag-brand"),
        Some(v) if v["type"] == "import" => ("文件导入", "tag-success"),
        _ => ("来源未知", "tag-pending"),
    }
}

fn payment_method_label(m: &str) -> &'static str {
    match m {
        "debit_card" => "借记卡",
        "credit_card" => "信用卡",
        "wechat" => "微信",
        "alipay" => "支付宝",
        _ => m,
    }
}

#[component]
pub fn DataSummary() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (date_from, set_date_from) = signal(Option::<String>::None);
    let (date_to, set_date_to) = signal(Option::<String>::None);
    let (category_filter, set_category_filter) = signal(Option::<String>::None);
    let (project_filter, set_project_filter) = signal(Option::<String>::None);

    let data = LocalResource::new(move || async move {
        ipc::list_data_summaries(&ipc::DataSummaryFilter {
            date_from: date_from.get(),
            date_to: date_to.get(),
            category: category_filter.get(),
            project: project_filter.get(),
            page: page.get(),
            page_size: PAGE_SIZE,
        })
        .await
    });

    let refresh = move || data.refetch();

    let (edit_open, set_edit_open) = signal(false);
    let (editing, set_editing) = signal(Option::<DataSummaryRecord>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let (rate_open, set_rate_open) = signal(false);

    let open_new = move || {
        set_editing.set(None);
        set_error.set(None);
        set_edit_open.set(true);
    };

    let open_edit = move |r: DataSummaryRecord| {
        set_editing.set(Some(r));
        set_error.set(None);
        set_edit_open.set(true);
    };

    let on_delete = move |id: i64| {
        leptos::task::spawn_local(async move {
            match ipc::delete_data_summary(id).await {
                Ok(_) => refresh(),
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    let do_search = move |_| {
        set_page.set(1);
        refresh();
    };

    view! {
        <div class="page-content">
            <div class="action-bar">
                <div class="action-bar-group">
                    <button class="btn btn-primary" on:click=move |_| open_new()>
                        <Plus size=14 />
                        "新增汇总"
                    </button>
                </div>
                <div class="action-bar-group">
                    <input
                        type="date"
                        class="form-input"
                        style="width:140px"
                        prop:value=move || date_from.get().unwrap_or_default()
                        on:input=move |ev| set_date_from.set(Some(event_target_value(&ev)))
                    />
                    <span class="text-tertiary text-12">"至"</span>
                    <input
                        type="date"
                        class="form-input"
                        style="width:140px"
                        prop:value=move || date_to.get().unwrap_or_default()
                        on:input=move |ev| set_date_to.set(Some(event_target_value(&ev)))
                    />
                    <select
                        class="form-select"
                        style="width:100px"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_category_filter.set(if v.is_empty() { None } else { Some(v) });
                        }
                    >
                        <option value="">"全部分类"</option>
                        <option value="收入">"收入"</option>
                        <option value="支出">"支出"</option>
                    </select>
                    <button class="btn btn-outline" on:click=do_search>"查询"</button>
                </div>
                <div class="action-bar-group ml-auto">
                    <button class="btn btn-outline" on:click=move |_| set_rate_open.set(true)>
                        <Settings2 size=14 />
                        "手续费率"
                    </button>
                </div>
            </div>
            <Show when=move || error.get().is_some()>
                <div class="login-error">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Suspense fallback=|| view! { <div class="text-tertiary p-4">"加载中…"</div> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(p) => {
                            view! {
                                <div class="card">
                                    <table class="data-table" style="border:none">
                                        <thead>
                                            <tr>
                                                <th class="text-center" style="width:48px">"#"</th>
                                                <th>"日期"</th>
                                                <th>"收据编号"</th>
                                                <th>"分类"</th>
                                                <th>"项目"</th>
                                                <th>"事由"</th>
                                                <th>"支付方式"</th>
                                                <th class="data-table-num">"支付金额"</th>
                                                <th class="data-table-num">"手续费"</th>
                                                <th class="data-table-num">"实际收入"</th>
                                                <th class="data-table-num">"支出"</th>
                                                <th class="data-table-num">"余额"</th>
                                                <th>"来源"</th>
                                                <th>"备注"</th>
                                                <th class="text-center border-l border-border" style="width:80px">"操作"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For each=move || p.items.clone() key=|r| r.id let:row>
                                                <tr>
                                                    <td class="data-table-num text-tertiary">{row.id}</td>
                                                    <td>{row.summary_date.clone()}</td>
                                                    <td class="text-tertiary">{row.receipt_no.clone().unwrap_or_default()}</td>
                                                    <td>
                                                        <span class=format!("tag {}", if row.category == "收入" { "tag-success" } else { "tag-draft" })>
                                                            {row.category.clone()}
                                                        </span>
                                                    </td>
                                                    <td>{row.project.clone()}</td>
                                                    <td class="text-secondary">{row.reason.clone().unwrap_or_default()}</td>
                                                    <td>{row.payment_method.as_ref().map(|m| payment_method_label(m)).unwrap_or_default()}</td>
                                                    <td class="data-table-num">{row.payment_amount.clone()}</td>
                                                    <td class="data-table-num">{row.fee.clone()}</td>
                                                    <td class="data-table-num text-money">{row.actual_income.clone()}</td>
                                                    <td class="data-table-num text-money">{row.expense.clone()}</td>
                                                    <td class="data-table-num">{row.balance.clone().unwrap_or_default()}</td>
                                                    <td>
                                                        {let (label, tag) = source_label(&row.source_info);
                                                        view! { <span class=format!("tag {tag}")>{label}</span> }}
                                                    </td>
                                                    <td class="text-tertiary" style="max-width:150px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
                                                        {row.remarks.clone().unwrap_or_default()}
                                                    </td>
                                                    <td class="text-center border-l border-border">
                                                        <div class="flex items-center justify-center gap-4">
                                                            <button
                                                                class="text-xs text-brand inline-flex"
                                                                on:click={
                                                                    let r = row.clone();
                                                                    move |_| open_edit(r.clone())
                                                                }
                                                            >
                                                                <Pen size=12 />
                                                            </button>
                                                            <button
                                                                class="text-xs text-danger inline-flex"
                                                                on:click={
                                                                    let id = row.id;
                                                                    move |_| on_delete(id)
                                                                }
                                                            >
                                                                <Trash2 size=12 />
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            </For>
                                        </tbody>
                                    </table>
                                    <Show when=move || p.items.is_empty()>
                                        <div class="empty-state">
                                            <p class="empty-state-desc">"暂无数据汇总记录,点击「新增汇总」开始。"</p>
                                        </div>
                                    </Show>
                })}
            </Suspense>
        </div>
        <DataSummaryEditModal
            open=edit_open
            editing=editing
            set_open=set_edit_open
            on_saved=Callback::new(move |_| refresh())
        />
        <FeeRateModal open=rate_open set_open=set_rate_open />
    }
}

#[component]
fn DataSummaryEditModal(
    open: ReadSignal<bool>,
    editing: ReadSignal<Option<DataSummaryRecord>>,
    set_open: WriteSignal<bool>,
    on_saved: Callback<()>,
) -> impl IntoView {
    let (summary_date, set_summary_date) = signal(String::new());
    let (receipt_no, set_receipt_no) = signal(String::new());
    let (category, set_category) = signal("收入".to_string());
    let (project, set_project) = signal(String::new());
    let (reason, set_reason) = signal(String::new());
    let (payment_method, set_payment_method) = signal(String::new());
    let (payment_amount, set_payment_amount) = signal(String::new());
    let (fee, set_fee) = signal(String::new());
    let (actual_income, set_actual_income) = signal(String::new());
    let (expense, set_expense) = signal(String::new());
    let (balance, set_balance) = signal(String::new());
    let (remarks, set_remarks) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);

    Effect::new(move |_| {
        if let Some(r) = editing.get() {
            set_summary_date.set(r.summary_date.clone());
            set_receipt_no.set(r.receipt_no.unwrap_or_default());
            set_category.set(r.category.clone());
            set_project.set(r.project.clone());
            set_reason.set(r.reason.unwrap_or_default());
            set_payment_method.set(r.payment_method.unwrap_or_default());
            set_payment_amount.set(r.payment_amount.clone());
            set_fee.set(r.fee.clone());
            set_actual_income.set(r.actual_income.clone());
            set_expense.set(r.expense.clone());
            set_balance.set(r.balance.unwrap_or_default());
            set_remarks.set(r.remarks.unwrap_or_default());
        } else if open.get() {
            set_summary_date.set(String::new());
            set_receipt_no.set(String::new());
            set_category.set("收入".to_string());
            set_project.set(String::new());
            set_reason.set(String::new());
            set_payment_method.set(String::new());
            set_payment_amount.set(String::new());
            set_fee.set(String::new());
            set_actual_income.set(String::new());
            set_expense.set(String::new());
            set_balance.set(String::new());
            set_remarks.set(String::new());
        }
    });

    let close = move |_| set_open.set(false);

    let on_submit = Callback::new(move |_| {
        let editing_id = editing.get().map(|r| r.id);
        let input = DataSummaryInput {
            summary_date: summary_date.get(),
            receipt_no: if receipt_no.get().is_empty() {
                None
            } else {
                Some(receipt_no.get())
            },
            category: category.get(),
            project: project.get(),
            reason: if reason.get().is_empty() {
                None
            } else {
                Some(reason.get())
            },
            payment_method: if payment_method.get().is_empty() {
                None
            } else {
                Some(payment_method.get())
            },
            payment_amount: if payment_amount.get().is_empty() {
                None
            } else {
                Some(payment_amount.get())
            },
            fee: if fee.get().is_empty() {
                None
            } else {
                Some(fee.get())
            },
            actual_income: if actual_income.get().is_empty() {
                None
            } else {
                Some(actual_income.get())
            },
            expense: if expense.get().is_empty() {
                None
            } else {
                Some(expense.get())
            },
            balance: if balance.get().is_empty() {
                None
            } else {
                Some(balance.get())
            },
            remarks: if remarks.get().is_empty() {
                None
            } else {
                Some(remarks.get())
            },
        };
        set_saving.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = editing_id {
                ipc::update_data_summary(id, &input).await
            } else {
                ipc::create_data_summary(&input).await
            };
            set_saving.set(false);
            match res {
                Ok(_) => {
                    set_open.set(false);
                    on_saved.run(());
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let title_static: &'static str = "数据汇总编辑";
    view! {
        <Modal open=open title=title_static size="lg" on_close=Callback::new(close)>
            <div class="modal-form">
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"日期"</label>
                        <input
                            class="form-input"
                            type="date"
                            prop:value=summary_date
                            on:input=move |ev| set_summary_date.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"收据编号"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="如 0000502"
                            prop:value=receipt_no
                            on:input=move |ev| set_receipt_no.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"分类"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_category.set(event_target_value(&ev))
                        >
                            <option value="收入" selected=move || category.get() == "收入">"收入"</option>
                            <option value="支出" selected=move || category.get() == "支出">"支出"</option>
                        </select>
                    </div>
                    <div class="form-field">
                        <label class="form-label">"项目"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="如 营业收入、手续费"
                            prop:value=project
                            on:input=move |ev| set_project.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="form-field">
                    <label class="form-label">"事由"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="业务描述"
                        prop:value=reason
                        on:input=move |ev| set_reason.set(event_target_value(&ev))
                    />
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"支付方式"</label>
                        <select
                            class="form-select"
                            on:change=move |ev| set_payment_method.set(event_target_value(&ev))
                        >
                            <option value="">"(未选择)"</option>
                            <option value="debit_card" selected=move || payment_method.get() == "debit_card">"借记卡"</option>
                            <option value="credit_card" selected=move || payment_method.get() == "credit_card">"信用卡"</option>
                            <option value="wechat" selected=move || payment_method.get() == "wechat">"微信"</option>
                            <option value="alipay" selected=move || payment_method.get() == "alipay">"支付宝"</option>
                        </select>
                    </div>
                    <div class="form-field">
                        <label class="form-label">"支付金额"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="0.00"
                            prop:value=payment_amount
                            on:input=move |ev| set_payment_amount.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"手续费"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="0.00"
                            prop:value=fee
                            on:input=move |ev| set_fee.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"实际收入"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="0.00"
                            prop:value=actual_income
                            on:input=move |ev| set_actual_income.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="modal-form-row">
                    <div class="form-field">
                        <label class="form-label">"支出"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="0.00"
                            prop:value=expense
                            on:input=move |ev| set_expense.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"余额"</label>
                        <input
                            class="form-input"
                            type="text"
                            placeholder="0.00"
                            prop:value=balance
                            on:input=move |ev| set_balance.set(event_target_value(&ev))
                        />
                    </div>
                </div>
                <div class="form-field">
                    <label class="form-label">"备注"</label>
                    <input
                        class="form-input"
                        type="text"
                        placeholder="对方单位、清算账户等"
                        prop:value=remarks
                        on:input=move |ev| set_remarks.set(event_target_value(&ev))
                    />
                </div>
                <Show when=move || error.get().is_some()>
                    <div class="login-error">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" type="button" on:click=move |_| close(())>"取消"</button>
                <button class="btn btn-primary" type="button" disabled=saving on:click=move |_| on_submit.run(())>
                    {move || if saving.get() { "保存中…" } else { "保存" }}
                </button>
            </div>
        </Modal>
    }
}

#[component]
fn FeeRateModal(open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let rates =
        LocalResource::new(move || async move { ipc::list_fee_rates().await.unwrap_or_default() });

    let (error, set_error) = signal(Option::<String>::None);
    let (saving_id, set_saving_id) = signal(Option::<i64>::None);

    let close = move |_| set_open.set(false);

    let on_save = move |id: i64, method: String, rate_val: String| {
        set_saving_id.set(Some(id));
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match ipc::update_fee_rate(
                id,
                &ipc::FeeRateInput {
                    payment_method: method,
                    rate: rate_val,
                    description: None,
                    is_active: Some(true),
                },
            )
            .await
            {
                Ok(_) => {
                    rates.refetch();
                    set_saving_id.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_saving_id.set(None);
                }
            }
        });
    };

    let title_static: &'static str = "手续费率管理";
    view! {
        <Modal open=open title=title_static size="sm" on_close=Callback::new(close)>
            <div class="modal-form">
                <div class="text-13 text-secondary mb-3">
                    "设置各支付方式的默认手续费率(百分比)。修改后新增汇总记录时自动引用。"
                </div>
                <Suspense fallback=|| view! { <div class="text-tertiary p-2">"加载中…"</div> }>
                    {move || Suspend::new(async move {
                        let list = rates.await;
                        view! {
                            <table class="data-table" style="border:none">
                                <thead>
                                    <tr>
                                        <th>"支付方式"</th>
                                        <th class="data-table-num">"费率(%)"</th>
                                        <th class="text-center" style="width:60px">"操作"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {list.iter().map(|r| {
                                        let id = r.id;
                                        let method = r.payment_method.clone();
                                        let (val, set_val) = signal(r.rate.clone());
                                        let saving = move || saving_id.get() == Some(id);
                                        view! {
                                            <tr>
                                                <td>{payment_method_label(&r.payment_method)}</td>
                                                <td class="data-table-num">
                                                    <input
                                                        class="form-input"
                                                        type="text"
                                                        style="width:80px;text-align:right"
                                                        prop:value=val
                                                        on:input=move |ev| set_val.set(event_target_value(&ev))
                                                    />
                                                </td>
                                                <td class="text-center">
                                                    <button
                                                        class="btn btn-sm btn-primary"
                                                        disabled=saving
                                                        on:click=move |_| on_save(id, method.clone(), val.get())
                                                    >
                                                        {move || if saving() { "…" } else { "保存" }}
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }.into_any()
                    })}
                </Suspense>
                <Show when=move || error.get().is_some()>
                    <div class="login-error mt-2">{move || error.get().unwrap_or_default()}</div>
                </Show>
            </div>
            <div class="modal-footer">
                <button class="btn btn-outline" type="button" on:click=move |_| close(())>"关闭"</button>
            </div>
        </Modal>
    }
}
