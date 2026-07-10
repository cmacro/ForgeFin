pub mod backup;
pub mod company;
pub mod company_reg;
pub mod schema;
pub mod system;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

/// 应用数据根目录。
///
/// 统一放置系统库与各公司独立库：
/// - `<data_dir>/forgefin_system.db`
/// - `<data_dir>/companies/forgefin_company_{id}.db`
/// - `<data_dir>/backups/...`
pub fn data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "无法定位应用数据目录".to_string())?;
    let dir = base.join("ForgeFin");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    Ok(dir)
}

pub fn companies_dir() -> Result<PathBuf, String> {
    let dir = data_dir()?.join("companies");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建公司目录失败: {e}"))?;
    }
    Ok(dir)
}

pub fn backups_dir() -> Result<PathBuf, String> {
    let dir = data_dir()?.join("backups");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建备份目录失败: {e}"))?;
    }
    Ok(dir)
}

pub fn system_db_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("forgefin_system.db"))
}

/// 公司库文件路径。
///
/// 命名规则: `forgefin_company_{id}.db`,id 为不带连字符的 UUID 字符串。
pub fn company_db_path(company_id: &str) -> Result<PathBuf, String> {
    Ok(companies_dir()?.join(format!("forgefin_company_{company_id}.db")))
}

/// 打开并启用外键与 WAL。
fn open_connection(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败: {e}"))?;
        }
    }
    let conn =
        Connection::open(path).map_err(|e| format!("打开数据库失败 ({}): {e}", path.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|e| format!("设置 PRAGMA 失败: {e}"))?;
    Ok(conn)
}

/// 进程内数据库连接管理器。
///
/// - `system`: 系统库单例,惰性打开。
/// - `companies`: 各公司库连接缓存,key 为 company_id。
///
/// 所有连接使用 `Mutex` 保证线程安全,不引入连接池。
pub struct DbState {
    system: Mutex<Option<Connection>>,
    companies: Mutex<HashMap<String, Connection>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            system: Mutex::new(None),
            companies: Mutex::new(HashMap::new()),
        }
    }

    /// 获取系统库连接(惰性初始化并建表)。
    pub fn system(&self) -> Result<MutexGuard<'_, Option<Connection>>, String> {
        let mut guard = self
            .system
            .lock()
            .map_err(|e| format!("系统库锁获取失败: {e}"))?;
        if guard.is_none() {
            let path = system_db_path()?;
            let conn = open_connection(&path)?;
            schema::init_system(&conn)?;
            *guard = Some(conn);
        }
        Ok(guard)
    }

    /// 获取指定公司的库连接(惰性初始化并建表)。
    pub fn company(
        &self,
        company_id: &str,
    ) -> Result<MutexGuard<'_, HashMap<String, Connection>>, String> {
        let mut guard = self
            .companies
            .lock()
            .map_err(|e| format!("公司库锁获取失败: {e}"))?;
        if !guard.contains_key(company_id) {
            let path = company_db_path(company_id)?;
            let conn = open_connection(&path)?;
            schema::init_company(&conn)?;
            guard.insert(company_id.to_string(), conn);
        }
        Ok(guard)
    }

    /// 关闭某个公司连接(切换公司或备份时调用)。
    pub fn close_company(&self, company_id: &str) {
        if let Ok(mut guard) = self.companies.lock() {
            guard.remove(company_id);
        }
    }

    /// 关闭所有公司连接(全系统备份前调用)。
    pub fn close_all_companies(&self) {
        if let Ok(mut guard) = self.companies.lock() {
            guard.clear();
        }
    }

    /// 关闭所有连接并 checkpoint WAL。
    pub fn close_all(&self) {
        if let Ok(mut guard) = self.companies.lock() {
            for (_, conn) in guard.drain() {
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            }
        }
        if let Ok(mut guard) = self.system.lock() {
            if let Some(conn) = guard.take() {
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            }
        }
    }
}

impl Default for DbState {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DbState {
    fn drop(&mut self) {
        self.close_all();
    }
}
