use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::page_layout::{PageLayout, PageLayoutInput, PageLayoutUpdate};
use lanesra_core::services::page_layout_service::{self, EffectivePage};

#[tauri::command]
pub fn list_page_layouts(state: State<AppState>, entity_type: String) -> AppResult<Vec<PageLayout>> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::list_layouts(&conn, &require_workspace_id(&conn)?, &entity_type)
}

#[tauri::command]
pub fn create_page_layout(state: State<AppState>, input: PageLayoutInput) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    page_layout_service::create_layout(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_page_layout(state: State<AppState>, id: String, update: PageLayoutUpdate) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::update_layout(&conn, &id, &update, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_page_layout(state: State<AppState>, id: String) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::publish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn unpublish_page_layout(state: State<AppState>, id: String) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::unpublish_layout(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn revert_page_layout_draft(state: State<AppState>, id: String) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::revert_layout_draft(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn make_page_layout_default(state: State<AppState>, id: String) -> AppResult<PageLayout> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::make_default(&conn, &id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_page_layout(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    page_layout_service::delete_layout(&conn, &id, current_actor(&state).as_deref())
}

/// The page a create/edit (and eventually detail, in 5b) screen should
/// render for `entity_type`, resolved against the current signed-in
/// user's roles - any authenticated user can call this, same as
/// `effective_screen_layout`.
#[tauri::command]
pub fn effective_page_layout(state: State<AppState>, entity_type: String) -> AppResult<EffectivePage> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let page = page_layout_service::resolve_effective_page(&conn, &workspace_id, &entity_type, current_actor(&state).as_deref())?;
    Ok(EffectivePage { page })
}
