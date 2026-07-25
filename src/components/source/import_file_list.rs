use leptos::prelude::*;
use lucide_leptos::FileText;

use crate::ipc::ImportBatch;

fn source_type_label(source_type: &str) -> String {
    match source_type {
        "bank_flow" => "银行流水",
        "order_flow" => "订单流水",
        "pos_flow" => "POS流水",
        "summary_flow" => "数据汇总",
        _ => source_type,
    }
    .to_string()
}

#[component]
pub fn ImportFileList(
    batches: Signal<Vec<ImportBatch>>,
    selected_id: ReadSignal<Option<i64>>,
    on_select: Callback<i64>,
) -> impl IntoView {
    view! {
        <div class="card flex flex-col min-h-0">
            <div class="card-header">
                <span class="card-title">"导入的文件列表"</span>
                <span class="text-13 text-tertiary">
                    {move || format!("共 {} 个文件", batches.get().len())}
                </span>
            </div>
            <div class="flex-1 overflow-auto p-2">
                <Show
                    when=move || !batches.get().is_empty()
                    fallback=|| view! {
                        <div class="empty-state">
                            <p class="empty-state-desc">"暂无导入文件"</p>
                        </div>
                    }
                >
                    <FileTableRows
                        batches=batches
                        selected_id=selected_id
                        on_select=on_select
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn FileTableRows(
    batches: Signal<Vec<ImportBatch>>,
    selected_id: ReadSignal<Option<i64>>,
    on_select: Callback<i64>,
) -> impl IntoView {
    view! {
        <table class="w-full">
            <thead>
                <tr>
                    <th>"文件名"</th>
                    <th>"类型"</th>
                    <th>"导入时间"</th>
                </tr>
            </thead>
            <tbody>
                <For each=move || batches.get().clone() key=|b| b.id let:batch>
                    <FileRow
                        batch=batch
                        selected_id=selected_id
                        on_select=on_select
                    />
                </For>
            </tbody>
        </table>
    }
}

#[component]
fn FileRow(
    batch: ImportBatch,
    selected_id: ReadSignal<Option<i64>>,
    on_select: Callback<i64>,
) -> impl IntoView {
    let is_selected = move || selected_id.get() == Some(batch.id);
    let tr_class = move || {
        if is_selected() {
            "cursor-pointer hover:bg-surface transition-colors bg-surface border-l-2 border-l-brand"
        } else {
            "cursor-pointer hover:bg-surface transition-colors"
        }
    };
    view! {
        <tr
            class=tr_class
            on:click=move |_| on_select.run(batch.id)
        >
            <td class="flex items-center gap-2">
                <span class="text-tertiary"><FileText size=14 /></span>
                <span class="truncate" title={batch.file_name.clone()}>
                    {batch.file_name.clone()}
                </span>
            </td>
            <td>
                <span class="text-13 px-2 py-0.5 bg-surface-light rounded">
                    {source_type_label(&batch.source_type)}
                </span>
            </td>
            <td class="text-13 text-tertiary whitespace-nowrap">
                {batch.imported_at.clone()}
            </td>
        </tr>
    }
}
