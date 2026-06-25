use leptos::prelude::*;

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
    #[prop(default = std::rc::Rc::new(|| {}))] on_search: std::rc::Rc<dyn Fn()>,
    #[prop(default = std::rc::Rc::new(|| {}))] on_reset: std::rc::Rc<dyn Fn()>,
    #[prop(default = false)] expandable: bool,
) -> impl IntoView {
    let search = on_search.clone();
    let reset = on_reset.clone();
    let (expanded, set_expanded) = signal(false);
    let visible_fields = move || {
        if !expandable || expanded.get() {
            fields.clone()
        } else {
            fields.iter().take(5).cloned().collect::<Vec<_>>()
        }
    };

    view! {
        <div class="bg-surface border border-main rounded-md p-4 shadow-sm">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-3">
                <For each=move || visible_fields() key=|f| f.key let:field>
                    <div class="flex flex-col gap-1" style=field.width.map(|w| format!("width: {w}"))>
                        <span class="text-xs text-secondary">{field.label}</span>
                        {match field.kind.clone() {
                            FieldKind::Text { placeholder } => view! {
                                <input
                                    type="text"
                                    class="h-8 px-2 text-sm border border-main rounded-md bg-surface text-primary focus:outline-none focus:border-brand focus:ring-1 focus:ring-brand/30"
                                    placeholder=placeholder.unwrap_or("")
                                />
                            }.into_any(),
                            FieldKind::DateRange => view! {
                                <div class="h-8 flex items-center gap-1 px-2 text-sm border border-main rounded-md bg-surface text-primary focus-within:border-brand">
                                    <input
                                        type="text"
                                        class="flex-1 bg-transparent outline-none text-sm placeholder:text-disabled"
                                        placeholder="2024-06-01"
                                    />
                                    <span class="text-disabled text-xs">"~"</span>
                                    <input
                                        type="text"
                                        class="flex-1 bg-transparent outline-none text-sm placeholder:text-disabled"
                                        placeholder="2024-06-30"
                                    />
                                    <svg class="w-4 h-4 text-secondary shrink-0" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M5 3v2M15 3v2M3 7h14M4 5h12v12H4z" stroke-linecap="round" />
                                    </svg>
                                </div>
                            }.into_any(),
                            FieldKind::Select { options, placeholder } => view! {
                                <div class="relative">
                                    <select class="h-8 w-full px-2 pr-7 text-sm border border-main rounded-md bg-surface text-primary appearance-none focus:outline-none focus:border-brand focus:ring-1 focus:ring-brand/30">
                                        <option value="">{placeholder.unwrap_or("")}</option>
                                        <For each=move || options.clone() key=|o| o.value let:opt>
                                            <option value=opt.value>{opt.label}</option>
                                        </For>
                                    </select>
                                    <svg class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 w-3 h-3 text-secondary" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M3 4l3 3 3-3" stroke-linecap="round" stroke-linejoin="round" />
                                    </svg>
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </For>
            </div>
            <div class="flex items-center justify-between mt-3">
                <Show when=move || expandable>
                    <button
                        class="inline-flex items-center gap-1 text-xs text-secondary hover:text-primary"
                        on:click=move |_| set_expanded.update(|v| *v = !*v)
                    >
                        <svg class="w-3 h-3 transition-transform" class=("rotate-180", expanded) viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M3 4l3 3 3-3" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                        {move || if expanded.get() { "收起" } else { "更多条件" }}
                    </button>
                </Show>
                <div class="flex items-center gap-2 ml-auto">
                    <button
                        class="h-8 px-4 text-sm border border-main rounded-md text-primary bg-surface hover:bg-surface-hover"
                        on:click=move |_| reset()
                    >
                        "重置"
                    </button>
                    <button
                        class="h-8 px-4 text-sm rounded-md text-white bg-brand hover:bg-brand-hover"
                        on:click=move |_| search()
                    >
                        "查询"
                    </button>
                </div>
            </div>
        </div>
    }
}
