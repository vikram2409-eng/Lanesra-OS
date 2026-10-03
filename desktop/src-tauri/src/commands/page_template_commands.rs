use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::page_template::{PageTemplate, PageTemplateInput};
use lanesra_core::services::page_template_service;

#[tauri::command]
pub fn list_page_templates(state: State<AppState>, entity_type: String) -> AppResult<Vec<PageTemplate>> {
    let conn = state.conn.lock().unwrap();
    page_template_service::list_templates(&conn, &require_workspace_id(&conn)?, &entity_type)
}

#[tauri::command]
pub fn create_page_template(state: State<AppState>, page_id: String, input: PageTemplateInput) -> AppResult<PageTemplate> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    page_template_service::create_template_from_page(&conn, &workspace_id, &page_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_page_template(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    page_template_service::delete_template(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
