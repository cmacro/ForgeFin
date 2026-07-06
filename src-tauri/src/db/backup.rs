use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono::Utc;

/// 生成备份文件名: `forgefin_company_{id}_YYYYMMDD_HHMMSS.db`。
pub fn backup_company_path(company_id: &str) -> Result<PathBuf, String> {
    let dir = super::backups_dir()?;
    let now = Utc::now();
    let stamp = now.format("%Y%m%d_%H%M%S").to_string();
    Ok(dir.join(format!("forgefin_company_{company_id}_{stamp}.db")))
}

pub fn system_backup_path() -> Result<PathBuf, String> {
    let dir = super::backups_dir()?;
    let now = Utc::now();
    let stamp = now.format("%Y%m%d_%H%M%S").to_string();
    Ok(dir.join(format!("forgefin_system_{stamp}.db")))
}

/// 复制单文件(同步),保留元数据失败时回退普通复制。
fn copy_file(src: &Path, dst: &Path) -> Result<(), String> {
    fs::copy(src, dst)
        .map_err(|e| format!("复制文件失败 ({} -> {}): {e}", src.display(), dst.display()))?;
    Ok(())
}

/// 删除 WAL/SHM 副本对应的辅助文件(若存在)。
fn cleanup_aux(dst: &Path) {
    for ext in ["-wal", "-shm"] {
        let p = dst.with_extension(format!("db{}", ext));
        if p.exists() {
            fs::remove_file(&p).ok();
        }
    }
}

/// 备份单个公司数据库。
pub fn backup_company(company_id: &str) -> Result<PathBuf, String> {
    let src = super::company_db_path(company_id)?;
    if !src.exists() {
        return Err("公司数据库文件不存在".to_string());
    }
    let dst = backup_company_path(company_id)?;
    copy_file(&src, &dst)?;
    cleanup_aux(&dst);
    Ok(dst)
}

/// 备份系统库。
pub fn backup_system() -> Result<PathBuf, String> {
    let src = super::system_db_path()?;
    if !src.exists() {
        return Err("系统库文件不存在".to_string());
    }
    let dst = system_backup_path()?;
    copy_file(&src, &dst)?;
    cleanup_aux(&dst);
    Ok(dst)
}

/// 列出所有备份文件(按时间倒序)。
pub fn list_backups() -> Result<Vec<BackupEntry>, String> {
    let dir = super::backups_dir()?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("读取备份目录失败: {e}"))? {
        let entry = entry.map_err(|e| format!("读取备份条目失败: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if !name.ends_with(".db") {
                continue;
            }
            let meta = fs::metadata(&path).map_err(|e| format!("读取备份元数据失败: {e}"))?;
            let size = meta.len() as i64;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            entries.push(BackupEntry {
                path: path.display().to_string(),
                name: name.to_string(),
                size,
                modified,
            });
        }
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(entries)
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct BackupEntry {
    pub path: String,
    pub name: String,
    pub size: i64,
    pub modified: i64,
}

/// 从备份恢复公司库。
///
/// 恢复前必须先关闭该公司连接(`DbState::close_company`)。
/// 此操作会覆盖现有数据,调用方需确保二次确认。
pub fn restore_company_from_backup(company_id: &str, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err("备份文件不存在".to_string());
    }
    let dst = super::company_db_path(company_id)?;
    // 删除现有文件及 WAL/SHM
    if dst.exists() {
        fs::remove_file(&dst).map_err(|e| format!("删除现有库失败: {e}"))?;
    }
    for ext in ["-wal", "-shm"] {
        let p = dst.with_extension(format!("db{}", ext));
        if p.exists() {
            fs::remove_file(&p).ok();
        }
    }
    copy_file(backup_path, &dst)?;
    cleanup_aux(&dst);
    Ok(())
}
