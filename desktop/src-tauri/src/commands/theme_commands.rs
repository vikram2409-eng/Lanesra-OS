//! UX/UI Modernization, Phase A (issue #191): Theme Studio Tauri commands.

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::workspace_theme::{ThemeContrastIssue, ThemeTokens, WorkspaceTheme, WorkspaceThemeInput};
use lanesra_core::services::theme_service;

#[tauri::command]
pub fn list_theme_presets(_state: State<AppState>) -> Vec<(String, String, String, ThemeTokens)> {
    theme_service::built_in_presets()
        .into_iter()
        .map(|(key, name, blurb, tokens)| (key.to_string(), name.to_string(), blurb.to_string(), tokens))
        .collect()
}

#[tauri::command]
pub fn get_published_theme(state: State<AppState>) -> AppResult<Option<WorkspaceTheme>> {
    let conn = state.conn.lock().unwrap();
    theme_service::get_published(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn list_theme_versions(state: State<AppState>) -> AppResult<Vec<WorkspaceTheme>> {
    let conn = state.conn.lock().unwrap();
    theme_service::list_versions(&conn, &require_workspace_id(&conn)?)
}

#[tauri::command]
pub fn validate_theme_tokens(_state: State<AppState>, tokens: ThemeTokens) -> Vec<ThemeContrastIssue> {
    theme_service::validate_tokens(&tokens)
}

#[tauri::command]
pub fn save_theme_draft(state: State<AppState>, existing_id: Option<String>, input: WorkspaceThemeInput) -> AppResult<WorkspaceTheme> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    theme_service::save_draft(&conn, &workspace_id, existing_id.as_deref(), &input, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn publish_theme(state: State<AppState>, id: String) -> AppResult<WorkspaceTheme> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    theme_service::publish(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn rollback_theme(state: State<AppState>, from_version: i64) -> AppResult<WorkspaceTheme> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    theme_service::rollback_to_version(&conn, &workspace_id, from_version, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn delete_theme_draft(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    theme_service::delete_draft(&conn, &id, &workspace_id, current_actor(&state).as_deref())
}
