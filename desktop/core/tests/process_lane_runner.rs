//! Phase 7, WP-0707: the lane runner's core surface (ADR-0023, ADR-0024).
//!
//! The runner screen (stage bar, step list, writes summary, inbox state)
//! and Object 360 read this from the core; the engine itself
//! (`get_run_view` / `submit_person_step` / `skip_step` / `start_lane_run`)
//! is WP-0704/0705/0706 and is used as-is. These tests cover:
//!   AC-1  `get_run_view` lists every step in definition order with its
//!         progress (completed / current / locked / skipped / waiting) and
//!         the run's posted writes as human-readable summaries
//!   AC-2  a run waiting on an approval reports its inbox state
//!         (`waiting`), so the runner can show "continue in the approvals
//!         inbox" instead of the step's work area
//!   AC-3  `list_lane_runs_for_record` lists every run on a record across
//!         lanes, and every submit/skip is recorded in the audit trail with
//!         actor and time
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    action_type_service, custom_field_service, custom_object_service, custom_record_service,
    process_lane_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Runner Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "secret-password".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [83u8; 32]
}

/// The `widget` object with a `tracking` text field and one record.
fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    custom_object_service::create_with_key(
        conn,
        ws,
        "widget",
        &CustomObjectDefinitionInput {
            singular_label: "Widget".into(),
            plural_label: "Widgets".into(),
            icon: "▣".into(),
            prefix: "WID".into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
    custom_field_service::create_definition_with_key(
        conn,
        ws,
        "tracking",
        &CustomFieldDefinitionInput {
            entity_type: "widget".into(),
            label: "Tracking".into(),
            field_type: "text".into(),
            options: vec![],
            required: false,
            show_in_list: false,
            sort_order: 0,
            min_value: None,
            max_value: None,
            max_length: None,
            regex_pattern: None,
            is_searchable: false,
            is_filterable: false,
            is_reportable: false,
            default_value: None,
            is_unique: false,
            help_text: None,
            placeholder: None,
            is_hidden_by_default: false,
        },
        Some(admin),
    )
    .unwrap();
    custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: "widget".into(),
            primary_name: "Widget one".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(admin),
    )
    .unwrap()
    .id
}

/// Defines the `Set Tracking` Action Type on `widget`: a `set_fields` effect
/// that writes its `val` parameter into the `tracking` field.
fn create_set_tracking_action(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    action_type_service::create(
        conn,
        ws,
        &ActionTypeDefinitionInput {
            name: "Set Tracking".into(),
            description: String::new(),
            target_object_type: "widget".into(),
            required_capability: "update".into(),
            parameters: vec![ActionParameterSpec {
                key: "val".into(),
                label: "Value".into(),
                field_type: "text".into(),
                required: true,
                default_value: None,
            }],
            effect: "set_fields".into(),
            effect_config_json: json!({"field_sources": {"tracking": "val"}}).to_string(),
        },
        Some(admin),
    )
    .unwrap();
}

fn publish_lane(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    id: &str,
    name: &str,
    stages: Value,
) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": name, "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true},
                {"key": "note", "type": "string", "required": false}
            ]},
            "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, id, Some(admin)).unwrap();
}

/// The runner reference lane: a required form step, an optional form step,
/// then a record step that writes. Two stages, three steps, definition
/// order is capture -> post.
fn runner_stages() -> Value {
    json!([
        {"id": "capture", "name": "Capture", "steps": [
            {"id": "enter-details", "name": "Enter details", "kind": "form", "role": "r",
             "required": true,
             "fields": [
                {"key": "carrier", "label": "Carrier", "type": "string",
                 "binds_to": "run_data.carrier", "required": true},
                {"key": "tracking", "label": "Tracking", "type": "string",
                 "binds_to": "record.tracking", "required": false}
             ]},
            {"id": "add-note", "name": "Add note", "kind": "form", "role": "r",
             "required": false,
             "fields": [
                {"key": "note", "label": "Note", "type": "string",
                 "binds_to": "run_data.note", "required": false}
             ]}
        ]},
        {"id": "post", "name": "Post", "steps": [
            {"id": "post-tracking", "name": "Post tracking", "kind": "record", "role": "r",
             "required": true,
             "writes": [
                {"action_type": "Set Tracking", "inputs": {"val": "$run_data.carrier"}}
             ]}
        ]}
    ])
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect()
}

async fn start(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    lane: &str,
    record: &str,
) -> lanesra_core::models::graph_run::GraphRun {
    process_lane_service::start_lane_run(conn, ws, &master_key(), lane, record, Some(admin))
        .await
        .unwrap()
}

fn step_statuses(view: &process_lane_service::LaneRunView) -> Vec<(&str, &str)> {
    view.steps
        .iter()
        .map(|s| (s.step_id.as_str(), s.status.as_str()))
        .collect()
}

// --------------------------------------------------------------------------
// AC-1: the view lists every step in definition order with its progress.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-014 (WP-0707 AC-1)
#[tokio::test]
async fn run_view_lists_every_step_with_progress_in_definition_order() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    create_set_tracking_action(&conn, &ws, &admin);
    publish_lane(
        &conn,
        &ws,
        &admin,
        "lane-runner",
        "Runner Lane",
        runner_stages(),
    );

    let run = start(&conn, &ws, &admin, "lane-runner", &record).await;
    assert_eq!(run.status, "waiting_person");

    // Before any step is done: the first step is current, the rest locked,
    // none has times, nothing has been written, nothing waits on approval.
    let view = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&admin)).unwrap();
    assert_eq!(
        step_statuses(&view),
        vec![
            ("enter-details", "current"),
            ("add-note", "locked"),
            ("post-tracking", "locked"),
        ]
    );
    assert_eq!(view.steps[0].stage_id, "capture");
    assert_eq!(view.steps[2].stage_id, "post");
    // The current step's row already has a start time (the run reached it)
    // but no finish time; the locked steps have no row and no times.
    assert!(view.steps[0].started_at.is_some());
    assert!(view.steps[0].finished_at.is_none());
    assert!(view.steps[1].started_at.is_none() && view.steps[1].finished_at.is_none());
    assert!(view.steps[2].started_at.is_none() && view.steps[2].finished_at.is_none());
    assert!(view.writes.is_empty());
    assert!(view.waiting.is_none());

    // Submit the first step: it completes with a finish time at or after its
    // start, and the optional second step becomes current.
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME"), ("tracking", "TRK-1")]),
        Some(&admin),
    )
    .await
    .unwrap();
    let view = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&admin)).unwrap();
    assert_eq!(
        step_statuses(&view),
        vec![
            ("enter-details", "completed"),
            ("add-note", "current"),
            ("post-tracking", "locked"),
        ]
    );
    let first = &view.steps[0];
    let started = first.started_at.as_ref().expect("a start time");
    let finished = first.finished_at.as_ref().expect("a finish time");
    assert!(finished >= started);
}

// --------------------------------------------------------------------------
// AC-1 + AC-3: a skip marks the step skipped, and the record step's write
// appears in the view's write summaries once the run completes.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-014 (WP-0707 AC-1), REQ-LANE-015 (WP-0707 AC-3)
#[tokio::test]
async fn skipped_step_and_posted_write_show_in_the_run_view() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    create_set_tracking_action(&conn, &ws, &admin);
    publish_lane(
        &conn,
        &ws,
        &admin,
        "lane-runner",
        "Runner Lane",
        runner_stages(),
    );

    let run = start(&conn, &ws, &admin, "lane-runner", &record).await;
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME")]),
        Some(&admin),
    )
    .await
    .unwrap();
    let done = process_lane_service::skip_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        "nothing to add",
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(
        custom_field_service::get_entity_values(&conn, &record)
            .unwrap()
            .get("tracking")
            .map(String::as_str),
        Some("ACME"),
        "the record step posted its write on the way out"
    );

    let view = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&admin)).unwrap();
    assert_eq!(
        step_statuses(&view),
        vec![
            ("enter-details", "completed"),
            ("add-note", "skipped"),
            ("post-tracking", "completed"),
        ]
    );
    assert!(view.steps[1].finished_at.is_some());
    assert_eq!(view.writes.len(), 1);
    assert!(
        view.writes[0].contains("Post tracking") && view.writes[0].contains("Set Tracking"),
        "the write summary names the step and what it applied: {}",
        view.writes[0]
    );
    assert!(view.waiting.is_none());
}

// --------------------------------------------------------------------------
// AC-2: a run waiting on an approval reports its inbox state.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-014 (WP-0707 AC-2)
#[tokio::test]
async fn run_waiting_on_an_approval_reports_its_inbox_state() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    let stages = json!([
        {"id": "decide", "name": "Decide", "steps": [{
            "id": "decide", "name": "Decide", "kind": "approval", "role": "r",
            "required": true,
            "writes": [{"action_type": "noop", "inputs": {}}],
            "branches": [
                {"on": "approved", "target": {"stage": "post", "step": "approved-path"}},
                {"on": "rejected", "target": {"stage": "post", "step": "rejected-path"}}
            ]
        }]},
        {"id": "post", "name": "Post", "steps": [
            {"id": "approved-path", "name": "Approved path", "kind": "form", "role": "r",
             "required": true, "fields": [], "ends_run": {"outcome": "done"}},
            {"id": "rejected-path", "name": "Rejected path", "kind": "form", "role": "r",
             "required": true, "fields": [], "ends_run": {"outcome": "held"}}
        ]}
    ]);
    publish_lane(&conn, &ws, &admin, "lane-approval", "Approval Lane", stages);

    let run = start(&conn, &ws, &admin, "lane-approval", &record).await;
    assert_eq!(run.status, "waiting_approval");

    let view = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&admin)).unwrap();
    // The runner must not render the approval step's work area: the view
    // says so plainly, with the step it waits on.
    let waiting = view.waiting.as_ref().expect("an inbox state");
    assert_eq!(waiting.step_id, "decide");
    assert_eq!(waiting.step_name, "Decide");
    assert_eq!(waiting.kind, "approval");
    // The approval step itself is marked waiting, not current; the branch
    // targets are still locked.
    assert_eq!(
        step_statuses(&view),
        vec![
            ("decide", "waiting"),
            ("approved-path", "locked"),
            ("rejected-path", "locked"),
        ]
    );
    assert!(view.current.is_none());
}

// --------------------------------------------------------------------------
// AC-3: list-runs-for-record lists every run on the record across lanes.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-015 (WP-0707 AC-3), REQ-LANE-014 (WP-0707 AC-1)
#[tokio::test]
async fn list_runs_for_record_lists_every_run_on_the_record_across_lanes() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    create_set_tracking_action(&conn, &ws, &admin);
    let other_record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput {
            object_key: "widget".into(),
            primary_name: "Widget two".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(&admin),
    )
    .unwrap()
    .id;
    publish_lane(
        &conn,
        &ws,
        &admin,
        "lane-one",
        "First Lane",
        runner_stages(),
    );
    // A second, form-only lane on the same object: one active run per lane
    // per record, so both lanes may have a run on the same record.
    let form_only = json!([
        {"id": "capture", "name": "Capture", "steps": [
            {"id": "enter-details", "name": "Enter details", "kind": "form", "role": "r",
             "required": true, "fields": [
                {"key": "carrier", "label": "Carrier", "type": "string",
                 "binds_to": "run_data.carrier", "required": true}
             ]}
        ]}
    ]);
    publish_lane(&conn, &ws, &admin, "lane-two", "Second Lane", form_only);

    // Lane one runs to completion on the record; lane two waits; a run of
    // lane one on a different record must not leak into the list.
    let run_one = start(&conn, &ws, &admin, "lane-one", &record).await;
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_one.id,
        &values(&[("carrier", "ACME")]),
        Some(&admin),
    )
    .await
    .unwrap();
    process_lane_service::skip_step(
        &conn,
        &ws,
        &master_key(),
        &run_one.id,
        "nothing to add",
        Some(&admin),
    )
    .await
    .unwrap();
    let run_two = start(&conn, &ws, &admin, "lane-two", &record).await;
    let _other = start(&conn, &ws, &admin, "lane-one", &other_record).await;

    let list =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &record).unwrap();
    assert_eq!(list.runs.len(), 2, "both lanes' runs, and no one else's");

    // Newest first.
    let (first, second) = (&list.runs[0], &list.runs[1]);
    assert_eq!(first.lane_id, "lane-two");
    assert_eq!(first.lane_name, "Second Lane");
    assert_eq!(first.status, "waiting_person");
    assert_eq!(first.current_stage_id.as_deref(), Some("capture"));
    assert_eq!(first.current_stage_name.as_deref(), Some("Capture"));
    assert_eq!(first.current_step_id.as_deref(), Some("enter-details"));
    assert_eq!(first.current_step_name.as_deref(), Some("Enter details"));
    assert!(first.finished_at.is_none());
    assert!(!first.started_at.is_empty());

    assert_eq!(second.run_id, run_one.id);
    assert_eq!(second.lane_id, "lane-one");
    assert_eq!(second.lane_name, "First Lane");
    assert_eq!(second.status, "completed");
    assert!(second.finished_at.is_some());
    // A finished run has no current step.
    assert!(second.current_step_id.is_none());
    assert!(second.current_stage_id.is_none());

    // The other record's own list holds only its run.
    let other_list =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &other_record)
            .unwrap();
    assert_eq!(other_list.runs.len(), 1);
    assert_eq!(other_list.runs[0].lane_id, "lane-one");

    // A record with no runs lists none.
    let empty =
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", "no-such").unwrap();
    assert!(empty.runs.is_empty());
    let _ = run_two;
}

// --------------------------------------------------------------------------
// AC-3: every submit and skip is recorded in the audit trail with actor
// and time.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-015 (WP-0707 AC-3)
#[tokio::test]
async fn submit_and_skip_are_audited_with_actor_and_time() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    create_set_tracking_action(&conn, &ws, &admin);
    publish_lane(
        &conn,
        &ws,
        &admin,
        "lane-runner",
        "Runner Lane",
        runner_stages(),
    );

    let run = start(&conn, &ws, &admin, "lane-runner", &record).await;
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME")]),
        Some(&admin),
    )
    .await
    .unwrap();
    process_lane_service::skip_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        "nothing to add",
        Some(&admin),
    )
    .await
    .unwrap();

    let events =
        lanesra_core::repositories::audit_repo::list_for_entity(&conn, "widget", &record).unwrap();
    let submitted = events
        .iter()
        .find(|e| e.event_type == "lane_step_submitted")
        .expect("a submit audit event");
    assert_eq!(submitted.user_id.as_deref(), Some(admin.as_str()));
    assert!(!submitted.occurred_at.is_empty());
    assert!(submitted.summary.contains("Enter details"));
    let details: Value =
        serde_json::from_str(submitted.details_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(details["step_id"], json!("enter-details"));
    assert_eq!(details["run_id"], json!(run.id));
    assert_eq!(details["outcome"], json!("submitted"));

    let skipped = events
        .iter()
        .find(|e| e.event_type == "lane_step_skipped")
        .expect("a skip audit event");
    assert_eq!(skipped.user_id.as_deref(), Some(admin.as_str()));
    assert!(!skipped.occurred_at.is_empty());
    let details: Value =
        serde_json::from_str(skipped.details_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(details["step_id"], json!("add-note"));
    assert_eq!(details["reason"], json!("nothing to add"));
    assert_eq!(details["outcome"], json!("skipped"));
}
