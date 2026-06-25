use std::sync::Arc;

use leptos::prelude::*;

#[derive(Clone)]
pub struct ColumnConfig {
    pub key: &'static str,
    pub label: &'static str,
    pub width: Option<&'static str>,
    pub align: Align,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Clone)]
pub struct RowActionsFn(pub Arc<dyn Fn(Vec<String>) -> Vec<AnyView> + Send + Sync>);

impl RowActionsFn {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Vec<String>) -> Vec<AnyView> + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }
}

#[component]
pub fn DataTable(
    columns: Vec<ColumnConfig>,
    rows: Vec<Vec<String>>,
    #[prop(optional)] row_actions: Option<RowActionsFn>,
) -> impl IntoView {
    let col_count = columns.len();
    let has_actions = row_actions.is_some();
    let columns_for_header = columns.clone();
    let columns_for_cells = columns.clone();
    let rows_for_body = rows.clone();
    let rows_for_empty = rows.clone();
    view! {
        <div class="bg-surface border border-main rounded-md shadow-sm overflow-hidden flex flex-col min-h-0">
            <div class="overflow-auto">
                <table class="w-full text-sm border-collapse">
                    <thead class="bg-surface-alt sticky top-0 z-10">
                        <tr class="border-b border-main">
                            <For each=move || columns_for_header.clone() key=|col| col.key let:col>
                                <th
                                    class=th_class(col.align)
                                    style=col.width.map(|w| format!("width: {w}"))
                                >
                                    {col.label}
                                </th>
                            </For>
                            <Show when=move || has_actions>
                                <th class="px-3 py-2 text-left text-secondary font-medium border-l border-main">"操作"</th>
                            </Show>
                        </tr>
                    </thead>
                    <tbody>
                        <For each=move || rows_for_body.clone() key=|r| r.join("|") let:row>
                            {
                                let cells = (0..col_count).map(|i| {
                                    let align = columns_for_cells.get(i).map(|c| c.align).unwrap_or(Align::Left);
                                    let text = row.get(i).cloned().unwrap_or_default();
                                    view! {
                                        <td class=td_class(align)>{text}</td>
                                    }.into_any()
                                }).collect::<Vec<_>>();
                                let actions = row_actions.clone();
                                let actions_for_show = actions.clone();
                                let actions_stored = StoredValue::new(actions);
                                view! {
                                    <tr class="border-b border-muted hover:bg-surface-hover h-10">
                                        {cells}
                                        <Show when=move || actions_for_show.is_some()>
                                            <td class="px-3 py-2 border-l border-main">
                                                {
                                                    let a = actions_stored.get_value();
                                                    let r = row.clone();
                                                    view! { <RowActions actions=a row=r /> }
                                                }
                                            </td>
                                        </Show>
                                    </tr>
                                }
                            }
                        </For>
                        <Show when=move || rows_for_empty.is_empty()>
                            <tr>
                                <td colspan={col_count + 1} class="text-center text-secondary py-10 text-sm">
                                    "暂无数据"
                                </td>
                            </tr>
                        </Show>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

fn th_class(align: Align) -> &'static str {
    match align {
        Align::Left => "px-3 py-2 text-left text-secondary font-medium",
        Align::Right => "px-3 py-2 text-right text-secondary font-medium",
        Align::Center => "px-3 py-2 text-center text-secondary font-medium",
    }
}

fn td_class(align: Align) -> &'static str {
    match align {
        Align::Left => "px-3 py-2 text-left text-primary",
        Align::Right => "px-3 py-2 text-right text-primary tabular-nums",
        Align::Center => "px-3 py-2 text-center text-primary",
    }
}

#[component]
fn RowActions(actions: Option<RowActionsFn>, row: Vec<String>) -> impl IntoView {
    let acts = actions.map(|a| (a.0)(row)).unwrap_or_default();
    view! {
        <div class="flex items-center gap-1">
            {acts}
        </div>
    }
}
