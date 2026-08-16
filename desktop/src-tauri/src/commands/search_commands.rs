use tauri::State;

use crate::commands::require_workspace_id;
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::services::search_service::{self, SearchResult};

/// Global search (spec §5.3/§9.3): any authenticated user can search,
/// matching every other read command's access model - nothing here is
/// admin-only.
#[tauri::command]
pub fn global_search(state: State<AppState>, query: String) -> AppResult<Vec<SearchResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    search_service::global_search(&conn, &workspace_id, &query)
}
