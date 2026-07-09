use leptos::prelude::*;

use crate::auth::Session;
use crate::components::layout::shell::AppShell;
use crate::nav::{NavKey, NavState};
use crate::pages::accounts::Accounts;
use crate::pages::contacts::Contacts;
use crate::pages::dashboard::Dashboard;
use crate::pages::general_ledger::GeneralLedger;
use crate::pages::login::Login;
use crate::pages::placeholder::Placeholder;
use crate::pages::settings::Settings;
use crate::pages::voucher::VoucherManagement;
use crate::pages::voucher_entry::VoucherEntry;

#[component]
pub fn App() -> impl IntoView {
    // 启动时从后端恢复会话。
    leptos::task::spawn_local(async {
        Session::init().await;
    });

    view! {
        <AppRouter />
    }
}

#[component]
fn AppRouter() -> impl IntoView {
    let loading = Session::loading();
    let user = Session::user();

    view! {
        <Show when=move || loading.get() fallback=|| view! { <LoadingScreen /> }>
            <div />
        </Show>
        <Show when=move || !loading.get() && user.get().is_none()>
            <Login />
        </Show>
        <Show when=move || !loading.get() && user.get().is_some()>
            <MainShell />
        </Show>
    }
}

#[component]
fn LoadingScreen() -> impl IntoView {
    view! {
        <div class="login-layout">
            <div class="text-tertiary text-13">"正在加载…"</div>
        </div>
    }
}

#[component]
fn MainShell() -> impl IntoView {
    let nav = NavState::new();
    let active = nav.active;
    let has_company = Session::has_company();

    let children = move || {
        if !has_company {
            return view! { <NoCompany /> }.into_any();
        }
        let key = active.get();
        view! {
            {match key {
                NavKey::Home => view! { <Dashboard /> }.into_any(),
                NavKey::VoucherManagement | NavKey::VoucherQuery => {
                    view! { <VoucherManagement /> }.into_any()
                }
                NavKey::VoucherEntry => view! { <VoucherEntry /> }.into_any(),
                NavKey::VoucherAudit => view! { <VoucherManagement audit_mode=true /> }.into_any(),
                NavKey::ChartOfAccounts => view! { <Accounts /> }.into_any(),
                NavKey::Contacts => view! { <Contacts /> }.into_any(),
                NavKey::AccountBalance => view! { <Placeholder title="科目余额" /> }.into_any(),
                NavKey::DetailedLedger => view! { <Placeholder title="明细账" /> }.into_any(),
                NavKey::GeneralLedger => view! { <GeneralLedger /> }.into_any(),
                NavKey::TrialBalance => view! { <Placeholder title="试算平衡表" /> }.into_any(),
                NavKey::ReportCenter => view! { <Placeholder title="报表中心" /> }.into_any(),
                NavKey::AccountsReceivable | NavKey::AccountsPayable => {
                    view! { <Contacts /> }.into_any()
                }
                NavKey::FixedAssets => view! { <Placeholder title="固定资产" /> }.into_any(),
                NavKey::CashierManagement => view! { <Placeholder title="出纳管理" /> }.into_any(),
                NavKey::BudgetManagement => view! { <Placeholder title="预算管理" /> }.into_any(),
                NavKey::TaxManagement => view! { <Placeholder title="税务管理" /> }.into_any(),
                NavKey::SystemSettings => view! { <Settings /> }.into_any(),
            }}
        }
        .into_any()
    };

    view! {
        <AppShell nav=nav.clone()>
            {children}
        </AppShell>
    }
}

#[component]
fn NoCompany() -> impl IntoView {
    view! {
        <div class="empty-state">
            <h2 class="empty-state-title">"尚未选择账套"</h2>
            <p class="empty-state-desc">
                "请在顶栏选择一个账套,或前往系统设置创建新账套。所有业务数据按账套隔离。"
            </p>
        </div>
    }
}
