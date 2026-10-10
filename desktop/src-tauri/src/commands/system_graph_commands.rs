//! Next-Gen program, Domain A, FND-01: Tauri commands for
//! `system_graph_service`'s read-only query API. Writes happen only as a
//! side effect of each owning service's own create/update/delete - there
//! is no `create_system_node`/`update_system_node` command, since nothing
//! outside the 9 owning services should ever author a node directly.

use tauri::State;

use crate::commands::require_workspace_id;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::system_graph::{SystemGraphHit, SystemNode};
use lanesra_core::services::system_graph_service;

#[tauri::command]
pub fn list_system_nodes(state: State<AppState>, node_type: String) -> AppResult<Vec<SystemNode>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::list_nodes_by_type(&conn, &workspace_id, &node_type)
}

#[tauri::command]
pub fn get_system_node(state: State<AppState>, node_type: String, component_id: String) -> AppResult<Option<SystemNode>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::get_node(&conn, &workspace_id, &node_type, &component_id)
}

#[tauri::command]
pub fn get_system_node_dependencies(state: State<AppState>, node_type: String, component_id: String) -> AppResult<Vec<SystemGraphHit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::get_dependencies(&conn, &workspace_id, &node_type, &component_id)
}

#[tauri::command]
pub fn get_system_node_dependents(state: State<AppState>, node_type: String, component_id: String) -> AppResult<Vec<SystemGraphHit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::get_dependents(&conn, &workspace_id, &node_type, &component_id)
}

/// Full transitive downstream closure ("if I change this, what's the
/// blast radius of things it feeds into") - the Dependency Explorer's
/// "Lineage" direction.
#[tauri::command]
pub fn get_system_node_lineage(state: State<AppState>, node_type: String, component_id: String) -> AppResult<Vec<SystemGraphHit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::get_lineage(&conn, &workspace_id, &node_type, &component_id)
}

/// Full transitive upstream closure ("if I change this, what's the blast
/// radius of things that depend on it") - the Dependency Explorer's
/// "Impact" direction, and the direct generalization of
/// `custom_object_service::count_references` to a real transitive answer.
#[tauri::command]
pub fn get_system_node_impact(state: State<AppState>, node_type: String, component_id: String) -> AppResult<Vec<SystemGraphHit>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    system_graph_service::get_impact(&conn, &workspace_id, &node_type, &component_id)
}
