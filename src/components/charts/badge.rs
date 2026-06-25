use leptos::prelude::*;

use crate::components::charts::kpi_card::KpiAccent;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeVariant {
    Draft,
    Pending,
    Approved,
    Posted,
    Archived,
    Info,
    Neutral,
}

impl BadgeVariant {
    fn class(&self) -> &'static str {
        match self {
            BadgeVariant::Draft => "bg-draft-soft text-draft",
            BadgeVariant::Pending => "bg-pending-soft text-pending",
            BadgeVariant::Approved => "bg-approved-soft text-approved",
            BadgeVariant::Posted => "bg-posted-soft text-posted",
            BadgeVariant::Archived => "bg-archived-soft text-archived",
            BadgeVariant::Info => "bg-info-soft text-info",
            BadgeVariant::Neutral => "bg-surface-hover text-secondary",
        }
    }
}

impl From<KpiAccent> for BadgeVariant {
    #[allow(dead_code)]
    fn from(value: KpiAccent) -> Self {
        match value {
            KpiAccent::Neutral => BadgeVariant::Neutral,
            KpiAccent::Primary => BadgeVariant::Approved,
            KpiAccent::Success => BadgeVariant::Approved,
            KpiAccent::Warning => BadgeVariant::Pending,
            KpiAccent::Danger => BadgeVariant::Archived,
            KpiAccent::Info => BadgeVariant::Info,
            KpiAccent::Brand => BadgeVariant::Posted,
        }
    }
}

#[component]
pub fn Badge(label: String, variant: BadgeVariant) -> impl IntoView {
    let class = variant.class();
    view! {
        <span class=format!("inline-flex items-center px-2 h-5 rounded text-xs font-medium {class}")>
            {label}
        </span>
    }
}

pub fn status_variant(label: &str) -> BadgeVariant {
    match label {
        "已审核" | "已过账" => BadgeVariant::Approved,
        "未审核" | "待审核" => BadgeVariant::Pending,
        "草稿" => BadgeVariant::Draft,
        "已驳回" => BadgeVariant::Archived,
        _ => BadgeVariant::Neutral,
    }
}
