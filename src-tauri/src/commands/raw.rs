use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

// =====================================================================
// 扫描目录
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

    let entries = fs::read_dir(&dir)
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

    with_company_conn(&db, &session, |conn| {
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
    })
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

// =====================================================================
// 自动导入目录
// =====================================================================

#[tauri::command]
pub fn auto_import_raw_directory_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    path: String,
) -> Result<ImportDirResult, String> {
    let files = scan_raw_directory_cmd(db.clone(), session.clone(), path.clone())?;
    let mut result = ImportDirResult {
        imported: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
    };

    for file in files {
        if file.status == "imported" {
            result.skipped.push(file.file_name);
            continue;
        }
        if file.status == "unsupported" {
            result
                .errors
                .push(format!("{}: 不支持的文件类型", file.file_name));
            continue;
        }
        match import_raw_file_cmd(db.clone(), session.clone(), file.file_path, None, None) {
            Ok(r) => result.imported.push(r),
            Err(e) => result.errors.push(format!("{}: {e}", file.file_name)),
        }
    }

    Ok(result)
}

// =====================================================================
// 单文件导入
// =====================================================================

#[tauri::command]
pub fn import_raw_file_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    file_path: String,
    batch_id: Option<String>,
    source_type: Option<String>,
) -> Result<ImportResult, String> {
    let path = PathBuf::from(&file_path);
    if !path.is_file() {
        return Err(format!("文件不存在: {file_path}"));
    }

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let detected = detect_source_type(&file_name);
    let source_type = source_type
        .as_deref()
        .or(detected)
        .ok_or_else(|| format!("无法识别文件类型: {file_name}"))?
        .to_string();

    let file_hash = sha256_file(&path)?;
    let created_by = current_user_id(&session.lock().map_err(|e| format!("会话锁失败: {e}"))?);

    with_company_conn(&db, &session, |conn| {
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
                rusqlite::params![&source_type],
                |row| row.get(0),
            )
            .map_err(|e| format!("未找到来源类型 {source_type}: {e}"))?;

        // 写入批次记录
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

        // 解析文件
        let row_count =
            parse_and_insert_records(conn, &path, source_type_id, import_batch_id, &source_type)?;

        // 更新批次行数
        conn.execute(
            "UPDATE import_batches SET row_count = ?1 WHERE id = ?2",
            rusqlite::params![row_count, import_batch_id],
        )
        .map_err(|e| format!("更新批次行数失败: {e}"))?;

        Ok(ImportResult {
            file_name,
            batch_id: import_batch_id,
            source_type,
            row_count,
            file_hash,
        })
    })
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

    let mut row_no = 0;
    let mut inserted = 0;

    for (idx, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        row_no = idx as i32 + 2; // 1-based, 第 1 行是表头

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
