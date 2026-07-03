use leptos::prelude::*;
use lucide_leptos::{Banknote, Check, CircleCheckBig, Info, LayoutDashboard, TriangleAlert, X};

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
    fn icon(&self) -> impl IntoView {
        match self {
            KpiAccent::Brand => view! { <Banknote size=16 /> }.into_any(),
            KpiAccent::Success => view! { <Check size=16 /> }.into_any(),
            KpiAccent::Warning => view! { <TriangleAlert size=16 /> }.into_any(),
            KpiAccent::Danger => view! { <X size=16 /> }.into_any(),
            KpiAccent::Info => view! { <Info size=16 /> }.into_any(),
            KpiAccent::Primary => view! { <CircleCheckBig size=16 /> }.into_any(),
            KpiAccent::Neutral => view! { <LayoutDashboard size=16 /> }.into_any(),
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
    view! {
        <div class="card stat-card">
            <div class="flex items-center gap-3 mb-2">
                <div class="w-9 h-9 flex items-center justify-center rounded-md bg-surface-hover text-secondary">
                    {accent.icon()}
                </div>
                <span class="stat-card-label">{label}</span>
            </div>
            <div class="flex items-baseline gap-1">
                <span class="stat-card-value">{value}</span>
                {unit.map(|u| view! { <span class="stat-card-delta">{u}</span> })}
            </div>
        </div>
    }
}
