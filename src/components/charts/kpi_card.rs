use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum KpiAccent {
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Brand,
}

impl KpiAccent {
    fn bg_class(&self) -> &'static str {
        match self {
            KpiAccent::Neutral => "bg-surface-hover text-secondary",
            KpiAccent::Primary => "bg-brand-soft text-brand",
            KpiAccent::Success => "bg-success-soft text-success",
            KpiAccent::Warning => "bg-warning-soft text-warning",
            KpiAccent::Danger => "bg-danger-soft text-danger",
            KpiAccent::Info => "bg-info-soft text-info",
            KpiAccent::Brand => "bg-brand-soft text-brand",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            KpiAccent::Brand => "M10 3v14M6 7h8M4 7l-2 5h4zM16 7l-2 5h4z",
            KpiAccent::Success => "M4 10l4 4 8-8",
            KpiAccent::Warning => "M10 4v6m0 4h.01M3 18h14L10 4z",
            KpiAccent::Danger => "M6 6l8 8M14 6l-8 8M3 3h14v14H3z",
            KpiAccent::Info => "M10 4a6 6 0 1 0 0 12 6 6 0 0 0 0-12Zm0 3v4m0 2h.01",
            KpiAccent::Primary => "M4 4h12v12H4zM4 9l3 3 5-5",
            KpiAccent::Neutral => "M5 5h10v10H5z",
        }
    }
}

#[component]
pub fn KpiCard(
    label: &'static str,
    value: String,
    unit: Option<&'static str>,
    accent: KpiAccent,
) -> impl IntoView {
    let accent_class = accent.bg_class();
    let icon_path = accent.icon_path();
    view! {
        <div class="bg-surface border border-main rounded-md p-3 shadow-sm flex items-center gap-3">
            <div class=format!(
                "w-9 h-9 flex items-center justify-center rounded-md shrink-0 {accent_class}"
            )>
                <svg class="w-4 h-4" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d=icon_path stroke-linecap="round" stroke-linejoin="round" />
                </svg>
            </div>
            <div class="flex-1 min-w-0">
                <div class="text-xs text-secondary truncate">{label}</div>
                <div class="flex items-baseline gap-1 mt-0.5">
                    <span class="text-base font-semibold text-primary tabular-nums truncate">{value}</span>
                    {unit.map(|u| view! { <span class="text-xs text-secondary shrink-0">{u}</span> })}
                </div>
            </div>
        </div>
    }
}
