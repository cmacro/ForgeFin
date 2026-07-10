use serde::{Deserialize, Serialize};

// =====================================================================
// IPC 类型定义: 前后端共享的数据结构,镜像 src-tauri 命令签名。
// =====================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub department: Option<String>,
    pub is_admin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanyBrief {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: UserInfo,
    pub companies: Vec<CompanyBrief>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user: Option<UserInfo>,
    pub company_id: Option<String>,
    pub available_companies: Vec<CompanyBrief>,
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub account_type: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Voucher {
    pub id: String,
    pub voucher_no: String,
    pub voucher_date: String,
    pub voucher_type: String,
    pub summary: String,
    pub attachments: i32,
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
    pub debit: String,
    pub credit: String,
    pub contact_id: Option<String>,
    pub contact_name: Option<String>,
    pub created_at: String,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct VoucherFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub voucher_no: Option<String>,
    pub voucher_type: Option<String>,
    pub status: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherPage {
    pub items: Vec<Voucher>,
    pub total: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoucherDetail {
    pub voucher: Voucher,
    pub entries: Vec<VoucherEntry>,
    pub audit_logs: Vec<AuditLog>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupEntry {
    pub path: String,
    pub name: String,
    pub size: i64,
    pub modified: i64,
}

// =====================================================================
// invoke 封装。统一错误为 String。
//
// 通过 `window.__TAURI__.core.invoke` 调用 Rust 命令(tauri.conf.json
// 已开启 withGlobalTauri)。避免在前端 crate 引入 tauri 重依赖。
// =====================================================================

use js_sys::Reflect;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

fn tauri_invoke(
    cmd: String,
    args: JsValue,
) -> impl std::future::Future<Output = Result<JsValue, String>> {
    async move {
        let global = js_sys::global();
        let tauri = Reflect::get(&global, &JsValue::from_str("__TAURI__"))
            .map_err(|e| format!("TAURI 未注入: {e:?}"))?;
        let core = Reflect::get(&tauri, &JsValue::from_str("core"))
            .map_err(|e| format!("TAURI.core 未注入: {e:?}"))?;
        let invoke_fn = Reflect::get(&core, &JsValue::from_str("invoke"))
            .map_err(|e| format!("TAURI.core.invoke 未注入: {e:?}"))?;
        let invoke_fn = invoke_fn
            .dyn_into::<js_sys::Function>()
            .map_err(|e| format!("invoke 不是 Function: {e:?}"))?;
        let cmd_val = JsValue::from_str(&cmd);
        let args_arr = js_sys::Array::new();
        args_arr.push(&cmd_val);
        args_arr.push(&args);
        let promise = Reflect::apply(&invoke_fn, &JsValue::undefined(), &args_arr)
            .map_err(|e| format!("invoke 调用失败: {e:?}"))?;
        let promise = wasm_bindgen::JsCast::dyn_into::<js_sys::Promise>(promise)
            .map_err(|e| format!("invoke 返回非 Promise: {e:?}"))?;
        JsFuture::from(promise)
            .await
            .map_err(|e| format!("命令 {cmd} 执行失败: {e:?}"))
    }
}

pub async fn invoke<T: serde::de::DeserializeOwned>(
    cmd: &str,
    args: &impl Serialize,
) -> Result<T, String> {
    let args_val = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let res = tauri_invoke(cmd.to_string(), args_val).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn login(username: String, password: String) -> Result<AuthResult, String> {
    invoke(
        "login_cmd",
        &[("username", &username), ("password", &password)],
    )
    .await
}

pub async fn logout() -> Result<(), String> {
    invoke("logout_cmd", &()).await
}

pub async fn current_user() -> Result<CurrentUser, String> {
    invoke("current_user_cmd", &()).await
}

pub async fn set_current_company(company_id: String) -> Result<(), String> {
    invoke("set_current_company_cmd", &[("company_id", &company_id)]).await
}

pub async fn list_companies() -> Result<Vec<Company>, String> {
    invoke("list_companies_cmd", &()).await
}

pub async fn create_company(input: &CompanyInput) -> Result<Company, String> {
    invoke("create_company_cmd", &serde_json::json!({"input": input})).await
}

pub async fn update_company(id: String, input: &CompanyInput) -> Result<Company, String> {
    invoke(
        "update_company_cmd",
        &serde_json::json!({"id": id, "input": input}),
    )
    .await
}

pub async fn delete_company(id: String) -> Result<(), String> {
    invoke("delete_company_cmd", &[("id", &id)]).await
}

pub async fn list_accounts() -> Result<Vec<Account>, String> {
    invoke("list_accounts_cmd", &()).await
}

pub async fn create_account(input: &AccountInput) -> Result<Account, String> {
    invoke("create_account_cmd", &serde_json::json!({"input": input})).await
}

pub async fn update_account(id: String, input: &AccountInput) -> Result<Account, String> {
    invoke(
        "update_account_cmd",
        &serde_json::json!({"id": id, "input": input}),
    )
    .await
}

pub async fn delete_account(id: String) -> Result<(), String> {
    invoke("delete_account_cmd", &[("id", &id)]).await
}

pub async fn list_contacts(contact_type: Option<String>) -> Result<Vec<Contact>, String> {
    invoke("list_contacts_cmd", &[("contact_type", &contact_type)]).await
}

pub async fn create_contact(input: &ContactInput) -> Result<Contact, String> {
    invoke("create_contact_cmd", &serde_json::json!({"input": input})).await
}

pub async fn update_contact(id: String, input: &ContactInput) -> Result<Contact, String> {
    invoke(
        "update_contact_cmd",
        &serde_json::json!({"id": id, "input": input}),
    )
    .await
}

pub async fn delete_contact(id: String) -> Result<(), String> {
    invoke("delete_contact_cmd", &[("id", &id)]).await
}

pub async fn create_voucher(input: &VoucherInput) -> Result<Voucher, String> {
    invoke("create_voucher_cmd", &[("input", input)]).await
}

pub async fn list_vouchers(filter: &VoucherFilter) -> Result<VoucherPage, String> {
    invoke("list_vouchers_cmd", &[("filter", filter)]).await
}

pub async fn get_voucher(id: String) -> Result<VoucherDetail, String> {
    invoke("get_voucher_cmd", &[("id", &id)]).await
}

pub async fn delete_voucher(id: String) -> Result<(), String> {
    invoke("delete_voucher_cmd", &[("id", &id)]).await
}

pub async fn audit_voucher(id: String, comment: Option<String>) -> Result<Voucher, String> {
    invoke(
        "audit_voucher_cmd",
        &serde_json::json!({"id": id, "comment": comment}),
    )
    .await
}

pub async fn next_voucher_no(voucher_type: String, voucher_date: String) -> Result<String, String> {
    invoke(
        "next_voucher_no_cmd",
        &[
            ("voucher_type", &voucher_type),
            ("voucher_date", &voucher_date),
        ],
    )
    .await
}

pub async fn backup_company(company_id: String) -> Result<String, String> {
    invoke("backup_company_cmd", &[("company_id", &company_id)]).await
}

pub async fn backup_system() -> Result<String, String> {
    invoke("backup_system_cmd", &()).await
}

pub async fn list_backups() -> Result<Vec<BackupEntry>, String> {
    invoke("list_backups_cmd", &()).await
}

pub async fn restore_company(
    company_id: String,
    backup_path: String,
    confirm: bool,
) -> Result<(), String> {
    invoke(
        "restore_company_cmd",
        &serde_json::json!({"company_id": company_id, "backup_path": backup_path, "confirm": confirm}),
    )
    .await
}
