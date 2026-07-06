use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::company_reg::{self, Company, CompanyInput};
use crate::db::DbState;

#[tauri::command]
pub fn list_companies_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<Vec<Company>, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    company_reg::list_companies(conn)
}

#[tauri::command]
pub fn create_company_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    input: CompanyInput,
) -> Result<Company, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    let company = company_reg::create_company(conn, &input)?;
    // 创建公司后,初始化其独立数据库文件(惰性触发即可,这里主动建一次)。
    let _companies_guard = guard.company(&company.id)?;
    // 授予当前用户对该公司管理权限(admin 自动拥有;非 admin 也授予基础权限)。
    if let Some(user) = session
        .lock()
        .map_err(|e| format!("会话锁失败: {e}"))?
        .user
        .lock()
        .ok()
        .and_then(|g| g.clone())
    {
        crate::db::system::grant_permission(
            conn,
            &user.id,
            &company.id,
            "accountant",
            !user.is_admin,
            !user.is_admin,
            !user.is_admin,
            !user.is_admin,
        )?;
    }
    Ok(company)
}

#[tauri::command]
pub fn update_company_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
    input: CompanyInput,
) -> Result<Company, String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    company_reg::update_company(conn, &id, &input)
}

#[tauri::command]
pub fn delete_company_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
) -> Result<(), String> {
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    // 先关闭该公司连接,再删库文件,最后删系统库记录。
    guard.close_company(&id);
    company_reg::delete_company_db(&id)?;
    let sys_guard = guard.system()?;
    let conn = sys_guard
        .as_ref()
        .ok_or_else(|| "系统库未初始化".to_string())?;
    company_reg::delete_company(conn, &id)
}
