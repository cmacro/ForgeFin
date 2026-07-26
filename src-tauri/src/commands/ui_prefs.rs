use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::raw::{now_str, with_company_conn};
use crate::commands::session::SessionState;
use crate::db::DbState;

/// 单个来源类型下的列显示偏好。
///
/// `columns` 字典的 key 是稳定列标识(见 [`COLUMN_KEYS`])，
/// value 表示该列是否在工具条/表格中可见。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ColumnPrefs {
    pub source_type: String,
    pub columns: std::collections::BTreeMap<String, bool>,
}

/// 稳定列标识清单(也供前端 import 引用)。
pub const COLUMN_KEYS: &[&str] = &[
    "source_type",
    "source_file_name",
    "source_row_no",
    "record_no",
    "record_date",
    "amount_total",
    "balance",
    "counterpart_info",
    "summary",
    "status",
];

/// 默认:全部列可见。
pub(crate) fn default_columns() -> std::collections::BTreeMap<String, bool> {
    COLUMN_KEYS
        .iter()
        .map(|k| ((*k).to_string(), true))
        .collect()
}

/// 读取某来源类型的列显示偏好。
/// 若尚无记录则返回默认值(全可见)。
pub fn get_column_prefs_core(
    conn: &rusqlite::Connection,
    source_type: &str,
) -> Result<ColumnPrefs, String> {
    let mut stmt = conn
        .prepare(
            "SELECT column_key, visible FROM ui_column_prefs
             WHERE source_type = ?1",
        )
        .map_err(|e| format!("查询列偏好失败: {e}"))?;
    let rows: Vec<(String, bool)> = stmt
        .query_map(rusqlite::params![source_type], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
        })
        .map_err(|e| format!("查询列偏好失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询列偏好失败: {e}"))?;

    if rows.is_empty() {
        return Ok(ColumnPrefs {
            source_type: source_type.to_string(),
            columns: default_columns(),
        });
    }

    // 以默认全集为基,DB 记录覆盖之(新增列会自动可见,不会因 DB 没记录而隐藏)
    let mut columns = default_columns();
    for (k, v) in rows {
        columns.insert(k, v);
    }
    Ok(ColumnPrefs {
        source_type: source_type.to_string(),
        columns,
    })
}

/// 整体替换某来源类型的列显示偏好。
/// `columns` 只需列出用户关心的列,其它列保持当前可见状态;若不指定 `false` 显式写入则不会被关闭。
/// 为简化实现,本接口采用"全量提交":传入的 `columns` 即权威值,未在 `COLUMN_KEYS` 中出现的 key 会被忽略。
pub fn save_column_prefs_core(
    conn: &rusqlite::Connection,
    source_type: &str,
    columns: &std::collections::BTreeMap<String, bool>,
) -> Result<ColumnPrefs, String> {
    let now = now_str();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;

    // 先清空该 source_type 的所有现有记录,再插入新集合
    tx.execute(
        "DELETE FROM ui_column_prefs WHERE source_type = ?1",
        rusqlite::params![source_type],
    )
    .map_err(|e| format!("清除列偏好失败: {e}"))?;

    let mut final_columns = std::collections::BTreeMap::new();
    for key in COLUMN_KEYS {
        // 用户传入的值优先,未传入则默认可见
        let visible = columns.get(*key).copied().unwrap_or(true);
        final_columns.insert((*key).to_string(), visible);
        tx.execute(
            "INSERT INTO ui_column_prefs (source_type, column_key, visible, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![source_type, *key, visible as i64, now],
        )
        .map_err(|e| format!("写入列偏好失败: {e}"))?;
    }

    tx.commit().map_err(|e| format!("提交列偏好失败: {e}"))?;

    Ok(ColumnPrefs {
        source_type: source_type.to_string(),
        columns: final_columns,
    })
}

#[tauri::command]
pub fn get_column_prefs_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    source_type: String,
) -> Result<ColumnPrefs, String> {
    with_company_conn(&db, &session, |conn| {
        get_column_prefs_core(conn, &source_type)
    })
}

#[tauri::command]
pub fn save_column_prefs_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    source_type: String,
    columns: std::collections::BTreeMap<String, bool>,
) -> Result<ColumnPrefs, String> {
    with_company_conn(&db, &session, |conn| {
        save_column_prefs_core(conn, &source_type, &columns)
    })
}
