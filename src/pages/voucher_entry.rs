use leptos::prelude::*;
use lucide_leptos::{Plus, Printer, Trash2};

use crate::ipc::{self, Account, VoucherEntryInput, VoucherInput};

/// 凭证录入页。
///
/// 表单: 日期 / 凭证类型 / 摘要 / 附件数 + 分录行(科目、摘要、借方、贷方)。
/// 实时校验借贷平衡,保存调用后端命令。
#[component]
pub fn VoucherEntry() -> impl IntoView {
    let accounts = Resource::new(|| (), move |_| async { ipc::list_accounts().await });
    let (voucher_date, set_voucher_date) = signal(today_iso());
    let (voucher_type, set_voucher_type) = signal("记账".to_string());
    let (voucher_no, set_voucher_no) = signal(String::new());
    let (summary, set_summary) = signal(String::new());
    let (attachments, set_attachments) = signal(0i32);
    let (entries, set_entries) = signal(vec![empty_entry(), empty_entry()]);
    let (error, set_error) = signal(Option::<String>::None);
    let (saving, set_saving) = signal(false);
    let (success, set_success) = signal(Option::<String>::None);
    let (no_loading, set_no_loading) = signal(false);

    // 生成凭证字号。
    let gen_no = move || {
        let date = voucher_date.get();
        let vtype = voucher_type.get();
        leptos::task::spawn_local(async move {
            match ipc::next_voucher_no(vtype, date).await {
                Ok(n) => set_voucher_no.set(n),
                Err(e) => set_error.set(Some(format!("生成凭证字号失败: {e}"))),
            }
        });
    };

    // 日期或类型变化时重新生成字号。
    Effect::new(move |_| {
        let _ = (voucher_date.get(), voucher_type.get());
        if !no_loading.get() {
            gen_no();
        }
    });

    let debit_total = move || {
        entries
            .get()
            .iter()
            .map(|e| e.debit.parse::<i64>().unwrap_or(0))
            .sum::<i64>()
    };
    let credit_total = move || {
        entries
            .get()
            .iter()
            .map(|e| e.credit.parse::<i64>().unwrap_or(0))
            .sum::<i64>()
    };
    let balanced = move || debit_total.get() == credit_total.get() && debit_total.get() > 0;

    let add_row = move || {
        set_entries.update(|v| v.push(empty_entry()));
    };

    let remove_row = move |idx: usize| {
        set_entries.update(|v| {
            if v.len() > 2 {
                v.remove(idx);
            }
        });
    };

    let on_save = move || {
        set_error.set(None);
        set_success.set(None);
        if !balanced.get() {
            set_error.set(Some("借贷不平衡,无法保存".to_string()));
            return;
        }
        let input = VoucherInput {
            voucher_no: voucher_no.get(),
            voucher_date: voucher_date.get(),
            voucher_type: voucher_type.get(),
            summary: summary.get(),
            attachments: Some(attachments.get()),
            operator_id: None,
            operator_name: None,
            entries: entries
                .get()
                .into_iter()
                .filter(|e| !e.account_id.is_empty())
                .map(|e| VoucherEntryInput {
                    account_id: e.account_id,
                    account_code: e.account_code,
                    account_name: e.account_name,
                    summary: if e.summary.is_empty() { None } else { Some(e.summary) },
                    debit: e.debit,
                    credit: e.credit,
                    contact_id: None,
                    contact_name: None,
                })
                .collect(),
        };
        if input.entries.is_empty() {
            set_error.set(Some("请至少添加一条分录".to_string()));
            return;
        }
        set_saving.set(true);
        leptos::task::spawn_local(async move {
            match ipc::create_voucher(&input).await {
                Ok(v) => {
                    set_success.set(Some(format!("凭证 {} 保存成功", v.voucher_no)));
                    set_entries.set(vec![empty_entry(), empty_entry()]);
                    gen_no();
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="page-content">
            <div class="card">
                <div class="card-header">
                    <span class="card-title">"新增凭证"</span>
                    <button class="btn btn-outline btn-sm" on:click=move |_| gen_no()>
                        "重新生成字号"
                    </button>
                </div>
                <div class="card-body">
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                        <div class="form-field">
                            <label class="form-label">"凭证日期"</label>
                            <input
                                class="form-input"
                                type="date"
                                prop:value=voucher_date
                                on:input=move |ev| set_voucher_date.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-field">
                            <label class="form-label">"凭证类型"</label>
                            <select
                                class="form-select"
                                on:change=move |ev| set_voucher_type.set(event_target_value(&ev))
                            >
                                <option value="记账" selected=move || voucher_type.get() == "记账">"记账凭证"</option>
                                <option value="付款" selected=move || voucher_type.get() == "付款">"付款凭证"</option>
                                <option value="收款" selected=move || voucher_type.get() == "收款">"收款凭证"</option>
                                <option value="转账" selected=move || voucher_type.get() == "转账">"转账凭证"</option>
                            </select>
                        </div>
                        <div class="form-field">
                            <label class="form-label">"凭证字号"</label>
                            <input
                                class="form-input"
                                prop:value=voucher_no
                                on:input=move |ev| set_voucher_no.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-field">
                            <label class="form-label">"附件张数"</label>
                            <input
                                class="form-input"
                                type="number"
                                min="0"
                                prop:value=move || attachments.get().to_string()
                                on:input=move |ev| set_attachments.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                    </div>
                    <div class="form-field mt-3">
                        <label class="form-label">"摘要"</label>
                        <input
                            class="form-input"
                            placeholder="本凭证摘要"
                            prop:value=summary
                            on:input=move |ev| set_summary.set(event_target_value(&ev))
                        />
                    </div>
                </div>
            </div>

            <div class="card mt-4">
                <div class="card-header">
                    <span class="card-title">"分录"</span>
                    <button class="btn btn-outline btn-sm" on:click=move |_| add_row()>
                        <Plus size=14 />
                        "添加行"
                    </button>
                </div>
                <div class="card-body dense">
                    <table class="data-table" style="border:none">
                        <thead>
                            <tr>
                                <th class="text-center" style="width:48px">"序号"</th>
                                <th>"科目"</th>
                                <th>"摘要"</th>
                                <th class="data-table-num" style="width:160px">"借方(分)"</th>
                                <th class="data-table-num" style="width:160px">"贷方(分)"</th>
                                <th class="text-center" style="width:60px">"操作"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || entries.get().into_iter().enumerate().collect::<Vec<_>>()
                                key=|(_, e)| e.uid let:(idx, e)>
                                <tr>
                                    <td class="data-table-num">{idx + 1}</td>
                                    <td>
                                        <Suspense fallback=|| view! { <input class="form-input" disabled /> }>
                                            {move || Suspend::with(async move {
                                                let list = accounts.get().await.unwrap_or_default().unwrap_or_default();
                                                let leaves: Vec<&Account> = list.iter().filter(|a| a.is_leaf).collect();
                                                view! {
                                                    <select
                                                        class="form-select"
                                                        on:change=move |ev| {
                                                            let id = event_target_value(&ev);
                                                            let acc = list.iter().find(|a| a.id == id).cloned();
                                                            set_entries.update(|v| {
                                                                if let Some(a) = acc {
                                                                    v[idx].account_id = a.id;
                                                                    v[idx].account_code = a.code;
                                                                    v[idx].account_name = a.name;
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <option value="">"(选择科目)"</option>
                                                        {leaves.iter().map(|a| {
                                                            let id = a.id.clone();
                                                            let label = format!("{} · {}", a.code, a.name);
                                                            view! {
                                                                <option value=id selected=move || e.account_id == id>{label}</option>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </select>
                                                }
                                            })}
                                        </Suspense>
                                    </td>
                                    <td>
                                        <input
                                            class="form-input"
                                            placeholder="分录摘要(可选)"
                                            prop:value=e.summary
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_entries.update(|v| v[idx].summary = val);
                                            }
                                        />
                                    </td>
                                    <td>
                                        <input
                                            class="form-input data-table-num"
                                            type="number"
                                            min="0"
                                            prop:value=e.debit
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_entries.update(|v| {
                                                    v[idx].debit = val;
                                                    if !val.is_empty() && val.parse::<i64>().unwrap_or(0) > 0 {
                                                        v[idx].credit = "0".to_string();
                                                    }
                                                });
                                            }
                                        />
                                    </td>
                                    <td>
                                        <input
                                            class="form-input data-table-num"
                                            type="number"
                                            min="0"
                                            prop:value=e.credit
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_entries.update(|v| {
                                                    v[idx].credit = val;
                                                    if !val.is_empty() && val.parse::<i64>().unwrap_or(0) > 0 {
                                                        v[idx].debit = "0".to_string();
                                                    }
                                                });
                                            }
                                        />
                                    </td>
                                    <td class="text-center">
                                        <button class="text-danger inline-flex" on:click=move |_| remove_row(idx)>
                                            <Trash2 size=14 />
                                        </button>
                                    </td>
                                </tr>
                            </For>
                            <tr class="bg-surface-alt">
                                <td class="text-secondary font-medium" colspan="2">"合计"</td>
                                <td></td>
                                <td class="data-table-num font-semibold text-primary">{debit_total}</td>
                                <td class="data-table-num font-semibold text-primary">{credit_total}</td>
                                <td></td>
                            </tr>
                        </tbody>
                    </table>
                    <div class="flex items-center gap-2 mt-3 text-13">
                        <Show when=move || balanced.get()>
                            <span class="tag tag-success">"借贷平衡"</span>
                        </Show>
                        <Show when=move || !balanced.get()>
                            <span class="tag tag-danger">
                                {move || format!("不平衡: 借方 {} / 贷方 {}", debit_total.get(), credit_total.get())}
                            </span>
                        </Show>
                    </div>
                </div>
                <div class="card-footer">
                    <button class="btn btn-outline" on:click=move |_| window_print()>
                        <Printer size=14 />
                        "打印"
                    </button>
                    <button class="btn btn-primary" disabled=saving on:click=move |_| on_save()>
                        {move || if saving.get() { "保存中…" } else { "保存凭证" }}
                    </button>
                </div>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="login-error mt-3">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || success.get().is_some()>
                <div class="tag tag-success mt-3">{move || success.get().unwrap_or_default()}</div>
            </Show>
        </div>
    }
}

#[derive(Clone)]
struct EntryRow {
    uid: usize,
    account_id: String,
    account_code: String,
    account_name: String,
    summary: String,
    debit: String,
    credit: String,
}

fn empty_entry() -> EntryRow {
    EntryRow {
        uid: next_uid(),
        account_id: String::new(),
        account_code: String::new(),
        account_name: String::new(),
        summary: String::new(),
        debit: "0".to_string(),
        credit: "0".to_string(),
    }
}

static UID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
fn next_uid() -> usize {
    UID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn today_iso() -> String {
    // 简单返回 yyyy-MM-dd,使用 js Date。
    let d = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    )
}

fn window_print() {
    if let Some(w) = web_sys::window() {
        let _ = w.print();
    }
}