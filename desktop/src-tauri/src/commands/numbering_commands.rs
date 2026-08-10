use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::numbering_override::{EffectiveNumbering, NumberingOverrideInput};
use lanesra_core::services::numbering_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_numbering_formats(state: State<AppState>) -> AppResult<Vec<EffectiveNumbering>> {
    let conn = state.conn.lock().unwrap();
    numbering_service::list_effective(&conn, &require_workspace_id(&conn)?, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_numbering_format(state: State<AppState>, input: NumberingOverrideInput) -> AppResult<EffectiveNumbering> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    numbering_service::set_override(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn reset_numbering_format(state: State<AppState>, entity_type: String) -> AppResult<EffectiveNumbering> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    numbering_service::reset_override(&conn, &workspace_id, &entity_type, current_actor(&state).as_deref())
}
