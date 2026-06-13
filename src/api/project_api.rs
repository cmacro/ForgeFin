pub mod project_api {
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;
    use gloo_net::http::Request;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProjectRecordRequest {
        pub project_id: i64,
        pub amount: f64,
        pub is_income: bool,
        pub account_id: i64,
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProjectRecordResponse {
        pub voucher_id: i64,
        pub status: String,
    }

    pub async fn add_project_record(req: ProjectRecordRequest) -> Result<ProjectRecordResponse, String> {
        // This is a placeholder for actual Tauri invoke call
        // In a real Leptos-Tauri app, you would use `invoke("add_project_record", ...)`
        Ok(ProjectRecordResponse {
            voucher_id: 123,
            status: "Mock Success".to_string(),
        })
    }
}
