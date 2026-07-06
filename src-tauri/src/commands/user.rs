use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::DbState;
use crate::db::{company_reg, system};

#[tauri::command]
pub fn list_users_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<Vec<system::User>, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    system::list_users(conn)
}

#[derive(serde::Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub department: Option<String>,
    pub is_admin: Option<bool>,
}

#[tauri::command]
pub fn create_user_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    input: CreateUserInput,
) -> Result<system::User, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    system::create_user(
        conn,
        &input.username,
        &input.display_name,
        &input.password,
        input.department.as_deref(),
        input.is_admin.unwrap_or(false),
    )
}

#[derive(serde::Deserialize)]
pub struct GrantPermissionInput {
    pub user_id: String,
    pub company_id: String,
    pub role: String,
    pub can_audit: Option<bool>,
    pub can_post: Option<bool>,
    pub can_manage: Option<bool>,
    pub can_backup: Option<bool>,
}

#[tauri::command]
pub fn grant_permission_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    input: GrantPermissionInput,
) -> Result<system::UserCompanyPermission, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    system::grant_permission(
        conn,
        &input.user_id,
        &input.company_id,
        &input.role,
        input.can_audit.unwrap_or(false),
        input.can_post.unwrap_or(false),
        input.can_manage.unwrap_or(false),
        input.can_backup.unwrap_or(false),
    )
}

/// 当前用户可访问的公司列表(也包含公司详情)。
#[tauri::command]
pub fn user_companies_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<Vec<company_reg::Company>, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    let session_guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
    let user = session_guard.user.lock().ok().and_then(|g| g.clone());
    drop(session_guard);
    let user = user.ok_or_else(|| "未登录".to_string())?;
    if user.is_admin {
        return company_reg::list_companies(conn);
    }
    let ids = system::user_company_ids(conn, &user.id)?;
    let mut out = Vec::new();
    for id in ids {
        if let Ok(Some(c)) = company_reg::get_company(conn, &id) {
            out.push(c);
        }
    }
    Ok(out)
}
