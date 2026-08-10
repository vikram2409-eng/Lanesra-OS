use tauri::State;

use crate::commands::current_actor;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::custom_record::{CustomRecord, CustomRecordInput, CustomRecordUpdate};
use lanesra_core::services::custom_record_service;

#[tauri::command]
pub fn list_custom_records(state: State<AppState>, object_key: String) -> AppResult<Vec<CustomRecord>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = crate::commands::require_workspace_id(&conn)?;
    custom_record_service::list(&conn, &workspace_id, &object_key)
}

#[tauri::command]
pub fn get_custom_record(state: State<AppState>, id: String) -> AppResult<CustomRecord> {
    let conn = state.conn.lock().unwrap();
    custom_record_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_custom_record(state: State<AppState>, input: CustomRecordInput) -> AppResult<CustomRecord> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = crate::commands::require_workspace_id(&conn)?;
    custom_record_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_custom_record(state: State<AppState>, id: String, input: CustomRecordUpdate) -> AppResult<CustomRecord> {
    let conn = state.conn.lock().unwrap();
    custom_record_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_custom_record(state: State<AppState>, id: String) -> AppResult<CustomRecord> {
    let conn = state.conn.lock().unwrap();
    custom_record_service::archive(&conn, &id, current_actor(&state).as_deref())
}
