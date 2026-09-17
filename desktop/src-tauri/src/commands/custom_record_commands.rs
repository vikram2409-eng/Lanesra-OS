use tauri::State;

use crate::commands::current_actor;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::access_role::Capability;
use lanesra_core::models::custom_record::{CustomRecord, CustomRecordInput, CustomRecordUpdate};
use lanesra_core::services::{access_service, custom_record_service};

#[tauri::command]
pub fn list_custom_records(state: State<AppState>, object_key: String) -> AppResult<Vec<CustomRecord>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = crate::commands::require_workspace_id(&conn)?;
    let mut records = custom_record_service::list(&conn, &workspace_id, &object_key)?;
    let visible = access_service::filter_visible(&conn, current_actor(&state).as_deref(), &object_key, records.iter().map(|r| r.id.as_str()))?;
    records.retain(|r| visible.contains(&r.id));
    Ok(records)
}

#[tauri::command]
pub fn get_custom_record(state: State<AppState>, id: String) -> AppResult<CustomRecord> {
    let conn = state.conn.lock().unwrap();
    let record = custom_record_service::get(&conn, &id)?;
    access_service::require_capability(&conn, current_actor(&state).as_deref(), &record.object_key, Capability::Read, Some(&id))?;
    Ok(record)
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
