use tauri::State;

use crate::domain::AppResult;
use crate::models::user::User;
use crate::models::workspace::{Workspace, WorkspaceSetup};
use crate::repositories::workspace_repo;
use crate::services::workspace_service;
use crate::state::AppState;

#[tauri::command]
pub fn workspace_status(state: State<AppState>) -> AppResult<Option<Workspace>> {
    let conn = state.conn.lock().unwrap();
    Ok(workspace_repo::get_current(&conn)?)
}

#[tauri::command]
pub fn first_run_setup(
    state: State<AppState>,
    setup: WorkspaceSetup,
) -> AppResult<(Workspace, User)> {
    let conn = state.conn.lock().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup)?;
    *state.session_user_id.lock().unwrap() = Some(admin.id.clone());
    Ok((workspace, admin))
}
