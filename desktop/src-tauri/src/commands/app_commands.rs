use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::app_definition::{AccessibleApp, AppDefinition, AppDefinitionInput, AppDefinitionUpdate, AppPermission, AppPermissionInput};
use lanesra_core::services::app_service;

#[tauri::command]
pub fn list_apps(state: State<AppState>) -> AppResult<Vec<AppDefinition>> {
    let conn = state.conn.lock().unwrap();
    app_service::list(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_app(state: State<AppState>, input: AppDefinitionInput) -> AppResult<AppDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    app_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_app(state: State<AppState>, id: String, update: AppDefinitionUpdate) -> AppResult<AppDefinition> {
    let conn = state.conn.lock().unwrap();
    app_service::update(&conn, &id, &update, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_app(state: State<AppState>, id: String) -> AppResult<AppDefinition> {
    let conn = state.conn.lock().unwrap();
    app_service::publish(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn unpublish_app(state: State<AppState>, id: String) -> AppResult<AppDefinition> {
    let conn = state.conn.lock().unwrap();
    app_service::unpublish(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_app(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    app_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_app_permissions(state: State<AppState>, app_id: String) -> AppResult<Vec<AppPermission>> {
    let conn = state.conn.lock().unwrap();
    app_service::list_permissions(&conn, &app_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn grant_app_permission(state: State<AppState>, app_id: String, input: AppPermissionInput) -> AppResult<AppPermission> {
    let conn = state.conn.lock().unwrap();
    app_service::grant_permission(&conn, &app_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn revoke_app_permission(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    app_service::revoke_permission(&conn, &id, current_actor(&state).as_deref())
}

/// Every published app the current signed-in user can see, with their
/// resolved access level on each - drives the sidebar's app switcher. Any
/// authenticated user can call this (not admin-only), same as
/// `effective_dashboard_layout`.
#[tauri::command]
pub fn list_accessible_apps(state: State<AppState>) -> AppResult<Vec<AccessibleApp>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    app_service::list_accessible(&conn, &workspace_id, current_actor(&state).as_deref())
}
