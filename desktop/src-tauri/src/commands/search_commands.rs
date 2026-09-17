use tauri::State;

use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::services::{access_service, search_service};
use lanesra_core::services::search_service::SearchResult;

/// Global search (spec §5.3/§9.3): any authenticated user can search,
/// matching every other read command's access model - nothing here is
/// admin-only, but a hit is only ever returned for a record the searching
/// user's Access Role Read grant actually covers (Access Control v1's
/// list-level companion to the single-record gate on opening it directly).
#[tauri::command]
pub fn global_search(state: State<AppState>, query: String) -> AppResult<Vec<SearchResult>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let mut results = search_service::global_search(&conn, &workspace_id, &query)?;
    let visible = access_service::filter_visible_mixed(&conn, current_actor(&state).as_deref(), results.iter().map(|r| (r.entity_type.as_str(), r.entity_id.as_str())))?;
    results.retain(|r| visible.contains(&(r.entity_type.clone(), r.entity_id.clone())));
    Ok(results)
}
