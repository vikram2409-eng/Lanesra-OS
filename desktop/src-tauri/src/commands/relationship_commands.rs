use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::relationship::{
    RelatedRecord, RelationshipDefinition, RelationshipDefinitionInput, RelationshipDefinitionUpdate, RelationshipInstance,
};
use lanesra_core::services::relationship_service;

#[tauri::command]
pub fn list_relationship_definitions(state: State<AppState>, active_only: bool) -> AppResult<Vec<RelationshipDefinition>> {
    let conn = state.conn.lock().unwrap();
    relationship_service::list(&conn, &require_workspace_id(&conn)?, active_only)
}

#[tauri::command]
pub fn create_relationship_definition(state: State<AppState>, input: RelationshipDefinitionInput) -> AppResult<RelationshipDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    relationship_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_relationship_definition(state: State<AppState>, id: String, input: RelationshipDefinitionUpdate) -> AppResult<RelationshipDefinition> {
    let conn = state.conn.lock().unwrap();
    relationship_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_relationship_definition(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    relationship_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn link_records(
    state: State<AppState>,
    definition_id: String,
    source_entity_type: String,
    source_id: String,
    target_entity_type: String,
    target_id: String,
) -> AppResult<RelationshipInstance> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    relationship_service::link(
        &conn, &workspace_id, &definition_id, &source_entity_type, &source_id, &target_entity_type, &target_id,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn unlink_records(state: State<AppState>, instance_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    relationship_service::unlink(&conn, &instance_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_related_records(state: State<AppState>, entity_type: String, entity_id: String) -> AppResult<Vec<RelatedRecord>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    relationship_service::related_records_for(&conn, &workspace_id, &entity_type, &entity_id)
}
