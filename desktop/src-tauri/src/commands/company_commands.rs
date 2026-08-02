use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::domain::AppResult;
use crate::models::company::{Company, CompanyInput};
use crate::services::company_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_companies(state: State<AppState>) -> AppResult<Vec<Company>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    company_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn get_company(state: State<AppState>, id: String) -> AppResult<Company> {
    let conn = state.conn.lock().unwrap();
    company_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_company(state: State<AppState>, input: CompanyInput) -> AppResult<Company> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    company_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_company(state: State<AppState>, id: String, input: CompanyInput) -> AppResult<Company> {
    let conn = state.conn.lock().unwrap();
    company_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_company(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    company_service::archive(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn check_company_duplicates(
    state: State<AppState>,
    name: String,
    exclude_id: Option<String>,
) -> AppResult<Vec<Company>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    company_service::check_duplicates(&conn, &workspace_id, &name, exclude_id.as_deref())
}
