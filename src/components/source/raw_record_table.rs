use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::ipc::RawRecord;

#[component]
pub fn RawRecordTable(
    rows: Vec<RawRecord>,
    selected_id: ReadSignal<Option<i64>>,
    set_selected_id: WriteSignal<Option<i64>>,
    #[prop(optional)] show_source_type: Option<bool>,
) -> impl IntoView {
    let show_source = show_source_type.unwrap_or(true);
    let total_all = rows.len();
    let tbody_ref = NodeRef::<leptos::html::Tbody>::new();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let (query_text, set_query_text) = signal(String::new());
    let (mode, set_mode) = signal("filter");

    let keywords = Memo::new(move |_| {
        let raw = query_text.get();
        if raw.is_empty() {
            return Vec::<String>::new();
        }
        raw.split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    });

    let display_rows = Memo::new(move |_| {
        let kws = keywords.get();
        let m = mode.get();
        if kws.is_empty() || m == "search" {
            return rows.clone();
        }
        rows.clone()
            .into_iter()
            .filter(|r| row_matches_keywords(r, &kws, show_source))
            .collect::<Vec<_>>()
    });

    let display_ids =
        Memo::new(move |_| display_rows.get().iter().map(|r| r.id).collect::<Vec<_>>());

    let select_next = {
        let tbody_ref = tbody_ref.clone();
        move || {
            let current = selected_id.get();
            let ids = display_ids.get();
            let pos = current.and_then(|c| ids.iter().position(|x| *x == c));
            let next = match pos {
                Some(i) if i + 1 < ids.len() => i + 1,
                Some(_) => 0,
                None => 0,
            };
            if !ids.is_empty() {
                set_selected_id.set(Some(ids[next]));
                focus_row(tbody_ref.clone(), next);
            }
        }
    };

    let select_prev = {
        let tbody_ref = tbody_ref.clone();
        move || {
            let current = selected_id.get();
            let ids = display_ids.get();
            let pos = current.and_then(|c| ids.iter().position(|x| *x == c));
            let prev = match pos {
                Some(0) => ids.len().saturating_sub(1),
                Some(i) => i - 1,
                None => 0,
            };
            if !ids.is_empty() {
                set_selected_id.set(Some(ids[prev]));
                focus_row(tbody_ref.clone(), prev);
            }
        }
    };

    let focus_input = {
        let input_ref = input_ref.clone();
        move || {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    };

    let on_container_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        let ctrl = ev.ctrl_key() || ev.meta_key();

        if (ctrl && key == "f") || key == "/" {
            ev.prevent_default();
            focus_input();
            return;
        }

        if key == "Escape" {
            set_query_text.set(String::new());
            return;
        }

        if key == "ArrowDown" {
            ev.prevent_default();
            select_next();
        } else if key == "ArrowUp" {
            ev.prevent_default();
            select_prev();
        }
    };

    let clear_query = move |_: leptos::ev::MouseEvent| {
        set_query_text.set(String::new());
        focus_input();
    };

    let switch_filter = move |_: leptos::ev::MouseEvent| {
        set_mode.set("filter");
        focus_input();
    };

    let switch_search = move |_: leptos::ev::MouseEvent| {
        set_mode.set("search");
        focus_input();
    };

    let is_filter = move || mode.get() == "filter";
    let is_search = move || mode.get() == "search";
    let filter_class = move || {
        if is_filter() {
            "btn btn-primary btn-sm"
        } else {
            "btn btn-outline btn-sm"
        }
    };
    let search_class = move || {
        if is_search() {
            "btn btn-primary btn-sm"
        } else {
            "btn btn-outline btn-sm"
        }
    };
    let total_after = move || display_rows.get().len();

    view! {
        <div class="data-table flex flex-col min-h-0 flex-1">
            <div class="data-table-bar">
                <div class="flex items-center gap-1 flex-shrink-0">
                    <button class=filter_class on:click=switch_filter>"过滤"</button>
                    <button class=search_class on:click=switch_search>"查找"</button>
                </div>
                <input
                    node_ref=input_ref
                    class="data-table-bar-input"
                    type="text"
                    placeholder=move || {
                        if is_filter() { "输入关键词过滤（空格分隔多关键词）隐藏不匹配行" } else { "输入关键词查找（空格分隔多关键词）高亮匹配文本" }
                    }
                    prop:value=query_text
                    on:input=move |ev| set_query_text.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Escape" {
                            set_query_text.set(String::new());
                        }
                    }
                />
                <span class="data-table-bar-hint">
                    {move || {
                        let m = mode.get();
                        if m == "filter" {
                            let t = total_after();
                            if t == total_all { String::new() } else { format!("{} / {} 条", t, total_all) }
                        } else {
                            String::new()
                        }
                    }}
                </span>
                <button class="btn btn-sm btn-outline" on:click=clear_query>"✕"</button>
            </div>
            <div class="flex-1 overflow-auto" tabindex="-1" on:keydown=on_container_keydown>
                <table>
                    <thead>
                        <tr>
                            <Show when=move || show_source>
                                <th>"来源类型"</th>
                            </Show>
                            <th>"来源文件"</th>
                            <th class="data-table-num">"行号"</th>
                            <th>"业务单号"</th>
                            <th class="data-table-num">"日期"</th>
                            <th class="data-table-num">"金额"</th>
                            <th>"对方信息"</th>
                            <th>"摘要"</th>
                            <th>"状态"</th>
                        </tr>
                    </thead>
                    <tbody node_ref=tbody_ref>
                        <For each=move || display_rows.get() key=|r| r.id let:row>
                            <RowItem
                                row=row
                                selected=selected_id
                                set_selected=set_selected_id
                                show_source_type=show_source
                                query=query_text
                            />
                        </For>
                    </tbody>
                </table>
                {move || {
                    if total_after() == 0 {
                        view! {
                            <div class="text-center py-40 text-tertiary">"暂无原始记录"</div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

fn row_matches_keywords(row: &RawRecord, keywords: &[String], show_source: bool) -> bool {
    let row_no = row.source_row_no.to_string();
    let haystack = {
        let mut parts: Vec<&str> = Vec::new();
        if show_source {
            parts.push(&row.source_type_name);
        }
        parts.push(&row.source_file_name);
        parts.push(&row_no);
        if let Some(ref v) = row.record_no {
            parts.push(v);
        }
        if let Some(ref v) = row.record_date {
            parts.push(v);
        }
        if let Some(ref v) = row.amount_total {
            parts.push(v);
        }
        if let Some(ref v) = row.counterpart_info {
            parts.push(v);
        }
        if let Some(ref v) = row.summary {
            parts.push(v);
        }
        parts.join(" ").to_lowercase()
    };
    keywords.iter().all(|kw| haystack.contains(kw.as_str()))
}

#[component]
fn RowItem(
    row: RawRecord,
    selected: ReadSignal<Option<i64>>,
    set_selected: WriteSignal<Option<i64>>,
    show_source_type: bool,
    query: ReadSignal<String>,
) -> impl IntoView {
    let id = row.id;
    let is_active = move || selected.get() == Some(id);
    let status_label = status_cn(&row.status);
    view! {
        <tr
            class=("selected", is_active)
            tabindex="0"
            on:click=move |_| set_selected.set(Some(id))
            on:keydown=move |ev| {
                if ev.key() == "Enter" || ev.key() == " " {
                    set_selected.set(Some(id));
                }
            }
        >
            {show_source_type.then(|| view! { <td>{highlight_text(&row.source_type_name, query.get())}</td> })}
            <td>{highlight_text(&row.source_file_name, query.get())}</td>
            <td class="data-table-num">{row.source_row_no}</td>
            <td>{highlight_text(&row.record_no.as_deref().unwrap_or("—"), query.get())}</td>
            <td class="data-table-num">{highlight_text(&row.record_date.as_deref().unwrap_or("—"), query.get())}</td>
            <td class="data-table-num">{highlight_text(&row.amount_total.as_deref().unwrap_or("—"), query.get())}</td>
            <td>{highlight_text(&row.counterpart_info.as_deref().unwrap_or("—"), query.get())}</td>
            <td>{highlight_text(&row.summary.as_deref().unwrap_or("—"), query.get())}</td>
            <td>
                <span class={format!("text-13 {}", status_class(&row.status))}>
                    {status_label}
                </span>
            </td>
        </tr>
    }
}

fn highlight_text(text: &str, query: String) -> Vec<AnyView> {
    if query.is_empty() {
        return vec![view! { {text.to_string()} }.into_any()];
    }
    let text_lower = text.to_lowercase();
    let keywords: Vec<&str> = query.split_whitespace().filter(|s| !s.is_empty()).collect();
    if keywords.is_empty() {
        return vec![view! { {text.to_string()} }.into_any()];
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for kw in &keywords {
        let kw_lower = kw.to_lowercase();
        for (start, matched) in text_lower.match_indices(&kw_lower) {
            let end = start + matched.len();
            ranges.push((start, end));
        }
    }

    ranges.sort_by_key(|r| r.0);
    ranges.dedup();

    if ranges.is_empty() {
        return vec![view! { {text.to_string()} }.into_any()];
    }

    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in ranges {
        if let Some(last) = merged.last_mut() {
            if s <= last.1 {
                last.1 = last.1.max(e);
                continue;
            }
        }
        merged.push((s, e));
    }

    let mut result: Vec<AnyView> = Vec::new();
    let mut last_end = 0;
    for (s, e) in &merged {
        if *s > last_end {
            result.push(view! { {text[last_end..*s].to_string()} }.into_any());
        }
        result.push(
            view! { <span class="search-highlight">{text[*s..*e].to_string()}</span> }.into_any(),
        );
        last_end = *e;
    }
    if last_end < text.len() {
        result.push(view! { {text[last_end..].to_string()} }.into_any());
    }
    result
}

fn focus_row(tbody: NodeRef<leptos::html::Tbody>, index: usize) {
    if let Some(tbody) = tbody.get() {
        let collection = tbody.rows();
        if let Some(row) = collection.item(index as u32) {
            if let Some(el) = row.dyn_into::<web_sys::HtmlElement>().ok() {
                let _ = el.focus();
            }
        }
    }
}

fn status_cn(status: &str) -> String {
    match status {
        "pending" => "待处理",
        "matched" => "已匹配",
        "approved" => "已审核",
        "rejected" => "已驳回",
        _ => status,
    }
    .to_string()
}

fn status_class(status: &str) -> &'static str {
    match status {
        "pending" => "text-warning",
        "matched" => "text-success",
        "approved" => "text-success",
        "rejected" => "text-danger",
        _ => "text-tertiary",
    }
}
