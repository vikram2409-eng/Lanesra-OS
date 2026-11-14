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
use lanesra_core::services::process_lane_blueprint::{LaneBlueprint, LaneBlueprintSummary};
use lanesra_core::services::process_lane_service::LaneRunView;
use lanesra_core::services::{process_lane_blueprint, process_lane_service};
use serde_json::{Map, Value};

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

/// The blueprint of a lane (WP-0712, REQ-LANE-020): the latest version's
/// definition, read-only, for any authenticated workspace member - a
/// Viewer reads the same blueprint an Administrator does, and it carries
/// no run and no record data.
#[tauri::command]
pub fn get_lane_blueprint(state: State<AppState>, lane_key: String) -> AppResult<LaneBlueprint> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_blueprint::get_lane_blueprint(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}

/// Every lane in the workspace, one summary row each (WP-0712).
#[tauri::command]
pub fn list_lane_blueprints(state: State<AppState>) -> AppResult<Vec<LaneBlueprintSummary>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_blueprint::list_lane_blueprints(
        &conn,
        &workspace_id,
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

/// WP-0715: describe a process - the Admin Assistant drafts a lane from a
/// plain-language description. The result is always a draft (see
/// `assistant_draft_lane_service`); publishing stays a separate admin
/// action, so this command has no publish path.
#[tauri::command]
pub async fn draft_lane_from_description(
    state: State<'_, AppState>,
    description: String,
    object_key: String,
) -> AppResult<ProcessLane> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        lanesra_core::services::assistant_draft_lane_service::draft_lane(
            &conn,
            &workspace_id,
            &master_key,
            actor.as_deref(),
            &description,
            &object_key,
        )
        .await
    })
    .await
}

/// WP-0704: the current state of a lane run for a viewer - status, run data,
/// and the current human step with its fields (record-bound values withheld
/// when the viewer cannot read the anchor record).
#[tauri::command]
pub fn get_process_lane_run_view(state: State<AppState>, run_id: String) -> AppResult<LaneRunView> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let actor = current_actor(&state);
    process_lane_service::get_run_view(&conn, &workspace_id, &run_id, actor.as_deref())
}

/// WP-0704: submits the current human step of a `waiting_person` lane run -
/// validates and stores the entered values and advances the run. Async
/// because resuming the run may reach an Agent node.
#[tauri::command]
pub async fn submit_process_lane_step(
    state: State<'_, AppState>,
    run_id: String,
    values: Map<String, Value>,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::submit_person_step(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            &values,
            actor.as_deref(),
        )
        .await
    })
    .await
}

/// WP-0705: skips the current optional step of a lane run, recording the
/// reason. A required step is refused by the service. Async because resuming
/// the run may reach an Agent node.
#[tauri::command]
pub async fn skip_process_lane_step(
    state: State<'_, AppState>,
    run_id: String,
    reason: String,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::skip_step(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            &reason,
            actor.as_deref(),
        )
        .await
    })
    .await
}
