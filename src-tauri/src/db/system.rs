use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub department: Option<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCompanyPermission {
    pub id: String,
    pub user_id: String,
    pub company_id: String,
    pub role: String,
    pub can_audit: bool,
    pub can_post: bool,
    pub can_manage: bool,
    pub can_backup: bool,
    pub created_at: String,
}

/// 仅用于校验密码,无明文存储。
///
/// 实现采用 SHA-256 盐值哈希(足够 MVP,生产可升级 bcrypt/argon2)。
pub fn hash_password(raw: &str) -> Result<String, String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let salt = "forgefin.v1";
    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    salt.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

pub fn verify_password(raw: &str, password_hash: &str) -> bool {
    match hash_password(raw) {
        Ok(h) => h == password_hash,
        Err(_) => false,
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn create_user(
    conn: &Connection,
    username: &str,
    display_name: &str,
    raw_password: &str,
    department: Option<&str>,
    is_admin: bool,
) -> Result<User, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();
    let hash = hash_password(raw_password)?;
    conn.execute(
        "INSERT INTO users (id, username, display_name, password_hash, department, is_admin, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
        params![
            id,
            username,
            display_name,
            hash,
            department,
            is_admin as i32,
            now
        ],
    )
    .map_err(|e| format!("创建用户失败: {e}"))?;
    Ok(User {
        id,
        username: username.to_string(),
        display_name: display_name.to_string(),
        password_hash: hash,
        department: department.map(String::from),
        is_admin,
        is_active: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn find_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, display_name, password_hash, department, is_admin, is_active, created_at, updated_at FROM users WHERE username = ?1")
        .map_err(|e| format!("查询用户失败: {e}"))?;
    let user = stmt
        .query_row(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                password_hash: row.get(3)?,
                department: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .optional();
    match user {
        Ok(u) => Ok(u),
        Err(e) => Err(format!("查询用户失败: {e}")),
    }
}

#[allow(dead_code)]
pub fn find_user_by_id(conn: &Connection, user_id: &str) -> Result<Option<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, display_name, password_hash, department, is_admin, is_active, created_at, updated_at FROM users WHERE id = ?1")
        .map_err(|e| format!("查询用户失败: {e}"))?;
    let user = stmt
        .query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                password_hash: row.get(3)?,
                department: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .optional();
    match user {
        Ok(u) => Ok(u),
        Err(e) => Err(format!("查询用户失败: {e}")),
    }
}

pub fn list_users(conn: &Connection) -> Result<Vec<User>, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, display_name, password_hash, department, is_admin, is_active, created_at, updated_at FROM users ORDER BY created_at")
        .map_err(|e| format!("查询用户列表失败: {e}"))?;
    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                password_hash: row.get(3)?,
                department: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("查询用户列表失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询用户列表失败: {e}"))?;
    Ok(users)
}

/// 尝试登录,返回用户(校验通过)或错误信息。
pub fn login(conn: &Connection, username: &str, raw_password: &str) -> Result<User, String> {
    let user =
        find_user_by_username(conn, username)?.ok_or_else(|| "用户名或密码错误".to_string())?;
    if !user.is_active {
        return Err("该用户已停用".to_string());
    }
    if !verify_password(raw_password, &user.password_hash) {
        return Err("用户名或密码错误".to_string());
    }
    Ok(user)
}

/// 校验用户能否访问指定公司,返回权限记录。
pub fn user_permission(
    conn: &Connection,
    user_id: &str,
    company_id: &str,
) -> Result<Option<UserCompanyPermission>, String> {
    let mut stmt = conn
        .prepare("SELECT id, user_id, company_id, role, can_audit, can_post, can_manage, can_backup, created_at FROM user_company_permissions WHERE user_id = ?1 AND company_id = ?2")
        .map_err(|e| format!("查询权限失败: {e}"))?;
    let perm = stmt
        .query_row(params![user_id, company_id], |row| {
            Ok(UserCompanyPermission {
                id: row.get(0)?,
                user_id: row.get(1)?,
                company_id: row.get(2)?,
                role: row.get(3)?,
                can_audit: row.get::<_, i32>(4)? != 0,
                can_post: row.get::<_, i32>(5)? != 0,
                can_manage: row.get::<_, i32>(6)? != 0,
                can_backup: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
            })
        })
        .optional();
    match perm {
        Ok(p) => Ok(p),
        Err(e) => Err(format!("查询权限失败: {e}")),
    }
}

/// 获取用户可访问的所有公司 ID。
pub fn user_company_ids(conn: &Connection, user_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT company_id FROM user_company_permissions WHERE user_id = ?1")
        .map_err(|e| format!("查询用户公司列表失败: {e}"))?;
    let ids = stmt
        .query_map(params![user_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("查询用户公司列表失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("查询用户公司列表失败: {e}"))?;
    Ok(ids)
}

/// 授予/更新用户对公司的权限。
pub fn grant_permission(
    conn: &Connection,
    user_id: &str,
    company_id: &str,
    role: &str,
    can_audit: bool,
    can_post: bool,
    can_manage: bool,
    can_backup: bool,
) -> Result<UserCompanyPermission, String> {
    let now = now_iso();
    let existing = user_permission(conn, user_id, company_id)?;
    if existing.is_some() {
        conn.execute(
            "UPDATE user_company_permissions SET role = ?1, can_audit = ?2, can_post = ?3, can_manage = ?4, can_backup = ?5 WHERE user_id = ?6 AND company_id = ?7",
            params![
                role,
                can_audit as i32,
                can_post as i32,
                can_manage as i32,
                can_backup as i32,
                user_id,
                company_id
            ],
        )
        .map_err(|e| format!("更新权限失败: {e}"))?;
        user_permission(conn, user_id, company_id)?.ok_or_else(|| "权限更新后查询失败".to_string())
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO user_company_permissions (id, user_id, company_id, role, can_audit, can_post, can_manage, can_backup, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                user_id,
                company_id,
                role,
                can_audit as i32,
                can_post as i32,
                can_manage as i32,
                can_backup as i32,
                now
            ],
        )
        .map_err(|e| format!("授予权限失败: {e}"))?;
        user_permission(conn, user_id, company_id)?.ok_or_else(|| "权限授予后查询失败".to_string())
    }
}
