use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::custom_object::{CustomObjectDefinition, CustomObjectDefinitionInput, CustomObjectDefinitionUpdate};
use lanesra_core::services::custom_object_service;

#[tauri::command]
pub fn list_custom_objects(state: State<AppState>, active_only: bool) -> AppResult<Vec<CustomObjectDefinition>> {
    let conn = state.conn.lock().unwrap();
    custom_object_service::list(&conn, &require_workspace_id(&conn)?, active_only)
}

#[tauri::command]
pub fn create_custom_object(state: State<AppState>, input: CustomObjectDefinitionInput) -> AppResult<CustomObjectDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    custom_object_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_custom_object(state: State<AppState>, id: String, input: CustomObjectDefinitionUpdate) -> AppResult<CustomObjectDefinition> {
    let conn = state.conn.lock().unwrap();
    custom_object_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_custom_object(state: State<AppState>, id: String) -> AppResult<CustomObjectDefinition> {
    let conn = state.conn.lock().unwrap();
    custom_object_service::deactivate(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_custom_object(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    custom_object_service::delete(&conn, &id, current_actor(&state).as_deref())
}
