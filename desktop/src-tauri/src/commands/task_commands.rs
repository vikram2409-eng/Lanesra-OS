use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::task::{Task, TaskInput};
use lanesra_core::services::task_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> AppResult<Vec<Task>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    task_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn list_tasks_by_related(state: State<AppState>, related_type: String, related_id: String) -> AppResult<Vec<Task>> {
    let conn = state.conn.lock().unwrap();
    task_service::list_by_related(&conn, &related_type, &related_id)
}

#[tauri::command]
pub fn get_task(state: State<AppState>, id: String) -> AppResult<Task> {
    let conn = state.conn.lock().unwrap();
    task_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_task(state: State<AppState>, input: TaskInput) -> AppResult<Task> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    task_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_task(state: State<AppState>, id: String, input: TaskInput) -> AppResult<Task> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    task_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn archive_task(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    task_service::archive(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
