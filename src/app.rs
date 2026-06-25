use leptos::prelude::*;

use crate::components::layout::shell::AppShell;
use crate::nav::{NavKey, NavState};
use crate::pages::dashboard::Dashboard;
use crate::pages::general_ledger::GeneralLedger;
use crate::pages::placeholder::Placeholder;
use crate::pages::voucher::VoucherManagement;

#[component]
pub fn App() -> impl IntoView {
    let nav = NavState::new();
    let active = nav.active;

    let children = move || {
        let key = active.get();
        view! {
            {match key {
                NavKey::Home => view! { <Dashboard /> }.into_any(),
                NavKey::VoucherManagement
                | NavKey::VoucherEntry
                | NavKey::VoucherAudit
                | NavKey::VoucherQuery => view! { <VoucherManagement /> }.into_any(),
                NavKey::AccountBalance => view! { <Placeholder title="科目余额" /> }.into_any(),
                NavKey::DetailedLedger => view! { <Placeholder title="明细账" /> }.into_any(),
                NavKey::GeneralLedger => view! { <GeneralLedger /> }.into_any(),
                NavKey::TrialBalance => view! { <Placeholder title="试算平衡表" /> }.into_any(),
                NavKey::ReportCenter => view! { <Placeholder title="报表中心" /> }.into_any(),
                NavKey::AccountsReceivable => view! { <Placeholder title="应收管理" /> }.into_any(),
                NavKey::AccountsPayable => view! { <Placeholder title="应付管理" /> }.into_any(),
                NavKey::FixedAssets => view! { <Placeholder title="固定资产" /> }.into_any(),
                NavKey::CashierManagement => view! { <Placeholder title="出纳管理" /> }.into_any(),
                NavKey::BudgetManagement => view! { <Placeholder title="预算管理" /> }.into_any(),
                NavKey::TaxManagement => view! { <Placeholder title="税务管理" /> }.into_any(),
                NavKey::SystemSettings => view! { <Placeholder title="系统设置" /> }.into_any(),
            }}
        }
    };

    view! {
        <AppShell nav=nav.clone()>
            {children}
        </AppShell>
    }
}
