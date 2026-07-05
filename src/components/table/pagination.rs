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
        <div class="pagination">
            <span>
                {format!("共 {} 条", total)}
            </span>
            <div class="pagination-controls">
                <select class="pagination-select">
                    <option>{format!("{page_size}条/页")}</option>
                    <option>"50条/页"</option>
                    <option>"100条/页"</option>
                </select>
                <button class="pagination-btn">
                    <span class="block leading-none">"‹"</span>
                </button>
                <For each=move || visible.clone() key=|p| *p let:p>
                    {if p == -1 {
                        view! { <span class="pagination-ellipsis">"..."</span> }.into_any()
                    } else if p == current {
                        view! {
                            <button class="pagination-btn pagination-btn-active">
                                {p}
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <button class="pagination-btn">
                                {p}
                            </button>
                        }.into_any()
                    }}
                </For>
                <button class="pagination-btn">
                    <span class="block leading-none">"›"</span>
                </button>
                <span class="pagination-jump">
                    {format!("前往 {} 页", current)}
                </span>
            </div>
        </div>
    }
}
