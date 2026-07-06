use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::company::{self, Account, AccountInput};
use crate::db::DbState;

fn require_company(
    db: &tauri::State<'_, Mutex<DbState>>,
    session: &tauri::State<'_, Mutex<SessionState>>,
) -> Result<String, String> {
    let session_guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
    let company_id = session_guard
        .company_id
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .ok_or_else(|| "未选择公司,请先切换账套".to_string())?;
    drop(session_guard);
    // 校验库存在(惰性建库)
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let _guard = guard.company(&company_id)?;
    Ok(company_id)
}

#[tauri::command]
pub fn list_accounts_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<Vec<Account>, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::list_accounts(conn)
}

#[tauri::command]
pub fn create_account_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    input: AccountInput,
) -> Result<Account, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::create_account(conn, &input)
}

#[tauri::command]
pub fn update_account_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
    input: AccountInput,
) -> Result<Account, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::update_account(conn, &id, &input)
}

#[tauri::command]
pub fn delete_account_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
) -> Result<(), String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::delete_account(conn, &id)
}
