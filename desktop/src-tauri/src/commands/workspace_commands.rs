use tauri::State;

use lanesra_core::domain::AppResult;
use lanesra_core::models::user::User;
use lanesra_core::models::workspace::{DashboardKpiPrefs, Workspace, WorkspaceLogo, WorkspaceSetup, WorkspaceUpdate};
use lanesra_core::repositories::workspace_repo;
use lanesra_core::services::workspace_service;
use crate::commands::current_actor;
use crate::state::AppState;

#[tauri::command]
pub fn workspace_status(state: State<AppState>) -> AppResult<Option<Workspace>> {
    let conn = state.conn.lock().unwrap();
    Ok(workspace_repo::get_current(&conn)?)
}

#[tauri::command]
pub fn first_run_setup(
    state: State<AppState>,
    setup: WorkspaceSetup,
) -> AppResult<(Workspace, User)> {
    let conn = state.conn.lock().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup)?;
    *state.session_user_id.lock().unwrap() = Some(admin.id.clone());
    Ok((workspace, admin))
}

#[tauri::command]
pub fn update_workspace(state: State<AppState>, input: WorkspaceUpdate) -> AppResult<Workspace> {
    let conn = state.conn.lock().unwrap();
    workspace_service::update(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_workspace_logo(state: State<AppState>, input: WorkspaceLogo) -> AppResult<Workspace> {
    let conn = state.conn.lock().unwrap();
    workspace_service::set_logo(&conn, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn clear_workspace_logo(state: State<AppState>) -> AppResult<Workspace> {
    let conn = state.conn.lock().unwrap();
    workspace_service::clear_logo(&conn, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_dashboard_kpis(state: State<AppState>, prefs: DashboardKpiPrefs) -> AppResult<Workspace> {
    let conn = state.conn.lock().unwrap();
    workspace_service::set_dashboard_kpis(&conn, &prefs, current_actor(&state).as_deref())
}
