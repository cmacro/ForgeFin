use std::sync::Mutex;

use crate::db::system::User;

/// 简易会话状态: 进程内单用户会话(桌面应用,本地运行)。
///
/// 当前选中公司 ID 也存放于此,所有业务命令据此路由数据库。
pub struct SessionState {
    pub user: Mutex<Option<User>>,
    pub company_id: Mutex<Option<String>>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            user: Mutex::new(None),
            company_id: Mutex::new(None),
        }
    }

    pub fn set_user(&self, user: User) {
        if let Ok(mut g) = self.user.lock() {
            *g = Some(user);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.user.lock() {
            *g = None;
        }
        if let Ok(mut g) = self.company_id.lock() {
            *g = None;
        }
    }

    pub fn set_company(&self, company_id: String) {
        if let Ok(mut g) = self.company_id.lock() {
            *g = Some(company_id);
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize)]
pub struct AuthResult {
    pub user: UserInfo,
    pub companies: Vec<CompanyBrief>,
}

#[derive(serde::Serialize, Clone)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub department: Option<String>,
    pub is_admin: bool,
}

impl From<&User> for UserInfo {
    fn from(u: &User) -> Self {
        Self {
            id: u.id.clone(),
            username: u.username.clone(),
            display_name: u.display_name.clone(),
            department: u.department.clone(),
            is_admin: u.is_admin,
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct CompanyBrief {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub fn login_cmd(
    db: tauri::State<'_, std::sync::Mutex<crate::db::DbState>>,
    session: tauri::State<'_, std::sync::Mutex<SessionState>>,
    username: String,
    password: String,
) -> Result<AuthResult, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    let user = crate::db::system::login(conn, &username, &password)?;
    // 加载可访问公司
    let company_ids = crate::db::system::user_company_ids(conn, &user.id)?;
    let mut companies = Vec::new();
    for cid in &company_ids {
        if let Ok(Some(c)) = crate::db::company_reg::get_company(conn, cid) {
            companies.push(CompanyBrief {
                id: c.id,
                name: c.name,
            });
        }
    }
    // admin 可看到所有公司
    if user.is_admin {
        companies = crate::db::company_reg::list_companies(conn)?
            .into_iter()
            .map(|c| CompanyBrief {
                id: c.id,
                name: c.name,
            })
            .collect();
    }
    let info = UserInfo::from(&user);
    session
        .lock()
        .map_err(|e| format!("会话锁失败: {e}"))?
        .set_user(user);
    Ok(AuthResult {
        user: info,
        companies,
    })
}

#[tauri::command]
pub fn logout_cmd(session: tauri::State<'_, std::sync::Mutex<SessionState>>) -> Result<(), String> {
    session
        .lock()
        .map_err(|e| format!("会话锁失败: {e}"))?
        .clear();
    Ok(())
}

#[derive(serde::Serialize)]
pub struct CurrentUser {
    pub user: Option<UserInfo>,
    pub company_id: Option<String>,
    pub available_companies: Vec<CompanyBrief>,
}

#[tauri::command]
pub fn current_user_cmd(
    db: tauri::State<'_, std::sync::Mutex<crate::db::DbState>>,
    session: tauri::State<'_, std::sync::Mutex<SessionState>>,
) -> Result<CurrentUser, String> {
    let session_guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
    let user = session_guard.user.lock().ok().and_then(|g| g.clone());
    let company_id = session_guard.company_id.lock().ok().and_then(|g| g.clone());
    drop(session_guard);

    let mut available = Vec::new();
    if let Some(u) = &user {
        let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
        let sys_guard = guard.system()?;
        let conn = sys_guard
            .as_ref()
            .ok_or_else(|| "系统库未初始化".to_string())?;
        if u.is_admin {
            available = crate::db::company_reg::list_companies(conn)?
                .into_iter()
                .map(|c| CompanyBrief {
                    id: c.id,
                    name: c.name,
                })
                .collect();
        } else {
            let ids = crate::db::system::user_company_ids(conn, &u.id)?;
            for cid in ids {
                if let Ok(Some(c)) = crate::db::company_reg::get_company(conn, &cid) {
                    available.push(CompanyBrief {
                        id: c.id,
                        name: c.name,
                    });
                }
            }
        }
    }

    Ok(CurrentUser {
        user: user.as_ref().map(UserInfo::from),
        company_id,
        available_companies: available,
    })
}

#[tauri::command]
pub fn set_current_company_cmd(
    session: tauri::State<'_, std::sync::Mutex<SessionState>>,
    company_id: String,
) -> Result<(), String> {
    session
        .lock()
        .map_err(|e| format!("会话锁失败: {e}"))?
        .set_company(company_id);
    Ok(())
}
