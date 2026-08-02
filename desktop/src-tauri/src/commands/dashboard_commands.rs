use tauri::State;

use crate::commands::require_workspace_id;
use crate::domain::AppResult;
use crate::models::dashboard::DashboardSummary;
use crate::services::dashboard_service;
use crate::state::AppState;

#[tauri::command]
pub fn dashboard_summary(state: State<AppState>) -> AppResult<DashboardSummary> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    dashboard_service::summary(&conn, &workspace_id)
}
