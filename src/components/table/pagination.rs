use leptos::prelude::*;

#[component]
pub fn Pagination(
    #[prop(default = 245)] total: i32,
    #[prop(default = 1)] current: i32,
    #[prop(default = 20)] page_size: i32,
) -> impl IntoView {
    let pages = (total + page_size - 1) / page_size;
    let visible: Vec<i32> = if pages <= 7 {
        (1..=pages).collect()
    } else {
        let mut v = vec![1, 2, 3, 4, 5];
        if current > 3 && current < pages - 2 {
            v = vec![current - 1, current, current + 1];
        }
        let mut result = v.clone();
        if result.last().copied().unwrap_or(0) < pages - 1 {
            result.push(-1);
        }
        result.push(pages);
        result.dedup();
        result
    };

    view! {
        <div class="flex items-center justify-between text-xs px-3 py-2" style="color: var(--color-tertiary)">
            <span>
                {format!("共 {} 条", total)}
            </span>
            <div class="flex items-center gap-1">
                <select class="h-7 px-2 text-xs border rounded" style="border-color: var(--color-border); background: var(--color-surface); color: var(--color-primary)">
                    <option>{format!("{page_size}条/页")}</option>
                    <option>"50条/页"</option>
                    <option>"100条/页"</option>
                </select>
                <button class="w-7 h-7 border rounded" style="border-color: var(--color-border); color: var(--color-primary); background: var(--color-surface);">
                    <span class="block leading-none">"‹"</span>
                </button>
                <For each=move || visible.clone() key=|p| *p let:p>
                    {if p == -1 {
                        view! { <span class="px-1" style="color: var(--color-disabled)">"..."</span> }.into_any()
                    } else if p == current {
                        view! {
                            <button class="w-7 h-7 border rounded font-medium text-white" style="border-color: var(--color-brand); background: var(--color-brand)">
                                {p}
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button class="w-7 h-7 border rounded" style="border-color: var(--color-border); color: var(--color-primary); background: var(--color-surface);">
                                {p}
                            </button>
                        }.into_any()
                    }}
                </For>
                <button class="w-7 h-7 border rounded" style="border-color: var(--color-border); color: var(--color-primary); background: var(--color-surface);">
                    <span class="block leading-none">"›"</span>
                </button>
                <span class="ml-2 hidden md:inline">
                    {format!("前往 {} 页", current)}
                </span>
            </div>
        </div>
    }
}
