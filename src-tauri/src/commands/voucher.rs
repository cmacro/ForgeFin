use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::company::{self, AuditLog, Voucher, VoucherEntry, VoucherFilter, VoucherInput};
use crate::db::DbState;

fn require_company(
    db: &tauri::State<'_, Mutex<DbState>>,
    session: &tauri::State<'_, Mutex<SessionState>>,
) -> Result<(String, Option<crate::db::system::User>), String> {
    let session_guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
    let user = session_guard.user.lock().ok().and_then(|g| g.clone());
    let company_id = session_guard
        .company_id
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .ok_or_else(|| "未选择公司,请先切换账套".to_string())?;
    drop(session_guard);
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let _guard = guard.company(&company_id)?;
    Ok((company_id, user))
}

#[tauri::command]
pub fn create_voucher_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    input: VoucherInput,
) -> Result<Voucher, String> {
    let (company_id, user) = require_company(&db, &session)?;
    let mut input = input;
    if let Some(u) = &user {
        if input.operator_id.is_none() {
            input.operator_id = Some(u.id.clone());
        }
        if input.operator_name.is_none() {
            input.operator_name = Some(u.display_name.clone());
        }
    }
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::create_voucher(conn, &input)
}

#[derive(serde::Serialize)]
pub struct VoucherPage {
    pub items: Vec<Voucher>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[tauri::command]
pub fn list_vouchers_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    filter: VoucherFilter,
) -> Result<VoucherPage, String> {
    let company_id = require_company(&db, &session)?.0;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    let page = company::list_vouchers(conn, &filter)?;
    Ok(VoucherPage {
        items: page.items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    })
}

#[derive(serde::Serialize)]
pub struct VoucherDetail {
    pub voucher: Voucher,
    pub entries: Vec<VoucherEntry>,
    pub audit_logs: Vec<AuditLog>,
}

#[tauri::command]
pub fn get_voucher_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
) -> Result<VoucherDetail, String> {
    let company_id = require_company(&db, &session)?.0;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    let voucher = company::get_voucher(conn, &id)?.ok_or_else(|| "凭证不存在".to_string())?;
    let entries = company::list_voucher_entries(conn, &id)?;
    let audit_logs = company::list_audit_logs(conn, &id)?;
    Ok(VoucherDetail {
        voucher,
        entries,
        audit_logs,
    })
}

#[tauri::command]
pub fn delete_voucher_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
) -> Result<(), String> {
    let company_id = require_company(&db, &session)?.0;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::delete_voucher(conn, &id)
}

#[tauri::command]
pub fn audit_voucher_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    id: String,
    comment: Option<String>,
) -> Result<Voucher, String> {
    let (company_id, user) = require_company(&db, &session)?;
    let operator_id = user.as_ref().map(|u| u.id.clone());
    let operator_name = user.as_ref().map(|u| u.display_name.clone());
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::audit_voucher(
        conn,
        &id,
        operator_id.as_deref(),
        operator_name.as_deref(),
        comment.as_deref(),
    )
}

#[tauri::command]
pub fn list_audit_logs_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    voucher_id: String,
) -> Result<Vec<AuditLog>, String> {
    let company_id = require_company(&db, &session)?.0;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::list_audit_logs(conn, &voucher_id)
}

#[tauri::command]
pub fn next_voucher_no_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    session: tauri::State<'_, Mutex<SessionState>>,
    voucher_type: String,
    voucher_date: String,
) -> Result<String, String> {
    let company_id = require_company(&db, &session)?.0;
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let mut companies_guard = guard.company(&company_id)?;
    let conn = companies_guard
        .get_mut(&company_id)
        .ok_or_else(|| "公司库未初始化".to_string())?;
    company::next_voucher_no(conn, &voucher_type, &voucher_date)
}
