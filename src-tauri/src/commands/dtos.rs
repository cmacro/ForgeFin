use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProjectRecordRequest {
    pub project_id: i64,
    pub amount: f64,
    pub is_income: bool,
    pub account_id: i64,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectRecordResponse {
    pub voucher_id: i64,
    pub status: String,
}
