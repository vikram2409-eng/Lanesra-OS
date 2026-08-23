use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::saved_view::{SavedView, SavedViewInput};
use lanesra_core::services::saved_view_service;

#[tauri::command]
pub fn create_saved_view(state: State<AppState>, input: SavedViewInput) -> AppResult<SavedView> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    saved_view_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn list_saved_views(state: State<AppState>, object_key: String) -> AppResult<Vec<SavedView>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    saved_view_service::list_for_object(&conn, &workspace_id, &object_key, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_saved_view(state: State<AppState>, id: String, input: SavedViewInput) -> AppResult<SavedView> {
    let conn = state.conn.lock().unwrap();
    saved_view_service::update(&conn, &id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_saved_view(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    saved_view_service::delete(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_saved_view_default(state: State<AppState>, id: String) -> AppResult<SavedView> {
    let conn = state.conn.lock().unwrap();
    saved_view_service::set_default(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn clear_saved_view_default(state: State<AppState>, object_key: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    saved_view_service::clear_default(&conn, &workspace_id, &object_key, current_actor(&state).as_deref())
}
