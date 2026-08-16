use tauri::State;

use crate::commands::require_workspace_id;
use lanesra_core::domain::AppResult;
use lanesra_core::services::dashboard_widget_service::{self, RecordListRow};
use crate::state::AppState;

/// Backs a record-list dashboard widget (Dashboard customization Phase 3) -
/// see `dashboard_widget_service::run`'s own doc comment for what "recent"
/// vs "due_soon" mean per entity type.
#[tauri::command]
pub fn run_dashboard_record_list(
    state: State<AppState>,
    entity_type: String,
    mode: String,
    limit: i64,
) -> AppResult<Vec<RecordListRow>> {
    let conn = state.conn.lock().unwrap();
    dashboard_widget_service::run(&conn, &require_workspace_id(&conn)?, &entity_type, &mode, limit)
}
