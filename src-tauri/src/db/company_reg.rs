use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::system::now_iso;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub tax_id: Option<String>,
    pub legal_person: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub currency: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanyInput {
    pub name: String,
    pub tax_id: Option<String>,
    pub legal_person: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub currency: Option<String>,
    pub is_active: Option<bool>,
}

pub fn create_company(sys_conn: &Connection, input: &CompanyInput) -> Result<Company, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    sys_conn
        .execute(
            "INSERT INTO companies (id, name, tax_id, legal_person, address, phone, currency, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                id,
                input.name,
                input.tax_id,
                input.legal_person,
                input.address,
                input.phone,
                input.currency.as_deref().unwrap_or("CNY"),
                input.is_active.unwrap_or(true) as i32,
                now
            ],
        )
        .map_err(|e| format!("创建公司失败: {e}"))?;
    get_company(sys_conn, &id)?.ok_or_else(|| "公司创建后查询失败".to_string())
}

pub fn get_company(sys_conn: &Connection, id: &str) -> Result<Option<Company>, String> {
    let company = sys_conn
        .query_row(
            "SELECT id, name, tax_id, legal_person, address, phone, currency, is_active, created_at, updated_at FROM companies WHERE id = ?1",
            params![id],
            map_company,
        )
        .optional();
    match company {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("查询公司失败: {e}")),
    }
}

pub fn list_companies(sys_conn: &Connection) -> Result<Vec<Company>, String> {
    let mut stmt = sys_conn
        .prepare("SELECT id, name, tax_id, legal_person, address, phone, currency, is_active, created_at, updated_at FROM companies ORDER BY created_at")
        .map_err(|e| format!("查询公司列表失败: {e}"))?;
    let companies = stmt
        .query_map([], map_company)
        .map_err(|e| format!("查询公司列表失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询公司列表失败: {e}"))?;
    Ok(companies)
}

fn map_company(row: &rusqlite::Row) -> rusqlite::Result<Company> {
    Ok(Company {
        id: row.get(0)?,
        name: row.get(1)?,
        tax_id: row.get(2)?,
        legal_person: row.get(3)?,
        address: row.get(4)?,
        phone: row.get(5)?,
        currency: row.get(6)?,
        is_active: row.get::<_, i32>(7)? != 0,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub fn update_company(
    sys_conn: &Connection,
    id: &str,
    input: &CompanyInput,
) -> Result<Company, String> {
    let now = now_iso();
    sys_conn
        .execute(
            "UPDATE companies SET name = ?1, tax_id = ?2, legal_person = ?3, address = ?4, phone = ?5, currency = ?6, is_active = ?7, updated_at = ?8 WHERE id = ?9",
            params![
                input.name,
                input.tax_id,
                input.legal_person,
                input.address,
                input.phone,
                input.currency.as_deref().unwrap_or("CNY"),
                input.is_active.unwrap_or(true) as i32,
                now,
                id
            ],
        )
        .map_err(|e| format!("更新公司失败: {e}"))?;
    get_company(sys_conn, id)?.ok_or_else(|| "公司更新后查询失败".to_string())
}

pub fn delete_company(sys_conn: &Connection, id: &str) -> Result<(), String> {
    sys_conn
        .execute("DELETE FROM companies WHERE id = ?1", params![id])
        .map_err(|e| format!("删除公司失败: {e}"))?;
    Ok(())
}

/// 删除公司库文件(谨慎使用,建议先备份)。
pub fn delete_company_db(id: &str) -> Result<(), String> {
    let path = super::company_db_path(id)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除公司库文件失败: {e}"))?;
        for ext in ["-wal", "-shm"] {
            let p = path.with_extension(format!("db{}", ext));
            if p.exists() {
                std::fs::remove_file(&p).ok();
            }
        }
    }
    Ok(())
}
