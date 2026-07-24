mod commands;
mod db;

use commands::*;
use db::DbState;
use std::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(DbState::new()))
        .manage(std::sync::Mutex::new(commands::session::SessionState::new()))
        .invoke_handler(tauri::generate_handler![
            greet,
            // auth / users
            session::login_cmd,
            session::logout_cmd,
            session::current_user_cmd,
            session::set_current_company_cmd,
            user::list_users_cmd,
            user::create_user_cmd,
            user::grant_permission_cmd,
            // companies
            company::list_companies_cmd,
            company::create_company_cmd,
            company::update_company_cmd,
            company::delete_company_cmd,
            user::user_companies_cmd,
            // accounts
            account::list_accounts_cmd,
            account::create_account_cmd,
            account::update_account_cmd,
            account::delete_account_cmd,
            // contacts
            contact::list_contacts_cmd,
            contact::create_contact_cmd,
            contact::update_contact_cmd,
            contact::delete_contact_cmd,
            // vouchers
            voucher::create_voucher_cmd,
            voucher::list_vouchers_cmd,
            voucher::get_voucher_cmd,
            voucher::delete_voucher_cmd,
            voucher::audit_voucher_cmd,
            voucher::list_audit_logs_cmd,
            voucher::next_voucher_no_cmd,
            // raw data
            raw::scan_raw_directory_cmd,
            raw::auto_import_raw_directory_cmd,
            raw::import_raw_file_cmd,
            raw::list_raw_records_cmd,
            raw::get_raw_record_cmd,
            raw::reconcile_cmd,
            raw::list_reconciliation_items_cmd,
            raw::review_summary_cmd,
            raw::list_raw_audit_logs_cmd,
            // backup
            backup::backup_company_cmd,
            backup::backup_system_cmd,
            backup::list_backups_cmd,
            backup::restore_company_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
