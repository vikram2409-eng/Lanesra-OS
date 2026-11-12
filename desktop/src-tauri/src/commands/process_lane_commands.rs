//! AI Agent Platform v2, Phase 7: Tauri commands for
//! `process_lane_service` (WP-0703) - import/publish a lane definition and
//! start a run on an anchor record. The sync commands call the service
//! directly; starting a run is genuinely async, since the graph runtime may
//! call an Agent node through `chat_service`.
//!
//! The Team Workspace server exposes the same operations over HTTP (see
//! `desktop/server/src/dispatch.rs` and `admin_actions.rs`), so desktop
//! and Team Workspace behave the same from the same service code.

use tauri::State;

use crate::commands::integration_commands::run_with_own_connection;
use crate::commands::{current_actor, require_workspace_id};
use crate::state::AppState;
use lanesra_core::domain::AppResult;
use lanesra_core::models::graph_run::GraphRun;
use lanesra_core::models::process_lane::{ProcessLane, ProcessLaneVersion};
use lanesra_core::services::process_lane_service;

#[tauri::command]
pub fn import_process_lane(
    state: State<AppState>,
    definition_json: String,
) -> AppResult<ProcessLane> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::import_lane_definition(
        &conn,
        &workspace_id,
        &definition_json,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn list_process_lanes(state: State<AppState>) -> AppResult<Vec<ProcessLane>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::list_lanes(&conn, &workspace_id, current_actor(&state).as_deref())
}

#[tauri::command]
pub fn get_process_lane(state: State<AppState>, lane_key: String) -> AppResult<ProcessLane> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::get_lane(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn create_process_lane_version(
    state: State<AppState>,
    lane_key: String,
    definition_json: String,
) -> AppResult<ProcessLaneVersion> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::create_version(
        &conn,
        &workspace_id,
        &lane_key,
        &definition_json,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub fn publish_process_lane(
    state: State<AppState>,
    lane_key: String,
) -> AppResult<ProcessLaneVersion> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::publish_lane(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}

#[tauri::command]
pub async fn start_process_lane_run(
    state: State<'_, AppState>,
    lane_key: String,
    anchor_record_id: String,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::start_lane_run(
            &conn,
            &workspace_id,
            &master_key,
            &lane_key,
            &anchor_record_id,
            actor.as_deref(),
        )
        .await
    })
    .await
}
