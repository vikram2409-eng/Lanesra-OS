use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::screen_layout::{ScreenLayout, ScreenLayoutInput, ScreenLayoutUpdate};
use lanesra_core::services::screen_layout_service::{self, EffectiveLayout};

#[tauri::command]
pub fn list_screen_layouts(state: State<AppState>, entity_type: String) -> AppResult<Vec<ScreenLayout>> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::list_layouts(&conn, &require_workspace_id(&conn)?, &entity_type)
}

#[tauri::command]
pub fn create_screen_layout(state: State<AppState>, input: ScreenLayoutInput) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    screen_layout_service::create_layout(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_screen_layout(state: State<AppState>, id: String, update: ScreenLayoutUpdate) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::update_layout(&conn, &id, &update, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_screen_layout(state: State<AppState>, id: String) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::publish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn unpublish_screen_layout(state: State<AppState>, id: String) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::unpublish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn revert_screen_layout_draft(state: State<AppState>, id: String) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::revert_layout_draft(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn make_screen_layout_default(state: State<AppState>, id: String) -> AppResult<ScreenLayout> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::make_default(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_screen_layout(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    screen_layout_service::delete_layout(&conn, &id, current_actor(&state).as_deref())
}

/// The tabs a create/edit (and eventually detail) screen should render
/// for `entity_type`, resolved against the current signed-in user's
/// roles - any authenticated user can call this (not admin-only), the
/// same as every other read a record-editing screen needs.
#[tauri::command]
pub fn effective_screen_layout(state: State<AppState>, entity_type: String) -> AppResult<EffectiveLayout> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let tabs = screen_layout_service::resolve_effective_layout(&conn, &workspace_id, &entity_type, current_actor(&state).as_deref())?;
    Ok(EffectiveLayout { tabs })
}
