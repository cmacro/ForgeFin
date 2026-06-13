use crate::commands::{add_project_record, ping, ProjectServiceState};
use crate::infrastructure::persistence::repositories::sqlite_project_repository::SqliteProjectRepository;
use crate::application::services::project_service::ProjectService;
use sqlx::sqlite::SqlitePool;

#[tauri::command]
fn setup_db(pool: SqlitePool) {
    // Migration and pool setup logic
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri::plugin::shell::run())
        .invoke_handler(tauri::generate_handler![ping, add_project_record])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
