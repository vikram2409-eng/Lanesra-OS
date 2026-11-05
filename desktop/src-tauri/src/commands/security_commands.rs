//! WP-0106: desktop Tauri commands for the security settings and active
//! sessions — the same core services the Team Workspace server exposes over
//! HTTP (`get`/`update_security_settings`, `list_active_sessions`,
//! `sign_out_all_sessions`), so the admin screen works in both editions.

use tauri::State;

use lanesra_core::domain::AppResult;
use lanesra_core::models::security::{ActiveSession, SecuritySettings, SecuritySettingsUpdate};
use lanesra_core::services::{security_settings_service, web_session_service};

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;

#[tauri::command]
pub fn get_security_settings(state: State<AppState>) -> AppResult<SecuritySettings> {
    let conn = state.conn.lock().unwrap();
    security_settings_service::get_settings(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn update_security_settings(
    state: State<AppState>,
    input: SecuritySettingsUpdate,
) -> AppResult<SecuritySettings> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    security_settings_service::update_settings(
        &conn,
        &workspace_id,
        &input,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn list_active_sessions(state: State<AppState>) -> AppResult<Vec<ActiveSession>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    // The desktop app has no web sessions of its own; this lists the Team
    // Workspace server's when the same core runs there.
    web_session_service::list_active_sessions(
        &conn,
        &workspace_id,
        current_actor(&state).as_deref(),
        None,
    )
}

#[tauri::command]
pub fn sign_out_all_sessions(state: State<AppState>) -> AppResult<serde_json::Value> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let ended =
        web_session_service::sign_out_all(&conn, &workspace_id, current_actor(&state).as_deref())?;
    Ok(serde_json::json!({ "ended": ended }))
}
