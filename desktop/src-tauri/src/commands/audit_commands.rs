use tauri::State;

use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::audit::AuditEvent;
use lanesra_core::services::audit_service;

/// Any authenticated user can view an entity's audit history - matches
/// `global_search`'s access model, since viewing a record's history needs
/// no more privilege than viewing the record itself.
#[tauri::command]
pub fn list_audit_events(state: State<AppState>, entity_type: String, entity_id: String) -> AppResult<Vec<AuditEvent>> {
    let conn = state.conn.lock().unwrap();
    audit_service::list_for_entity(&conn, &entity_type, &entity_id)
}
