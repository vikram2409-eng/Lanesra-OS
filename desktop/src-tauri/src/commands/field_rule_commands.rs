use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::field_rule::{FieldRule, FieldRuleInput, FieldRuleUpdate};
use lanesra_core::services::field_rule_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_field_rules(state: State<AppState>, entity_type: String, active_only: bool) -> AppResult<Vec<FieldRule>> {
    let conn = state.conn.lock().unwrap();
    field_rule_service::list_rules(&conn, &require_workspace_id(&conn)?, &entity_type, active_only)
}

#[tauri::command]
pub fn create_field_rule(state: State<AppState>, input: FieldRuleInput) -> AppResult<FieldRule> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    field_rule_service::create_rule(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_field_rule(state: State<AppState>, id: String, input: FieldRuleUpdate) -> AppResult<FieldRule> {
    let conn = state.conn.lock().unwrap();
    field_rule_service::update_rule(&conn, &id, &input, current_actor(&state).as_deref())
}
