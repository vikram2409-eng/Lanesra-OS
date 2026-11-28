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
use lanesra_core::models::lane_run_note::{LaneRunAttachment, LaneRunNote};
use lanesra_core::models::process_lane::{ProcessLane, ProcessLaneVersion};
use lanesra_core::services::process_lane_blueprint::{LaneBlueprint, LaneBlueprintSummary};
use lanesra_core::services::process_lane_service::{
    LaneBoard, LaneBoardItem, LaneRunList, LaneRunView, LaneTiming, MySteps,
};
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

/// WP-0714 (REQ-LANE-021 AC-2): the publish report for a lane - one entry
/// per P-01..P-16 - with no side effects. The frontend renders the report
/// before offering publish, and `publish_lane` itself refuses on any REFUSE
/// entry.
#[tauri::command]
pub fn check_process_lane_publish(
    state: State<AppState>,
    lane_key: String,
) -> AppResult<lanesra_core::services::process_lane_publish::PublishReport> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_publish::check_publish(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}

/// WP-0714 (REQ-LANE-021 AC-1): walks the lane in definition order against
/// the anchor record with the given fixtures and reports what would
/// happen - stages visited, steps executed/skipped, checks passed/failed,
/// branches taken, writes simulated, zero-quantity flags, unevaluated
/// checks, end outcome vs expected. Posts nothing and changes nothing.
#[tauri::command]
pub fn dry_run_process_lane(
    state: State<AppState>,
    lane_key: String,
    anchor_record_id: String,
    fixtures: lanesra_core::services::process_lane_publish::DryRunFixtures,
    expected_outcome: Option<String>,
) -> AppResult<lanesra_core::services::process_lane_publish::DryRunReport> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_publish::dry_run_lane(
        &conn,
        &workspace_id,
        &lane_key,
        &anchor_record_id,
        &fixtures,
        expected_outcome.as_deref(),
        current_actor(&state).as_deref(),
    )
}

/// WP-0714 (REQ-LANE-021 §4): the cases stored against a lane (all
/// versions, or one version).
#[tauri::command]
pub fn list_lane_cases(
    state: State<AppState>,
    lane_key: String,
    version: Option<i64>,
) -> AppResult<Vec<lanesra_core::services::process_lane_publish::LaneCaseRecord>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_publish::list_lane_cases(
        &conn,
        &workspace_id,
        &lane_key,
        version,
        current_actor(&state).as_deref(),
    )
}

/// WP-0714 (REQ-LANE-021 §4): stores a case (JSON) with the lane version
/// it tests. The case names its lane and version; both must match.
#[tauri::command]
pub fn store_lane_case(
    state: State<AppState>,
    lane_key: String,
    version: i64,
    case_json: String,
) -> AppResult<lanesra_core::services::process_lane_publish::LaneCaseRecord> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_publish::store_lane_case(
        &conn,
        &workspace_id,
        &lane_key,
        version,
        &case_json,
        current_actor(&state).as_deref(),
    )
}

/// WP-0714 (REQ-LANE-021 §4): runs one lane case for real against the
/// seeded workspace, in a savepoint that always rolls back - the case
/// leaves no trace. Genuinely async, like starting a run.
#[tauri::command]
pub async fn run_lane_case(
    state: State<'_, AppState>,
    case_json: String,
) -> AppResult<lanesra_core::services::process_lane_publish::LaneCaseResult> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        lanesra_core::services::process_lane_publish::run_lane_case(
            &conn,
            &workspace_id,
            &master_key,
            &case_json,
            actor.as_deref(),
        )
        .await
    })
    .await
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

/// WP-0723: revise an existing lane from a plain-language change
/// description. The result is always a draft version of the same lane
/// (see `assistant_edit_lane_service`); publishing stays a separate
/// admin action, so this command has no publish path.
#[tauri::command]
pub async fn edit_lane_from_description(
    state: State<'_, AppState>,
    lane_key: String,
    description: String,
) -> AppResult<ProcessLaneVersion> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        lanesra_core::services::assistant_edit_lane_service::edit_lane(
            &conn,
            &workspace_id,
            &master_key,
            actor.as_deref(),
            &lane_key,
            &description,
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

/// WP-0707 (REQ-LANE-015 AC-3): every lane run on one record, across lanes,
/// newest first - what Object 360 lists. The record is identified by the
/// same `{object_key, record_id}` pair the demo's `laneRunsForRecord` and
/// the core's one-active-run guard use.
#[tauri::command]
pub fn list_lane_runs_for_record(
    state: State<AppState>,
    object_key: String,
    record_id: String,
) -> AppResult<LaneRunList> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::list_lane_runs_for_record(&conn, &workspace_id, &object_key, &record_id)
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

/// WP-0710: completes one item of a repeating step (`repeat_over`) on a
/// `waiting_person` lane run - validates and writes just that item's values
/// (to its child record or a per-child run-data field) and, when it is the
/// last open item, advances the run past the loop. Two people can submit
/// different items at once without overwriting each other. Async because
/// resuming the run may reach an Agent node.
#[tauri::command]
pub async fn submit_process_lane_step_item(
    state: State<'_, AppState>,
    run_id: String,
    item_id: String,
    values: Map<String, Value>,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::submit_person_step_item(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            &item_id,
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

/// WP-0706: resolves the approval a lane run waits on, routing it down the
/// step's approved/rejected branch.
#[tauri::command]
pub async fn resolve_process_lane_approval(
    state: State<'_, AppState>,
    run_id: String,
    approve: bool,
    notes: Option<String>,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::resolve_lane_approval(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            approve,
            notes.as_deref(),
            actor.as_deref(),
        )
        .await
    })
    .await
}

/// WP-0706: cancels a lane run; posted writes stay, and the cancellation is
/// recorded in the audit trail.
#[tauri::command]
pub async fn cancel_process_lane_run(
    state: State<'_, AppState>,
    run_id: String,
    reason: String,
    #[allow(non_snake_case)] cancelChild: bool,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::cancel_lane_run(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            &reason,
            cancelChild,
            actor.as_deref(),
        )
        .await
    })
    .await
}

/// WP-0718 (REQ-LANE-024 AC-1/AC-2): saves a builder lane - a new lane
/// when `lane_key` is None (the import path), a new draft version of the
/// existing lane otherwise (the version path). The definition JSON is the
/// builder's own output; the service validates it either way.
#[tauri::command]
pub fn save_builder_lane(
    state: State<AppState>,
    lane_key: Option<String>,
    definition_json: String,
) -> AppResult<lanesra_core::services::process_lane_builder::BuilderSaveResult> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_builder::save_builder_lane(
        &conn,
        &workspace_id,
        lane_key.as_deref(),
        &definition_json,
        current_actor(&state).as_deref(),
    )
}

/// WP-0718 (REQ-LANE-024 AC-2): the publish report for an unsaved draft -
/// the same P-01..P-16 checks `check_process_lane_publish` runs on a
/// stored lane, evaluated on the definition the builder holds. No side
/// effects.
#[tauri::command]
pub fn builder_check_lane(
    state: State<AppState>,
    definition_json: String,
) -> AppResult<lanesra_core::services::process_lane_publish::PublishReport> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_builder::builder_check(
        &conn,
        &workspace_id,
        &definition_json,
        current_actor(&state).as_deref(),
    )
}

/// WP-0718 (REQ-LANE-024 AC-2): dry-runs an unsaved draft against an
/// anchor record - the same walk `dry_run_process_lane` runs on a stored
/// lane. Posts nothing and changes nothing.
#[tauri::command]
pub fn builder_dry_run_lane(
    state: State<AppState>,
    definition_json: String,
    anchor_record_id: String,
    fixtures: lanesra_core::services::process_lane_publish::DryRunFixtures,
    expected_outcome: Option<String>,
) -> AppResult<lanesra_core::services::process_lane_publish::DryRunReport> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_builder::builder_dry_run(
        &conn,
        &workspace_id,
        &definition_json,
        &anchor_record_id,
        &fixtures,
        expected_outcome.as_deref(),
        current_actor(&state).as_deref(),
    )
}

/// WP-0718 (REQ-LANE-024 AC-4, design §4): the per-step reads/writes map
/// and the lane-level run-data plumbing table for an unsaved draft. Pure
/// function over the definition; no side effects.
#[tauri::command]
pub fn builder_reads_writes(
    state: State<AppState>,
    definition_json: String,
) -> AppResult<lanesra_core::services::process_lane_builder::ReadsWritesReport> {
    let _ = &state;
    lanesra_core::services::process_lane_builder::builder_reads_writes(&definition_json)
}

/// WP-0718 (REQ-LANE-024 AC-4, design §6): the Action-Type picker
/// catalogue for an object (or every object when `object_key` is None):
/// the adopted catalog entries plus the provisional ids the stored
/// lanes use, each marked provisional.
#[tauri::command]
pub fn builder_action_types(
    state: State<AppState>,
    object_key: Option<String>,
) -> AppResult<Vec<lanesra_core::services::process_lane_builder::BuilderActionType>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_builder::builder_action_types(
        &conn,
        &workspace_id,
        object_key.as_deref(),
        current_actor(&state).as_deref(),
    )
}

/// WP-0718 follow-up (builder gaps fix, gap 8): adopts one warehouse
/// DOMAIN Action Type (`<object>.set_status`) into the adopted catalog,
/// so a lane step using it can pass P-12. Admin-only; `run_data.set`
/// and cross-object writes are refused by the service, never adopted.
#[tauri::command]
pub fn adopt_builder_action_type(
    state: State<AppState>,
    name: String,
    target_object_type: String,
) -> AppResult<lanesra_core::models::action_type::ActionTypeDefinition> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    lanesra_core::services::process_lane_builder::adopt_builder_action_type(
        &conn,
        &workspace_id,
        &name,
        &target_object_type,
        current_actor(&state).as_deref(),
    )
}

/// WP-0706: hands the current form step of a lane run to a different role.
#[tauri::command]
pub fn handoff_process_lane_step(
    state: State<'_, AppState>,
    run_id: String,
    to_role: String,
    reason: String,
) -> AppResult<GraphRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let actor = current_actor(&state);
    process_lane_service::handoff_step(
        &conn,
        &workspace_id,
        &run_id,
        &to_role,
        &reason,
        actor.as_deref(),
    )
}

/// WP-0720 (REQ-LANE-009, AC-2): pauses a run waiting on a person, parking
/// it on the same step. Sync - a pure status transition, no master key.
#[tauri::command]
pub fn pause_process_lane_run(
    state: State<'_, AppState>,
    run_id: String,
    reason: Option<String>,
) -> AppResult<GraphRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let actor = current_actor(&state);
    process_lane_service::pause_run(
        &conn,
        &workspace_id,
        &run_id,
        reason.as_deref(),
        actor.as_deref(),
    )
}

/// WP-0720 (REQ-LANE-009, AC-2): resumes a paused run back to the step it
/// was paused on. Sync - a pure status transition, no master key.
#[tauri::command]
pub fn resume_process_lane_run(state: State<'_, AppState>, run_id: String) -> AppResult<GraphRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let actor = current_actor(&state);
    process_lane_service::resume_paused_run(&conn, &workspace_id, &run_id, actor.as_deref())
}

/// WP-0720 (REQ-LANE-009, AC-2): reopens an earlier completed step in the
/// run's current stage, so the person can redo it and re-walk forward.
/// Async because resuming the run at the step may reach an Agent node.
#[tauri::command]
pub async fn reopen_process_lane_step(
    state: State<'_, AppState>,
    run_id: String,
    step_id: String,
    reason: Option<String>,
) -> AppResult<GraphRun> {
    let master_key = crate::commands::resolve_master_key(&state)?;
    let db_path = state.db_path.clone();
    let (workspace_id, actor) = {
        let conn = state.conn.lock().unwrap();
        (require_workspace_id(&conn)?, current_actor(&state))
    };
    run_with_own_connection(db_path, move |conn| async move {
        process_lane_service::reopen_step(
            &conn,
            &workspace_id,
            &master_key,
            &run_id,
            &step_id,
            reason.as_deref(),
            actor.as_deref(),
        )
        .await
    })
    .await
}

/// WP-0711 (REQ-LANE-018, AC-1): adds a note to a lane run at any point.
#[tauri::command]
pub fn add_lane_run_note(
    state: State<AppState>,
    run_id: String,
    body: String,
) -> AppResult<LaneRunNote> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::add_lane_run_note(
        &conn,
        &workspace_id,
        &run_id,
        &body,
        current_actor(&state).as_deref(),
    )
}

/// WP-0711 (REQ-LANE-018, AC-1/AC-3): attaches a file to a lane run at any
/// point. The content arrives base64; the bytes land under the data dir
/// (the db file's parent).
#[tauri::command]
pub fn attach_lane_run_file(
    state: State<AppState>,
    run_id: String,
    file_name: String,
    mime_type: Option<String>,
    content_base64: String,
) -> AppResult<LaneRunAttachment> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let data_dir = state
        .db_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| lanesra_core::domain::AppError::Validation("no data directory".into()))?;
    process_lane_service::attach_lane_run_file(
        &conn,
        &workspace_id,
        &data_dir,
        &run_id,
        &file_name,
        mime_type.as_deref(),
        &content_base64,
        current_actor(&state).as_deref(),
    )
}

/// WP-0711 (REQ-LANE-018, AC-3): reads one attachment's bytes through its
/// run, for download. The read is run-scoped in the service.
#[tauri::command]
pub fn read_lane_run_attachment(
    state: State<AppState>,
    run_id: String,
    attachment_id: String,
) -> AppResult<(LaneRunAttachment, Vec<u8>)> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let data_dir = state
        .db_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| lanesra_core::domain::AppError::Validation("no data directory".into()))?;
    process_lane_service::read_lane_run_attachment(
        &conn,
        &workspace_id,
        &data_dir,
        &run_id,
        &attachment_id,
        current_actor(&state).as_deref(),
    )
}

/// WP-0711 (REQ-LANE-018, AC-2): reassigns the current step of a lane run
/// to one person who holds the step's role.
#[tauri::command]
pub fn reassign_lane_run_step(
    state: State<'_, AppState>,
    run_id: String,
    to_user_id: String,
    reason: String,
) -> AppResult<GraphRun> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    let actor = current_actor(&state);
    process_lane_service::reassign_lane_step(
        &conn,
        &workspace_id,
        &run_id,
        &to_user_id,
        &reason,
        actor.as_deref(),
    )
}

/// The board of a lane (WP-0713 AC-1, REQ-LANE-019): its active runs in
/// their stages, only records the signed-in person may read.
#[tauri::command]
pub fn list_lane_board(state: State<'_, AppState>, lane_key: String) -> AppResult<LaneBoard> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::list_lane_board(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}

/// The signed-in person's lane steps and approvals together, oldest first
/// (WP-0713 AC-2, REQ-LANE-019).
#[tauri::command]
pub fn list_my_lane_steps(state: State<'_, AppState>) -> AppResult<MySteps> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::list_my_steps(&conn, &workspace_id, current_actor(&state).as_deref())
}

/// A lane's timing figures (WP-0713 AC-3, REQ-LANE-019): time per stage,
/// completion rate and rework, from the recorded timestamps.
#[tauri::command]
pub fn get_lane_timing(state: State<'_, AppState>, lane_key: String) -> AppResult<LaneTiming> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::lane_timing(&conn, &workspace_id, &lane_key)
}

/// The ageing view of a lane (WP-0713 AC-4, REQ-LANE-019): its active
/// runs, longest in their current stage first, only records the
/// signed-in person may read.
#[tauri::command]
pub fn list_lane_ageing(
    state: State<'_, AppState>,
    lane_key: String,
) -> AppResult<Vec<LaneBoardItem>> {
    let conn = state.conn.lock().unwrap();
    let workspace_id = require_workspace_id(&conn)?;
    process_lane_service::list_lane_ageing(
        &conn,
        &workspace_id,
        &lane_key,
        current_actor(&state).as_deref(),
    )
}
