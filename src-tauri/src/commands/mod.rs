pub mod dtos;

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub async fn add_project_record(
    request: dtos::ProjectRecordRequest,
    state: tauri::State<'_, ProjectServiceState>,
) -> Result<dtos::ProjectRecordResponse, String> {
    state.service.record_income_expense(
        request.project_id,
        request.amount,
        request.is_income,
        request.account_id,
        request.description,
    ).await
    .map(|id| dtos::ProjectRecordResponse {
        voucher_id: id,
        status: "Success".to_string(),
    })
    .map_err(|e| e.to_string())
}

pub struct ProjectServiceState {
    pub service: crate::application::services::project_service::ProjectService,
}
