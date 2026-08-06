use std::collections::BTreeMap;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::ipc::RawRecord;

/// 稳定列标识清单(与后端 `ui_prefs::COLUMN_KEYS` 保持一致)。
pub const COLUMN_KEYS: &[&str] = &[
    "source_type",
    "source_file_name",
    "source_row_no",
    "record_no",
    "record_date",
    "amount_total",
    "balance",
    "counterpart_info",
    "summary",
    "status",
];

/// 列 key → 中文标签,供工具条/表头使用。
pub fn column_label(key: &str) -> &'static str {
    match key {
        "source_type" => "来源类型",
        "source_file_name" => "来源文件",
        "source_row_no" => "行号",
        "record_no" => "业务单号",
        "record_date" => "日期",
        "amount_total" => "金额",
        "balance" => "余额",
        "counterpart_info" => "对方信息",
        "summary" => "摘要",
        "status" => "状态",
        _ => "?",
    }
}

/// 全部列默认可见(账套首次打开时使用)。
/// source_file_name / source_row_no 默认隐藏,来源信息在详情面板展示。
pub fn default_columns() -> BTreeMap<String, bool> {
    COLUMN_KEYS
        .iter()
        .map(|k| {
            let visible = !matches!(*k, "source_file_name" | "source_row_no");
            ((*k).to_string(), visible)
        })
        .collect()
}

/// Filter / search + keyboard-navigation state shared between the toolbar and table body.
///
/// Create once with [`RawRecordFilterState::new`] then pass clones to
/// [`RawRecordToolbar`] and [`RawRecordTableBody`].
#[derive(Clone)]
pub struct RawRecordFilterState {
    pub source_type: String,
    pub query_text: ReadSignal<String>,
    pub set_query_text: WriteSignal<String>,
    pub mode: ReadSignal<&'static str>,
    pub set_mode: WriteSignal<&'static str>,
    pub display_rows: Memo<Vec<RawRecord>>,
    pub display_ids: Memo<Vec<i64>>,
    pub total_all: usize,
    pub columns: ReadSignal<BTreeMap<String, bool>>,
    pub set_columns: WriteSignal<BTreeMap<String, bool>>,
    pub on_columns_change: Option<Callback<BTreeMap<String, bool>>>,
    pub tbody_ref: NodeRef<leptos::html::Tbody>,
    pub input_ref: NodeRef<leptos::html::Input>,
}

impl RawRecordFilterState {
    /// 构造过滤状态(默认全部列可见)。
    pub fn new(rows: &[RawRecord], source_type: impl Into<String>) -> Self {
        Self::with_columns(rows, source_type, default_columns(), None)
    }

    /// 构造过滤状态(指定初始列可见集合)。
    pub fn with_columns(
        rows: &[RawRecord],
        source_type: impl Into<String>,
        columns: BTreeMap<String, bool>,
        on_columns_change: Option<Callback<BTreeMap<String, bool>>>,
    ) -> Self {
        let total_all = rows.len();
        let tbody_ref = NodeRef::<leptos::html::Tbody>::new();
        let input_ref = NodeRef::<leptos::html::Input>::new();

        let (query_text, set_query_text) = signal(String::new());
        let (mode, set_mode) = signal("filter");
        let (cols, set_cols) = signal(columns);

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

        let rows_owned: Vec<RawRecord> = rows.to_vec();
        let display_rows = Memo::new(move |_| {
            let kws = keywords.get();
            let m = mode.get();
            if kws.is_empty() || m == "search" {
                return rows_owned.clone();
            }
            let show_source = cols.get().get("source_type").copied().unwrap_or(false);
            rows_owned
                .clone()
                .into_iter()
                .filter(|r| row_matches_keywords(r, &kws, show_source))
                .collect::<Vec<_>>()
        });

        let display_ids =
            Memo::new(move |_| display_rows.get().iter().map(|r| r.id).collect::<Vec<_>>());

        Self {
            source_type: source_type.into(),
            query_text,
            set_query_text,
            mode,
            set_mode,
            display_rows,
            display_ids,
            total_all,
            columns: cols,
            set_columns: set_cols,
            on_columns_change,
            tbody_ref,
            input_ref,
        }
    }

    pub fn total_after(&self) -> usize {
        self.display_rows.get().len()
    }

    pub fn focus_input(&self) {
        if let Some(input) = self.input_ref.get() {
            let _ = input.focus();
        }
    }

    /// 切换某列可见性(同时触发 on_columns_change 回调,用于落库)。
    pub fn toggle_column(&self, key: &str) {
        let mut next = self.columns.get();
        let cur = next.get(key).copied().unwrap_or(true);
        next.insert(key.to_string(), !cur);
        self.set_columns.set(next.clone());
        if let Some(cb) = &self.on_columns_change {
            cb.run(next);
        }
    }

    pub fn select_next(
        &self,
        selected_id: ReadSignal<Option<i64>>,
        set_selected_id: WriteSignal<Option<i64>>,
    ) {
        let current = selected_id.get();
        let ids = self.display_ids.get();
        let pos = current.and_then(|c| ids.iter().position(|x| *x == c));
        let next = match pos {
            Some(i) if i + 1 < ids.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        if !ids.is_empty() {
            set_selected_id.set(Some(ids[next]));
            focus_row(self.tbody_ref.clone(), next);
        }
    }

    pub fn select_prev(
        &self,
        selected_id: ReadSignal<Option<i64>>,
        set_selected_id: WriteSignal<Option<i64>>,
    ) {
        let current = selected_id.get();
        let ids = self.display_ids.get();
        let pos = current.and_then(|c| ids.iter().position(|x| *x == c));
        let prev = match pos {
            Some(0) => ids.len().saturating_sub(1),
            Some(i) => i - 1,
            None => 0,
        };
        if !ids.is_empty() {
            set_selected_id.set(Some(ids[prev]));
            focus_row(self.tbody_ref.clone(), prev);
        }
    }
}

/// 工具条:过滤/搜索 + 列设置(下拉)。
///
/// `on_columns_change` 用于在用户切换列可见性时持久化到后端。
/// 通常由页面层在构造 state 时填入,内部 state.toggle_column 触发。
#[component]
pub fn RawRecordToolbar(state: RawRecordFilterState) -> impl IntoView {
    let RawRecordFilterState {
        query_text,
        set_query_text,
        mode,
        set_mode,
        input_ref,
        total_all,
        columns,
        ..
    } = state.clone();

    let clear_query = {
        let state = state.clone();
        move |_: leptos::ev::MouseEvent| {
            set_query_text.set(String::new());
            state.focus_input();
        }
    };

    let switch_filter = {
        let state = state.clone();
        move |_: leptos::ev::MouseEvent| {
            set_mode.set("filter");
            state.focus_input();
        }
    };

    let switch_search = {
        let state = state.clone();
        move |_: leptos::ev::MouseEvent| {
            set_mode.set("search");
            state.focus_input();
        }
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

    let state_for_hint = state.clone();
    let hint = move || {
        let m = mode.get();
        if m == "filter" {
            let t = state_for_hint.total_after();
            if t == total_all {
                String::new()
            } else {
                format!("{} / {} 条", t, total_all)
            }
        } else {
            String::new()
        }
    };

    // 列设置下拉(开关)
    let (column_menu_open, set_column_menu_open) = signal(false);
    let toggle_menu = move |_: leptos::ev::MouseEvent| {
        set_column_menu_open.update(|v| *v = !*v);
    };

    // 全局 mousedown 监听:点下拉以外的任何位置时关闭菜单。
    // - 通过 create_effect 监听 column_menu_open:仅在打开时挂监听、关闭时移除。
    // - 下拉 DOM 通过 stop_propagation 阻止 mousedown 冒泡到 document,
    //   实现"点下拉内部不关"。
    // - 持有 js_sys::Function 句柄(由 Closure::as_ref() 转换),方便 close 时
    //   removeEventListener 找到同一对象引用。
    {
        use std::cell::RefCell;
        use std::rc::Rc;
        let set_open_signal = set_column_menu_open;
        let active_listener: Rc<RefCell<Option<js_sys::Function>>> = Rc::new(RefCell::new(None));
        Effect::new(move |_| {
            // 状态变 false:清除挂载的 listener
            if !column_menu_open.get() {
                if let Some(f) = active_listener.borrow_mut().take() {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            let _ = document.remove_event_listener_with_callback("mousedown", &f);
                        }
                    }
                    // f 在这里被 Drop,但因为 Closure 已被 forget() 转移给 JS,
                    // 没有对应 Rust 端 closure,JS 端 listener 引用会随 document GC
                }
                return;
            }

            // 状态变 true:挂监听(若已存在则先移除旧的)
            if let Some(old) = active_listener.borrow_mut().take() {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        let _ = document.remove_event_listener_with_callback("mousedown", &old);
                    }
                }
            }

            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };
            let set_open_clone = set_open_signal;
            let closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)> =
                wasm_bindgen::closure::Closure::new(move |_ev: web_sys::MouseEvent| {
                    set_open_clone.set(false);
                });
            // 取出 JS 端函数句柄(独立于 Closure 生命周期的 JsValue,Send+Sync)
            let func: js_sys::Function = closure.as_ref().clone().dyn_into().unwrap();
            let _ = document.add_event_listener_with_callback("mousedown", &func);
            // closure 转移给 JS(避免 rust 端 Drop 时 invalidate JS 回调)
            closure.forget();
            *active_listener.borrow_mut() = Some(func);
        });
    }

    let visible_count = move || columns.get().values().filter(|v| **v).count();
    let total_cols = COLUMN_KEYS.len();

    view! {
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
            <span class="data-table-bar-hint">{hint}</span>

            <div
                class="relative flex-shrink-0"
                on:mousedown=move |ev| ev.stop_propagation()
            >
                <button
                    class="btn btn-sm btn-outline flex items-center gap-1"
                    on:click=toggle_menu
                >
                    "列设置"
                    <span class="text-12 text-tertiary">
                        {move || format!("({}/{})", visible_count(), total_cols)}
                    </span>
                </button>
                <Show when=move || column_menu_open.get()>
                    <div class="absolute right-0 top-full mt-1 z-10 bg-surface border border-border rounded shadow-md p-2 min-w-44">
                        {COLUMN_KEYS.iter().map(|key| {
                            let key_str = (*key).to_string();
                            let label = column_label(key);
                            let key_for_toggle = key_str.clone();
                            let key_for_checked = key_str.clone();
                            let state_for_checked = state.clone();
                            let state_for_toggle = state.clone();
                            view! {
                                <label class="flex items-center gap-2 px-2 py-1 text-13 hover:bg-surface-alt rounded cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked=move || state_for_checked.columns.get().get(&key_for_checked).copied().unwrap_or(true)
                                        on:change=move |_| state_for_toggle.toggle_column(&key_for_toggle)
                                    />
                                    <span>{label}</span>
                                </label>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </Show>
            </div>

            <button class="btn btn-sm btn-outline" on:click=clear_query>"✕"</button>
        </div>
    }
}

/// Scrollable table body. Render this inside the grid's left column; it fills
/// available height and scrolls internally.
#[component]
pub fn RawRecordTableBody(
    state: RawRecordFilterState,
    selected_id: ReadSignal<Option<i64>>,
    set_selected_id: WriteSignal<Option<i64>>,
) -> impl IntoView {
    let RawRecordFilterState {
        query_text,
        mode,
        display_rows,
        tbody_ref,
        columns,
        set_query_text,
        ..
    } = state.clone();

    let state_for_keys = state.clone();

    let selected_id_for_next = selected_id;
    let set_selected_id_for_next = set_selected_id;
    let on_container_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();
        let ctrl = ev.ctrl_key() || ev.meta_key();

        if (ctrl && key == "f") || key == "/" {
            ev.prevent_default();
            state.focus_input();
            return;
        }

        if key == "Escape" {
            set_query_text.set(String::new());
            return;
        }

        if key == "ArrowDown" {
            ev.prevent_default();
            state.select_next(selected_id_for_next, set_selected_id_for_next);
        } else if key == "ArrowUp" {
            ev.prevent_default();
            state.select_prev(selected_id_for_next, set_selected_id_for_next);
        }
    };

    let total_after = move || display_rows.get().len();
    let _ = mode.get();

    view! {
        <div class="data-table data-table-compact data-table-nowrap flex flex-col min-h-0 flex-1">
            <div class="flex-1 overflow-auto" tabindex="-1" on:keydown=on_container_keydown>
                <table>
                    <thead>
                        <tr>
                            <Show when=move || state_for_keys.columns.get().get("source_type").copied().unwrap_or(true)>
                                <th>"来源类型"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("source_file_name").copied().unwrap_or(true)>
                                <th>"来源文件"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("source_row_no").copied().unwrap_or(true)>
                                <th class="data-table-num">"行号"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("record_no").copied().unwrap_or(true)>
                                <th>"业务单号"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("record_date").copied().unwrap_or(true)>
                                <th class="data-table-num">"日期"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("amount_total").copied().unwrap_or(true)>
                                <th class="data-table-num">"金额"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("balance").copied().unwrap_or(true)>
                                <th class="data-table-num">"余额"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("counterpart_info").copied().unwrap_or(true)>
                                <th>"对方信息"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("summary").copied().unwrap_or(true)>
                                <th>"摘要"</th>
                            </Show>
                            <Show when=move || state_for_keys.columns.get().get("status").copied().unwrap_or(true)>
                                <th>"状态"</th>
                            </Show>
                        </tr>
                    </thead>
                    <tbody node_ref=tbody_ref>
                        <For each=move || display_rows.get() key=|r| r.id let:row>
                            <RowItem
                                row=row
                                selected=selected_id
                                set_selected=set_selected_id
                                columns=columns
                                query=query_text
                            />
                        </For>
                    </tbody>
                </table>
                {move || {
                    if total_after() == 0 {
                        view! {
                            <div class="text-center py-40 text-tertiary">"暂无原始记录"</div>
                        }
                            .into_any()
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
        if let Some(ref v) = row.balance {
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
    columns: ReadSignal<BTreeMap<String, bool>>,
    query: ReadSignal<String>,
) -> impl IntoView {
    let id = row.id;
    let is_active = move || selected.get() == Some(id);
    // 余额连续性检查结果 → 行 class
    // 注意:view! 中 class=(name, bool) 是"按 bool 切换 name 类名"的语法,
    // 这里的 name 必须是字面量字符串,不能用变量名。所以用 class=balance_class
    // 把字符串值直接绑上去。
    // 业务规则:Decimal 严格加减不应出现四舍五入误差,所以只有"真不一致"才上红底;
    // 一旦财务人员确认(`balance_confirmed_at` 非空),mismatch 行也按 ok 样式
    // 显示(余额 cell 绿色加粗、整行无背景)。
    let is_confirmed = row.balance_confirmed_at.is_some();
    let balance_class = match (row.balance_check_status.as_deref(), is_confirmed) {
        (_, true) => "balance-ok",
        (Some("ok"), false) => "balance-ok",
        (Some("mismatch"), false) => "balance-mismatch",
        (Some("skip"), false) | (None, false) => "balance-skip",
        (_, false) => "balance-skip",
    };
    view! {
        <tr
            class=balance_class
            class=("selected", is_active)
            tabindex="0"
            on:click=move |_| set_selected.set(Some(id))
            on:keydown=move |ev| {
                if ev.key() == "Enter" || ev.key() == " " {
                    set_selected.set(Some(id));
                }
            }
        >
            <Show when=move || columns.get().get("source_type").copied().unwrap_or(true)>
                <td>{highlight_text(&row.source_type_name, query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("source_file_name").copied().unwrap_or(true)>
                <td>{highlight_text(&row.source_file_name, query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("source_row_no").copied().unwrap_or(true)>
                <td class="data-table-num">{row.source_row_no}</td>
            </Show>
            <Show when=move || columns.get().get("record_no").copied().unwrap_or(true)>
                <td>{highlight_text(&row.record_no.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("record_date").copied().unwrap_or(true)>
                <td class="data-table-num">{highlight_text(&row.record_date.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("amount_total").copied().unwrap_or(true)>
                <td class="data-table-num">{highlight_text(&row.amount_total.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("balance").copied().unwrap_or(true)>
                <td class="data-table-num balance-cell">{highlight_text(&row.balance.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("counterpart_info").copied().unwrap_or(true)>
                <td>{highlight_text(&row.counterpart_info.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("summary").copied().unwrap_or(true)>
                <td>{highlight_text(&row.summary.as_deref().unwrap_or("—"), query.get())}</td>
            </Show>
            <Show when=move || columns.get().get("status").copied().unwrap_or(true)>
                <td>
                    <StatusCell status=row.status.clone() />
                </td>
            </Show>
        </tr>
    }
}

#[component]
fn StatusCell(status: String) -> impl IntoView {
    let label = status_cn(&status);
    let class_name = status_class(&status);
    view! {
        <span class={format!("text-13 {class_name}")}>{label}</span>
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
