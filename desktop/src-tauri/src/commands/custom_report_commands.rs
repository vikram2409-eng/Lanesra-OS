use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::custom_report::{CustomReport, CustomReportInput, CustomReportRow, CustomReportUpdate};
use lanesra_core::services::custom_report_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_custom_reports(state: State<AppState>) -> AppResult<Vec<CustomReport>> {
    let conn = state.conn.lock().unwrap();
    custom_report_service::list(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_custom_report(state: State<AppState>, input: CustomReportInput) -> AppResult<CustomReport> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    custom_report_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_custom_report(state: State<AppState>, id: String, input: CustomReportUpdate) -> AppResult<CustomReport> {
    let conn = state.conn.lock().unwrap();
    custom_report_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_custom_report(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    custom_report_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn run_custom_report(state: State<AppState>, id: String) -> AppResult<Vec<CustomReportRow>> {
    let conn = state.conn.lock().unwrap();
    let report = lanesra_core::repositories::custom_report_repo::get(&conn, &id)?
        .ok_or_else(|| lanesra_core::domain::AppError::NotFound("Custom report".into()))?;
    custom_report_service::run(&conn, &report)
}
