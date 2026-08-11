use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::{AppError, AppResult};
use lanesra_core::models::workflow::Notification;
use lanesra_core::repositories::notification_repo;

#[tauri::command]
pub fn list_notifications(state: State<AppState>, unread_only: bool) -> AppResult<Vec<Notification>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    Ok(notification_repo::list_for_user(&conn, &workspace_id, &user_id, unread_only)?)
}

#[tauri::command]
pub fn mark_notification_read(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    Ok(notification_repo::mark_read(&conn, &id)?)
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<AppState>) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let user_id = current_actor(&state).ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    Ok(notification_repo::mark_all_read(&conn, &workspace_id, &user_id)?)
}
