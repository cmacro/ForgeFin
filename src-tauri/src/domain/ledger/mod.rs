use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    pub id: i64,
    pub voucher_date: NaiveDate,
    pub description: Option<String>,
    pub project_id: Option<i64>,
    pub status: VoucherStatus,
    pub items: Vec<VoucherItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoucherStatus {
    Unposted,
    Posted,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherItem {
    pub id: i64,
    pub voucher_id: i64,
    pub account_id: i64,
    pub debit: f64,
    pub credit: f64,
    pub description: Option<String>,
}

impl Voucher {
    pub fn is_balanced(&self) -> bool {
        let total_debit: f64 = self.items.iter().map(|i| i.debit).sum();
        let total_credit: f64 = self.items.iter().map(|i| i.credit).sum();
        (total_debit - total_credit).abs() < 0.0001
    }
}

#[cfg(test)]
mod tests;
