use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum BadgeVariant {
    Draft,
    Pending,
    Approved,
    Posted,
    Archived,
    // Info,
    Neutral,
}

impl BadgeVariant {
    fn class(&self) -> &'static str {
        match self {
            BadgeVariant::Draft => "tag-draft",
            BadgeVariant::Pending => "tag-pending",
            BadgeVariant::Approved => "tag-approved",
            BadgeVariant::Posted => "tag-posted",
            BadgeVariant::Archived => "tag-archived",
            // BadgeVariant::Info => "tag-info",
            BadgeVariant::Neutral => "",
        }
    }
}

#[component]
pub fn Badge(label: String, variant: BadgeVariant) -> impl IntoView {
    let class = variant.class();
    view! {
        <span class=format!("tag {class}")>
            <span class="tag-dot"></span>
            {label}
        </span>
    }
}

pub fn status_variant(label: &str) -> BadgeVariant {
    match label {
        "已过账" => BadgeVariant::Posted,
        "已审核" => BadgeVariant::Approved,
        // "已审核" | "已过账" => BadgeVariant::Approved,
        "未审核" | "待审核" => BadgeVariant::Pending,
        "草稿" => BadgeVariant::Draft,
        "已驳回" => BadgeVariant::Archived,
        _ => BadgeVariant::Neutral,
    }
}
