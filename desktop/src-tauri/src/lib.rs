pub mod commands;
pub mod state;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("lanesra.sqlite3");
            let conn = lanesra_core::db::open_workspace_db(&db_path)?;
            app.manage(AppState::new(conn, db_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace_commands::workspace_status,
            commands::workspace_commands::first_run_setup,
            commands::workspace_commands::update_workspace,
            commands::workspace_commands::set_workspace_logo,
            commands::workspace_commands::clear_workspace_logo,
            commands::auth_commands::login,
            commands::auth_commands::logout,
            commands::auth_commands::current_user,
            commands::auth_commands::change_my_password,
            commands::company_commands::list_companies,
            commands::company_commands::get_company,
            commands::company_commands::create_company,
            commands::company_commands::update_company,
            commands::company_commands::archive_company,
            commands::company_commands::check_company_duplicates,
            commands::contact_commands::list_contacts,
            commands::contact_commands::list_contacts_by_company,
            commands::contact_commands::get_contact,
            commands::contact_commands::create_contact,
            commands::contact_commands::update_contact,
            commands::contact_commands::archive_contact,
            commands::contact_commands::check_contact_duplicates,
            commands::product_commands::list_products,
            commands::product_commands::get_product,
            commands::product_commands::create_product,
            commands::product_commands::update_product,
            commands::product_commands::archive_product,
            commands::opportunity_commands::list_opportunities,
            commands::opportunity_commands::list_opportunities_by_company,
            commands::opportunity_commands::get_opportunity,
            commands::opportunity_commands::create_opportunity,
            commands::opportunity_commands::update_opportunity,
            commands::opportunity_commands::archive_opportunity,
            commands::opportunity_commands::set_opportunity_products,
            commands::opportunity_commands::list_opportunity_products,
            commands::quote_commands::list_quotes,
            commands::quote_commands::get_quote,
            commands::quote_commands::create_quote,
            commands::quote_commands::set_quote_status,
            commands::quote_commands::convert_quote_to_order,
            commands::order_commands::list_orders,
            commands::order_commands::get_order,
            commands::order_commands::create_order,
            commands::order_commands::set_order_status,
            commands::order_commands::convert_order_to_invoice,
            commands::invoice_commands::list_invoices,
            commands::invoice_commands::get_invoice,
            commands::invoice_commands::create_invoice,
            commands::invoice_commands::issue_invoice,
            commands::invoice_commands::void_invoice,
            commands::invoice_commands::record_invoice_payment,
            commands::invoice_commands::refresh_overdue_invoices,
            commands::contract_commands::list_contracts,
            commands::contract_commands::list_contracts_by_company,
            commands::contract_commands::get_contract,
            commands::contract_commands::create_contract,
            commands::contract_commands::update_contract,
            commands::contract_commands::archive_contract,
            commands::task_commands::list_tasks,
            commands::task_commands::list_tasks_by_related,
            commands::task_commands::get_task,
            commands::task_commands::create_task,
            commands::task_commands::update_task,
            commands::task_commands::archive_task,
            commands::user_commands::list_users,
            commands::user_commands::create_user,
            commands::user_commands::update_user,
            commands::user_commands::set_user_password,
            commands::dashboard_commands::dashboard_summary,
            commands::backup_commands::create_backup,
            commands::backup_commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lanesra OS");
}
