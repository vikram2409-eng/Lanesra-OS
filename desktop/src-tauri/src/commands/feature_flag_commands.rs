//! Workspace feature flags (WP-0010, ADR-0012) for the desktop edition.

use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::feature_flag::{Edition, FeatureFlagList, FeatureFlagOverrideInput};
use lanesra_core::services::feature_flag_service;

#[tauri::command]
pub fn list_feature_flags(state: State<AppState>) -> AppResult<FeatureFlagList> {
    let conn = state.conn.lock().unwrap();
    feature_flag_service::list(
        &conn,
        &require_workspace_id(&conn)?,
        Edition::Desktop,
        feature_flag_service::developer_mode(),
    )
}

#[tauri::command]
pub fn set_feature_flag_override(
    state: State<AppState>,
    input: FeatureFlagOverrideInput,
) -> AppResult<FeatureFlagList> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    feature_flag_service::set_override(
        &conn,
        &workspace_id,
        Edition::Desktop,
        &input,
        current_actor(&state).as_deref(),
        feature_flag_service::developer_mode(),
    )
}
