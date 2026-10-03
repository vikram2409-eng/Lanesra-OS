use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::admin_home::AdminHomeSummary;
use lanesra_core::models::admin_nav::AdminNavItem;
use lanesra_core::services::{admin_home_service, admin_nav_service, admin_search_service};
use lanesra_core::services::admin_search_service::AdminSearchResult;

/// Admin Control Center Modernization (issue #197): the Admin landing
/// page's own Setup Progress / Needs Attention / Platform Health data.
#[tauri::command]
pub fn get_admin_home_summary(state: State<AppState>) -> AppResult<AdminHomeSummary> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_home_service::get_summary(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn admin_search(state: State<AppState>, query: String) -> AppResult<Vec<AdminSearchResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_search_service::admin_search(&conn, &workspace_id, &query, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn record_admin_visit(state: State<AppState>, admin_tab: String, label: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_nav_service::record_visit(&conn, &workspace_id, &admin_tab, &label, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_admin_pinned(state: State<AppState>, admin_tab: String, pinned: bool) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_nav_service::set_pinned(&conn, &workspace_id, &admin_tab, pinned, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_recent_admin_visits(state: State<AppState>) -> AppResult<Vec<AdminNavItem>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_nav_service::list_recent(&conn, &workspace_id, 10, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_pinned_admin_items(state: State<AppState>) -> AppResult<Vec<AdminNavItem>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    admin_nav_service::list_pinned(&conn, &workspace_id, current_actor(&state).as_deref())
}
