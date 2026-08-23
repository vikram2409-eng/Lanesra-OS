use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::services::bulk_action_service::{self, BulkActionResult};

#[tauri::command]
pub fn bulk_update_builtin_field(state: State<AppState>, object_key: String, ids: Vec<String>, field_key: String, value: String) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    bulk_action_service::bulk_update_builtin_field(&conn, &workspace_id, &object_key, &ids, &field_key, &value, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_update_custom_field(state: State<AppState>, object_key: String, ids: Vec<String>, field_key: String, value: String) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    bulk_action_service::bulk_update_custom_field(&conn, &object_key, &ids, &field_key, &value, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_reassign_owner(state: State<AppState>, object_key: String, ids: Vec<String>, owner_user_id: Option<String>) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    bulk_action_service::bulk_reassign_owner(&conn, &workspace_id, &object_key, &ids, owner_user_id.as_deref(), current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_change_status(state: State<AppState>, object_key: String, ids: Vec<String>, new_status: String) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    bulk_action_service::bulk_change_status(&conn, &workspace_id, &object_key, &ids, &new_status, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_update_tags(state: State<AppState>, object_key: String, ids: Vec<String>, tags: Vec<String>, add: bool) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    bulk_action_service::bulk_update_tags(&conn, &workspace_id, &object_key, &ids, &tags, add, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn bulk_archive(state: State<AppState>, object_key: String, ids: Vec<String>) -> AppResult<Vec<BulkActionResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    bulk_action_service::bulk_archive(&conn, &workspace_id, &object_key, &ids, current_actor(&state).as_deref())
}
