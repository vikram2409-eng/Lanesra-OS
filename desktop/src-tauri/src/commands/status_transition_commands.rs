use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::status_transition::{StatusTransition, StatusTransitionInput};
use lanesra_core::services::status_transition_service;

#[tauri::command]
pub fn list_status_transitions(state: State<AppState>, entity_type: String) -> AppResult<Vec<StatusTransition>> {
    let conn = state.conn.lock().unwrap();
    status_transition_service::list(&conn, &require_workspace_id(&conn)?, &entity_type, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_status_transition(state: State<AppState>, input: StatusTransitionInput) -> AppResult<StatusTransition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    status_transition_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_status_transition_active(state: State<AppState>, id: String, is_active: bool) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    status_transition_service::set_active(&conn, &id, is_active, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_status_transition(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    status_transition_service::delete(&conn, &id, current_actor(&state).as_deref())
}
