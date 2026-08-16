use std::collections::HashMap;

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::custom_field::{
    CustomFieldDefinition, CustomFieldDefinitionInput, CustomFieldDefinitionUpdate, CustomFieldValues,
};
use lanesra_core::services::custom_field_service::{self, SaveNotices};
use crate::state::AppState;

#[tauri::command]
pub fn list_custom_field_definitions(
    state: State<AppState>,
    entity_type: String,
    active_only: bool,
) -> AppResult<Vec<CustomFieldDefinition>> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::list_definitions(&conn, &require_workspace_id(&conn)?, &entity_type, active_only)
}

#[tauri::command]
pub fn create_custom_field_definition(
    state: State<AppState>,
    input: CustomFieldDefinitionInput,
) -> AppResult<CustomFieldDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    custom_field_service::create_definition(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_custom_field_definition(
    state: State<AppState>,
    id: String,
    input: CustomFieldDefinitionUpdate,
) -> AppResult<CustomFieldDefinition> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::update_definition(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn deactivate_custom_field_definition(state: State<AppState>, id: String) -> AppResult<CustomFieldDefinition> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::deactivate_definition(&conn, &id, current_actor(&state).as_deref())
}

/// Admin UX polish (spec §10): the frontend calls this before either
/// deactivation path above and shows a confirmation dialog when the result
/// isn't empty.
#[tauri::command]
pub fn describe_custom_field_dependents(state: State<AppState>, id: String) -> AppResult<Vec<String>> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::describe_active_dependents(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_custom_field_values(
    state: State<AppState>,
    entity_type: String,
    entity_id: String,
    values: CustomFieldValues,
) -> AppResult<SaveNotices> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::set_entity_values(&conn, &entity_type, &entity_id, &values, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_custom_field_values(state: State<AppState>, entity_id: String) -> AppResult<CustomFieldValues> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::get_entity_values(&conn, &entity_id)
}

/// List-view filtering: every `is_filterable` value for one entity type,
/// keyed by entity id then field key - see
/// custom_field_service::get_filterable_values.
#[tauri::command]
pub fn list_filterable_custom_field_values(state: State<AppState>, entity_type: String) -> AppResult<HashMap<String, CustomFieldValues>> {
    let conn = state.conn.lock().unwrap();
    custom_field_service::get_filterable_values(&conn, &require_workspace_id(&conn)?, &entity_type)
}
