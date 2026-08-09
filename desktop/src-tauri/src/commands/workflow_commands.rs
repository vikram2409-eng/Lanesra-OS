use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::workflow_rule::{WorkflowRule, WorkflowRuleInput, WorkflowRuleUpdate};
use lanesra_core::services::workflow_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_workflow_rules(state: State<AppState>, entity_type: String) -> AppResult<Vec<WorkflowRule>> {
    let conn = state.conn.lock().unwrap();
    workflow_service::list_rules(&conn, &require_workspace_id(&conn)?, &entity_type, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_workflow_rule(state: State<AppState>, input: WorkflowRuleInput) -> AppResult<WorkflowRule> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    workflow_service::create_rule(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_workflow_rule(state: State<AppState>, id: String, input: WorkflowRuleUpdate) -> AppResult<WorkflowRule> {
    let conn = state.conn.lock().unwrap();
    workflow_service::update_rule(&conn, &id, &input, current_actor(&state).as_deref())
}
