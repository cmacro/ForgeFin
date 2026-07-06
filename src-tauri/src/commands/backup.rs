use std::sync::Mutex;

use crate::commands::session::SessionState;
use crate::db::{backup, DbState};

#[tauri::command]
pub fn backup_company_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    company_id: String,
) -> Result<String, String> {
    // 备份前关闭该公司连接,确保数据刷盘。
    db.lock()
        .map_err(|e| format!("数据库锁失败: {e}"))?
        .close_company(&company_id);
    let path = backup::backup_company(&company_id)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn backup_system_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<String, String> {
    // 全系统备份前关闭所有公司连接并关闭系统库。
    let guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    guard.close_all_companies();
    if let Ok(mut sys_guard) = guard.system() {
        *sys_guard = None;
    }
    let path = backup::backup_system()?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn list_backups_cmd(
    _db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
) -> Result<Vec<backup::BackupEntry>, String> {
    backup::list_backups()
}

#[tauri::command]
pub fn restore_company_cmd(
    db: tauri::State<'_, Mutex<DbState>>,
    _session: tauri::State<'_, Mutex<SessionState>>,
    company_id: String,
    backup_path: String,
    confirm: bool,
) -> Result<(), String> {
    if !confirm {
        return Err("恢复操作必须二次确认(confirm=true)".to_string());
    }
    let path = std::path::PathBuf::from(&backup_path);
    db.lock()
        .map_err(|e| format!("数据库锁失败: {e}"))?
        .close_company(&company_id);
    backup::restore_company_from_backup(&company_id, &path)
}
