use leptos::prelude::*;
use lucide_leptos::{Calendar, ChevronDown, ChevronUp};

#[derive(Clone)]
pub struct SearchField {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    pub width: Option<&'static str>,
}

#[derive(Clone)]
pub enum FieldKind {
    Text {
        placeholder: Option<&'static str>,
    },
    DateRange,
    Select {
        options: Vec<SelectOption>,
        placeholder: Option<&'static str>,
    },
}

#[derive(Clone)]
pub struct SelectOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[component]
pub fn SearchForm(
    fields: Vec<SearchField>,
    #[prop(default = Callback::new(|_| {}))] on_search: Callback<()>,
    #[prop(default = Callback::new(|_| {}))] on_reset: Callback<()>,
    #[prop(default = false)] expandable: bool,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let visible_fields = move || {
        if !expandable || expanded.get() {
            fields.clone()
        } else {
            fields.iter().take(5).cloned().collect::<Vec<_>>()
        }
    };

    view! {
        <div class="card p-4 shadow-xs">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3">
                <For each=move || visible_fields() key=|f| f.key let:field>
                    <div class="form-field" style=field.width.map(|w| format!("width: {w}"))>
                        <span class="form-label">{field.label}</span>
                        {match field.kind.clone() {
                            FieldKind::Text { placeholder } => view! {
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder=placeholder.unwrap_or("")
                                />
                            }.into_any(),
                            FieldKind::DateRange => view! {
                                <div class="flex items-center gap-1">
                                    <input
                                        type="text"
                                        class="form-input flex-1"
                                        placeholder="2024-06-01"
                                    />
                                    <span class="text-disabled">"~"</span>
                                    <input
                                        type="text"
                                        class="form-input flex-1"
                                        placeholder="2024-06-30"
                                    />
                                    <span class="text-tertiary inline-flex items-center flex-shrink-0">
                                        <Calendar size=16 />
                                    </span>
                                </div>
                            }.into_any(),
                            FieldKind::Select { options, placeholder } => view! {
                                <div class="relative">
                                    <select class="form-select w-full">
                                        <option value="">{placeholder.unwrap_or("")}</option>
                                        <For each=move || options.clone() key=|o| o.value let:opt>
                                            <option value=opt.value>{opt.label}</option>
                                        </For>
                                    </select>
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </For>
            </div>
            <div class="search-form-actions">
                <Show when=move || expandable>
                    <button
                        class="search-form-toggle"
                        on:click=move |_| set_expanded.update(|v| *v = !*v)
                    >
                        <span class="chevron" class=("rotated", expanded)>
                            <Show when=move || expanded.get() fallback=|| view! { <ChevronDown size=12 /> }>
                                <ChevronUp size=12 />
                            </Show>
                        </span>
                        {move || if expanded.get() { "收起" } else { "更多条件" }}
                    </button>
                </Show>
                <div class="search-form-btn-group">
                    <button class="btn btn-outline" on:click=move |_| on_reset.run(())>
                        "重置"
                    </button>
                    <button class="btn btn-primary" on:click=move |_| on_search.run(())>
                        "查询"
                    </button>
                </div>
            </div>
        </div>
    }
}
