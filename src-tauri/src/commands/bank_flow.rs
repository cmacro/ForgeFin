use std::collections::HashMap;
use std::path::Path;

use chrono::Local;
use rusqlite::OptionalExtension;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::commands::raw::{
    compute_balance_check_status, fetch_balance_check_rows, now_str, with_company_conn,
    BalanceCheckRow,
};
use crate::commands::session::SessionState;
use crate::db::DbState;

fn cents_to_yuan(cents: i64) -> String {
    let d = Decimal::new(cents, 2);
    format!("{:.2}", d)
}

// =====================================================================
// 数据结构
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankFlowRecord {
    pub id: i64,
    pub import_batch_id: i64,
    pub source_file_name: String,
    pub source_row_no: i32,
    pub record_no: Option<String>,
    pub record_date: Option<String>,
    pub amount_in: Option<String>,
    pub amount_out: Option<String>,
    pub amount_total: Option<String>,
    pub balance: Option<String>,
    pub currency: String,
    pub counterpart_info: Option<String>,
    pub summary: Option<String>,
    pub status: String,
    pub created_at: String,
    pub file_path: String,
    #[serde(default)]
    pub balance_check_status: Option<String>,
    #[serde(default)]
    pub balance_confirmed_at: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BankFlowFilter {
    pub batch_id: Option<i64>,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankFlowPage {
    pub items: Vec<BankFlowRecord>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankFlowDetail {
    pub record: BankFlowRecord,
    pub raw_data: String,
}

// =====================================================================
// 核心逻辑
// =====================================================================

pub fn list_bank_flows_core(
    conn: &rusqlite::Connection,
    filter: &BankFlowFilter,
) -> Result<BankFlowPage, String> {
    let page = filter.page.max(1);
    let page_size = filter.page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(b) = filter.batch_id {
        where_clause.push_str(" AND bf.import_batch_id = ?");
        params.push(Box::new(b));
    }

    // 余额连续性校验
    let balance_check_map: HashMap<i64, String> = {
        let rows = fetch_bank_balance_check_rows(conn)?;
        compute_balance_check_status(&rows)
    };

    let count_sql = format!("SELECT COUNT(*) FROM bank_flows bf{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计银行流水失败: {e}"))?;

    let list_sql = format!(
        "SELECT bf.id, bf.import_batch_id, bf.source_file_name, bf.source_row_no,
                bf.record_no, bf.record_date, bf.amount_in, bf.amount_out, bf.amount_total,
                bf.balance, bf.currency, bf.counterpart_info, bf.summary,
                bf.status, bf.created_at, ib.file_path, bf.balance_confirmed_at
         FROM bank_flows bf
         LEFT JOIN import_batches ib ON bf.import_batch_id = ib.id
         {where_clause}
         ORDER BY bf.record_date DESC, bf.id DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询银行流水失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            let id: i64 = row.get(0)?;
            let balance_check_status = balance_check_map.get(&id).cloned();
            let amount_in_cents: Option<i64> = row.get(6)?;
            let amount_out_cents: Option<i64> = row.get(7)?;
            let amount_total_cents: Option<i64> = row.get(8)?;
            let balance_cents: Option<i64> = row.get(9)?;
            Ok(BankFlowRecord {
                id,
                import_batch_id: row.get(1)?,
                source_file_name: row.get(2)?,
                source_row_no: row.get(3)?,
                record_no: row.get(4)?,
                record_date: row.get(5)?,
                amount_in: amount_in_cents.map(cents_to_yuan),
                amount_out: amount_out_cents.map(cents_to_yuan),
                amount_total: amount_total_cents.map(cents_to_yuan),
                balance: balance_cents.map(cents_to_yuan),
                currency: row.get(10)?,
                counterpart_info: row.get(11)?,
                summary: row.get(12)?,
                status: row.get(13)?,
                created_at: row.get(14)?,
                file_path: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                balance_check_status,
                balance_confirmed_at: row.get(16)?,
            })
        })
        .map_err(|e| format!("查询银行流水失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询银行流水失败: {e}"))?;

    Ok(BankFlowPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn get_bank_flow_core(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<BankFlowDetail>, String> {
    let record_opt = conn
        .query_row(
            "SELECT bf.id, bf.import_batch_id, bf.source_file_name, bf.source_row_no,
                    bf.record_no, bf.record_date, bf.amount_in, bf.amount_out, bf.amount_total,
                    bf.balance, bf.currency, bf.counterpart_info, bf.summary,
                    bf.status, bf.created_at, bf.raw_data, ib.file_path, bf.balance_confirmed_at
             FROM bank_flows bf
             LEFT JOIN import_batches ib ON bf.import_batch_id = ib.id
             WHERE bf.id = ?1",
            rusqlite::params![id],
            |row| {
                let amount_in_cents: Option<i64> = row.get(6)?;
                let amount_out_cents: Option<i64> = row.get(7)?;
                let amount_total_cents: Option<i64> = row.get(8)?;
                let balance_cents: Option<i64> = row.get(9)?;
                Ok((
                    BankFlowRecord {
                        id: row.get(0)?,
                        import_batch_id: row.get(1)?,
                        source_file_name: row.get(2)?,
                        source_row_no: row.get(3)?,
                        record_no: row.get(4)?,
                        record_date: row.get(5)?,
                        amount_in: amount_in_cents.map(cents_to_yuan),
                        amount_out: amount_out_cents.map(cents_to_yuan),
                        amount_total: amount_total_cents.map(cents_to_yuan),
                        balance: balance_cents.map(cents_to_yuan),
                        currency: row.get(10)?,
                        counterpart_info: row.get(11)?,
                        summary: row.get(12)?,
                        status: row.get(13)?,
                        created_at: row.get(14)?,
                        file_path: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                        balance_check_status: None,
                        balance_confirmed_at: row.get(17)?,
                    },
                    row.get::<_, String>(15)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("查询银行流水详情失败: {e}"))?;

    let Some((mut record, raw_data)) = record_opt else {
        return Ok(None);
    };

    // 余额连续性校验
    let check_rows = fetch_bank_balance_check_rows(conn)?;
    let map = compute_balance_check_status(&check_rows);
    record.balance_check_status = map.get(&record.id).cloned();

    Ok(Some(BankFlowDetail { record, raw_data }))
}

/// 从 bank_flows 表拉取余额连续性校验所需的轻量数据
fn fetch_bank_balance_check_rows(
    conn: &rusqlite::Connection,
) -> Result<Vec<BalanceCheckRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bf.id, bf.balance, bf.raw_data
             FROM bank_flows bf
             ORDER BY bf.record_date ASC, bf.id ASC",
        )
        .map_err(|e| format!("查询余额校验数据失败: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BalanceCheckRow {
                id: row.get(0)?,
                balance: row.get(1)?,
                raw_data: row.get(2)?,
            })
        })
        .map_err(|e| format!("查询余额校验数据失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询余额校验数据失败: {e}"))?;
    Ok(rows)
}

/// 财务人员对某一批次银行流水的余额连续性进行整体确认。
pub fn confirm_balance_batch_core(
    conn: &rusqlite::Connection,
    batch_id: i64,
) -> Result<i64, String> {
    let ts = now_str();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;

    let updated = tx
        .execute(
            "UPDATE bank_flows
             SET balance_confirmed_at = ?1
             WHERE import_batch_id = ?2
               AND balance_confirmed_at IS NULL",
            rusqlite::params![ts, batch_id],
        )
        .map_err(|e| format!("更新余额确认时间失败: {e}"))?;

    tx.commit().map_err(|e| format!("提交事务失败: {e}"))?;
    Ok(updated as i64)
}

/// 撤销对某一批次银行流水余额连续性的整体确认。
pub fn unconfirm_balance_batch_core(
    conn: &rusqlite::Connection,
    batch_id: i64,
) -> Result<i64, String> {
    let updated = conn
        .execute(
            "UPDATE bank_flows
             SET balance_confirmed_at = NULL
             WHERE import_batch_id = ?1
               AND balance_confirmed_at IS NOT NULL",
            rusqlite::params![batch_id],
        )
        .map_err(|e| format!("撤销余额确认失败: {e}"))?;
    Ok(updated as i64)
}

// =====================================================================
// Tauri 命令封装
// =====================================================================

#[tauri::command]
pub fn list_bank_flows_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    filter: BankFlowFilter,
) -> Result<BankFlowPage, String> {
    with_company_conn(&db, &session, |conn| list_bank_flows_core(conn, &filter))
}

#[tauri::command]
pub fn get_bank_flow_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
) -> Result<Option<BankFlowDetail>, String> {
    with_company_conn(&db, &session, |conn| get_bank_flow_core(conn, id))
}

#[tauri::command]
pub fn confirm_bank_balance_batch_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    batch_id: i64,
) -> Result<i64, String> {
    with_company_conn(&db, &session, |conn| {
        confirm_balance_batch_core(conn, batch_id)
    })
}

#[tauri::command]
pub fn unconfirm_bank_balance_batch_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    batch_id: i64,
) -> Result<i64, String> {
    with_company_conn(&db, &session, |conn| {
        unconfirm_balance_batch_core(conn, batch_id)
    })
}
