use tauri::State;

use crate::commands::current_actor;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::backup::{BackupManifest, BackupPackage};
use lanesra_core::services::backup_service;

#[tauri::command]
pub fn create_backup(state: State<AppState>) -> AppResult<BackupPackage> {
    let conn = state.conn.lock().unwrap();
    backup_service::create_backup(&conn, current_actor(&state).as_deref())
}

/// Unlike every other command, this replaces the connection held in
/// `state.conn` entirely rather than running SQL through it - see
/// `backup_service::restore_from_package` for why.
#[tauri::command]
pub fn restore_backup(state: State<AppState>, package_base64: String) -> AppResult<BackupManifest> {
    let actor = current_actor(&state);
    let mut conn = state.conn.lock().unwrap();
    backup_service::restore_from_package(&mut conn, &state.db_path, &package_base64, actor.as_deref())
}
