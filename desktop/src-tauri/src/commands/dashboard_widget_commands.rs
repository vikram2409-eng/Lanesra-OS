use tauri::State;

use crate::commands::require_workspace_id;
use lanesra_core::domain::AppResult;
use lanesra_core::services::dashboard_widget_service::{self, RecordListRow};
use lanesra_core::services::saved_view_service;
use crate::state::AppState;

/// Backs a record-list dashboard widget (Dashboard customization Phase 3) -
/// see `dashboard_widget_service::run`'s own doc comment for what "recent"
/// vs "due_soon" mean per entity type. `saved_view_id`, when the widget's
/// config names one, narrows the rows to that view's saved filters - see
/// `dashboard_widget_service::matches_filters`'s own doc comment for the
/// (deliberately simpler than list-screen) matching it does.
#[tauri::command]
pub fn run_dashboard_record_list(
    state: State<AppState>,
    entity_type: String,
    mode: String,
    limit: i64,
    saved_view_id: Option<String>,
) -> AppResult<Vec<RecordListRow>> {
    let conn = state.conn.lock().unwrap();
    let filters = match &saved_view_id {
        Some(id) => saved_view_service::get(&conn, id)?.map(|v| v.filters).unwrap_or_default(),
        None => Default::default(),
    };
    dashboard_widget_service::run(&conn, &require_workspace_id(&conn)?, &entity_type, &mode, limit, &filters)
}
