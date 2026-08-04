use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::OptionalExtension;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

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
    #[serde(default)]
    pub skipped_count: i32,
    #[serde(default)]
    pub balance_check_warning: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImportDirResult {
    pub imported: Vec<ImportResult>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportBatch {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub source_type: String,
    pub row_count: i32,
    pub imported_at: String,
    pub created_by: Option<String>,
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

pub(crate) fn with_company_conn<F, T>(
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

fn yuan_to_cents(s: &str) -> i64 {
    let cleaned = s.replace(',', "").trim().to_string();
    if cleaned.is_empty() {
        return 0;
    }
    if let Some(d) = Decimal::from_str_exact(&cleaned).ok() {
        let cents = d * Decimal::new(100, 0);
        cents.round_dp(0).try_into().unwrap_or(0)
    } else {
        0
    }
}

pub(crate) fn now_str() -> String {
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

/// 计算原始行的指纹 hash,用于审计与排查(不参与数据库唯一约束)。
///
/// 组成:来源类型 + 业务单号(record_no) + 日期 + 金额 + 对方单位。
/// 任一字段为空时仍能稳定产出指纹,空字符串参与拼接而非被省略。
fn compute_row_hash(source_type: &str, fields: &[Option<&String>; 4]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_type.as_bytes());
    hasher.update([0x1f]);
    for field in fields {
        let value: &str = field.as_deref().map_or("", |v| v.as_str());
        hasher.update(value.as_bytes());
        hasher.update([0x1e]);
    }
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
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

        // 数据汇总是由系统按银行流水 / POS 流水 / 微信备注自动派生,
        // 不允许导入。扫描时把这类文件标记为 unsupported,
        // 导入中心就不会展示"导入"按钮。
        if source_type == "summary_flow" {
            result.push(RawFileInfo {
                file_path,
                file_name,
                source_type: source_type.to_string(),
                status: "unsupported".to_string(),
                row_count: 0,
            });
            continue;
        }

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

    // 业务规则:数据汇总(summary_flow) 不允许导入。
    // 实际汇总由系统按"银行流水 + POS 流水 + 微信聊天备注"自动生成,
    // 参见 `generate_summary_core`。这里统一拒绝任何 summary_flow 导入,
    // 不论显式传入还是根据文件名自动识别。
    if source_type == "summary_flow" {
        return Err(format!(
            "数据汇总不允许导入: 数据汇总由系统自动从银行流水、POS 流水和微信备注生成,无需上传 ({file_name})"
        ));
    }

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

    let (row_count, skipped_count, balance_check_warning) =
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
        skipped_count,
        balance_check_warning,
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
) -> Result<(i32, i32, Option<String>), String> {
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

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启导入事务失败: {e}"))?;

    let mut inserted: i32 = 0;
    let mut skipped: i32 = 0;
    let ts_now = now_str();

    // 仅对银行流水启用余额连续性校验。
    // 跟踪:首行余额、末行余额、累计 Σ转入 - Σ转出、上行余额。
    let is_bank_flow = source_type == "bank_flow";
    let is_order_flow = source_type == "order_flow";
    let is_pos_flow = source_type == "pos_flow";
    let mut first_balance: Option<Decimal> = None;
    let mut last_balance: Option<Decimal> = None;
    let mut sum_in: Decimal = Decimal::ZERO;
    let mut sum_out: Decimal = Decimal::ZERO;
    let mut prev_balance: Option<Decimal> = None;
    let mut discontinuity_rows: i32 = 0;

    for (idx, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row_no = idx as i32 + 1; // 1-based data row (header already consumed)

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
        let balance = extract_balance(&map);

        let row_hash = compute_row_hash(
            source_type,
            &[
                record_no.as_ref(),
                record_date.as_ref(),
                amount_total.as_ref(),
                counterpart_info.as_ref(),
            ],
        );

        // 余额连续性校验(银行流水专属)
        let mut balance_disc: Option<String> = None;
        if is_bank_flow {
            let cur_in = extract_decimal(&map, &["转入金额"]);
            let cur_out = extract_decimal(&map, &["转出金额"]);
            let cur_bal_decimal = balance.as_ref().and_then(|v| parse_amount(v));

            if let (Some(prev), Some(cur_bal), Some(in_amt), Some(out_amt)) =
                (prev_balance, cur_bal_decimal, cur_in, cur_out)
            {
                let expected = prev + in_amt - out_amt;
                if expected != cur_bal {
                    let diff = expected - cur_bal;
                    balance_disc = Some(format!(
                        "余额不连续:上一行余额 {prev} + 转入 {in_amt} - 转出 {out_amt} = {expected},实际 {cur_bal},差额 {diff}"
                    ));
                }
            }

            if let Some(b) = cur_bal_decimal {
                if first_balance.is_none() {
                    first_balance = Some(b);
                }
                last_balance = Some(b);
                prev_balance = Some(b);
            }
            if let Some(in_amt) = cur_in {
                sum_in += in_amt;
            }
            if let Some(out_amt) = cur_out {
                sum_out += out_amt;
            }
        }

        // 银行流水:写入 bank_flows 独立表(金额以分为单位)
        if is_bank_flow {
            let amount_in_str = extract_value(&map, &["转入金额"]).unwrap_or_default();
            let amount_out_str = extract_value(&map, &["转出金额"]).unwrap_or_default();
            let amount_total_str = amount_total.clone().unwrap_or_default();
            let balance_str = balance.clone().unwrap_or_default();

            let amount_in_cents = yuan_to_cents(&amount_in_str);
            let amount_out_cents = yuan_to_cents(&amount_out_str);
            let amount_total_cents = yuan_to_cents(&amount_total_str);
            let balance_cents = if balance_str.is_empty() {
                None
            } else {
                Some(yuan_to_cents(&balance_str))
            };

            let insert_res = tx.execute(
                "INSERT INTO bank_flows
                 (import_batch_id, source_file_name, source_row_no, record_no, record_date,
                  amount_in, amount_out, amount_total, balance, currency, counterpart_info,
                  summary, raw_data, row_hash, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11, ?12, ?13, 'pending', ?14)",
                rusqlite::params![
                    import_batch_id,
                    source_file_name,
                    row_no,
                    record_no,
                    record_date,
                    amount_in_cents,
                    amount_out_cents,
                    amount_total_cents,
                    balance_cents,
                    counterpart_info,
                    summary,
                    raw_data,
                    row_hash,
                    ts_now,
                ],
            );

            match insert_res {
                Ok(_) => {
                    inserted += 1;
                    if let Some(msg) = balance_disc {
                        tx.execute(
                            "INSERT INTO import_errors
                             (import_batch_id, source_row_no, field_name, field_value, error_message, created_at)
                             VALUES (?1, ?2, 'balance_discontinuity', ?3, ?4, ?5)",
                            rusqlite::params![
                                import_batch_id,
                                row_no,
                                balance.as_deref().unwrap_or(""),
                                msg,
                                ts_now,
                            ],
                        )
                        .map_err(|e| format!("写入 import_errors 失败 (行 {row_no}): {e}"))?;
                        discontinuity_rows += 1;
                    }
                }
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    let dup_value = record_no
                        .clone()
                        .or_else(|| record_date.clone())
                        .unwrap_or_default();
                    tx.execute(
                        "INSERT INTO import_errors
                         (import_batch_id, source_row_no, field_name, field_value, error_message, created_at)
                         VALUES (?1, ?2, 'duplicate_row', ?3, ?4, ?5)",
                        rusqlite::params![
                            import_batch_id,
                            row_no,
                            dup_value,
                            "原始行已存在,跳过入库(可能来源文件存在重叠)",
                            ts_now,
                        ],
                    )
                    .map_err(|e| format!("写入 import_errors 失败 (行 {row_no}): {e}"))?;
                    skipped += 1;
                }
                Err(e) => {
                    return Err(format!("写入 bank_flows 失败 (行 {row_no}): {e}"));
                }
            }
        } else if is_order_flow {
            // 订单流水:写入 order_flows 独立表(金额以分为单位)
            let amount_total_str = amount_total.clone().unwrap_or_default();
            let amount_total_cents = yuan_to_cents(&amount_total_str);

            let insert_res = tx.execute(
                "INSERT INTO order_flows
                 (import_batch_id, source_file_name, source_row_no, record_no, record_date,
                  amount_total, currency, counterpart_info, summary, raw_data, row_hash, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9, ?10, 'pending', ?11)",
                rusqlite::params![
                    import_batch_id,
                    source_file_name,
                    row_no,
                    record_no,
                    record_date,
                    amount_total_cents,
                    counterpart_info,
                    summary,
                    raw_data,
                    row_hash,
                    ts_now,
                ],
            );

            match insert_res {
                Ok(_) => {
                    inserted += 1;
                }
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    let dup_value = record_no
                        .clone()
                        .or_else(|| record_date.clone())
                        .unwrap_or_default();
                    tx.execute(
                        "INSERT INTO import_errors
                         (import_batch_id, source_row_no, field_name, field_value, error_message, created_at)
                         VALUES (?1, ?2, 'duplicate_row', ?3, ?4, ?5)",
                        rusqlite::params![
                            import_batch_id,
                            row_no,
                            dup_value,
                            "原始行已存在,跳过入库(可能来源文件存在重叠)",
                            ts_now,
                        ],
                    )
                    .map_err(|e| format!("写入 import_errors 失败 (行 {row_no}): {e}"))?;
                    skipped += 1;
                }
                Err(e) => {
                    return Err(format!("写入 order_flows 失败 (行 {row_no}): {e}"));
                }
            }
        } else if is_pos_flow {
            // POS 流水:写入 source_records(尚无独立表)
            let insert_res = tx.execute(
                "INSERT INTO source_records
                 (source_type_id, import_batch_id, source_file_name, source_row_no, record_no, record_date, amount_total, balance, currency, counterpart_info, summary, raw_data, row_hash, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'CNY', ?9, ?10, ?11, ?12, 'pending', ?13)",
                rusqlite::params![
                    source_type_id,
                    import_batch_id,
                    source_file_name,
                    row_no,
                    record_no,
                    record_date,
                    amount_total,
                    balance,
                    counterpart_info,
                    summary,
                    raw_data,
                    row_hash,
                    ts_now,
                ],
            );

            match insert_res {
                Ok(_) => {
                    inserted += 1;
                }
                Err(rusqlite::Error::SqliteFailure(err, _))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    let dup_value = record_no
                        .clone()
                        .or_else(|| record_date.clone())
                        .unwrap_or_default();
                    tx.execute(
                        "INSERT INTO import_errors
                         (import_batch_id, source_row_no, field_name, field_value, error_message, created_at)
                         VALUES (?1, ?2, 'duplicate_row', ?3, ?4, ?5)",
                        rusqlite::params![
                            import_batch_id,
                            row_no,
                            dup_value,
                            "原始行已存在,跳过入库(可能来源文件存在重叠)",
                            ts_now,
                        ],
                    )
                    .map_err(|e| format!("写入 import_errors 失败 (行 {row_no}): {e}"))?;
                    skipped += 1;
                }
                Err(e) => {
                    return Err(format!("写入 source_records 失败 (行 {row_no}): {e}"));
                }
            }
        }
    }

    // 首末余额差额自检(银行流水专属)
    let balance_warning = if is_bank_flow {
        check_balance_first_last(
            first_balance,
            last_balance,
            sum_in,
            sum_out,
            discontinuity_rows,
        )
    } else {
        None
    };

    tx.commit().map_err(|e| format!("提交导入事务失败: {e}"))?;

    Ok((inserted, skipped, balance_warning))
}

/// 从 TSV 行 map 中抽出"余额"字段(Decimal-as-string)。
fn extract_balance(map: &HashMap<String, String>) -> Option<String> {
    if let Some(v) = map.get("余额") {
        if !v.is_empty() {
            if let Some(d) = parse_amount(v) {
                return Some(d.to_string());
            }
        }
    }
    None
}

/// 从 TSV 行 map 中抽出指定键的第一个非空值,转成 Decimal。
fn extract_decimal(map: &HashMap<String, String>, keys: &[&str]) -> Option<Decimal> {
    for key in keys {
        if let Some(v) = map.get(*key) {
            if !v.is_empty() {
                if let Some(d) = parse_amount(v) {
                    return Some(d);
                }
            }
        }
    }
    None
}

/// 首末余额 vs Σ差额自检。
///
/// 当文件至少 2 行有余额时计算 `末行余额 - 首行余额` 与 `Σ转入 - Σ转出`,
/// 不一致时返回警告字符串。Decimal 严格加减,不应用容差 —— 任何不平都是真问题,
/// 应由财务人员人工确认。
fn check_balance_first_last(
    first: Option<Decimal>,
    last: Option<Decimal>,
    sum_in: Decimal,
    sum_out: Decimal,
    discontinuity_rows: i32,
) -> Option<String> {
    if first.is_none() || last.is_none() {
        return None;
    }
    let first = first?;
    let last = last?;
    let by_balance = last - first;
    let by_amount = sum_in - sum_out;
    let mut parts: Vec<String> = Vec::new();
    if by_balance != by_amount {
        let diff = by_balance - by_amount;
        parts.push(format!(
            "首末余额差额 {by_balance} 与 Σ(转入) - Σ(转出) = {by_amount} 不一致,差额 {diff}"
        ));
    }
    if discontinuity_rows > 0 {
        parts.push(format!(
            "{discontinuity_rows} 行余额连续性校验失败(疑似文件截断/篡改)"
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

// =====================================================================
// Tauri 命令封装
// =====================================================================

#[tauri::command]
pub async fn select_raw_directory_cmd(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();

    match folder {
        Some(path) => Ok(path.to_string()),
        None => Err("用户取消了选择".to_string()),
    }
}

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
pub fn list_import_batches_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    source_type: Option<String>,
    months: Option<i32>,
) -> Result<Vec<ImportBatch>, String> {
    with_company_conn(&db, &session, |conn| {
        list_import_batches_core(conn, source_type.as_deref(), months)
    })
}

#[tauri::command]
pub fn get_import_batch_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    batch_id: i64,
) -> Result<Option<ImportBatch>, String> {
    with_company_conn(&db, &session, |conn| get_import_batch_core(conn, batch_id))
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
pub fn generate_summary_cmd(
    db: State<'_, std::sync::Mutex<DbState>>,
    session: State<'_, std::sync::Mutex<SessionState>>,
    date_from: String,
    date_to: String,
) -> Result<GenerateSummaryResult, String> {
    with_company_conn(&db, &session, |conn| {
        generate_summary_core(conn, &date_from, &date_to)
    })
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

#[tauri::command]
pub async fn read_source_file_cmd(file_path: String) -> Result<String, String> {
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("读取文件失败: {e}"))?;
    Ok(content)
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
    pub balance: Option<String>,
    pub currency: String,
    pub counterpart_info: Option<String>,
    pub summary: Option<String>,
    pub status: String,
    pub created_at: String,
    pub file_path: String,
    /// 余额连续性校验结果:
    /// - `"ok"`:本行余额与(上一行余额 + 转入 - 转出)一致
    /// - `"mismatch"`:不一致
    /// - `"skip"`:无法计算(余额为空/缺转入或转出/无上一行)
    /// - `None`:不适用(非 bank_flow)
    #[serde(default)]
    pub balance_check_status: Option<String>,
    /// 余额连续性财务确认时间(ISO8601 字符串)。
    /// 非 NULL 时表示该行已被财务人员确认,
    /// 即便 `balance_check_status` 为 `mismatch` 也不在 UI 展示红底告警。
    #[serde(default)]
    pub balance_confirmed_at: Option<String>,
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

/// 数据汇总生成结果(占位):后续实现按日期范围从三类源数据派生汇总行。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GenerateSummaryResult {
    pub date_from: String,
    pub date_to: String,
    pub generated_count: i32,
    pub skipped_count: i32,
    pub errors: Vec<String>,
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

pub fn list_import_batches_core(
    conn: &rusqlite::Connection,
    source_type: Option<&str>,
    months: Option<i32>,
) -> Result<Vec<ImportBatch>, String> {
    let mut sql = String::from(
        "SELECT id, file_path, file_name, source_type, row_count, imported_at, created_by
         FROM import_batches WHERE 1 = 1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(t) = source_type {
        sql.push_str(" AND source_type = ?");
        params.push(Box::new(t.to_string()));
    }
    if let Some(m) = months {
        sql.push_str(" AND imported_at >= date('now', ?)");
        params.push(Box::new(format!("-{} months", m)));
    }
    sql.push_str(" ORDER BY imported_at DESC");

    let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("查询导入批次失败: {e}"))?;
    let items = stmt
        .query_map(rusqlite::params_from_iter(refs.iter()), |row| {
            Ok(ImportBatch {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                source_type: row.get(3)?,
                row_count: row.get(4)?,
                imported_at: row.get(5)?,
                created_by: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询导入批次失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询导入批次失败: {e}"))?;
    Ok(items)
}

pub fn get_import_batch_core(
    conn: &rusqlite::Connection,
    batch_id: i64,
) -> Result<Option<ImportBatch>, String> {
    conn.query_row(
        "SELECT id, file_path, file_name, source_type, row_count, imported_at, created_by
         FROM import_batches WHERE id = ?1",
        rusqlite::params![batch_id],
        |row| {
            Ok(ImportBatch {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                source_type: row.get(3)?,
                row_count: row.get(4)?,
                imported_at: row.get(5)?,
                created_by: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(|e| format!("查询导入批次失败: {e}"))
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

    // 计算余额连续性(仅对 bank_flow 适用)
    let balance_check_map: std::collections::HashMap<i64, String> = if filter
        .source_type
        .as_deref()
        .map(|t| t == "bank_flow")
        .unwrap_or(true)
    {
        let rows = fetch_balance_check_rows(conn, filter.source_type.as_deref())?;
        compute_balance_check_status(&rows)
    } else {
        std::collections::HashMap::new()
    };

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
                sr.record_no, sr.record_date, sr.amount_total, sr.balance, sr.currency, sr.counterpart_info,
                sr.summary, sr.status, sr.created_at, ib.file_path, sr.balance_confirmed_at
         FROM source_records sr
         JOIN source_types st ON sr.source_type_id = st.id
         LEFT JOIN import_batches ib ON sr.import_batch_id = ib.id
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
            let id: i64 = row.get(0)?;
            let balance_check_status = balance_check_map.get(&id).cloned();
            Ok(RawRecord {
                id,
                source_type: row.get(1)?,
                source_type_name: row.get(2)?,
                import_batch_id: row.get(3)?,
                source_file_name: row.get(4)?,
                source_row_no: row.get(5)?,
                record_no: row.get(6)?,
                record_date: row.get(7)?,
                amount_total: row.get(8)?,
                balance: row.get(9)?,
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

/// 余额连续性校验的轻量数据(从 raw_data 抽取必要字段)
#[derive(Debug)]
pub(crate) struct BalanceCheckRow {
    pub(crate) id: i64,
    pub(crate) balance: Option<String>,
    /// raw_data JSON 字符串
    pub(crate) raw_data: String,
}

/// 解析 raw_data JSON,提取"转入金额"、"转出金额"。
/// 银行流水 TSV 字段名为"转入金额"和"转出金额",返回的元组表示(in, out)绝对值。
/// 任意字段缺失或解析失败返回 None。
fn parse_amount_from_raw_data(raw_data: &str) -> Option<(Decimal, Decimal)> {
    let v: serde_json::Value = serde_json::from_str(raw_data).ok()?;
    let obj = v.as_object()?;
    let in_amt = obj
        .get("转入金额")
        .and_then(|x| x.as_str())
        .and_then(|s| parse_amount(s))?;
    let out_amt = obj
        .get("转出金额")
        .and_then(|x| x.as_str())
        .and_then(|s| parse_amount(s))?;
    Some((in_amt, out_amt))
}

/// 对一批银行流水的轻量数据,正序遍历,计算每行的余额连续性状态。
/// 返回 `id -> status` 映射,status 取值:
/// - `"ok"`:本行余额与(prev + in - out)严格相等(Decimal 严格加减,不需要容差)
/// - `"mismatch"`:不一致 — 需财务人员确认;确认后 UI 不再显示红底
/// - `"skip"`:无法计算(余额为空/缺转入或转出/无上一行)
pub fn compute_balance_check_status(
    rows: &[BalanceCheckRow],
) -> std::collections::HashMap<i64, String> {
    let mut out = std::collections::HashMap::with_capacity(rows.len());
    let mut prev_balance: Option<Decimal> = None;

    for row in rows {
        let cur_balance = row.balance.as_deref().and_then(parse_amount);

        let status = match (prev_balance, cur_balance) {
            (Some(prev), Some(cur)) => {
                if let Some((in_amt, out_amt)) = parse_amount_from_raw_data(&row.raw_data) {
                    let expected = prev + in_amt - out_amt;
                    if expected == cur {
                        "ok".to_string()
                    } else {
                        "mismatch".to_string()
                    }
                } else {
                    "skip".to_string()
                }
            }
            _ => "skip".to_string(),
        };

        if let Some(b) = cur_balance {
            prev_balance = Some(b);
        }
        out.insert(row.id, status);
    }
    out
}

/// 拉取所有 bank_flows 的轻量数据(用于余额连续性校验)。
///
/// 用于计算余额连续性,需要全量正序遍历,因此不分页。
pub(crate) fn fetch_balance_check_rows(
    conn: &rusqlite::Connection,
    _source_type: Option<&str>,
) -> Result<Vec<BalanceCheckRow>, String> {
    let mut sql = String::from(
        "SELECT bf.id, bf.balance, bf.raw_data
         FROM bank_flows bf",
    );
    let params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    sql.push_str(" ORDER BY bf.record_date ASC, bf.id ASC");

    let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("查询余额校验数据失败: {e}"))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(refs.iter()), |row| {
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

pub fn get_raw_record_core(
    conn: &rusqlite::Connection,
    id: i64,
) -> Result<Option<RawRecordDetail>, String> {
    let record_opt = conn
        .query_row(
            "SELECT sr.id, st.code, st.name, sr.import_batch_id, sr.source_file_name, sr.source_row_no,
                    sr.record_no, sr.record_date, sr.amount_total, sr.balance, sr.currency, sr.counterpart_info,
                    sr.summary, sr.status, sr.created_at, sr.raw_data, ib.file_path, sr.balance_confirmed_at
             FROM source_records sr
             JOIN source_types st ON sr.source_type_id = st.id
             LEFT JOIN import_batches ib ON sr.import_batch_id = ib.id
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
                        balance: row.get(9)?,
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
        .map_err(|e| format!("查询原始记录详情失败: {e}"))?;

    let Some((mut record, raw_data)) = record_opt else {
        return Ok(None);
    };

    // 计算余额连续性(仅 bank_flow)
    if record.source_type == "bank_flow" {
        let check_rows = fetch_balance_check_rows(conn, Some("bank_flow"))?;
        let map = compute_balance_check_status(&check_rows);
        record.balance_check_status = map.get(&record.id).cloned();
    }

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

/// 自动生成数据汇总(骨架函数)。
///
/// 业务规则:数据汇总(`summary_flow`) 不允许用户手动导入,
/// 而是由系统根据以下三项源数据自动派生:
///   1. 银行流水(`bank_flow`)
///   2. 工商商户 POS 流水(`pos_flow`)
///   3. 微信聊天记录的说明信息(待 source_type 落地后启用)
///
/// 当前函数仅记录调用并返回占位结果;具体生成规则待后续迭代。
/// 已生成的汇总行应写入 `source_records` 表(`source_type='summary_flow'`),
/// 标记为系统生成,可被既有 raw_record_table / 对账 / 凭证生成流程复用。
pub fn generate_summary_core(
    conn: &rusqlite::Connection,
    date_from: &str,
    date_to: &str,
) -> Result<GenerateSummaryResult, String> {
    let _ = conn; // 占位:未来实现需在此打开事务并写入 source_records

    // 校验日期参数
    if date_from.trim().is_empty() || date_to.trim().is_empty() {
        return Err("日期范围不能为空".to_string());
    }
    if date_from > date_to {
        return Err(format!("起始日期 {date_from} 晚于结束日期 {date_to}"));
    }

    // 占位实现:当前不真正生成汇总行,仅返回空结果。
    // 后续在此实现以下逻辑:
    //   - 联表 bank_flow / pos_flow 按日期分组,聚合 Σ(转入-转出) 与 Σ(订单金额-手续费)
    //   - 对每条待生成行写入 source_records(source_type_id=summary_flow, status='auto_generated')
    //   - 微信备注单独查表合并;在本轮尚未落地,先跳过
    Ok(GenerateSummaryResult {
        date_from: date_from.to_string(),
        date_to: date_to.to_string(),
        generated_count: 0,
        skipped_count: 0,
        errors: Vec::new(),
    })
}

pub fn reconcile_core(conn: &rusqlite::Connection, date: &str) -> Result<ReconcileResult, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;

    // 计算当日银行流水合计(金额以分为单位,除以 100.0 转为元)
    let bank_total_cents: i64 = tx
        .query_row(
            "SELECT COALESCE(SUM(amount_total), 0)
             FROM bank_flows
             WHERE record_date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .map_err(|e| format!("汇总银行金额失败: {e}"))?;
    let bank_total: Decimal = Decimal::new(bank_total_cents, 2);

    // 计算当日订单实收合计(金额以分为单位,除以 100.0 转为元)
    let order_total_cents: i64 = tx
        .query_row(
            "SELECT COALESCE(SUM(amount_total), 0)
             FROM order_flows
             WHERE record_date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .map_err(|e| format!("汇总订单金额失败: {e}"))?;
    let order_total: Decimal = Decimal::new(order_total_cents, 2);

    let diff = bank_total - order_total;
    let matched = diff.abs() < Decimal::new(1, 2); // 差额 < 0.01 视为无差异

    // 获取当日银行/订单记录 ID
    let bank_ids: Vec<i64> = {
        let mut stmt = tx
            .prepare(
                "SELECT id FROM bank_flows
                 WHERE record_date = ?1
                 ORDER BY id",
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
                "SELECT id FROM order_flows
                 WHERE record_date = ?1
                 ORDER BY id",
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

    // 若自动匹配,更新 bank_flows 和 order_flows 状态为 matched
    if matched {
        for id in bank_ids.iter() {
            tx.execute(
                "UPDATE bank_flows SET status = 'matched' WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("更新银行流水状态失败: {e}"))?;
        }
        for id in order_ids.iter() {
            tx.execute(
                "UPDATE order_flows SET status = 'matched' WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| format!("更新订单流水状态失败: {e}"))?;
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
        match source_type {
            "bank_flow" => conn
                .query_row("SELECT COUNT(*) FROM bank_flows", [], |row| row.get(0))
                .expect("查询 bank_flows 记录数失败"),
            "order_flow" => conn
                .query_row("SELECT COUNT(*) FROM order_flows", [], |row| row.get(0))
                .expect("查询 order_flows 记录数失败"),
            _ => conn
                .query_row(
                    "SELECT COUNT(*) FROM source_records sr
                     JOIN source_types st ON sr.source_type_id = st.id
                     WHERE st.code = ?1",
                    rusqlite::params![source_type],
                    |row| row.get(0),
                )
                .expect("查询记录数失败"),
        }
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
                "SELECT counterpart_info FROM bank_flows LIMIT 1",
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
        let err = import_file_core(&conn, &path, None, None).expect_err("数据汇总应被拒绝导入");
        assert!(err.contains("不允许导入"), "实际错误: {err}");
        assert_eq!(count_records(&conn, "summary_flow"), 0);
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

        // 数据汇总不允许导入,扫描时标记为 unsupported;
        // 其它三类文件(bank/order/pos) 仍为 pending。
        let pending: Vec<_> = files.iter().filter(|f| f.status == "pending").collect();
        assert_eq!(
            pending.len(),
            3,
            "应检测到 3 个待导入文件(数据汇总是 unsupported)"
        );

        let unsupported: Vec<_> = files.iter().filter(|f| f.status == "unsupported").collect();
        assert!(unsupported
            .iter()
            .any(|f| f.file_name == "summary_raw.tsv" && f.source_type == "summary_flow"));

        assert!(files
            .iter()
            .any(|f| f.file_name == "bank_raw.tsv" && f.source_type == "bank_flow"));
        assert!(files
            .iter()
            .any(|f| f.file_name == "order_raw.tsv" && f.source_type == "order_flow"));
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

    /// 写入临时 TSV,测试结束后删除。
    fn write_temp_tsv(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("forgefin_raw_tests");
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        let path = dir.join(name);
        std::fs::write(&path, content).expect("写入临时 TSV 失败");
        path
    }

    fn unique_temp_name(prefix: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{prefix}_{nanos}.tsv")
    }

    /// 模拟"同一个订单流水分两次导出,日期范围重叠"。
    /// 第一次导入 3 条,第二次导入 2 条(其中 1 条与第一次重叠)——
    /// 期望第二次 inserted=1, skipped=1。
    #[test]
    fn test_overlapping_order_import_dedups_by_record_no() {
        let conn = in_memory_company_conn();

        let file_a = write_temp_tsv(
            &unique_temp_name("order_a"),
            "工行订单号\t商户实收金额\t交易时间\n\
             ORD-001\t100.00\t2026-07-01 10:00:00\n\
             ORD-002\t200.00\t2026-07-02 10:00:00\n\
             ORD-003\t300.00\t2026-07-03 10:00:00\n",
        );
        let file_b = write_temp_tsv(
            &unique_temp_name("order_b"),
            "工行订单号\t商户实收金额\t交易时间\n\
             ORD-002\t200.00\t2026-07-02 10:00:00\n\
             ORD-004\t400.00\t2026-07-04 10:00:00\n",
        );

        let r1 = import_file_core(&conn, &file_a, None, None).expect("首次导入失败");
        assert_eq!(r1.row_count, 3);
        assert_eq!(r1.skipped_count, 0);

        let r2 = import_file_core(&conn, &file_b, None, None).expect("二次导入应成功");
        assert_eq!(r2.row_count, 1, "仅新订单 ORD-004 应入库");
        assert_eq!(r2.skipped_count, 1, "重叠的 ORD-002 应被跳过");

        // 总计仍为 4 条,无重复
        assert_eq!(count_records(&conn, "order_flow"), 4);

        // import_errors 应记录 1 条 duplicate_row
        let err_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM import_errors WHERE field_name = 'duplicate_row'",
                [],
                |row| row.get(0),
            )
            .expect("查询 import_errors 失败");
        assert_eq!(err_count, 1);

        let _ = std::fs::remove_file(&file_a);
        let _ = std::fs::remove_file(&file_b);
    }

    /// 银行流水的 record_no 几乎全是占位符 `000000000`,
    /// 因此不参与业务级去重 —— 应保证样本能完整入库,不被误伤。
    #[test]
    fn test_bank_flow_with_placeholder_record_no_imports_all_rows() {
        let conn = in_memory_company_conn();

        let bank = write_temp_tsv(
            &unique_temp_name("bank_placeholder"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t摘要\n\
             000000000\t2026-07-10 10:00:00\t安泊酒店\t0.00\t4552.00\t房费\n\
             000000000\t2026-07-10 13:39:00\t明瑞科技\t0.00\t4800.00\t货款\n\
             000000000\t2026-07-10 22:13:00\t银行\t0.00\t9.00\t跨行手续费\n",
        );

        let r = import_file_core(&conn, &bank, None, None).expect("银行流水应完整入库");
        assert_eq!(r.row_count, 3);
        assert_eq!(r.skipped_count, 0);
        assert_eq!(count_records(&conn, "bank_flow"), 3);

        let _ = std::fs::remove_file(&bank);
    }

    /// 数据汇总是由系统按银行流水/POS 流水/微信备注自动派生,不允许用户导入。
    /// 此测试验证即便文件名不包含 "summary" 关键字,只要显式传入 summary_flow 也应被拒绝。
    #[test]
    fn test_summary_import_is_blocked() {
        let conn = in_memory_company_conn();

        // 文件名故意不含 "summary"/"汇总" 等关键字,但显式 source_type="summary_flow"
        let file = write_temp_tsv(
            &unique_temp_name("plain"),
            "日期\t收据编号\t事由\t实际收入\t支出\t备注\n\
             2026-07-10\tR001\t王浩然产康充值\t11970.00\t0.00\t微信\n",
        );

        let err = import_file_core(&conn, &file, Some("summary_flow"), None)
            .expect_err("summary_flow 显式导入应被拒绝");
        assert!(err.contains("不允许导入"), "实际错误: {err}");
        assert_eq!(count_records(&conn, "summary_flow"), 0);

        let _ = std::fs::remove_file(&file);
    }

    /// 银行流水导入后,source_records.balance 列应被正确填充。
    #[test]
    fn test_bank_flow_import_writes_balance_column() {
        let conn = in_memory_company_conn();

        // 连续性 OK 且首末差额 OK:
        //   第 1 行: in=2000, out=500, balance=100000
        //   第 2 行: in=500,  out=0,   balance=101500 (100000+2000-500+500-0=102000...见下)
        // 重新设计:
        //   第 1 行: in=0, out=0, balance=100000
        //   第 2 行: in=1000, out=0, balance=101000
        //     连续性: 100000 + 1000 - 0 = 101000 ✓
        //   Σ(末-首) = 1000, Σ(转入-转出) = 1000 ✓
        let bank = write_temp_tsv(
            &unique_temp_name("bank_balance"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\t安泊酒店\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 13:39:00\t明瑞科技\t1000.00\t0.00\t101000.00\n",
        );

        let r = import_file_core(&conn, &bank, None, None).expect("导入失败");
        assert_eq!(r.row_count, 2);
        assert!(
            r.balance_check_warning.is_none(),
            "正常银行流水不应产生余额警告: {:?}",
            r.balance_check_warning
        );

        let balances: Vec<i64> = conn
            .prepare("SELECT balance FROM bank_flows ORDER BY source_row_no")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(balances, vec![10000000, 10100000]);

        let _ = std::fs::remove_file(&bank);
    }

    /// 余额连续性校验:文件被篡改时,差额行应写入 import_errors。
    #[test]
    fn test_bank_flow_balance_continuity_warning_on_tampered_file() {
        let conn = in_memory_company_conn();

        // 第二行的"余额"被改小 100.00,导致 prev + in - out != cur
        let bank = write_temp_tsv(
            &unique_temp_name("bank_tampered"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\t安泊酒店\t0.00\t1000.00\t100000.00\n\
             000000000\t2026-07-10 13:39:00\t明瑞科技\t500.00\t0.00\t99500.00\n\
             000000000\t2026-07-11 09:00:00\t明瑞科技\t0.00\t200.00\t99300.00\n",
        );

        let r = import_file_core(&conn, &bank, None, None).expect("导入失败");
        assert_eq!(r.row_count, 3);
        let warning = r.balance_check_warning.expect("应返回余额校验警告");
        assert!(
            warning.contains("余额不连续") || warning.contains("连续性"),
            "warning={warning}"
        );

        let err_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM import_errors WHERE field_name = 'balance_discontinuity'",
                [],
                |row| row.get(0),
            )
            .expect("查询 import_errors 失败");
        assert_eq!(err_count, 1, "仅中间一行连续性失败");

        let _ = std::fs::remove_file(&bank);
    }

    /// 首末余额差额自检:文件整体 Σ(转入-转出) 不等于 末-首 余额。
    #[test]
    fn test_bank_flow_first_last_balance_check_warns_on_mismatch() {
        let conn = in_memory_company_conn();

        // 首余额 100000,末余额 105000,差 5000
        // 但 Σ(转入-转出) = (0-1000) + (1000-0) + (0-0) + (0-0) = 0
        // → 首末差额自检触发
        let bank = write_temp_tsv(
            &unique_temp_name("bank_first_last"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t1000.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 12:00:00\tC\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 13:00:00\tD\t0.00\t0.00\t105000.00\n",
        );

        let r = import_file_core(&conn, &bank, None, None).expect("导入失败");
        let warning = r.balance_check_warning.expect("应返回余额校验警告");
        assert!(
            warning.contains("首末余额差额") || warning.contains("Σ"),
            "warning={warning}"
        );

        let _ = std::fs::remove_file(&bank);
    }

    /// 订单流水的导入不应触发任何余额校验。
    #[test]
    fn test_order_flow_has_no_balance_check() {
        let conn = in_memory_company_conn();

        let order = write_temp_tsv(
            &unique_temp_name("order_no_balance"),
            "工行订单号\t商户实收金额\t交易时间\n\
             ORD-A\t100.00\t2026-07-10 10:00:00\n\
             ORD-B\t200.00\t2026-07-11 10:00:00\n",
        );

        let r = import_file_core(&conn, &order, None, None).expect("导入失败");
        assert_eq!(r.row_count, 2);
        assert!(
            r.balance_check_warning.is_none(),
            "订单流水不应返回余额校验警告"
        );

        let _ = std::fs::remove_file(&order);
    }

    /// 数据汇总:即使显式传入 source_type,也不允许导入。
    #[test]
    fn test_summary_flow_explicit_import_rejected() {
        let conn = in_memory_company_conn();
        let order = write_temp_tsv(
            &unique_temp_name("disguised_summary"),
            "工行订单号\t商户实收金额\t交易时间\n\
             ORD-A\t100.00\t2026-07-10 10:00:00\n",
        );

        // 显式传入 summary_flow,应被拒绝
        let err = import_file_core(&conn, &order, Some("summary_flow"), None)
            .expect_err("显式 summary_flow 导入应被拒绝");
        assert!(err.contains("不允许导入"), "实际错误: {err}");

        let _ = std::fs::remove_file(&order);
    }

    /// generate_summary_core 占位实现:返回空结果,不写入 source_records。
    #[test]
    fn test_generate_summary_core_returns_placeholder() {
        let conn = in_memory_company_conn();

        // 正常区间
        let r = generate_summary_core(&conn, "2026-07-01", "2026-07-31").expect("生成失败");
        assert_eq!(r.date_from, "2026-07-01");
        assert_eq!(r.date_to, "2026-07-31");
        assert_eq!(r.generated_count, 0);
        assert!(r.errors.is_empty());

        // 起始日期晚于结束日期
        assert!(generate_summary_core(&conn, "2026-07-31", "2026-07-01").is_err());

        // 空日期
        assert!(generate_summary_core(&conn, "", "2026-07-01").is_err());
        assert!(generate_summary_core(&conn, "2026-07-01", "  ").is_err());
    }

    /// 列显示偏好:无记录时返回默认值(全可见)。
    #[test]
    fn test_get_column_prefs_returns_default_when_empty() {
        let conn = in_memory_company_conn();
        let prefs =
            crate::commands::ui_prefs::get_column_prefs_core(&conn, "bank_flow").expect("读取");
        assert_eq!(prefs.source_type, "bank_flow");
        for key in crate::commands::ui_prefs::COLUMN_KEYS {
            assert_eq!(
                prefs.columns.get(*key).copied(),
                Some(true),
                "默认应全可见,但 {key} 不是"
            );
        }
    }

    /// 列显示偏好:保存后再次读取应一致。
    #[test]
    fn test_save_and_reload_column_prefs() {
        let conn = in_memory_company_conn();

        // 关闭部分列
        let mut cols = crate::commands::ui_prefs::default_columns();
        cols.insert("balance".to_string(), false);
        cols.insert("summary".to_string(), false);
        crate::commands::ui_prefs::save_column_prefs_core(&conn, "bank_flow", &cols).expect("保存");

        // 重新读取
        let prefs =
            crate::commands::ui_prefs::get_column_prefs_core(&conn, "bank_flow").expect("读取");
        assert_eq!(prefs.columns.get("balance"), Some(&false));
        assert_eq!(prefs.columns.get("summary"), Some(&false));
        // 其它列保持默认 true
        assert_eq!(prefs.columns.get("amount_total"), Some(&true));
    }

    /// 列显示偏好:全量提交语义。再次保存时覆盖之前的全集合。
    #[test]
    fn test_save_column_prefs_overwrites() {
        let conn = in_memory_company_conn();

        // 第一次保存:关闭 balance
        let mut cols1 = crate::commands::ui_prefs::default_columns();
        cols1.insert("balance".to_string(), false);
        crate::commands::ui_prefs::save_column_prefs_core(&conn, "order_flow", &cols1)
            .expect("保存");

        // 第二次保存:全部默认(开启 balance)
        let cols2 = crate::commands::ui_prefs::default_columns();
        crate::commands::ui_prefs::save_column_prefs_core(&conn, "order_flow", &cols2)
            .expect("保存");

        let prefs =
            crate::commands::ui_prefs::get_column_prefs_core(&conn, "order_flow").expect("读取");
        // 第二次提交后 balance 应恢复 true
        assert_eq!(prefs.columns.get("balance"), Some(&true));
    }

    /// 列显示偏好:不同 source_type 的配置互不影响。
    #[test]
    fn test_column_prefs_isolated_per_source_type() {
        let conn = in_memory_company_conn();

        let mut cols = crate::commands::ui_prefs::default_columns();
        cols.insert("balance".to_string(), false);

        crate::commands::ui_prefs::save_column_prefs_core(&conn, "bank_flow", &cols).expect("保存");

        // order_flow 仍应保持默认
        let order_prefs =
            crate::commands::ui_prefs::get_column_prefs_core(&conn, "order_flow").expect("读取");
        assert_eq!(order_prefs.columns.get("balance"), Some(&true));
    }

    /// 余额连续性计算:导入连续银行流水,逐行计算应全部 ok。
    #[test]
    fn test_balance_check_status_ok() {
        let conn = in_memory_company_conn();
        // 第 1 行 in=0, out=0, balance=100000
        // 第 2 行 in=1000, out=0, balance=101000
        //   expected = 100000 + 1000 - 0 = 101000 ✓
        let bank = write_temp_tsv(
            &unique_temp_name("bank_ok"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t101000.00\n",
        );
        import_file_core(&conn, &bank, None, None).expect("导入失败");

        let rows = fetch_balance_check_rows(&conn, Some("bank_flow")).expect("fetch");
        let map = compute_balance_check_status(&rows);
        let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        assert_eq!(map.get(&ids[0]).map(|s| s.as_str()), Some("skip")); // 第 1 行无 prev
        assert_eq!(map.get(&ids[1]).map(|s| s.as_str()), Some("ok"));

        let _ = std::fs::remove_file(&bank);
    }

    /// 余额连续性计算:第 2 行余额被改,与 expected 不一致 → mismatch。
    #[test]
    fn test_balance_check_status_mismatch() {
        let conn = in_memory_company_conn();
        let bank = write_temp_tsv(
            &unique_temp_name("bank_mismatch"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t99999.00\n",
        );
        import_file_core(&conn, &bank, None, None).expect("导入失败");

        let rows = fetch_balance_check_rows(&conn, Some("bank_flow")).expect("fetch");
        let map = compute_balance_check_status(&rows);
        let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        assert_eq!(map.get(&ids[0]).map(|s| s.as_str()), Some("skip"));
        assert_eq!(map.get(&ids[1]).map(|s| s.as_str()), Some("mismatch"));

        let _ = std::fs::remove_file(&bank);
    }

    /// 余额连续性计算:raw_data 中缺转入或转出 → skip。
    #[test]
    fn test_balance_check_status_skip_when_amounts_missing() {
        let conn = in_memory_company_conn();
        // 第 1 行有完整数据,第 2 行缺转出金额(空列)
        let bank = write_temp_tsv(
            &unique_temp_name("bank_skip"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t\t101000.00\n",
        );
        import_file_core(&conn, &bank, None, None).expect("导入失败");

        let rows = fetch_balance_check_rows(&conn, Some("bank_flow")).expect("fetch");
        let map = compute_balance_check_status(&rows);
        let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        assert_eq!(map.get(&ids[1]).map(|s| s.as_str()), Some("skip"));

        let _ = std::fs::remove_file(&bank);
    }

    /// list_bank_flows_core 应当把 balance_check_status 写入 bank_flow 记录。
    #[test]
    fn test_list_raw_records_includes_balance_check_status_for_bank_flow() {
        let conn = in_memory_company_conn();
        let bank = write_temp_tsv(
            &unique_temp_name("bank_list"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t101000.00\n",
        );
        import_file_core(&conn, &bank, None, None).expect("导入失败");

        let page = crate::commands::bank_flow::list_bank_flows_core(
            &conn,
            &crate::commands::bank_flow::BankFlowFilter {
                batch_id: None,
                page: 1,
                page_size: 50,
            },
        )
        .expect("查询失败");

        // page 内按 record_date DESC 排序,但 balance_check_status 应已填好
        let statuses: Vec<Option<String>> = page
            .items
            .iter()
            .map(|r| r.balance_check_status.clone())
            .collect();
        assert!(
            statuses.iter().all(|s| s.is_some()),
            "所有银行流水行应填写 status"
        );
        // 其中第 1 行(更早时间)skip,第 2 行(更晚时间)ok(因为是上一行的延续)
        // 顺序按 record_date DESC:第 1 行(11:00)= ok,第 2 行(10:00)= skip
        assert_eq!(statuses[0], Some("ok".to_string()));
        assert_eq!(statuses[1], Some("skip".to_string()));

        let _ = std::fs::remove_file(&bank);
    }

    /// 严格相等回归:1 分的差异(曾经落入容差)现在应直接是 mismatch。
    /// 业务规则:Decimal 严格加减不应出现四舍五入误差,任何不平都是真问题。
    #[test]
    fn test_balance_check_status_strict_no_tolerance() {
        let conn = in_memory_company_conn();
        // 第 1 行 balance = 100000.00
        // 第 2 行 in = 1000.00, out = 0.00, balance = 100999.99 → expected = 101000.00,差 0.01
        let bank = write_temp_tsv(
            &unique_temp_name("bank_strict"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t100999.99\n",
        );
        import_file_core(&conn, &bank, None, None).expect("导入失败");

        let rows = fetch_balance_check_rows(&conn, Some("bank_flow")).expect("fetch");
        let map = compute_balance_check_status(&rows);
        let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        // 严格相等,0.01 差异也算 mismatch
        assert_eq!(map.get(&ids[1]).map(|s| s.as_str()), Some("mismatch"));

        let _ = std::fs::remove_file(&bank);
    }

    /// confirm_balance_batch_core:确认后 batch 内所有 bank_flow 行 balance_confirmed_at 非空,
    /// 且对应的 import_errors.balance_discontinuity 行被清理。
    #[test]
    fn test_confirm_balance_batch_writes_timestamp_and_clears_errors() {
        let conn = in_memory_company_conn();
        // 第 1 行 ok;第 2 行 mismatch(差 1.00);导入会产生 balance_discontinuity 错误。
        let bank = write_temp_tsv(
            &unique_temp_name("bank_confirm"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t99999.00\n",
        );
        let res = import_file_core(&conn, &bank, None, None).expect("导入失败");
        let batch_id = res.batch_id;

        // 确认前:batch 内 balance_confirmed_at 全为 NULL,且存在 balance_discontinuity 错误
        let confirmed_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bank_flows
                 WHERE import_batch_id = ?1
                   AND balance_confirmed_at IS NOT NULL",
                rusqlite::params![batch_id],
                |row| row.get(0),
            )
            .expect("query before");
        assert_eq!(confirmed_before, 0);

        let err_count_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM import_errors
                 WHERE import_batch_id = ?1 AND field_name = 'balance_discontinuity'",
                rusqlite::params![batch_id],
                |row| row.get(0),
            )
            .expect("query err before");
        assert!(err_count_before > 0, "mismatch 行应已写入 import_errors");

        // 确认
        let updated = crate::commands::bank_flow::confirm_balance_batch_core(&conn, batch_id)
            .expect("confirm");
        assert_eq!(updated, 2, "两行都应被确认");

        // 确认后:batch 内 balance_confirmed_at 全部非空,balance_discontinuity 错误清空
        let confirmed_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bank_flows
                 WHERE import_batch_id = ?1
                   AND balance_confirmed_at IS NOT NULL",
                rusqlite::params![batch_id],
                |row| row.get(0),
            )
            .expect("query after");
        assert_eq!(confirmed_after, 2);

        let err_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM import_errors
                 WHERE import_batch_id = ?1 AND field_name = 'balance_discontinuity'",
                rusqlite::params![batch_id],
                |row| row.get(0),
            )
            .expect("query err after");
        assert_eq!(err_count_after, 0);

        // list_bank_flows_core 返回的 balance_confirmed_at 也应为非空
        let page = crate::commands::bank_flow::list_bank_flows_core(
            &conn,
            &crate::commands::bank_flow::BankFlowFilter {
                batch_id: Some(batch_id),
                page: 1,
                page_size: 50,
            },
        )
        .expect("list");
        let snapshot: Vec<(
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = page
            .items
            .iter()
            .map(|r| {
                (
                    r.id,
                    r.record_date.clone(),
                    r.balance.clone(),
                    r.balance_check_status.clone(),
                    r.balance_confirmed_at.clone(),
                )
            })
            .collect();
        for r in &page.items {
            assert!(
                r.balance_confirmed_at.is_some(),
                "list 应返回 balance_confirmed_at: row {} = {:?}",
                r.id,
                r.balance_confirmed_at
            );
        }
        // 至少有一行 balance_check_status = mismatch(确认后 status 仍报不平,
        // 后端持续暴露事实,前端据此判断"已确认 → 仍按 ok 样式显示")。
        // 另一行通常为 skip(无 prev),不能要求"全部 mismatch"。
        assert!(
            page.items
                .iter()
                .any(|r| r.balance_check_status.as_deref() == Some("mismatch")),
            "DEBUG snapshot = {snapshot:#?}"
        );

        // 撤销
        let cleared = crate::commands::bank_flow::unconfirm_balance_batch_core(&conn, batch_id)
            .expect("unconfirm");
        assert_eq!(cleared, 2);
        let confirmed_cleared: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bank_flows
                 WHERE import_batch_id = ?1
                   AND balance_confirmed_at IS NOT NULL",
                rusqlite::params![batch_id],
                |row| row.get(0),
            )
            .expect("query cleared");
        assert_eq!(confirmed_cleared, 0);

        let _ = std::fs::remove_file(&bank);
    }

    /// confirm_balance_batch_core 不会影响其它 source_type 的记录。
    #[test]
    fn test_confirm_balance_batch_only_affects_bank_flow() {
        let conn = in_memory_company_conn();
        // 同时导入银行流水(确认目标)和订单流水(非目标)
        let bank = write_temp_tsv(
            &unique_temp_name("bank_isolate"),
            "凭证号\t交易时间\t对方单位\t转入金额\t转出金额\t余额\n\
             000000000\t2026-07-10 10:00:00\tA\t0.00\t0.00\t100000.00\n\
             000000000\t2026-07-10 11:00:00\tB\t1000.00\t0.00\t99999.00\n",
        );
        let order = write_temp_tsv(
            &unique_temp_name("order_isolate"),
            "凭证号\t交易时间\t对方单位\t订单金额\t手续费\t商户实收\n\
             ORD001\t2026-07-10 10:00:00\t客户X\t100.00\t0.25\t99.75\n",
        );
        let bank_res = import_file_core(&conn, &bank, None, None).expect("import bank");
        import_file_core(&conn, &order, None, None).expect("import order");

        let updated =
            crate::commands::bank_flow::confirm_balance_batch_core(&conn, bank_res.batch_id)
                .expect("confirm");
        assert_eq!(updated, 2, "只影响 2 行银行流水");

        // 订单流水的 balance_confirmed_at 应仍为 NULL
        let order_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM order_flows
                 WHERE balance_confirmed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .expect("query order");
        assert_eq!(order_count, 0, "订单流水不应被银行流水的确认动作影响");

        let _ = std::fs::remove_file(&bank);
        let _ = std::fs::remove_file(&order);
    }
}
