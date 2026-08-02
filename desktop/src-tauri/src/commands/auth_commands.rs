use tauri::State;

use crate::commands::require_workspace_id;
use lanesra_core::domain::AppResult;
use lanesra_core::models::user::{Credentials, User};
use lanesra_core::services::auth_service;
use crate::state::AppState;

#[tauri::command]
pub fn login(state: State<AppState>, credentials: Credentials) -> AppResult<User> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let user = auth_service::login(&conn, &workspace_id, &credentials)?;
    *state.session_user_id.lock().unwrap() = Some(user.id.clone());
    Ok(user)
}

#[tauri::command]
pub fn logout(state: State<AppState>) -> AppResult<()> {
    *state.session_user_id.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn current_user(state: State<AppState>) -> AppResult<Option<User>> {
    let session_user_id = state.session_user_id.lock().unwrap().clone();
    let Some(user_id) = session_user_id else {
        return Ok(None);
    };
    let conn = state.conn.lock().unwrap();
    auth_service::resolve_user(&conn, &user_id)
}
