use leptos::prelude::*;

#[derive(Clone)]
pub struct SearchField {
    pub key: &'static str,
    pub label: &'static str,
    pub placeholder: Option<&'static str>,
}

#[component]
pub fn SearchForm(
    fields: Vec<SearchField>,
    #[prop(default=std::rc::Rc::new(|| {}))] on_search: std::rc::Rc<dyn Fn()>,
    #[prop(default=std::rc::Rc::new(|| {}))] on_reset: std::rc::Rc<dyn Fn()>,
) -> impl IntoView {
    let search = on_search.clone();
    let reset = on_reset.clone();
    view! {
        <div class="bg-surface border border-main rounded-md p-4 shadow-sm">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
                <For each=move || fields.clone() key=|f| f.key let:field>
                    <label class="flex flex-col gap-1">
                        <span class="text-xs text-secondary">{field.label}</span>
                        <input
                            type="text"
                            class="h-8 px-2 text-sm border border-main rounded-md bg-surface text-primary focus:outline-none focus:border-brand"
                            placeholder={field.placeholder.unwrap_or("")}
                        />
                    </label>
                </For>
            </div>
            <div class="flex justify-end gap-2 mt-3">
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
    }
}
