use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::ipc::{self, RawRecord};

/// 银行流水页面工具条下方的"余额连续性确认"操作条。
///
/// 业务规则:`Decimal` 严格加减不应出现四舍五入误差,任何"余额不平"都视为真问题;
/// 财务核对后(可能是跨日补录、合并入账、银行分笔误差等)用本条对**整批**(import_batch)
/// 银行流水做整体确认,确认后该批 `mismatch` 行不再显示红底告警,余额 cell 仍保持绿色加粗
/// 样式(与 `ok` 行一致)。支持撤销。
///
/// 仅基于 `items` 中出现过的 `import_batch_id` 提供选择(不另行拉取全量批次列表),
/// 简单可靠;切换批次后通过 `on_changed` 回调通知父组件 refetch。
#[component]
pub fn BankFlowBalanceBar(items: Vec<RawRecord>, on_changed: Callback<()>) -> impl IntoView {
    // 当前页(items)中所有出现过的批次,按 id 升序排列。
    // 用一个 signal 持有,确保多个 `Fn` 闭包可以重复访问。
    let batches_signal = {
        let mut seen: BTreeMap<i64, BatchStats> = BTreeMap::new();
        for r in &items {
            if r.source_type != "bank_flow" {
                continue;
            }
            let entry = seen
                .entry(r.import_batch_id)
                .or_insert_with(BatchStats::default);
            entry.total += 1;
            if r.balance_check_status.as_deref() == Some("mismatch") {
                entry.mismatch += 1;
            }
            if r.balance_confirmed_at.is_some() {
                entry.confirmed_count += 1;
            }
        }
        signal(seen)
    };
    let batches = batches_signal.0;

    // 选中的 batch;items 变化时若当前选择不存在则回退到首个。
    let (selected_batch, set_selected_batch) = signal(Option::<i64>::None);
    let first_batch_id = batches.get().keys().next().copied();
    Effect::new(move |_| {
        let cur = selected_batch.get();
        let map = batches.get();
        let needs_reset = match cur {
            None => true,
            Some(id) => !map.contains_key(&id),
        };
        if needs_reset {
            set_selected_batch.set(first_batch_id);
        }
    });

    let (pending, set_pending) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let on_confirm = {
        let on_changed = on_changed;
        move |_: leptos::ev::MouseEvent| {
            let Some(batch_id) = selected_batch.get() else {
                return;
            };
            set_error.set(None);
            set_pending.set(true);
            let on_changed = on_changed;
            leptos::task::spawn_local(async move {
                match ipc::confirm_bank_balance_batch(batch_id).await {
                    Ok(_) => on_changed.run(()),
                    Err(e) => set_error.set(Some(format!("确认失败: {e}"))),
                }
                set_pending.set(false);
            });
        }
    };

    let on_unconfirm = {
        let on_changed = on_changed;
        move |_: leptos::ev::MouseEvent| {
            let Some(batch_id) = selected_batch.get() else {
                return;
            };
            set_error.set(None);
            set_pending.set(true);
            let on_changed = on_changed;
            leptos::task::spawn_local(async move {
                match ipc::unconfirm_bank_balance_batch(batch_id).await {
                    Ok(_) => on_changed.run(()),
                    Err(e) => set_error.set(Some(format!("撤销失败: {e}"))),
                }
                set_pending.set(false);
            });
        }
    };

    let has_batches = move || !batches.get().is_empty();
    let stats = move || {
        selected_batch
            .get()
            .and_then(|id| batches.get().get(&id).cloned())
    };

    let is_all_confirmed = move || {
        stats()
            .map(|s| s.mismatch > 0 && s.confirmed_count >= s.mismatch)
            .unwrap_or(false)
    };

    view! {
        <div class="data-table-bar">
            <Show
                when=has_batches
                fallback=|| view! {
                    <span class="text-12 text-tertiary">"暂无银行流水批次可确认"</span>
                }
            >
                <label class="flex items-center gap-1 text-12 text-secondary">
                    <span>"批次:"</span>
                    <select
                        class="form-input"
                        style:height="24px"
                        style:padding="0 6px"
                        style:font-size="12px"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            if let Ok(id) = v.parse::<i64>() {
                                set_selected_batch.set(Some(id));
                            }
                        }
                    >
                        {move || batches.get().iter().map(|(id, s)| {
                            let id_str = id.to_string();
                            let label = format!(
                                "批次 {} · 共 {} 行 · 余额不平 {} 行",
                                id, s.total, s.mismatch
                            );
                            view! {
                                <option value=id_str>{label}</option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </label>

                {move || stats().map(|s| {
                    let has_mismatch = s.mismatch > 0;
                    let all_pass = s.mismatch == 0 && s.total > 0;
                    view! {
                        <Show when=move || has_mismatch>
                            <span class="text-12 text-warning">
                                {format!("该批次有 {} 行余额不平", s.mismatch)}
                            </span>
                        </Show>
                        <Show when=move || all_pass>
                            <span class="text-12 text-success">"该批次余额连续性全部通过"</span>
                        </Show>
                    }
                })}

                <Show
                    when=move || is_all_confirmed()
                    fallback=move || view! {
                        <button
                            class="btn btn-sm btn-primary"
                            disabled=move || pending.get() || stats().map(|s| s.mismatch == 0).unwrap_or(true)
                            on:click=on_confirm
                        >
                            "确认余额"
                        </button>
                    }
                >
                    <span class="text-12 text-success">"已确认"</span>
                    <button
                        class="btn btn-sm btn-outline"
                        disabled=move || pending.get()
                        on:click=on_unconfirm
                    >
                        "撤销确认"
                    </button>
                </Show>

                <Show when=move || pending.get()>
                    <span class="text-12 text-tertiary">"处理中…"</span>
                </Show>
                <Show when=move || error.get().is_some()>
                    <span class="text-12 text-danger">{move || error.get().unwrap_or_default()}</span>
                </Show>
            </Show>
        </div>
    }
}

#[derive(Clone, Default)]
struct BatchStats {
    total: i64,
    mismatch: i64,
    confirmed_count: i64,
}
