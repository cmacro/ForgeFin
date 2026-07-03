use leptos::prelude::*;
use lucide_leptos::{
    Book, BookOpen, Building2, ChartNoAxesColumnIncreasing, Download, FilePlus, FileText, House,
    ListChecks, Receipt, Scale, Search, Settings, SquareCheck, Target, Upload, Wallet,
};

#[component]
pub fn Icon(name: &'static str, #[prop(default = 16)] size: usize) -> impl IntoView {
    match name {
        "home" => view! { <House size=size /> }.into_any(),
        "book" => view! { <Book size=size /> }.into_any(),
        "file" => view! { <FileText size=size /> }.into_any(),
        "file-plus" => view! { <FilePlus size=size /> }.into_any(),
        "check-square" => view! { <SquareCheck size=size /> }.into_any(),
        "search" => view! { <Search size=size /> }.into_any(),
        "scale" => view! { <Scale size=size /> }.into_any(),
        "list" => view! { <ListChecks size=size /> }.into_any(),
        "book-open" => view! { <BookOpen size=size /> }.into_any(),
        "bar-chart" => view! { <ChartNoAxesColumnIncreasing size=size /> }.into_any(),
        "report" => view! { <FileText size=size /> }.into_any(),
        "download" => view! { <Download size=size /> }.into_any(),
        "upload" => view! { <Upload size=size /> }.into_any(),
        "building" => view! { <Building2 size=size /> }.into_any(),
        "wallet" => view! { <Wallet size=size /> }.into_any(),
        "target" => view! { <Target size=size /> }.into_any(),
        "receipt" => view! { <Receipt size=size /> }.into_any(),
        "settings" => view! { <Settings size=size /> }.into_any(),
        _ => view! { <House size=size /> }.into_any(),
    }
}
