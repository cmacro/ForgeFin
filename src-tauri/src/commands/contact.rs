use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::company::{self, Contact, ContactInput};
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
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let _guard = guard.company(&company_id)?;
    Ok(company_id)
}

#[tauri::command]
pub fn list_contacts_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    contact_type: Option<String>,
) -> Result<Vec<Contact>, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::list_contacts(conn, contact_type.as_deref())
}

#[tauri::command]
pub fn create_contact_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    input: ContactInput,
) -> Result<Contact, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::create_contact(conn, &input)
}

#[tauri::command]
pub fn update_contact_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
    input: ContactInput,
) -> Result<Contact, String> {
    let company_id = require_company(&db, &session)?;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::update_contact(conn, &id, &input)
}

#[tauri::command]
pub fn delete_contact_cmd(
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
    company::delete_contact(conn, &id)
}
