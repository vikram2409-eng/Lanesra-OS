//! AI & Agentic Layer, Phase 3: the Unified Activity Timeline's two
//! commands - both plain sync, since logging a manually-recorded
//! interaction makes no network call (unlike `ai_commands`/
//! `integration_commands`'s genuinely-async ones).

use tauri::State;

use crate::commands::current_actor;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::activity::{Activity, ActivityInput};
use lanesra_core::services::activity_service;

#[tauri::command]
pub fn log_activity(state: State<AppState>, input: ActivityInput) -> AppResult<Activity> {
    let conn = state.conn.lock().unwrap();
    activity_service::log_activity(&conn, &input, current_actor(&state).as_deref())
}

/// Any authenticated user can view an entity's interactions - same
/// access model `list_audit_events` already uses (see that command's
/// own comment).
#[tauri::command]
pub fn list_activities(state: State<AppState>, entity_type: String, entity_id: String) -> AppResult<Vec<Activity>> {
    let conn = state.conn.lock().unwrap();
    activity_service::list_for_entity(&conn, &entity_type, &entity_id)
}
