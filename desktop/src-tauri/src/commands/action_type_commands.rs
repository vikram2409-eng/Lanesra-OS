//! Next-Gen program, Ontology Layer epic, issue #343: Tauri commands for
//! `action_type_service` - Action Type admin CRUD plus `invoke_action`,
//! the command a record page's generalized Quick Action button (and,
//! later, issue #344's governed AI agent tool) calls.

use std::collections::HashMap;

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::action_type::{ActionTypeDefinition, ActionTypeDefinitionInput};
use lanesra_core::services::action_type_service;

#[tauri::command]
pub fn list_action_types(state: State<AppState>, target_object_type: Option<String>, active_only: bool) -> AppResult<Vec<ActionTypeDefinition>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    action_type_service::list(&conn, &workspace_id, target_object_type.as_deref(), active_only)
}

#[tauri::command]
pub fn create_action_type(state: State<AppState>, input: ActionTypeDefinitionInput) -> AppResult<ActionTypeDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    action_type_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_action_type(state: State<AppState>, id: String, input: ActionTypeDefinitionInput) -> AppResult<ActionTypeDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    action_type_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_action_type(state: State<AppState>, id: String) -> AppResult<ActionTypeDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    action_type_service::deactivate(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn invoke_action_type(state: State<AppState>, action_type_id: String, record_id: String, input_params: HashMap<String, String>) -> AppResult<serde_json::Value> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    action_type_service::invoke(&conn, &workspace_id, &action_type_id, &record_id, &input_params, current_actor(&state).as_deref())
}
