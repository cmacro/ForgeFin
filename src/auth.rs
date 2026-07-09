use std::sync::LazyLock;

use leptos::prelude::*;

use crate::ipc::{self, CompanyBrief, CurrentUser, UserInfo};

/// 全局会话状态(进程内单例)。
///
/// - `user`: 当前登录用户,`None` 表示未登录(显示登录页)。
/// - `company_id`: 当前选中公司,`None` 表示未选择(显示账套切换提示)。
/// - `available_companies`: 当前用户可访问的公司列表。
pub struct Session;

static SESSION: LazyLock<RwSignal<Option<UserInfo>>> = LazyLock::new(|| RwSignal::new(None));
static COMPANY_ID: LazyLock<RwSignal<Option<String>>> = LazyLock::new(|| RwSignal::new(None));
static AVAILABLE: LazyLock<RwSignal<Vec<CompanyBrief>>> =
    LazyLock::new(|| RwSignal::new(Vec::new()));
static LOADING: LazyLock<RwSignal<bool>> = LazyLock::new(|| RwSignal::new(true));

impl Session {
    pub fn user() -> ReadSignal<Option<UserInfo>> {
        SESSION.read_only()
    }

    pub fn company_id() -> ReadSignal<Option<String>> {
        COMPANY_ID.read_only()
    }

    pub fn available_companies() -> ReadSignal<Vec<CompanyBrief>> {
        AVAILABLE.read_only()
    }

    pub fn loading() -> ReadSignal<bool> {
        LOADING.read_only()
    }

    pub fn set_user(user: Option<UserInfo>) {
        SESSION.set(user);
    }

    pub fn set_available(companies: Vec<CompanyBrief>) {
        AVAILABLE.set(companies);
    }

    pub fn set_company(id: Option<String>) {
        COMPANY_ID.set(id);
    }

    pub fn finish_loading() {
        LOADING.set(false);
    }

    pub fn is_logged_in() -> bool {
        SESSION.read().is_some()
    }

    pub fn has_company() -> bool {
        COMPANY_ID.read().is_some()
    }

    /// 启动时从后端恢复会话。
    pub async fn init() {
        match ipc::current_user().await {
            Ok(CurrentUser {
                user,
                company_id,
                available_companies,
            }) => {
                SESSION.set(user);
                COMPANY_ID.set(company_id);
                AVAILABLE.set(available_companies);
            }
            Err(e) => {
                leptos::logging::warn!("会话初始化失败: {e}");
            }
        }
        LOADING.set(false);
    }

    /// 登录成功后写入会话。
    pub async fn login(username: String, password: String) -> Result<(), String> {
        let res = ipc::login(username, password).await?;
        let companies = res.companies.clone();
        SESSION.set(Some(res.user));
        AVAILABLE.set(companies.clone());
        // 自动选第一个公司(若有)。
        if let Some(first) = companies.first() {
            let id = first.id.clone();
            ipc::set_current_company(id.clone()).await?;
            COMPANY_ID.set(Some(id));
        }
        Ok(())
    }

    pub async fn logout() -> Result<(), String> {
        ipc::logout().await?;
        SESSION.set(None);
        COMPANY_ID.set(None);
        AVAILABLE.set(Vec::new());
        Ok(())
    }

    pub async fn switch_company(id: String) -> Result<(), String> {
        ipc::set_current_company(id.clone()).await?;
        COMPANY_ID.set(Some(id));
        Ok(())
    }
}
