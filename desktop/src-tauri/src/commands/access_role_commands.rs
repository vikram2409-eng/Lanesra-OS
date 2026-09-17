use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::access_role::{AccessDecision, AccessInspectorResult, AccessRole, AccessRoleGrant, AccessRoleGrantInput, AccessRoleInput, AccessRoleUpdate};
use lanesra_core::services::{access_role_service, access_service};

#[tauri::command]
pub fn list_access_roles(state: State<AppState>) -> AppResult<Vec<AccessRole>> {
    let conn = state.conn.lock().unwrap();
    access_role_service::list(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn get_access_role(state: State<AppState>, id: String) -> AppResult<Option<AccessRole>> {
    let conn = state.conn.lock().unwrap();
    access_role_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_access_role(state: State<AppState>, input: AccessRoleInput) -> AppResult<AccessRole> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    access_role_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_access_role(state: State<AppState>, id: String, input: AccessRoleUpdate) -> AppResult<AccessRole> {
    let conn = state.conn.lock().unwrap();
    access_role_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_access_role(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    access_role_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_access_role_grants(state: State<AppState>, access_role_id: String) -> AppResult<Vec<AccessRoleGrant>> {
    let conn = state.conn.lock().unwrap();
    access_role_service::list_grants(&conn, &access_role_id)
}

#[tauri::command]
pub fn upsert_access_role_grant(state: State<AppState>, access_role_id: String, input: AccessRoleGrantInput) -> AppResult<AccessRoleGrant> {
    let conn = state.conn.lock().unwrap();
    access_role_service::upsert_grant(&conn, &access_role_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_access_role_grant(state: State<AppState>, access_role_id: String, object_key: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    access_role_service::delete_grant(&conn, &access_role_id, &object_key, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_user_access_roles(state: State<AppState>, user_id: String) -> AppResult<Vec<AccessRole>> {
    let conn = state.conn.lock().unwrap();
    access_role_service::list_roles_for_user(&conn, &user_id)
}

#[tauri::command]
pub fn list_access_role_assignees(state: State<AppState>, access_role_id: String) -> AppResult<Vec<String>> {
    let conn = state.conn.lock().unwrap();
    access_role_service::list_assignee_user_ids(&conn, &access_role_id)
}

#[tauri::command]
pub fn assign_access_role_to_user(state: State<AppState>, user_id: String, access_role_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    access_role_service::assign_to_user(&conn, &user_id, &access_role_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn remove_access_role_from_user(state: State<AppState>, user_id: String, access_role_id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    access_role_service::remove_from_user(&conn, &user_id, &access_role_id, current_actor(&state).as_deref())
}

/// Lets the frontend hide/disable an action before even attempting it -
/// same evaluator `inspect_access` narrates, just returning the decision
/// alone for the current user.
#[tauri::command]
pub fn check_capability(state: State<AppState>, object_key: String, capability: String, record_id: Option<String>) -> AppResult<AccessDecision> {
    let conn = state.conn.lock().unwrap();
    let actor_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let cap = access_role_service::parse_capability_str(&capability)?;
    Ok(access_service::explain_access(&conn, &actor_id, &object_key, cap, record_id.as_deref())?.decision)
}

/// The Access Inspector: a real, traceable answer to "why can/can't this
/// user do X to this record" - `actor_user_id` defaults to the current
/// user so an admin can also inspect on someone else's behalf.
#[tauri::command]
pub fn inspect_access(state: State<AppState>, object_key: String, capability: String, record_id: Option<String>, actor_user_id: Option<String>) -> AppResult<AccessInspectorResult> {
    let conn = state.conn.lock().unwrap();
    let actor_id = actor_user_id.or_else(|| current_actor(&state)).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let cap = access_role_service::parse_capability_str(&capability)?;
    access_service::explain_access(&conn, &actor_id, &object_key, cap, record_id.as_deref())
}
