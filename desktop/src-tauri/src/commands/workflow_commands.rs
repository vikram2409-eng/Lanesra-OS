use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::custom_field::CustomFieldValues;
use lanesra_core::models::workflow::{
    WorkflowDefinition, WorkflowDefinitionInput, WorkflowDefinitionUpdate, WorkflowRun, WorkflowRuleVersion,
};
use lanesra_core::services::workflow_service;
use lanesra_core::services::workflow_service::WorkflowTestResult;

#[tauri::command]
pub fn list_workflow_rules(state: State<AppState>, entity_type: String) -> AppResult<Vec<WorkflowDefinition>> {
    let conn = state.conn.lock().unwrap();
    workflow_service::list_rules(&conn, &require_workspace_id(&conn)?, &entity_type, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn create_workflow_rule(state: State<AppState>, input: WorkflowDefinitionInput) -> AppResult<WorkflowDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    workflow_service::create_rule(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_workflow_rule(state: State<AppState>, id: String, input: WorkflowDefinitionUpdate) -> AppResult<WorkflowDefinition> {
    let conn = state.conn.lock().unwrap();
    workflow_service::update_rule(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_workflow_runs(state: State<AppState>, workflow_id: String) -> AppResult<Vec<WorkflowRun>> {
    let conn = state.conn.lock().unwrap();
    workflow_service::list_runs(&conn, &require_workspace_id(&conn)?, &workflow_id, current_actor(&state).as_deref())
}

/// ADM-WF-11: polled periodically by the frontend while the app is open
/// (and once at startup) - see workflow_service::run_scheduled's own
/// comment for why this replaces a full OS-level background scheduler.
#[tauri::command]
pub fn run_scheduled_workflows(state: State<AppState>) -> AppResult<usize> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    workflow_service::run_scheduled(&conn, &workspace_id, current_actor(&state).as_deref())
}

/// Addendum Phase 3: dry-run every active workflow for `entity_type`
/// against a hypothetical field context, mirroring `test_business_rules`.
#[tauri::command]
pub fn test_workflows(state: State<AppState>, entity_type: String, context: CustomFieldValues) -> AppResult<WorkflowTestResult> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    workflow_service::test_workflows(&conn, &workspace_id, &entity_type, &context, current_actor(&state).as_deref())
}

/// Admin UX polish (spec §10).
#[tauri::command]
pub fn duplicate_workflow_rule(state: State<AppState>, id: String) -> AppResult<WorkflowDefinition> {
    let conn = state.conn.lock().unwrap();
    workflow_service::duplicate_rule(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_workflow_rule_versions(state: State<AppState>, workflow_id: String) -> AppResult<Vec<WorkflowRuleVersion>> {
    let conn = state.conn.lock().unwrap();
    workflow_service::list_versions(&conn, &workflow_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn restore_workflow_rule_version(state: State<AppState>, workflow_id: String, version_id: String) -> AppResult<WorkflowDefinition> {
    let conn = state.conn.lock().unwrap();
    workflow_service::restore_version(&conn, &workflow_id, &version_id, current_actor(&state).as_deref())
}
