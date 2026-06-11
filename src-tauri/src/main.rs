use tauri::Manager;

mod commands;
mod db;
mod ai;
mod services;
mod models;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            // Initialize DB and AI models here
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
