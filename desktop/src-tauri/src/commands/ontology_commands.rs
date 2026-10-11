//! Next-Gen program, Ontology Layer epic: Tauri commands for
//! `ontology_service` - the unified Object Type registry (issue #341)
//! and Link Type exposure (issue #342). Both plain, read-only sync
//! commands - no new write path, this is a registry view over data
//! that already exists.

use tauri::State;

use crate::commands::require_workspace_id;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::ontology::{ObjectTypeDetail, ObjectTypeSummary};
use lanesra_core::services::ontology_service;

#[tauri::command]
pub fn list_object_types(state: State<AppState>) -> AppResult<Vec<ObjectTypeSummary>> {
    let conn = state.conn.lock().unwrap();
    ontology_service::list_object_types(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn get_object_type_detail(state: State<AppState>, object_type: String) -> AppResult<ObjectTypeDetail> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    ontology_service::get_object_type_detail(&conn, &workspace_id, &object_type)
}
