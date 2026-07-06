use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NavKey {
    Home,
    VoucherManagement,
    VoucherEntry,
    VoucherAudit,
    VoucherQuery,
    ChartOfAccounts,
    Contacts,
    AccountBalance,
    DetailedLedger,
    GeneralLedger,
    TrialBalance,
    ReportCenter,
    AccountsReceivable,
    AccountsPayable,
    FixedAssets,
    CashierManagement,
    BudgetManagement,
    TaxManagement,
    SystemSettings,
}

#[derive(Clone, Debug)]
pub struct NavItem {
    pub key: NavKey,
    pub label: &'static str,
    pub icon: &'static str,
    #[allow(dead_code)]
    pub route: &'static str,
    pub children: Option<Vec<NavItem>>,
}

pub fn nav_tree() -> Vec<NavItem> {
    vec![
        NavItem {
            key: NavKey::Home,
            label: "首页",
            icon: "home",
            route: "/",
            children: None,
        },
        NavItem {
            key: NavKey::VoucherManagement,
            label: "总账",
            icon: "book",
            route: "/general-ledger",
            children: Some(vec![
                NavItem {
                    key: NavKey::VoucherManagement,
                    label: "凭证管理",
                    icon: "file",
                    route: "/general-ledger/voucher",
                    children: None,
                },
                NavItem {
                    key: NavKey::VoucherEntry,
                    label: "凭证录入",
                    icon: "file-plus",
                    route: "/general-ledger/voucher/entry",
                    children: None,
                },
                NavItem {
                    key: NavKey::VoucherAudit,
                    label: "凭证审核",
                    icon: "check-square",
                    route: "/general-ledger/voucher/audit",
                    children: None,
                },
                NavItem {
                    key: NavKey::VoucherQuery,
                    label: "凭证查询",
                    icon: "search",
                    route: "/general-ledger/voucher/query",
                    children: None,
                },
                NavItem {
                    key: NavKey::ChartOfAccounts,
                    label: "会计科目",
                    icon: "scale",
                    route: "/general-ledger/accounts",
                    children: None,
                },
                NavItem {
                    key: NavKey::Contacts,
                    label: "客户/供应商",
                    icon: "users",
                    route: "/contacts",
                    children: None,
                },
                NavItem {
                    key: NavKey::AccountBalance,
                    label: "科目余额",
                    icon: "scale",
                    route: "/general-ledger/account-balance",
                    children: None,
                },
                NavItem {
                    key: NavKey::DetailedLedger,
                    label: "明细账",
                    icon: "list",
                    route: "/general-ledger/detailed-ledger",
                    children: None,
                },
                NavItem {
                    key: NavKey::GeneralLedger,
                    label: "总账",
                    icon: "book-open",
                    route: "/general-ledger/general",
                    children: None,
                },
                NavItem {
                    key: NavKey::TrialBalance,
                    label: "试算平衡表",
                    icon: "bar-chart",
                    route: "/general-ledger/trial-balance",
                    children: None,
                },
            ]),
        },
        NavItem {
            key: NavKey::ReportCenter,
            label: "报表中心",
            icon: "report",
            route: "/reports",
            children: None,
        },
        NavItem {
            key: NavKey::AccountsReceivable,
            label: "应收管理",
            icon: "download",
            route: "/accounts-receivable",
            children: None,
        },
        NavItem {
            key: NavKey::AccountsPayable,
            label: "应付管理",
            icon: "upload",
            route: "/accounts-payable",
            children: None,
        },
        NavItem {
            key: NavKey::FixedAssets,
            label: "固定资产",
            icon: "building",
            route: "/fixed-assets",
            children: None,
        },
        NavItem {
            key: NavKey::CashierManagement,
            label: "出纳管理",
            icon: "wallet",
            route: "/cashier",
            children: None,
        },
        NavItem {
            key: NavKey::BudgetManagement,
            label: "预算管理",
            icon: "target",
            route: "/budget",
            children: None,
        },
        NavItem {
            key: NavKey::TaxManagement,
            label: "税务管理",
            icon: "receipt",
            route: "/tax",
            children: None,
        },
        NavItem {
            key: NavKey::SystemSettings,
            label: "系统设置",
            icon: "settings",
            route: "/settings",
            children: None,
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavState {
    pub active: RwSignal<NavKey>,
    pub collapsed: RwSignal<bool>,
    pub expanded: RwSignal<Vec<NavKey>>,
}

impl NavState {
    pub fn new() -> Self {
        let active = RwSignal::new(NavKey::VoucherManagement);
        let collapsed = RwSignal::new(false);
        let expanded = RwSignal::new(vec![NavKey::VoucherManagement]);
        Self {
            active,
            collapsed,
            expanded,
        }
    }

    pub fn toggle_collapse(&self) {
        self.collapsed.update(|v| *v = !*v);
    }

    pub fn toggle_expand(&self, key: NavKey) {
        self.expanded.update(|expanded| {
            if let Some(pos) = expanded.iter().position(|k| *k == key) {
                expanded.remove(pos);
            } else {
                expanded.push(key);
            }
        });
    }

    pub fn is_expanded(&self, key: NavKey) -> bool {
        self.expanded.read().iter().any(|k| *k == key)
    }

    pub fn activate(&self, key: NavKey) {
        self.active.set(key);
    }

    pub fn is_active(&self, key: NavKey) -> bool {
        *self.active.read() == key
    }
}

impl NavKey {
    pub fn title(&self) -> &'static str {
        match self {
            NavKey::Home => "财务概览",
            NavKey::VoucherManagement
            | NavKey::VoucherEntry
            | NavKey::VoucherAudit
            | NavKey::VoucherQuery => "凭证管理",
            NavKey::ChartOfAccounts => "会计科目",
            NavKey::Contacts => "客户/供应商",
            NavKey::AccountBalance => "科目余额",
            NavKey::DetailedLedger => "明细账",
            NavKey::GeneralLedger => "总账",
            NavKey::TrialBalance => "试算平衡表",
            NavKey::ReportCenter => "报表中心",
            NavKey::AccountsReceivable => "应收管理",
            NavKey::AccountsPayable => "应付管理",
            NavKey::FixedAssets => "固定资产",
            NavKey::CashierManagement => "出纳管理",
            NavKey::BudgetManagement => "预算管理",
            NavKey::TaxManagement => "税务管理",
            NavKey::SystemSettings => "系统设置",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            NavKey::VoucherManagement
            | NavKey::VoucherEntry
            | NavKey::VoucherAudit
            | NavKey::VoucherQuery => "查看并管理所有会计凭证",
            NavKey::ChartOfAccounts => "维护本公司会计科目表(按账套隔离)",
            NavKey::Contacts => "维护客户与供应商基础信息(按账套隔离)",
            NavKey::AccountBalance => "查看各科目期初余额、本期发生额及期末余额",
            NavKey::DetailedLedger => "查看各科目的明细账目记录",
            NavKey::GeneralLedger => "查看总账科目汇总数据",
            NavKey::TrialBalance => "检查科目借贷是否平衡",
            NavKey::ReportCenter => "生成各类财务报表",
            NavKey::AccountsReceivable => "管理应收账款及相关业务",
            NavKey::AccountsPayable => "管理应付账款及相关业务",
            NavKey::SystemSettings => "账套、用户与备份管理",
            _ => "",
        }
    }
}

impl Default for NavState {
    fn default() -> Self {
        Self::new()
    }
}
