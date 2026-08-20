use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::publisher::{Publisher, PublisherInput};
use lanesra_core::services::publisher_service;

#[tauri::command]
pub fn list_publishers(state: State<AppState>) -> AppResult<Vec<Publisher>> {
    let conn = state.conn.lock().unwrap();
    publisher_service::list(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn create_publisher(state: State<AppState>, input: PublisherInput) -> AppResult<Publisher> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    publisher_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}
