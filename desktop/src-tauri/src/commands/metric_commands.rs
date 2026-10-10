//! Next-Gen program, Domain A, FND-02: Tauri commands for
//! `metric_service` (Metric definitions).

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::semantic::{MetricDefinition, MetricDefinitionInput, MetricVersion};
use lanesra_core::services::metric_service;

#[tauri::command]
pub fn list_metric_definitions(state: State<AppState>, active_only: bool) -> AppResult<Vec<MetricDefinition>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    metric_service::list(&conn, &workspace_id, active_only)
}

#[tauri::command]
pub fn create_metric_definition(state: State<AppState>, input: MetricDefinitionInput) -> AppResult<MetricDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    metric_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_metric_definition(state: State<AppState>, id: String, input: MetricDefinitionInput) -> AppResult<MetricDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    metric_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_metric_versions(state: State<AppState>, metric_id: String) -> AppResult<Vec<MetricVersion>> {
    let conn = state.conn.lock().unwrap();
    metric_service::list_versions(&conn, &metric_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_metric_definition(state: State<AppState>, id: String) -> AppResult<MetricDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    metric_service::deactivate(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_metric_definition(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    metric_service::delete(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
