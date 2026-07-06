use chrono::Utc;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::system::now_iso;

// =====================================================================
// Accounts (会计科目)
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
    /// 资产/负债/权益/成本/损益
    pub account_type: String,
    /// debit / credit / auto
    pub balance_direction: String,
    pub is_leaf: bool,
    pub is_active: bool,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountInput {
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub account_type: String,
    pub balance_direction: Option<String>,
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

pub fn create_account(conn: &Connection, input: &AccountInput) -> Result<Account, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    let direction = input
        .balance_direction
        .clone()
        .unwrap_or_else(|| "auto".to_string());
    // 有 parent 即视为非叶子;无 parent 时默认叶子,可后续调整。
    let is_leaf = input.parent_id.is_none();
    conn.execute(
        "INSERT INTO accounts (id, code, name, parent_id, account_type, balance_direction, is_leaf, is_active, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            id,
            input.code,
            input.name,
            input.parent_id,
            input.account_type,
            direction,
            is_leaf as i32,
            input.is_active.unwrap_or(true) as i32,
            input.description,
            now
        ],
    )
    .map_err(|e| format!("创建科目失败: {e}"))?;
    // 若指定了 parent,把 parent 的 is_leaf 置为 0(已不可再作为叶子挂账)。
    if let Some(pid) = &input.parent_id {
        conn.execute(
            "UPDATE accounts SET is_leaf = 0, updated_at = ?1 WHERE id = ?2",
            params![now, pid],
        )
        .map_err(|e| format!("更新父科目失败: {e}"))?;
    }
    get_account(conn, &id)?.ok_or_else(|| "科目创建后查询失败".to_string())
}

pub fn get_account(conn: &Connection, id: &str) -> Result<Option<Account>, String> {
    let account = conn
        .query_row(
            "SELECT id, code, name, parent_id, account_type, balance_direction, is_leaf, is_active, description, created_at, updated_at FROM accounts WHERE id = ?1",
            params![id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    parent_id: row.get(3)?,
                    account_type: row.get(4)?,
                    balance_direction: row.get(5)?,
                    is_leaf: row.get::<_, i32>(6)? != 0,
                    is_active: row.get::<_, i32>(7)? != 0,
                    description: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        )
        .optional();
    match account {
        Ok(a) => Ok(a),
        Err(e) => Err(format!("查询科目失败: {e}")),
    }
}

pub fn list_accounts(conn: &Connection) -> Result<Vec<Account>, String> {
    let mut stmt = conn
        .prepare("SELECT id, code, name, parent_id, account_type, balance_direction, is_leaf, is_active, description, created_at, updated_at FROM accounts ORDER BY code")
        .map_err(|e| format!("查询科目列表失败: {e}"))?;
    let accounts = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                parent_id: row.get(3)?,
                account_type: row.get(4)?,
                balance_direction: row.get(5)?,
                is_leaf: row.get::<_, i32>(6)? != 0,
                is_active: row.get::<_, i32>(7)? != 0,
                description: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| format!("查询科目列表失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询科目列表失败: {e}"))?;
    Ok(accounts)
}

pub fn update_account(
    conn: &Connection,
    id: &str,
    input: &AccountInput,
) -> Result<Account, String> {
    let now = now_iso();
    conn.execute(
        "UPDATE accounts SET code = ?1, name = ?2, parent_id = ?3, account_type = ?4, balance_direction = ?5, is_active = ?6, description = ?7, updated_at = ?8 WHERE id = ?9",
        params![
            input.code,
            input.name,
            input.parent_id,
            input.account_type,
            input.balance_direction.clone().unwrap_or_else(|| "auto".to_string()),
            input.is_active.unwrap_or(true) as i32,
            input.description,
            now,
            id
        ],
    )
    .map_err(|e| format!("更新科目失败: {e}"))?;
    get_account(conn, id)?.ok_or_else(|| "科目更新后查询失败".to_string())
}

pub fn delete_account(conn: &Connection, id: &str) -> Result<(), String> {
    // 检查是否有子科目
    let child_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE parent_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("检查子科目失败: {e}"))?;
    if child_count > 0 {
        return Err("存在子科目,不能删除".to_string());
    }
    // 检查是否被凭证分录引用
    let entry_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM voucher_entries WHERE account_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("检查科目引用失败: {e}"))?;
    if entry_count > 0 {
        return Err("该科目已被凭证引用,不能删除".to_string());
    }
    conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])
        .map_err(|e| format!("删除科目失败: {e}"))?;
    Ok(())
}

// =====================================================================
// Contacts (客户/供应商)
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub code: String,
    pub name: String,
    /// customer / vendor
    pub contact_type: String,
    pub tax_id: Option<String>,
    pub bank_account: Option<String>,
    pub bank_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub remark: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContactInput {
    pub code: String,
    pub name: String,
    pub contact_type: String,
    pub tax_id: Option<String>,
    pub bank_account: Option<String>,
    pub bank_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub remark: Option<String>,
    pub is_active: Option<bool>,
}

pub fn create_contact(conn: &Connection, input: &ContactInput) -> Result<Contact, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    conn.execute(
        "INSERT INTO contacts (id, code, name, contact_type, tax_id, bank_account, bank_name, address, phone, email, remark, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
        params![
            id,
            input.code,
            input.name,
            input.contact_type,
            input.tax_id,
            input.bank_account,
            input.bank_name,
            input.address,
            input.phone,
            input.email,
            input.remark,
            input.is_active.unwrap_or(true) as i32,
            now
        ],
    )
    .map_err(|e| format!("创建客户/供应商失败: {e}"))?;
    get_contact(conn, &id)?.ok_or_else(|| "客户/供应商创建后查询失败".to_string())
}

pub fn get_contact(conn: &Connection, id: &str) -> Result<Option<Contact>, String> {
    let contact = conn
        .query_row(
            "SELECT id, code, name, contact_type, tax_id, bank_account, bank_name, address, phone, email, remark, is_active, created_at, updated_at FROM contacts WHERE id = ?1",
            params![id],
            |row| {
                Ok(Contact {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    contact_type: row.get(3)?,
                    tax_id: row.get(4)?,
                    bank_account: row.get(5)?,
                    bank_name: row.get(6)?,
                    address: row.get(7)?,
                    phone: row.get(8)?,
                    email: row.get(9)?,
                    remark: row.get(10)?,
                    is_active: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        )
        .optional();
    match contact {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("查询客户/供应商失败: {e}")),
    }
}

pub fn list_contacts(
    conn: &Connection,
    contact_type: Option<&str>,
) -> Result<Vec<Contact>, String> {
    let mut stmt = if contact_type.is_some() {
        conn.prepare("SELECT id, code, name, contact_type, tax_id, bank_account, bank_name, address, phone, email, remark, is_active, created_at, updated_at FROM contacts WHERE contact_type = ?1 ORDER BY code")
            .map_err(|e| format!("查询客户/供应商列表失败: {e}"))?
    } else {
        conn.prepare("SELECT id, code, name, contact_type, tax_id, bank_account, bank_name, address, phone, email, remark, is_active, created_at, updated_at FROM contacts ORDER BY code")
            .map_err(|e| format!("查询客户/供应商列表失败: {e}"))?
    };
    let rows = if let Some(t) = contact_type {
        stmt.query_map(params![t], map_contact)
            .map_err(|e| format!("查询客户/供应商列表失败: {e}"))?
    } else {
        stmt.query_map([], map_contact)
            .map_err(|e| format!("查询客户/供应商列表失败: {e}"))?
    };
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询客户/供应商列表失败: {e}"))
}

fn map_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        contact_type: row.get(3)?,
        tax_id: row.get(4)?,
        bank_account: row.get(5)?,
        bank_name: row.get(6)?,
        address: row.get(7)?,
        phone: row.get(8)?,
        email: row.get(9)?,
        remark: row.get(10)?,
        is_active: row.get::<_, i32>(11)? != 0,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

pub fn update_contact(
    conn: &Connection,
    id: &str,
    input: &ContactInput,
) -> Result<Contact, String> {
    let now = now_iso();
    conn.execute(
        "UPDATE contacts SET code = ?1, name = ?2, contact_type = ?3, tax_id = ?4, bank_account = ?5, bank_name = ?6, address = ?7, phone = ?8, email = ?9, remark = ?10, is_active = ?11, updated_at = ?12 WHERE id = ?13",
        params![
            input.code,
            input.name,
            input.contact_type,
            input.tax_id,
            input.bank_account,
            input.bank_name,
            input.address,
            input.phone,
            input.email,
            input.remark,
            input.is_active.unwrap_or(true) as i32,
            now,
            id
        ],
    )
    .map_err(|e| format!("更新客户/供应商失败: {e}"))?;
    get_contact(conn, id)?.ok_or_else(|| "客户/供应商更新后查询失败".to_string())
}

pub fn delete_contact(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])
        .map_err(|e| format!("删除客户/供应商失败: {e}"))?;
    Ok(())
}

// =====================================================================
// Vouchers (凭证)
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Voucher {
    pub id: String,
    pub voucher_no: String,
    pub voucher_date: String,
    pub voucher_type: String,
    pub summary: String,
    pub attachments: i32,
    /// draft / unaudited / audited / posted
    pub status: String,
    pub debit_total: String,
    pub credit_total: String,
    pub operator_id: Option<String>,
    pub operator_name: Option<String>,
    pub auditor_id: Option<String>,
    pub auditor_name: Option<String>,
    pub audited_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherEntry {
    pub id: String,
    pub voucher_id: String,
    pub line_no: i32,
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub summary: Option<String>,
    /// 以分(整数)存储的金额字符串,前端按精度格式化。
    pub debit: String,
    pub credit: String,
    pub contact_id: Option<String>,
    pub contact_name: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherInput {
    pub voucher_no: String,
    pub voucher_date: String,
    pub voucher_type: String,
    pub summary: String,
    pub attachments: Option<i32>,
    pub operator_id: Option<String>,
    pub operator_name: Option<String>,
    pub entries: Vec<VoucherEntryInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherEntryInput {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub summary: Option<String>,
    pub debit: String,
    pub credit: String,
    pub contact_id: Option<String>,
    pub contact_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub voucher_no: Option<String>,
    pub voucher_type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub struct VoucherPage {
    pub items: Vec<Voucher>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

pub fn create_voucher(conn: &Connection, input: &VoucherInput) -> Result<Voucher, String> {
    // 校验借贷平衡
    let (debit_total, credit_total) = sum_entries(&input.entries)?;
    if debit_total != credit_total {
        return Err(format!(
            "借贷不平衡: 借方 {debit_total} ≠ 贷方 {credit_total}"
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    conn.execute(
        "INSERT INTO vouchers (id, voucher_no, voucher_date, voucher_type, summary, attachments, status, debit_total, credit_total, operator_id, operator_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        params![
            id,
            input.voucher_no,
            input.voucher_date,
            input.voucher_type,
            input.summary,
            input.attachments.unwrap_or(0),
            "draft",
            debit_total,
            credit_total,
            input.operator_id,
            input.operator_name,
            now
        ],
    )
    .map_err(|e| format!("创建凭证失败: {e}"))?;
    for (idx, entry) in input.entries.iter().enumerate() {
        let entry_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO voucher_entries (id, voucher_id, line_no, account_id, account_code, account_name, summary, debit, credit, contact_id, contact_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry_id,
                id,
                (idx as i32) + 1,
                entry.account_id,
                entry.account_code,
                entry.account_name,
                entry.summary,
                entry.debit,
                entry.credit,
                entry.contact_id,
                entry.contact_name,
                now
            ],
        )
        .map_err(|e| format!("创建凭证分录失败: {e}"))?;
    }
    get_voucher(conn, &id)?.ok_or_else(|| "凭证创建后查询失败".to_string())
}

/// 计算借贷合计(分),并校验均为非负整数。
pub fn sum_entries(entries: &[VoucherEntryInput]) -> Result<(String, String), String> {
    let mut debit: i64 = 0;
    let mut credit: i64 = 0;
    for e in entries {
        let d: i64 = e
            .debit
            .parse()
            .map_err(|_| format!("借方金额非法: {}", e.debit))?;
        let c: i64 = e
            .credit
            .parse()
            .map_err(|_| format!("贷方金额非法: {}", e.credit))?;
        if d < 0 || c < 0 {
            return Err("金额不能为负".to_string());
        }
        debit += d;
        credit += c;
    }
    Ok((debit.to_string(), credit.to_string()))
}

pub fn get_voucher(conn: &Connection, id: &str) -> Result<Option<Voucher>, String> {
    let voucher = conn
        .query_row(
            "SELECT id, voucher_no, voucher_date, voucher_type, summary, attachments, status, debit_total, credit_total, operator_id, operator_name, auditor_id, auditor_name, audited_at, created_at, updated_at FROM vouchers WHERE id = ?1",
            params![id],
            map_voucher,
        )
        .optional();
    match voucher {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("查询凭证失败: {e}")),
    }
}

fn map_voucher(row: &rusqlite::Row) -> rusqlite::Result<Voucher> {
    Ok(Voucher {
        id: row.get(0)?,
        voucher_no: row.get(1)?,
        voucher_date: row.get(2)?,
        voucher_type: row.get(3)?,
        summary: row.get(4)?,
        attachments: row.get(5)?,
        status: row.get(6)?,
        debit_total: row.get(7)?,
        credit_total: row.get(8)?,
        operator_id: row.get(9)?,
        operator_name: row.get(10)?,
        auditor_id: row.get(11)?,
        auditor_name: row.get(12)?,
        audited_at: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

pub fn list_vouchers(conn: &Connection, filter: &VoucherFilter) -> Result<VoucherPage, String> {
    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 200);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from(" WHERE 1 = 1");
    let mut p: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(d) = &filter.date_from {
        where_clause.push_str(" AND voucher_date >= ?");
        p.push(Box::new(d.clone()));
    }
    if let Some(d) = &filter.date_to {
        where_clause.push_str(" AND voucher_date <= ?");
        p.push(Box::new(d.clone()));
    }
    if let Some(n) = &filter.voucher_no {
        where_clause.push_str(" AND voucher_no LIKE ?");
        p.push(Box::new(format!("%{n}%")));
    }
    if let Some(t) = &filter.voucher_type {
        where_clause.push_str(" AND voucher_type = ?");
        p.push(Box::new(t.clone()));
    }
    if let Some(s) = &filter.status {
        where_clause.push_str(" AND status = ?");
        p.push(Box::new(s.clone()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM vouchers{where_clause}");
    let count_refs: Vec<&dyn rusqlite::ToSql> = p.iter().map(|b| b.as_ref()).collect();
    let total: i32 = conn
        .query_row(&count_sql, params_from_iter(count_refs.iter()), |row| {
            row.get(0)
        })
        .map_err(|e| format!("统计凭证数量失败: {e}"))?;

    let list_sql = format!(
        "SELECT id, voucher_no, voucher_date, voucher_type, summary, attachments, status, debit_total, credit_total, operator_id, operator_name, auditor_id, auditor_name, audited_at, created_at, updated_at FROM vouchers{where_clause} ORDER BY voucher_date DESC, voucher_no DESC LIMIT ? OFFSET ?"
    );
    let mut list_params: Vec<Box<dyn rusqlite::ToSql>> = p.into_iter().collect();
    list_params.push(Box::new(page_size));
    list_params.push(Box::new(offset));
    let list_refs: Vec<&dyn rusqlite::ToSql> = list_params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn
        .prepare(&list_sql)
        .map_err(|e| format!("查询凭证列表失败: {e}"))?;
    let items = stmt
        .query_map(params_from_iter(list_refs.iter()), map_voucher)
        .map_err(|e| format!("查询凭证列表失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询凭证列表失败: {e}"))?;

    Ok(VoucherPage {
        items,
        total,
        page,
        page_size,
    })
}

pub fn list_voucher_entries(
    conn: &Connection,
    voucher_id: &str,
) -> Result<Vec<VoucherEntry>, String> {
    let mut stmt = conn
        .prepare("SELECT id, voucher_id, line_no, account_id, account_code, account_name, summary, debit, credit, contact_id, contact_name, created_at FROM voucher_entries WHERE voucher_id = ?1 ORDER BY line_no")
        .map_err(|e| format!("查询凭证分录失败: {e}"))?;
    let entries = stmt
        .query_map(params![voucher_id], |row| {
            Ok(VoucherEntry {
                id: row.get(0)?,
                voucher_id: row.get(1)?,
                line_no: row.get(2)?,
                account_id: row.get(3)?,
                account_code: row.get(4)?,
                account_name: row.get(5)?,
                summary: row.get(6)?,
                debit: row.get(7)?,
                credit: row.get(8)?,
                contact_id: row.get(9)?,
                contact_name: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| format!("查询凭证分录失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询凭证分录失败: {e}"))?;
    Ok(entries)
}

/// 状态流转: draft/unaudited -> audited, audited -> unaudited(反审核)。
pub fn audit_voucher(
    conn: &Connection,
    id: &str,
    operator_id: Option<&str>,
    operator_name: Option<&str>,
    comment: Option<&str>,
) -> Result<Voucher, String> {
    let voucher = get_voucher(conn, id)?.ok_or_else(|| "凭证不存在".to_string())?;
    let (new_status, action) = match voucher.status.as_str() {
        "draft" | "unaudited" => ("audited", "audit"),
        "audited" => ("unaudited", "unaudit"),
        "posted" => return Err("已过账凭证不能直接审核/反审核".to_string()),
        other => return Err(format!("未知凭证状态: {other}")),
    };
    let now = now_iso();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {e}"))?;
    tx.execute(
        "UPDATE vouchers SET status = ?1, auditor_id = ?2, auditor_name = ?3, audited_at = ?4, updated_at = ?4 WHERE id = ?5",
        params![new_status, operator_id, operator_name, now, id],
    )
    .map_err(|e| format!("更新凭证状态失败: {e}"))?;
    let log_id = uuid::Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO voucher_audit_logs (id, voucher_id, action, operator_id, operator_name, comment, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![log_id, id, action, operator_id, operator_name, comment, now],
    )
    .map_err(|e| format!("写入审核日志失败: {e}"))?;
    tx.commit().map_err(|e| format!("提交事务失败: {e}"))?;
    get_voucher(conn, id)?.ok_or_else(|| "凭证审核后查询失败".to_string())
}

pub fn delete_voucher(conn: &Connection, id: &str) -> Result<(), String> {
    let voucher = get_voucher(conn, id)?.ok_or_else(|| "凭证不存在".to_string())?;
    if voucher.status == "posted" {
        return Err("已过账凭证不能删除".to_string());
    }
    if voucher.status == "audited" {
        return Err("已审核凭证不能直接删除,请先反审核".to_string());
    }
    conn.execute("DELETE FROM vouchers WHERE id = ?1", params![id])
        .map_err(|e| format!("删除凭证失败: {e}"))?;
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub voucher_id: String,
    pub action: String,
    pub operator_id: Option<String>,
    pub operator_name: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
}

pub fn list_audit_logs(conn: &Connection, voucher_id: &str) -> Result<Vec<AuditLog>, String> {
    let mut stmt = conn
        .prepare("SELECT id, voucher_id, action, operator_id, operator_name, comment, created_at FROM voucher_audit_logs WHERE voucher_id = ?1 ORDER BY created_at DESC")
        .map_err(|e| format!("查询审核日志失败: {e}"))?;
    let logs = stmt
        .query_map(params![voucher_id], |row| {
            Ok(AuditLog {
                id: row.get(0)?,
                voucher_id: row.get(1)?,
                action: row.get(2)?,
                operator_id: row.get(3)?,
                operator_name: row.get(4)?,
                comment: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("查询审核日志失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询审核日志失败: {e}"))?;
    Ok(logs)
}

/// 计算下一凭证字号。
///
/// 规则: `{type_prefix}-{year}-{month}-{seq}`,如 `记-2024-06-0001`。
pub fn next_voucher_no(
    conn: &Connection,
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
    // 解析年月,失败则用当前年月。
    let (year, month) = parse_year_month(voucher_date);
    let like = format!("{prefix}-{year}-{month:02}-%");
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM vouchers WHERE voucher_no LIKE ?1",
            params![like],
            |row| row.get(0),
        )
        .map_err(|e| format!("生成凭证字号失败: {e}"))?;
    Ok(format!("{prefix}-{year}-{month:02}-{:04}", count + 1))
}

fn parse_year_month(date: &str) -> (i32, u32) {
    // 兼容 YYYY-MM-DD 或 YYYY-MM-DDTHH:MM:SS
    let s = date.split('T').next().unwrap_or(date);
    let mut parts = s.split('-');
    let y: i32 = parts
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| {
            let now = Utc::now();
            now.format("%Y").to_string().parse().unwrap_or(2024)
        });
    let m: u32 = parts
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| Utc::now().format("%m").to_string().parse().unwrap_or(1));
    (y, m)
}
