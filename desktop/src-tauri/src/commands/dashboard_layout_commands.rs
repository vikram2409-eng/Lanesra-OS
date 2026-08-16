use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::dashboard_layout::{DashboardLayout, DashboardLayoutInput, DashboardLayoutUpdate};
use lanesra_core::services::dashboard_layout_service::{self, EffectiveDashboard};

#[tauri::command]
pub fn list_dashboard_layouts(state: State<AppState>) -> AppResult<Vec<DashboardLayout>> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::list_layouts(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_dashboard_layout(state: State<AppState>, input: DashboardLayoutInput) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    dashboard_layout_service::create_layout(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_dashboard_layout(state: State<AppState>, id: String, update: DashboardLayoutUpdate) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::update_layout(&conn, &id, &update, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_dashboard_layout(state: State<AppState>, id: String) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::publish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn unpublish_dashboard_layout(state: State<AppState>, id: String) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::unpublish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn revert_dashboard_layout_draft(state: State<AppState>, id: String) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::revert_layout_draft(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn make_dashboard_layout_default(state: State<AppState>, id: String) -> AppResult<DashboardLayout> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::make_default(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_dashboard_layout(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    dashboard_layout_service::delete_layout(&conn, &id, current_actor(&state).as_deref())
}

/// The widgets the live Dashboard should render, resolved against the
/// current signed-in user's roles - any authenticated user can call this
/// (not admin-only), same as `effective_screen_layout`.
#[tauri::command]
pub fn effective_dashboard_layout(state: State<AppState>) -> AppResult<EffectiveDashboard> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let widgets = dashboard_layout_service::resolve_effective_dashboard(&conn, &workspace_id, current_actor(&state).as_deref())?;
    Ok(EffectiveDashboard { widgets })
}
