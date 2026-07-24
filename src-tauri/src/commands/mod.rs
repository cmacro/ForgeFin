pub mod account;
pub mod backup;
pub mod company;
pub mod contact;
pub mod raw;
pub mod session;
pub mod user;
pub mod voucher;

use std::sync::Mutex;

use crate::db::DbState;

#[allow(dead_code)]
/// 获取 DbState 的辅助函数,从 Tauri State 中取出锁。
pub fn db_state<'a>(
    state: &'a tauri::State<'a, Mutex<DbState>>,
) -> Result<std::sync::MutexGuard<'a, DbState>, String> {
    state.lock().map_err(|e| format!("数据库状态锁失败: {e}"))
}
