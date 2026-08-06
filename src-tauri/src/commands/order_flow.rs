use rusqlite::OptionalExtension;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::raw::{now_str, with_company_conn};
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
pub struct OrderFlowRecord {
    pub id: i64,
    pub import_batch_id: i64,
    pub record_no: Option<String>,
    pub record_date: Option<String>,
    pub amount_total: Option<String>,
    pub currency: String,
    pub counterpart_info: Option<String>,
    pub summary: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderFlowFilter {
    pub batch_id: Option<i64>,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderFlowPage {
    pub items: Vec<OrderFlowRecord>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderFlowDetail {
    pub record: OrderFlowRecord,
    pub raw_data: String,
    pub source_file_name: String,
    pub source_row_no: i32,
    pub file_path: String,
}

// =====================================================================
// 核心逻辑
// =====================================================================

pub fn list_order_flows_core(
    conn: &rusqlite::Connection,
    filter: &OrderFlowFilter,
) -> Result<OrderFlowPage, String> {
    let page = filter.page.max(1);
    let page_size = filter.page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(b) = filter.batch_id {
        where_clause.push_str(" AND of.import_batch_id = ?");
        params.push(Box::new(b));
    }

    let count_sql = format!("SELECT COUNT(*) FROM order_flows of{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计订单流水失败: {e}"))?;

    let list_sql = format!(
        "SELECT of.id, of.import_batch_id,
                of.record_no, of.record_date, of.amount_total,
                of.currency, of.counterpart_info, of.summary,
                of.status, of.created_at
         FROM order_flows of
         {where_clause}
         ORDER BY of.record_date DESC, of.id DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询订单流水失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            let amount_total_cents: Option<i64> = row.get(4)?;
            Ok(OrderFlowRecord {
                id: row.get(0)?,
                import_batch_id: row.get(1)?,
                record_no: row.get(2)?,
                record_date: row.get(3)?,
                amount_total: amount_total_cents.map(cents_to_yuan),
                currency: row.get(5)?,
                counterpart_info: row.get(6)?,
                summary: row.get(7)?,
                status: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("查询订单流水失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询订单流水失败: {e}"))?;

    Ok(OrderFlowPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn get_order_flow_core(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<OrderFlowDetail>, String> {
    let record_opt = conn
        .query_row(
            "SELECT of.id, of.import_batch_id,
                    of.record_no, of.record_date, of.amount_total,
                    of.currency, of.counterpart_info, of.summary,
                    of.status, of.created_at, of.raw_data,
                    of.source_file_name, of.source_row_no
             FROM order_flows of
             WHERE of.id = ?1",
            rusqlite::params![id],
            |row| {
                let amount_total_cents: Option<i64> = row.get(4)?;
                Ok((
                    OrderFlowRecord {
                        id: row.get(0)?,
                        import_batch_id: row.get(1)?,
                        record_no: row.get(2)?,
                        record_date: row.get(3)?,
                        amount_total: amount_total_cents.map(cents_to_yuan),
                        currency: row.get(5)?,
                        counterpart_info: row.get(6)?,
                        summary: row.get(7)?,
                        status: row.get(8)?,
                        created_at: row.get(9)?,
                    },
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i32>(12)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("查询订单流水详情失败: {e}"))?;

    let Some((record, raw_data, source_file_name, source_row_no)) = record_opt else {
        return Ok(None);
    };

    let file_path = conn
        .query_row(
            "SELECT file_path FROM import_batches WHERE id = ?1",
            rusqlite::params![record.import_batch_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .unwrap_or_default();

    Ok(Some(OrderFlowDetail {
        record,
        raw_data,
        source_file_name,
        source_row_no,
        file_path,
    }))
}

// =====================================================================
// Tauri 命令封装
// =====================================================================

#[tauri::command]
pub fn list_order_flows_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    filter: OrderFlowFilter,
) -> Result<OrderFlowPage, String> {
    with_company_conn(&db, &session, |conn| list_order_flows_core(conn, &filter))
}

#[tauri::command]
pub fn get_order_flow_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
) -> Result<Option<OrderFlowDetail>, String> {
    with_company_conn(&db, &session, |conn| get_order_flow_core(conn, id))
}
