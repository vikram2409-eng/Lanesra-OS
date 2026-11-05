//! WP-0133: desktop Tauri commands for guardrails — the same core service
//! the Team Workspace server exposes over HTTP (list/create/edit).

use tauri::State;

use lanesra_core::domain::AppResult;
use lanesra_core::models::ai_guardrail::{Guardrail, GuardrailEdit, GuardrailInput};
use lanesra_core::services::guardrail_service;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;

#[tauri::command]
pub fn list_guardrails(state: State<AppState>) -> AppResult<Vec<Guardrail>> {
    let conn = state.conn.lock().unwrap();
    let ws = require_workspace_id(&conn)?;
    guardrail_service::list(&conn, &ws, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_guardrail(state: State<AppState>, input: GuardrailInput) -> AppResult<Guardrail> {
    let conn = state.conn.lock().unwrap();
    let ws = require_workspace_id(&conn)?;
    guardrail_service::create(&conn, &ws, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn edit_guardrail(
    state: State<AppState>,
    id: String,
    edit: GuardrailEdit,
) -> AppResult<Guardrail> {
    let conn = state.conn.lock().unwrap();
    let ws = require_workspace_id(&conn)?;
    guardrail_service::edit(&conn, &ws, &id, &edit, current_actor(&state).as_deref())
}
