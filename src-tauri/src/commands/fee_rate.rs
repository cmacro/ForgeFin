use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::raw::{now_str, with_company_conn};
use crate::commands::session::SessionState;
use crate::db::DbState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeRate {
    pub id: i64,
    pub payment_method: String,
    pub rate: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeRateInput {
    pub payment_method: String,
    pub rate: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

pub fn list_fee_rates_core(conn: &rusqlite::Connection) -> Result<Vec<FeeRate>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, payment_method, rate, description, is_active, created_at, updated_at
             FROM fee_rates ORDER BY id",
        )
        .map_err(|e| format!("查询手续费率失败: {e}"))?;
    let items = stmt
        .query_map([], |row| {
            Ok(FeeRate {
                id: row.get(0)?,
                payment_method: row.get(1)?,
                rate: row.get(2)?,
                description: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询手续费率失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询手续费率失败: {e}"))?;
    Ok(items)
}

pub fn update_fee_rate_core(
    conn: &rusqlite::Connection,
    id: i64,
    input: &FeeRateInput,
) -> Result<FeeRate, String> {
    let now = now_str();
    conn.execute(
        "UPDATE fee_rates
         SET payment_method = ?1, rate = ?2, description = ?3, is_active = ?4, updated_at = ?5
         WHERE id = ?6",
        rusqlite::params![
            input.payment_method,
            input.rate,
            input.description,
            input.is_active.unwrap_or(true) as i32,
            now,
            id,
        ],
    )
    .map_err(|e| format!("更新手续费率失败: {e}"))?;

    let record = conn
        .query_row(
            "SELECT id, payment_method, rate, description, is_active, created_at, updated_at
             FROM fee_rates WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(FeeRate {
                    id: row.get(0)?,
                    payment_method: row.get(1)?,
                    rate: row.get(2)?,
                    description: row.get(3)?,
                    is_active: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("查询手续费率失败: {e}"))?;
    record.ok_or_else(|| "手续费率更新后查询失败".to_string())
}

#[tauri::command]
pub fn list_fee_rates_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
) -> Result<Vec<FeeRate>, String> {
    with_company_conn(&db, &session, |conn| list_fee_rates_core(conn))
}

#[tauri::command]
pub fn update_fee_rate_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
    input: FeeRateInput,
) -> Result<FeeRate, String> {
    with_company_conn(&db, &session, |conn| update_fee_rate_core(conn, id, &input))
}
