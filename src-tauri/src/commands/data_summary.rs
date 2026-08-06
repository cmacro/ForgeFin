use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::raw::{now_str, with_company_conn};
use crate::commands::session::SessionState;
use crate::db::DbState;

// =====================================================================
// 数据结构
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSummaryRecord {
    pub id: i64,
    pub summary_date: String,
    pub receipt_no: Option<String>,
    pub category: String,
    pub project: String,
    pub reason: Option<String>,
    pub payment_method: Option<String>,
    pub payment_amount: String,
    pub fee: String,
    pub actual_income: String,
    pub expense: String,
    pub balance: Option<String>,
    pub remarks: Option<String>,
    pub source_info: Option<String>,
    pub voucher_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSummaryInput {
    pub summary_date: String,
    pub receipt_no: Option<String>,
    pub category: String,
    pub project: String,
    pub reason: Option<String>,
    pub payment_method: Option<String>,
    pub payment_amount: Option<String>,
    pub fee: Option<String>,
    pub actual_income: Option<String>,
    pub expense: Option<String>,
    pub balance: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DataSummaryFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub category: Option<String>,
    pub project: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSummaryPage {
    pub items: Vec<DataSummaryRecord>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

// =====================================================================
// 核心逻辑
// =====================================================================

pub fn list_data_summaries_core(
    conn: &rusqlite::Connection,
    filter: &DataSummaryFilter,
) -> Result<DataSummaryPage, String> {
    let page = filter.page.max(1);
    let page_size = filter.page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(d) = &filter.date_from {
        where_clause.push_str(" AND ds.summary_date >= ?");
        params.push(Box::new(d.clone()));
    }
    if let Some(d) = &filter.date_to {
        where_clause.push_str(" AND ds.summary_date <= ?");
        params.push(Box::new(d.clone()));
    }
    if let Some(c) = &filter.category {
        where_clause.push_str(" AND ds.category = ?");
        params.push(Box::new(c.clone()));
    }
    if let Some(p) = &filter.project {
        where_clause.push_str(" AND ds.project = ?");
        params.push(Box::new(p.clone()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM data_summaries ds{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计数据汇总失败: {e}"))?;

    let list_sql = format!(
        "SELECT ds.id, ds.summary_date, ds.receipt_no, ds.category, ds.project,
                ds.reason, ds.payment_method, ds.payment_amount,
                ds.fee, ds.actual_income, ds.expense, ds.balance, ds.remarks,
                ds.source_info, ds.voucher_id, ds.created_at, ds.updated_at
         FROM data_summaries ds
         {where_clause}
         ORDER BY ds.summary_date DESC, ds.id DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询数据汇总失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            Ok(DataSummaryRecord {
                id: row.get(0)?,
                summary_date: row.get(1)?,
                receipt_no: row.get(2)?,
                category: row.get(3)?,
                project: row.get(4)?,
                reason: row.get(5)?,
                payment_method: row.get(6)?,
                payment_amount: row.get(7)?,
                fee: row.get(8)?,
                actual_income: row.get(9)?,
                expense: row.get(10)?,
                balance: row.get(11)?,
                remarks: row.get(12)?,
                source_info: row.get(13)?,
                voucher_id: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })
        .map_err(|e| format!("查询数据汇总失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询数据汇总失败: {e}"))?;

    Ok(DataSummaryPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn get_data_summary_core(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<DataSummaryRecord>, String> {
    let record = conn
        .query_row(
            "SELECT ds.id, ds.summary_date, ds.receipt_no, ds.category, ds.project,
                    ds.reason, ds.payment_method, ds.payment_amount,
                    ds.fee, ds.actual_income, ds.expense, ds.balance, ds.remarks,
                    ds.source_info, ds.voucher_id, ds.created_at, ds.updated_at
             FROM data_summaries ds
             WHERE ds.id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(DataSummaryRecord {
                    id: row.get(0)?,
                    summary_date: row.get(1)?,
                    receipt_no: row.get(2)?,
                    category: row.get(3)?,
                    project: row.get(4)?,
                    reason: row.get(5)?,
                    payment_method: row.get(6)?,
                    payment_amount: row.get(7)?,
                    fee: row.get(8)?,
                    actual_income: row.get(9)?,
                    expense: row.get(10)?,
                    balance: row.get(11)?,
                    remarks: row.get(12)?,
                    source_info: row.get(13)?,
                    voucher_id: row.get(14)?,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("查询数据汇总详情失败: {e}"))?;

    Ok(record)
}

pub fn create_data_summary_core(
    conn: &rusqlite::Connection,
    input: &DataSummaryInput,
    operator_name: Option<&str>,
) -> Result<DataSummaryRecord, String> {
    let now = now_str();
    let source_info = operator_name
        .map(|name| serde_json::json!({"type": "manual", "operator": name}).to_string());
    conn.execute(
        "INSERT INTO data_summaries
         (summary_date, receipt_no, category, project, reason,
          payment_method, payment_amount, fee,
          actual_income, expense, balance, remarks, source_info, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
        rusqlite::params![
            input.summary_date,
            input.receipt_no,
            input.category,
            input.project,
            input.reason,
            input.payment_method,
            input.payment_amount.as_deref().unwrap_or("0"),
            input.fee.as_deref().unwrap_or("0"),
            input.actual_income.as_deref().unwrap_or("0"),
            input.expense.as_deref().unwrap_or("0"),
            input.balance,
            input.remarks,
            source_info,
            now,
        ],
    )
    .map_err(|e| format!("创建数据汇总失败: {e}"))?;

    let id = conn.last_insert_rowid();
    get_data_summary_core(conn, id)?.ok_or_else(|| "数据汇总创建后查询失败".to_string())
}

pub fn update_data_summary_core(
    conn: &rusqlite::Connection,
    id: i64,
    input: &DataSummaryInput,
) -> Result<DataSummaryRecord, String> {
    let now = now_str();
    conn.execute(
        "UPDATE data_summaries
         SET summary_date = ?1, receipt_no = ?2, category = ?3, project = ?4, reason = ?5,
             payment_method = ?6, payment_amount = ?7, fee = ?8,
             actual_income = ?9, expense = ?10, balance = ?11, remarks = ?12, updated_at = ?13
         WHERE id = ?14",
        rusqlite::params![
            input.summary_date,
            input.receipt_no,
            input.category,
            input.project,
            input.reason,
            input.payment_method,
            input.payment_amount.as_deref().unwrap_or("0"),
            input.fee.as_deref().unwrap_or("0"),
            input.actual_income.as_deref().unwrap_or("0"),
            input.expense.as_deref().unwrap_or("0"),
            input.balance,
            input.remarks,
            now,
            id,
        ],
    )
    .map_err(|e| format!("更新数据汇总失败: {e}"))?;

    get_data_summary_core(conn, id)?.ok_or_else(|| "数据汇总更新后查询失败".to_string())
}

pub fn delete_data_summary_core(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "DELETE FROM data_summaries WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("删除数据汇总失败: {e}"))?;
    Ok(())
}

// =====================================================================
// Tauri 命令封装
// =====================================================================

#[tauri::command]
pub fn list_data_summaries_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    filter: DataSummaryFilter,
) -> Result<DataSummaryPage, String> {
    with_company_conn(&db, &session, |conn| {
        list_data_summaries_core(conn, &filter)
    })
}

#[tauri::command]
pub fn get_data_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
) -> Result<Option<DataSummaryRecord>, String> {
    with_company_conn(&db, &session, |conn| get_data_summary_core(conn, id))
}

#[tauri::command]
pub fn create_data_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    input: DataSummaryInput,
) -> Result<DataSummaryRecord, String> {
    let operator_name = {
        let guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        guard
            .user
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|u| u.display_name.clone()))
    };
    with_company_conn(&db, &session, |conn| {
        create_data_summary_core(conn, &input, operator_name.as_deref())
    })
}

#[tauri::command]
pub fn update_data_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
    input: DataSummaryInput,
) -> Result<DataSummaryRecord, String> {
    with_company_conn(&db, &session, |conn| {
        update_data_summary_core(conn, id, &input)
    })
}

#[tauri::command]
pub fn delete_data_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
) -> Result<(), String> {
    with_company_conn(&db, &session, |conn| delete_data_summary_core(conn, id))
}
