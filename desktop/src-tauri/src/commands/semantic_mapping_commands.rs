//! Next-Gen program, Domain A, FND-02: Tauri commands for
//! `semantic_mapping_service` (object/field -> glossary term / semantic
//! role mappings).

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::semantic::{SemanticMapping, SemanticMappingInput};
use lanesra_core::services::semantic_mapping_service;

#[tauri::command]
pub fn list_semantic_mappings_for_entity(state: State<AppState>, entity_type: String) -> AppResult<Vec<SemanticMapping>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    semantic_mapping_service::list_for_entity(&conn, &workspace_id, &entity_type)
}

#[tauri::command]
pub fn list_semantic_mappings_for_term(state: State<AppState>, term_id: String) -> AppResult<Vec<SemanticMapping>> {
    let conn = state.conn.lock().unwrap();
    semantic_mapping_service::list_for_term(&conn, &term_id)
}

#[tauri::command]
pub fn create_semantic_mapping(state: State<AppState>, input: SemanticMappingInput) -> AppResult<SemanticMapping> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    semantic_mapping_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_semantic_mapping(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    semantic_mapping_service::delete(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
