use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::OptionalExtension;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::commands::session::SessionState;
use crate::db::DbState;

// =====================================================================
// 数据结构
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawFileInfo {
    pub file_path: String,
    pub file_name: String,
    pub source_type: String,
    pub status: String, // imported | pending | unsupported
    pub row_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub file_name: String,
    pub batch_id: i64,
    pub source_type: String,
    pub row_count: i32,
    pub file_hash: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImportDirResult {
    pub imported: Vec<ImportResult>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

// =====================================================================
// 公共辅助函数
// =====================================================================

fn current_company_id(session: &SessionState) -> Result<String, String> {
    session
        .company_id
        .lock()
        .map_err(|e| format!("会话锁失败: {e}"))?
        .clone()
        .ok_or_else(|| "未选择账套".to_string())
}

fn current_user_id(session: &SessionState) -> Option<String> {
    session
        .user
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|u| u.id.clone()))
}

fn with_company_conn<F, T>(
    db: &State<'_, std::sync::Mutex<DbState>>,
    session: &State<'_, std::sync::Mutex<SessionState>>,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, String>,
{
    let company_id = {
        let sess = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        current_company_id(&sess)?
    };
    let db_guard = db.lock().map_err(|e| format!("数据库锁失败: {e}"))?;
    let companies = db_guard.company(&company_id)?;
    let conn = companies
        .get(&company_id)
        .ok_or_else(|| "公司库连接不存在".to_string())?;
    f(conn)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("读取文件失败 ({}): {e}", path.display()))?;
    let hash = Sha256::digest(&bytes);
    Ok(hash.iter().map(|b| format!("{b:02x}")).collect())
}

fn detect_source_type(file_name: &str) -> Option<&'static str> {
    let lower = file_name.to_lowercase();
    if lower.contains("bank") || lower.contains("银行") {
        Some("bank_flow")
    } else if lower.contains("order") || lower.contains("订单") {
        Some("order_flow")
    } else if lower.contains("pos") {
        Some("pos_flow")
    } else if lower.contains("summary") || lower.contains("汇总") || lower.contains("数据汇总")
    {
        Some("summary_flow")
    } else {
        None
    }
}

fn parse_amount(value: &str) -> Option<Decimal> {
    let cleaned = value.replace(',', "").trim().to_string();
    if cleaned.is_empty() {
        return None;
    }
    Decimal::from_str_exact(&cleaned).ok()
}

fn now_str() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn count_data_rows(path: &Path) -> Result<i32, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取文件失败 ({}): {e}", path.display()))?;
    let mut count = 0;
    for line in content.lines().skip(1) {
        if !line.trim().is_empty() {
            count += 1;
        }
    }
    Ok(count)
}

fn extract_value(map: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = map.get(*key) {
            if !v.is_empty() {
                return Some(v.clone());
            }
        }
    }
    None
}

fn compute_amount_total(map: &HashMap<String, String>, source_type: &str) -> Option<String> {
    match source_type {
        "bank_flow" => {
            if let Some(v) = extract_value(map, &["转入金额"]) {
                if let Some(d) = parse_amount(&v) {
                    if !d.is_zero() {
                        return Some(d.to_string());
                    }
                }
            }
            if let Some(v) = extract_value(map, &["转出金额"]) {
                if let Some(d) = parse_amount(&v) {
                    if !d.is_zero() {
                        return Some((-d).to_string());
                    }
                }
            }
            None
        }
        "order_flow" => extract_value(map, &["商户实收金额"])
            .and_then(|v| parse_amount(&v).map(|d| d.to_string())),
        "pos_flow" => {
            extract_value(map, &["订单金额"]).and_then(|v| parse_amount(&v).map(|d| d.to_string()))
        }
        "summary_flow" => {
            if let Some(v) = extract_value(map, &["实际收入"]) {
                if let Some(d) = parse_amount(&v) {
                    if !d.is_zero() {
                        return Some(d.to_string());
                    }
                }
            }
            if let Some(v) = extract_value(map, &["支出"]) {
                if let Some(d) = parse_amount(&v) {
                    if !d.is_zero() {
                        return Some((-d).to_string());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

// =====================================================================
// 核心逻辑（可直接测试）
// =====================================================================

pub fn scan_directory_core(
    conn: &rusqlite::Connection,
    dir: &Path,
) -> Result<Vec<RawFileInfo>, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败 ({}): {e}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                ext == "tsv" || ext == "csv" || ext == "xlsx"
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    let mut result = Vec::new();
    for entry in entries {
        let path = entry.path();
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_path = path.to_string_lossy().to_string();

        let Some(source_type) = detect_source_type(&file_name) else {
            result.push(RawFileInfo {
                file_path,
                file_name,
                source_type: "unknown".to_string(),
                status: "unsupported".to_string(),
                row_count: 0,
            });
            continue;
        };

        let file_hash = sha256_file(&path)?;
        let imported: bool = conn
            .query_row(
                "SELECT 1 FROM import_batches WHERE file_hash = ?1 LIMIT 1",
                rusqlite::params![&file_hash],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        let row_count = count_data_rows(&path).unwrap_or(0);

        result.push(RawFileInfo {
            file_path,
            file_name,
            source_type: source_type.to_string(),
            status: if imported {
                "imported".to_string()
            } else {
                "pending".to_string()
            },
            row_count,
        });
    }
    Ok(result)
}

pub fn import_file_core(
    conn: &rusqlite::Connection,
    path: &Path,
    source_type: Option<&str>,
    created_by: Option<&str>,
) -> Result<ImportResult, String> {
    if !path.is_file() {
        return Err(format!("文件不存在: {}", path.display()));
    }

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_path = path.to_string_lossy().to_string();

    let detected = detect_source_type(&file_name);
    let source_type = source_type
        .or(detected)
        .ok_or_else(|| format!("无法识别文件类型: {file_name}"))?;

    let file_hash = sha256_file(path)?;

    // 重复导入检查
    let existing: bool = conn
        .query_row(
            "SELECT 1 FROM import_batches WHERE file_hash = ?1 LIMIT 1",
            rusqlite::params![&file_hash],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    if existing {
        return Err(format!("文件已导入 (hash 重复): {file_name}"));
    }

    let source_type_id: i64 = conn
        .query_row(
            "SELECT id FROM source_types WHERE code = ?1",
            rusqlite::params![source_type],
            |row| row.get(0),
        )
        .map_err(|e| format!("未找到来源类型 {source_type}: {e}"))?;

    conn.execute(
        "INSERT INTO import_batches (file_path, file_name, file_hash, source_type, row_count, imported_at, created_by)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        rusqlite::params![
            file_path,
            file_name,
            file_hash,
            source_type,
            now_str(),
            created_by,
        ],
    )
    .map_err(|e| format!("写入导入批次失败: {e}"))?;

    let import_batch_id = conn.last_insert_rowid();

    let row_count =
        parse_and_insert_records(conn, path, source_type_id, import_batch_id, source_type)?;

    conn.execute(
        "UPDATE import_batches SET row_count = ?1 WHERE id = ?2",
        rusqlite::params![row_count, import_batch_id],
    )
    .map_err(|e| format!("更新批次行数失败: {e}"))?;

    Ok(ImportResult {
        file_name,
        batch_id: import_batch_id,
        source_type: source_type.to_string(),
        row_count,
        file_hash,
    })
}

pub fn auto_import_directory_core(
    conn: &rusqlite::Connection,
    dir: &Path,
    created_by: Option<&str>,
) -> Result<ImportDirResult, String> {
    let files = scan_directory_core(conn, dir)?;
    let mut result = ImportDirResult::default();
    for file in files {
        match file.status.as_str() {
            "imported" => result.skipped.push(file.file_name),
            "unsupported" => result
                .errors
                .push(format!("{}: 不支持的文件类型", file.file_name)),
            _ => {
                let path = PathBuf::from(&file.file_path);
                match import_file_core(conn, &path, Some(&file.source_type), created_by) {
                    Ok(r) => result.imported.push(r),
                    Err(e) => result.errors.push(format!("{}: {e}", file.file_name)),
                }
            }
        }
    }
    Ok(result)
}

fn parse_and_insert_records(
    conn: &rusqlite::Connection,
    path: &Path,
    source_type_id: i64,
    import_batch_id: i64,
    source_type: &str,
) -> Result<i32, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取文件失败 ({}): {e}", path.display()))?;
    let mut lines = content.lines();
    let header_line = lines.next().ok_or_else(|| "文件为空".to_string())?;
    let headers: Vec<String> = header_line
        .split('\t')
        .map(|s| s.trim().to_string())
        .collect();

    let source_file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut inserted = 0;

    for (idx, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row_no = idx as i32 + 2; // 1-based, 第 1 行是表头

        let values: Vec<String> = line.split('\t').map(|s| s.trim().to_string()).collect();
        let mut map = HashMap::<String, String>::new();
        for (h, v) in headers.iter().zip(values.iter()) {
            map.insert(h.clone(), v.clone());
        }

        let raw_data =
            serde_json::to_string(&map).map_err(|e| format!("序列化 raw_data 失败: {e}"))?;

        let record_date = extract_value(&map, &["交易时间", "日期", "支付时间", "结算日期"]);
        let counterpart_info =
            extract_value(&map, &["对方单位", "客户备注", "商户名称", "对方账号"]);
        let summary = extract_value(&map, &["摘要", "事由", "用途", "项目"]);
        let record_no = extract_value(&map, &["工行订单号", "商户订单号", "凭证号", "收据编号"]);
        let amount_total = compute_amount_total(&map, source_type);

        conn.execute(
            "INSERT INTO source_records
             (source_type_id, import_batch_id, source_file_name, source_row_no, record_no, record_date, amount_total, currency, counterpart_info, summary, raw_data, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'CNY', ?8, ?9, ?10, 'pending', ?11)",
            rusqlite::params![
                source_type_id,
                import_batch_id,
                source_file_name,
                row_no,
                record_no,
                record_date,
                amount_total,
                counterpart_info,
                summary,
                raw_data,
                now_str(),
            ],
        )
        .map_err(|e| format!("写入 source_records 失败 (行 {row_no}): {e}"))?;
        inserted += 1;
    }

    Ok(inserted)
}

// =====================================================================
// Tauri 命令封装
// =====================================================================

#[tauri::command]
pub fn scan_raw_directory_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    path: String,
) -> Result<Vec<RawFileInfo>, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("路径不是目录: {path}"));
    }
    with_company_conn(&db, &session, |conn| scan_directory_core(conn, &dir))
}

#[tauri::command]
pub fn auto_import_raw_directory_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    path: String,
) -> Result<ImportDirResult, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("路径不是目录: {path}"));
    }
    let created_by = {
        let guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        current_user_id(&guard)
    };
    with_company_conn(&db, &session, |conn| {
        auto_import_directory_core(conn, &dir, created_by.as_deref())
    })
}

#[tauri::command]
pub fn import_raw_file_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    file_path: String,
    _batch_id: Option<String>,
    source_type: Option<String>,
) -> Result<ImportResult, String> {
    let path = PathBuf::from(&file_path);
    let source_type = source_type.as_deref();
    let (created_by, operator_id, operator_name) = {
        let guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        let id = current_user_id(&guard);
        let name = guard
            .user
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|u| u.display_name.clone()));
        (id.clone(), id, name)
    };
    with_company_conn(&db, &session, |conn| {
        let result = import_file_core(conn, &path, source_type, created_by.as_deref())?;
        // 导入完成后写入审计日志
        let log_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO audit_logs (entity_type, entity_id, action, old_values, new_values,
                                     operator_id, operator_name, created_at)
             VALUES ('import_batch', ?1, 'import_raw_file', ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                result.batch_id.to_string(),
                serde_json::json!({"file_path": result.file_name}).to_string(),
                serde_json::json!({"row_count": result.row_count}).to_string(),
                operator_id.as_deref(),
                operator_name.as_deref(),
                now_str(),
            ],
        )
        .map_err(|e| format!("写入导入审计日志失败: {e}"))?;
        Ok(result)
    })
}

#[tauri::command]
pub fn list_raw_records_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    filter: RawRecordFilter,
) -> Result<RawRecordPage, String> {
    with_company_conn(&db, &session, |conn| list_raw_records_core(conn, &filter))
}

#[tauri::command]
pub fn get_raw_record_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    id: i64,
) -> Result<Option<RawRecordDetail>, String> {
    with_company_conn(&db, &session, |conn| get_raw_record_core(conn, id))
}

#[tauri::command]
pub fn reconcile_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    date: String,
) -> Result<ReconcileResult, String> {
    let (operator_id, operator_name) = {
        let guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        let id = current_user_id(&guard);
        let name = guard
            .user
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|u| u.display_name.clone()));
        (id, name)
    };
    with_company_conn(&db, &session, |conn| {
        let result = reconcile_core(conn, &date)?;
        let log_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO audit_logs (entity_type, entity_id, action, old_values, new_values,
                                     operator_id, operator_name, created_at)
             VALUES ('transaction_summary', ?1, 'reconcile', ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                result
                    .created_summary_ids
                    .first()
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                serde_json::json!({"date": date}).to_string(),
                serde_json::json!({
                    "matched_dates": result.matched_dates,
                    "diff_dates": result.diff_dates
                })
                .to_string(),
                operator_id.as_deref(),
                operator_name.as_deref(),
                now_str(),
            ],
        )
        .map_err(|e| format!("写入对账审计日志失败: {e}"))?;
        Ok(result)
    })
}

#[tauri::command]
pub fn list_reconciliation_items_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    date: Option<String>,
    status: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<ReconciliationPage, String> {
    with_company_conn(&db, &session, |conn| {
        list_reconciliation_items_core(conn, date.as_deref(), status.as_deref(), page, page_size)
    })
}

#[tauri::command]
pub fn review_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    summary_id: i64,
    approve: bool,
    comment: Option<String>,
) -> Result<Option<VoucherSummary>, String> {
    let (operator_id, operator_name) = {
        let guard = session.lock().map_err(|e| format!("会话锁失败: {e}"))?;
        let id = current_user_id(&guard);
        let name = guard
            .user
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|u| u.display_name.clone()));
        (id, name)
    };
    with_company_conn(&db, &session, |conn| {
        review_summary_core(
            conn,
            summary_id,
            approve,
            comment.as_deref(),
            operator_id.as_deref(),
            operator_name.as_deref(),
        )
    })
}

#[tauri::command]
pub fn list_raw_audit_logs_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<(Vec<AuditLogEntry>, i32), String> {
    with_company_conn(&db, &session, |conn| {
        list_audit_logs_core(
            conn,
            entity_type.as_deref(),
            entity_id.as_deref(),
            page,
            page_size,
        )
    })
}

// =====================================================================
// 功能单元测试
// =====================================================================
// 对账 / 差异审核 / 凭证生成 / 审计日志
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawRecord {
    pub id: i64,
    pub source_type: String,
    pub source_type_name: String,
    pub import_batch_id: i64,
    pub source_file_name: String,
    pub source_row_no: i32,
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
pub struct RawRecordFilter {
    pub source_type: Option<String>,
    pub batch_id: Option<i64>,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawRecordPage {
    pub items: Vec<RawRecord>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: String,
    pub file_name: String,
    pub file_size: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawRecordDetail {
    pub record: RawRecord,
    pub raw_data: String,
    pub attachments: Vec<AttachmentInfo>,
    pub audit_logs: Vec<AuditLogEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub action: String,
    pub operator_name: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconciliationItem {
    pub id: i64,
    pub summary_date: String,
    pub source_type: String,
    pub bank_amount: String,
    pub order_amount: String,
    pub diff_amount: String,
    pub review_status: String,
    pub voucher_id: Option<String>,
    pub voucher_no: Option<String>,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconciliationPage {
    pub items: Vec<ReconciliationItem>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconcileResult {
    pub matched_dates: Vec<String>,
    pub diff_dates: Vec<String>,
    pub created_summary_ids: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherSummary {
    pub id: String,
    pub voucher_no: String,
    pub voucher_date: String,
    pub summary: String,
    pub debit_total: String,
    pub credit_total: String,
}

pub fn list_raw_records_core(
    conn: &rusqlite::Connection,
    filter: &RawRecordFilter,
) -> Result<RawRecordPage, String> {
    let page = filter.page.max(1);
    let page_size = filter.page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(t) = &filter.source_type {
        where_clause.push_str(" AND st.code = ?");
        params.push(Box::new(t.clone()));
    }
    if let Some(b) = filter.batch_id {
        where_clause.push_str(" AND sr.import_batch_id = ?");
        params.push(Box::new(b));
    }

    let count_sql = format!(
        "SELECT COUNT(*) FROM source_records sr JOIN source_types st ON sr.source_type_id = st.id{where_clause}"
    );
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计原始记录失败: {e}"))?;

    let list_sql = format!(
        "SELECT sr.id, st.code, st.name, sr.import_batch_id, sr.source_file_name, sr.source_row_no,
                sr.record_no, sr.record_date, sr.amount_total, sr.currency, sr.counterpart_info,
                sr.summary, sr.status, sr.created_at
         FROM source_records sr
         JOIN source_types st ON sr.source_type_id = st.id
         {where_clause}
         ORDER BY sr.record_date DESC, sr.id DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询原始记录失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            Ok(RawRecord {
                id: row.get(0)?,
                source_type: row.get(1)?,
                source_type_name: row.get(2)?,
                import_batch_id: row.get(3)?,
                source_file_name: row.get(4)?,
                source_row_no: row.get(5)?,
                record_no: row.get(6)?,
                record_date: row.get(7)?,
                amount_total: row.get(8)?,
                currency: row.get(9)?,
                counterpart_info: row.get(10)?,
                summary: row.get(11)?,
                status: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("查询原始记录失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询原始记录失败: {e}"))?;

    Ok(RawRecordPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn get_raw_record_core(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<RawRecordDetail>, String> {
    let record = conn
        .query_row(
            "SELECT sr.id, st.code, st.name, sr.import_batch_id, sr.source_file_name, sr.source_row_no,
                    sr.record_no, sr.record_date, sr.amount_total, sr.currency, sr.counterpart_info,
                    sr.summary, sr.status, sr.created_at, sr.raw_data
             FROM source_records sr
             JOIN source_types st ON sr.source_type_id = st.id
             WHERE sr.id = ?1",
            rusqlite::params![id],
            |row| {
                Ok((
                    RawRecord {
                        id: row.get(0)?,
                        source_type: row.get(1)?,
                        source_type_name: row.get(2)?,
                        import_batch_id: row.get(3)?,
                        source_file_name: row.get(4)?,
                        source_row_no: row.get(5)?,
                        record_no: row.get(6)?,
                        record_date: row.get(7)?,
                        amount_total: row.get(8)?,
                        currency: row.get(9)?,
                        counterpart_info: row.get(10)?,
                        summary: row.get(11)?,
                        status: row.get(12)?,
                        created_at: row.get(13)?,
                    },
                    row.get::<_, String>(14)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("查询原始记录详情失败: {e}"))?;

    let Some((record, raw_data)) = record else {
        return Ok(None);
    };

    let entity_id = id.to_string();
    let mut stmt = conn
        .prepare(
            "SELECT id, entity_type, entity_id, file_name, file_size, created_at
             FROM attachments WHERE entity_type = 'source_record' AND entity_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("查询附件失败: {e}"))?;
    let attachments = stmt
        .query_map(rusqlite::params![&entity_id], |row| {
            Ok(AttachmentInfo {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                file_name: row.get(3)?,
                file_size: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("查询附件失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询附件失败: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, entity_type, entity_id, action, operator_name, comment, created_at
             FROM audit_logs
             WHERE entity_type = 'source_record' AND entity_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("查询审计日志失败: {e}"))?;
    let audit_logs = stmt
        .query_map(rusqlite::params![&entity_id], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                action: row.get(3)?,
                operator_name: row.get(4)?,
                comment: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询审计日志失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询审计日志失败: {e}"))?;

    Ok(Some(RawRecordDetail {
        record,
        raw_data,
        attachments,
        audit_logs,
    }))
}

pub fn reconcile_core(conn: &rusqlite::Connection, date: &str) -> Result<ReconcileResult, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;

    // 计算当日银行流水合计
    // 计算当日银行流水合计
    let bank_total_text: String = tx
        .query_row(
            "SELECT COALESCE(SUM(CAST(amount_total AS REAL)), 0)
             FROM source_records sr
             JOIN source_types st ON sr.source_type_id = st.id
             WHERE st.code = 'bank_flow' AND sr.record_date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .map_err(|e| format!("汇总银行金额失败: {e}"))?;
    let bank_total: Decimal = bank_total_text.parse().unwrap_or(Decimal::ZERO);

    // 计算当日订单实收合计
    let order_total_text: String = tx
        .query_row(
            "SELECT COALESCE(SUM(CAST(amount_total AS REAL)), 0)
             FROM source_records sr
             JOIN source_types st ON sr.source_type_id = st.id
             WHERE st.code = 'order_flow' AND sr.record_date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .map_err(|e| format!("汇总订单金额失败: {e}"))?;
    let order_total: Decimal = order_total_text.parse().unwrap_or(Decimal::ZERO);

    let diff = bank_total - order_total;
    let matched = diff.abs() < Decimal::new(1, 2); // 差额 < 0.01 视为无差异

    // 获取当日银行/订单记录 ID
    let bank_ids: Vec<i64> = {
        let mut stmt = tx
            .prepare(
                "SELECT sr.id FROM source_records sr
                 JOIN source_types st ON sr.source_type_id = st.id
                 WHERE st.code = 'bank_flow' AND sr.record_date = ?1
                 ORDER BY sr.id",
            )
            .map_err(|e| format!("查询银行记录失败: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![date], |row| row.get::<_, i64>(0))
            .map_err(|e| format!("查询银行记录失败: {e}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("查询银行记录失败: {e}"))?
    };
    let order_ids: Vec<i64> = {
        let mut stmt = tx
            .prepare(
                "SELECT sr.id FROM source_records sr
                 JOIN source_types st ON sr.source_type_id = st.id
                 WHERE st.code = 'order_flow' AND sr.record_date = ?1
                 ORDER BY sr.id",
            )
            .map_err(|e| format!("查询订单记录失败: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![date], |row| row.get::<_, i64>(0))
            .map_err(|e| format!("查询订单记录失败: {e}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("查询订单记录失败: {e}"))?
    };

    // 删除旧汇总
    tx.execute(
        "DELETE FROM transaction_summaries WHERE summary_date = ?1 AND source_type = 'bank_order'",
        rusqlite::params![date],
    )
    .map_err(|e| format!("清理旧汇总失败: {e}"))?;

    tx.execute(
        "INSERT INTO transaction_summaries
         (summary_date, source_type, bank_amount, order_amount, diff_amount, review_status,
          matched_bank_ids, matched_order_ids, comment, created_at, updated_at)
         VALUES (?1, 'bank_order', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        rusqlite::params![
            date,
            bank_total.to_string(),
            order_total.to_string(),
            diff.to_string(),
            if matched { "auto_matched" } else { "pending" },
            json_ids(&bank_ids),
            json_ids(&order_ids),
            if matched {
                Some("系统自动匹配")
            } else {
                None
            },
            now_str(),
        ],
    )
    .map_err(|e| format!("写入对账汇总失败: {e}"))?;
    let summary_id = tx.last_insert_rowid();

    // 若自动匹配,更新来源记录状态为 matched
    if matched {
        for id in bank_ids.iter().chain(order_ids.iter()) {
            tx.execute(
                "UPDATE source_records SET status = 'matched' WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("更新来源记录状态失败: {e}"))?;
        }
    }

    tx.commit().map_err(|e| format!("提交对账事务失败: {e}"))?;

    let mut matched_dates = Vec::new();
    let mut diff_dates = Vec::new();
    if matched {
        matched_dates.push(date.to_string());
    } else {
        diff_dates.push(date.to_string());
    }

    Ok(ReconcileResult {
        matched_dates,
        diff_dates,
        created_summary_ids: vec![summary_id],
    })
}

fn json_ids(ids: &[i64]) -> String {
    serde_json::to_string(ids).unwrap_or_else(|_| "[]".to_string())
}

pub fn list_reconciliation_items_core(
    conn: &rusqlite::Connection,
    date: Option<&str>,
    status: Option<&str>,
    page: i32,
    page_size: i32,
) -> Result<ReconciliationPage, String> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(d) = date {
        where_clause.push_str(" AND summary_date = ?");
        params.push(Box::new(d.to_string()));
    }
    if let Some(s) = status {
        where_clause.push_str(" AND review_status = ?");
        params.push(Box::new(s.to_string()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM transaction_summaries{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计对账记录失败: {e}"))?;

    let list_sql = format!(
        "SELECT ts.id, ts.summary_date, ts.source_type, ts.bank_amount, ts.order_amount,
                ts.diff_amount, ts.review_status, ts.voucher_id, v.voucher_no, ts.comment
         FROM transaction_summaries ts
         LEFT JOIN vouchers v ON ts.voucher_id = v.id
         {where_clause}
         ORDER BY ts.summary_date DESC, ts.id DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询对账记录失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            Ok(ReconciliationItem {
                id: row.get(0)?,
                summary_date: row.get(1)?,
                source_type: row.get(2)?,
                bank_amount: row.get(3)?,
                order_amount: row.get(4)?,
                diff_amount: row.get(5)?,
                review_status: row.get(6)?,
                voucher_id: row.get(7)?,
                voucher_no: row.get(8)?,
                comment: row.get(9)?,
            })
        })
        .map_err(|e| format!("查询对账记录失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询对账记录失败: {e}"))?;

    Ok(ReconciliationPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn review_summary_core(
    conn: &rusqlite::Connection,
    summary_id: i64,
    approve: bool,
    comment: Option<&str>,
    operator_id: Option<&str>,
    operator_name: Option<&str>,
) -> Result<Option<VoucherSummary>, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;

    let summary: (String, String, String, String, String) = tx
        .query_row(
            "SELECT summary_date, bank_amount, order_amount, diff_amount, review_status
             FROM transaction_summaries WHERE id = ?1",
            rusqlite::params![summary_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("查询对账汇总失败: {e}"))?;

    if summary.4 != "pending" {
        return Err(format!("当前状态 {} 不允许审核", summary.4));
    }

    if !approve {
        tx.execute(
            "UPDATE transaction_summaries
             SET review_status = 'rejected', comment = ?1, updated_at = ?2
             WHERE id = ?3",
            rusqlite::params![comment, now_str(), summary_id],
        )
        .map_err(|e| format!("更新对账状态失败: {e}"))?;
        let log_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO audit_logs (entity_type, entity_id, action, old_values, new_values,
                                     operator_id, operator_name, comment, created_at)
             VALUES ('transaction_summary', ?1, 'reject_review', ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                summary_id.to_string(),
                json_status(&summary.4),
                json_status("rejected"),
                operator_id,
                operator_name,
                None::<&str>,
                now_str(),
            ],
        )
        .map_err(|e| format!("写入审计日志失败: {e}"))?;
        tx.commit().map_err(|e| format!("提交事务失败: {e}"))?;
        return Ok(None);
    }

    let voucher = generate_voucher_in_tx(&tx, summary_id, &summary, operator_id, operator_name)?;

    tx.execute(
        "UPDATE transaction_summaries
         SET review_status = 'approved', voucher_id = ?1, comment = ?2, updated_at = ?3
         WHERE id = ?4",
        rusqlite::params![&voucher.id, comment, now_str(), summary_id],
    )
    .map_err(|e| format!("更新对账状态失败: {e}"))?;

    let log_id = uuid::Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO audit_logs (entity_type, entity_id, action, old_values, new_values,
                                 operator_id, operator_name, comment, created_at)
         VALUES ('transaction_summary', ?1, 'approve_review', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            summary_id.to_string(),
            json_status("pending"),
            json_status("approved"),
            operator_id,
            operator_name,
            comment,
            now_str(),
        ],
    )
    .map_err(|e| format!("写入审计日志失败: {e}"))?;

    tx.commit().map_err(|e| format!("提交事务失败: {e}"))?;
    Ok(Some(voucher))
}

fn json_status(status: &str) -> String {
    serde_json::json!({"review_status": status}).to_string()
}

fn generate_voucher_in_tx(
    tx: &rusqlite::Transaction,
    summary_id: i64,
    summary: &(String, String, String, String, String),
    operator_id: Option<&str>,
    operator_name: Option<&str>,
) -> Result<VoucherSummary, String> {
    let date = &summary.0;
    let bank_amount: Decimal = summary.1.parse().unwrap_or(Decimal::ZERO);
    let order_amount: Decimal = summary.2.parse().unwrap_or(Decimal::ZERO);
    let diff_amount: Decimal = summary.3.parse().unwrap_or(Decimal::ZERO);

    // 查找可用的默认科目: 银行存款/应收账款/主营业务收入/财务费用
    let cash_account = find_default_account(tx, "1002", "银行存款")?;
    let receivable_account = find_default_account(tx, "1122", "应收账款")?;
    let income_account = find_default_account(tx, "6001", "主营业务收入")?;

    let voucher_id = uuid::Uuid::new_v4().to_string();
    let voucher_no = next_voucher_no_in_tx(tx, "记账", date)?;
    let summary_text = format!("原始凭证对账生成 {}", date);
    let now = now_str();

    // 借: 银行存款(bank_amount); 贷: 主营业务收入(order_amount)+应收账款(-diff) 或反向
    // 以银行到账为借方,订单收入为贷方;差额为差异科目。
    let mut entries: Vec<(String, String, String, Decimal, Decimal)> = Vec::new();
    if !bank_amount.is_zero() {
        entries.push((
            cash_account.id.clone(),
            cash_account.code.clone(),
            cash_account.name.clone(),
            bank_amount,
            Decimal::ZERO,
        ));
    }
    if !order_amount.is_zero() {
        entries.push((
            income_account.id.clone(),
            income_account.code.clone(),
            income_account.name.clone(),
            Decimal::ZERO,
            order_amount,
        ));
    }
    if !diff_amount.is_zero() {
        // 正差额表示银行 > 订单 => 贷应收账款(少收);负差额表示银行 < 订单 => 借应收账款(多收)
        if diff_amount > Decimal::ZERO {
            entries.push((
                receivable_account.id.clone(),
                receivable_account.code.clone(),
                receivable_account.name.clone(),
                Decimal::ZERO,
                diff_amount,
            ));
        } else {
            entries.push((
                receivable_account.id.clone(),
                receivable_account.code.clone(),
                receivable_account.name.clone(),
                -diff_amount,
                Decimal::ZERO,
            ));
        }
    }

    let debit_total: Decimal = entries.iter().map(|e| e.3).sum();
    let credit_total: Decimal = entries.iter().map(|e| e.4).sum();
    if debit_total != credit_total {
        return Err(format!(
            "凭证借贷不平衡: 借方 {debit_total} ≠ 贷方 {credit_total}"
        ));
    }

    tx.execute(
        "INSERT INTO vouchers (id, voucher_no, voucher_date, voucher_type, summary, attachments,
                               status, debit_total, credit_total, operator_id, operator_name,
                               created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, 'draft', ?6, ?7, ?8, ?9, ?10, ?10)",
        rusqlite::params![
            &voucher_id,
            &voucher_no,
            date,
            "记账",
            &summary_text,
            debit_total.to_string(),
            credit_total.to_string(),
            operator_id,
            operator_name,
            &now,
        ],
    )
    .map_err(|e| format!("创建凭证失败: {e}"))?;

    for (idx, entry) in entries.iter().enumerate() {
        let entry_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO voucher_entries (id, voucher_id, line_no, account_id, account_code,
                                           account_name, summary, debit, credit, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                entry_id,
                &voucher_id,
                (idx as i32) + 1,
                &entry.0,
                &entry.1,
                &entry.2,
                &summary_text,
                entry.3.to_string(),
                entry.4.to_string(),
                &now,
            ],
        )
        .map_err(|e| format!("创建凭证分录失败: {e}"))?;
    }

    // 写入生成凭证审计日志
    let log_id = uuid::Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO audit_logs (entity_type, entity_id, action, old_values, new_values,
                                 operator_id, operator_name, comment, created_at)
         VALUES ('voucher', ?1, 'generate_voucher', ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            &voucher_id,
            serde_json::json!({"summary_id": summary_id}).to_string(),
            serde_json::json!({"voucher_no": voucher_no}).to_string(),
            operator_id,
            operator_name,
            None::<&str>,
            now_str(),
        ],
    )
    .map_err(|e| format!("写入凭证审计日志失败: {e}"))?;

    Ok(VoucherSummary {
        id: voucher_id,
        voucher_no,
        voucher_date: date.clone(),
        summary: summary_text,
        debit_total: debit_total.to_string(),
        credit_total: credit_total.to_string(),
    })
}

#[derive(Clone, Debug)]
struct DefaultAccount {
    id: String,
    code: String,
    name: String,
}

fn find_default_account(
    conn: &rusqlite::Connection,
    code: &str,
    fallback_name: &str,
) -> Result<DefaultAccount, String> {
    let row: Option<(String, String, String)> = conn
        .query_row(
            "SELECT id, code, name FROM accounts WHERE code = ?1 LIMIT 1",
            rusqlite::params![code],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| format!("查询科目失败: {e}"))?;
    if let Some((id, code, name)) = row {
        return Ok(DefaultAccount { id, code, name });
    }
    // 未找到则自动创建默认科目
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_str();
    conn.execute(
        "INSERT INTO accounts (id, code, name, account_type, balance_direction, is_leaf,
                               is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 1, ?6, ?6)",
        rusqlite::params![
            &id,
            code,
            fallback_name,
            if code.starts_with('1') {
                "asset"
            } else {
                "income"
            },
            if code.starts_with('1') {
                "debit"
            } else {
                "credit"
            },
            &now,
        ],
    )
    .map_err(|e| format!("创建默认科目失败: {e}"))?;
    Ok(DefaultAccount {
        id,
        code: code.to_string(),
        name: fallback_name.to_string(),
    })
}

fn next_voucher_no_in_tx(
    conn: &rusqlite::Connection,
    voucher_type: &str,
    voucher_date: &str,
) -> Result<String, String> {
    let prefix = match voucher_type {
        "记账" | "记账凭证" | "recording" => "记",
        "付款" | "付款凭证" | "payment" => "付",
        "收款" | "收款凭证" | "receipt" => "收",
        "转账" | "转账凭证" | "transfer" => "转",
        _ => "记",
    };
    let s = voucher_date.split('T').next().unwrap_or(voucher_date);
    let mut parts = s.split('-');
    let year: i32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(2024);
    let month: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(1);
    let like = format!("{prefix}-{year}-{month:02}-%");
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM vouchers WHERE voucher_no LIKE ?1",
            rusqlite::params![like],
            |row| row.get(0),
        )
        .map_err(|e| format!("生成凭证字号失败: {e}"))?;
    Ok(format!("{prefix}-{year}-{month:02}-{:04}", count + 1))
}

pub fn list_audit_logs_core(
    conn: &rusqlite::Connection,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    page: i32,
    page_size: i32,
) -> Result<(Vec<AuditLogEntry>, i32), String> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(t) = entity_type {
        where_clause.push_str(" AND entity_type = ?");
        params.push(Box::new(t.to_string()));
    }
    if let Some(id) = entity_id {
        where_clause.push_str(" AND entity_id = ?");
        params.push(Box::new(id.to_string()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM audit_logs{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(count_refs.iter()),
            |row| row.get(0),
        )
        .map_err(|e| format!("统计审计日志失败: {e}"))?;

    let list_sql = format!(
        "SELECT id, entity_type, entity_id, action, operator_name, comment, created_at
         FROM audit_logs
         {where_clause}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = params.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询审计日志失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(list_refs.iter()), |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                action: row.get(3)?,
                operator_name: row.get(4)?,
                comment: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询审计日志失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询审计日志失败: {e}"))?;
    Ok((items, total))
}

// =====================================================================
// 功能单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use std::path::PathBuf;

    fn in_memory_company_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("打开内存数据库失败");
        schema::init_company(&conn).expect("初始化公司库失败");
        conn
    }

    fn sample_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("CARGO_MANIFEST_DIR 没有父目录")
            .join("tests/sample_data/health_company")
    }

    fn sample_file(name: &str) -> PathBuf {
        sample_dir().join(name)
    }

    fn count_records(conn: &rusqlite::Connection, source_type: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM source_records sr
             JOIN source_types st ON sr.source_type_id = st.id
             WHERE st.code = ?1",
            rusqlite::params![source_type],
            |row| row.get(0),
        )
        .expect("查询记录数失败")
    }

    fn batch_row_count(conn: &rusqlite::Connection, batch_id: i64) -> i64 {
        conn.query_row(
            "SELECT row_count FROM import_batches WHERE id = ?1",
            rusqlite::params![batch_id],
            |row| row.get(0),
        )
        .expect("查询批次行数失败")
    }

    #[test]
    fn test_detect_source_type() {
        assert_eq!(detect_source_type("bank_raw.tsv"), Some("bank_flow"));
        assert_eq!(detect_source_type("银行流水.tsv"), Some("bank_flow"));
        assert_eq!(detect_source_type("order_raw.tsv"), Some("order_flow"));
        assert_eq!(detect_source_type("订单流水.tsv"), Some("order_flow"));
        assert_eq!(detect_source_type("pos_raw.tsv"), Some("pos_flow"));
        assert_eq!(detect_source_type("summary_raw.tsv"), Some("summary_flow"));
        assert_eq!(detect_source_type("数据汇总.tsv"), Some("summary_flow"));
        assert_eq!(detect_source_type("unknown.txt"), None);
    }

    #[test]
    fn test_parse_amount() {
        assert_eq!(
            parse_amount("15,639.00"),
            Some(Decimal::from_str_exact("15639.00").unwrap())
        );
        assert_eq!(
            parse_amount("31.5"),
            Some(Decimal::from_str_exact("31.5").unwrap())
        );
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("  "), None);
    }

    #[test]
    fn test_count_data_rows() {
        assert_eq!(count_data_rows(&sample_file("bank_raw.tsv")).unwrap(), 20);
        assert_eq!(count_data_rows(&sample_file("order_raw.tsv")).unwrap(), 13);
        assert_eq!(
            count_data_rows(&sample_file("summary_raw.tsv")).unwrap(),
            11
        );
        assert_eq!(count_data_rows(&sample_file("pos_raw.tsv")).unwrap(), 8);
    }

    #[test]
    fn test_import_bank_raw() {
        let conn = in_memory_company_conn();
        let path = sample_file("bank_raw.tsv");
        let result = import_file_core(&conn, &path, None, None).expect("导入银行流水失败");

        assert_eq!(result.source_type, "bank_flow");
        assert_eq!(result.row_count, 20);
        assert_eq!(batch_row_count(&conn, result.batch_id), 20);
        assert_eq!(count_records(&conn, "bank_flow"), 20);

        // 验证首条记录字段抽取
        let counterpart: String = conn
            .query_row(
                "SELECT counterpart_info FROM source_records LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("查询失败");
        assert_eq!(counterpart, "安泊酒店管理公司");
    }

    #[test]
    fn test_import_order_raw() {
        let conn = in_memory_company_conn();
        let path = sample_file("order_raw.tsv");
        let result = import_file_core(&conn, &path, None, None).expect("导入订单流水失败");

        assert_eq!(result.source_type, "order_flow");
        assert_eq!(result.row_count, 13);
        assert_eq!(count_records(&conn, "order_flow"), 13);
    }

    #[test]
    fn test_import_summary_raw() {
        let conn = in_memory_company_conn();
        let path = sample_file("summary_raw.tsv");
        let result = import_file_core(&conn, &path, None, None).expect("导入数据汇总失败");

        assert_eq!(result.source_type, "summary_flow");
        assert_eq!(result.row_count, 11);
        assert_eq!(count_records(&conn, "summary_flow"), 11);
    }

    #[test]
    fn test_import_pos_raw() {
        let conn = in_memory_company_conn();
        let path = sample_file("pos_raw.tsv");
        let result = import_file_core(&conn, &path, None, None).expect("导入 POS 流水失败");

        assert_eq!(result.source_type, "pos_flow");
        assert_eq!(result.row_count, 8);
        assert_eq!(count_records(&conn, "pos_flow"), 8);
    }

    #[test]
    fn test_scan_directory_pending() {
        let conn = in_memory_company_conn();
        let files = scan_directory_core(&conn, &sample_dir()).expect("扫描目录失败");

        let pending: Vec<_> = files.iter().filter(|f| f.status == "pending").collect();
        assert_eq!(pending.len(), 4, "应检测到 4 个待导入文件");
        assert!(files
            .iter()
            .any(|f| f.file_name == "bank_raw.tsv" && f.source_type == "bank_flow"));
        assert!(files
            .iter()
            .any(|f| f.file_name == "order_raw.tsv" && f.source_type == "order_flow"));
        assert!(files
            .iter()
            .any(|f| f.file_name == "summary_raw.tsv" && f.source_type == "summary_flow"));
        assert!(files
            .iter()
            .any(|f| f.file_name == "pos_raw.tsv" && f.source_type == "pos_flow"));
    }

    #[test]
    fn test_auto_import_directory() {
        let conn = in_memory_company_conn();
        let result = auto_import_directory_core(&conn, &sample_dir(), None).expect("自动导入失败");

        assert_eq!(result.imported.len(), 4, "应导入 4 个文件");
        assert_eq!(result.skipped.len(), 0, "首次导入不应有跳过");
        assert_eq!(result.errors.len(), 0, "不应有错误");

        // 第二次扫描应全部标记为已导入
        let files = scan_directory_core(&conn, &sample_dir()).expect("二次扫描失败");
        assert!(files.iter().all(|f| f.status == "imported"));

        // 再次自动导入应全部跳过
        let result2 =
            auto_import_directory_core(&conn, &sample_dir(), None).expect("二次自动导入失败");
        assert_eq!(result2.skipped.len(), 4, "应跳过 4 个已导入文件");
        assert_eq!(result2.imported.len(), 0);
    }

    #[test]
    fn test_duplicate_import_rejected() {
        let conn = in_memory_company_conn();
        let path = sample_file("pos_raw.tsv");
        import_file_core(&conn, &path, None, None).expect("首次导入失败");
        let err = import_file_core(&conn, &path, None, None).expect_err("重复导入应失败");
        assert!(err.contains("已导入"));
    }
}
