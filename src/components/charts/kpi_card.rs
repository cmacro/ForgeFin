use leptos::prelude::*;

#[component]
pub fn KpiCard(label: &'static str, value: ReadSignal<String>, accent: KpiAccent) -> impl IntoView {
    view! {
        <div class="bg-surface border border-main rounded-md p-4 shadow-sm flex flex-col gap-1">
            <span class="text-xs text-secondary">{label}</span>
            <span class="text-xl font-semibold text-primary">
                {move || value.get()}
            </span>
            <span class={accent_class(accent)}>"趋势指标"</span>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum KpiAccent {
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
}

fn accent_class(accent: KpiAccent) -> &'static str {
    match accent {
        KpiAccent::Neutral => "text-xs text-secondary",
        KpiAccent::Success => "text-xs text-success",
        KpiAccent::Warning => "text-xs text-warning",
        KpiAccent::Danger => "text-xs text-danger",
        KpiAccent::Info => "text-xs text-info",
    }
}
