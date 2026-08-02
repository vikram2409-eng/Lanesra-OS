use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use lanesra_core::domain::AppResult;
use lanesra_core::models::user::{NewUser, PasswordChange, User, UserUpdate};
use lanesra_core::services::user_service;
use crate::state::AppState;

#[tauri::command]
pub fn list_users(state: State<AppState>) -> AppResult<Vec<User>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    user_service::list(&conn, &workspace_id)
}

#[tauri::command]
pub fn create_user(state: State<AppState>, input: NewUser) -> AppResult<User> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    user_service::create(&conn, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn update_user(state: State<AppState>, id: String, input: UserUpdate) -> AppResult<User> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    user_service::update(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn set_user_password(state: State<AppState>, id: String, input: PasswordChange) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    user_service::set_password(&conn, &id, &workspace_id, &input, current_actor(&state).as_deref())
}
