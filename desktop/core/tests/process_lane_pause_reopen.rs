//! Phase 7, WP-0720: pause, resume and reopen on the lane runner
//! (REQ-LANE-012). The engine half of the runner's second cut - the run-level
//! actions the runner UI calls:
//!   - pause a run a person is working, and resume it to the same step;
//!   - reopen an earlier completed step in the current stage to redo it;
//!   - both audited; reopening across a posted governed write is refused.
//!
//! Cancel-with-posted-writes is `cancel_lane_run` (WP-0706) and is covered by
//! `process_lane_branches.rs`; these tests cover the three new actions.
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    action_type_service, custom_field_service, custom_object_service, custom_record_service,
    process_lane_service, user_service, workspace_service,
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
        admin_password: "correct horse battery staple".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [91u8; 32]
}

/// The `widget` object with a `tracking` text field and one record. The admin
/// bypasses step-role checks, so no access role needs to exist; the outsider
/// in the permission test holds none and is denied. Returns the record id.
fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    custom_object_service::create_with_key(
        conn,
        ws,
        "widget",
        &CustomObjectDefinitionInput {
            singular_label: "Widget".into(),
            plural_label: "Widgets".into(),
            icon: "W".into(),
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

/// A form step `id` collecting one required run-data field `field`.
fn form_step(id: &str, field: &str) -> Value {
    json!({
        "id": id, "name": format!("Step {id}"), "kind": "form", "role": "r", "required": true,
        "fields": [
            {"key": field, "label": field, "type": "string",
             "binds_to": format!("run_data.{field}"), "required": true}
        ]
    })
}

/// Publishes a one-object lane from a `stages` array, declaring the run-data
/// fields `fields`. Returns the lane key.
fn publish(conn: &rusqlite::Connection, ws: &str, admin: &str, fields: &[&str], stages: Value) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-widget",
            "name": "Widget Lane",
            "object_key": "widget",
            "version": 1,
            "status": "published",
            "run_data": {"fields": fields.iter().map(|f| json!({"key": f, "type": "string", "required": false})).collect::<Vec<_>>()},
            "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-widget", Some(admin)).unwrap();
}

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

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect()
}

fn start(conn: &rusqlite::Connection, ws: &str, admin: &str, record: &str) -> String {
    tokio_block(process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-widget",
        record,
        Some(admin),
    ))
    .unwrap()
    .id
}

/// Resolves a future on a fresh current-thread runtime (the suite is sync-ish).
fn tokio_block<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn current_step_id(conn: &rusqlite::Connection, ws: &str, run_id: &str) -> Option<String> {
    process_lane_service::get_run_view(conn, ws, run_id, None)
        .unwrap()
        .current
        .map(|c| c.step_id)
}

fn event_types(conn: &rusqlite::Connection, ws: &str, run_id: &str) -> Vec<String> {
    process_lane_service::get_run_view(conn, ws, run_id, None)
        .unwrap()
        .events
        .into_iter()
        .map(|e| e.event_type)
        .collect()
}

// --------------------------------------------------------------------------
// Pause and resume keep the run on the same step, and are audited.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-012 (WP-0720 AC-2)
#[test]
fn pause_then_resume_keeps_the_run_on_the_same_step() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    publish(
        &conn,
        &ws,
        &admin,
        &["va", "vb"],
        json!([{"id": "capture", "name": "Capture",
                "steps": [form_step("a", "va"), form_step("b", "vb")]}]),
    );
    let run_id = start(&conn, &ws, &admin, &record);
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("a"));

    // Pausing a non-waiting run is refused later; first pause the live run.
    let paused = process_lane_service::pause_run(
        &conn,
        &ws,
        &run_id,
        Some("back after lunch"),
        Some(&admin),
    )
    .unwrap();
    assert_eq!(paused.status, "paused");
    // A paused run cannot be paused again.
    assert!(process_lane_service::pause_run(&conn, &ws, &run_id, None, Some(&admin)).is_err());

    let resumed =
        process_lane_service::resume_paused_run(&conn, &ws, &run_id, Some(&admin)).unwrap();
    assert_eq!(resumed.status, "waiting_person");
    // Same step as before the pause.
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("a"));
    // Resuming a run that is not paused is refused.
    assert!(process_lane_service::resume_paused_run(&conn, &ws, &run_id, Some(&admin)).is_err());

    // Both actions are in the run's audit trail.
    let events = event_types(&conn, &ws, &run_id);
    assert!(
        events.contains(&"lane_run_paused".to_string()),
        "{events:?}"
    );
    assert!(
        events.contains(&"lane_run_resumed".to_string()),
        "{events:?}"
    );

    // The run still advances normally after a resume.
    let done = tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("va", "1")]),
        Some(&admin),
    ))
    .unwrap();
    assert_eq!(done.status, "waiting_person");
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("b"));
}

// --------------------------------------------------------------------------
// Reopen goes back to an earlier completed step in the current stage.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-012 (WP-0720 AC-2)
#[test]
fn reopen_goes_back_to_an_earlier_completed_step_and_is_audited() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    publish(
        &conn,
        &ws,
        &admin,
        &["va", "vb", "vc"],
        json!([{"id": "capture", "name": "Capture",
                "steps": [form_step("a", "va"), form_step("b", "vb"), form_step("c", "vc")]}]),
    );
    let run_id = start(&conn, &ws, &admin, &record);

    // Walk a, then b -> the run waits on c.
    tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("va", "1")]),
        Some(&admin),
    ))
    .unwrap();
    tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("vb", "2")]),
        Some(&admin),
    ))
    .unwrap();
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("c"));

    // Reopen the first step: the run goes back to it, still waiting on a person.
    let reopened = tokio_block(process_lane_service::reopen_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        "a",
        Some("mis-keyed the carrier"),
        Some(&admin),
    ))
    .unwrap();
    assert_eq!(reopened.status, "waiting_person");
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("a"));
    assert!(event_types(&conn, &ws, &run_id).contains(&"lane_run_step_reopened".to_string()));

    // The person re-walks forward from the reopened step.
    tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("va", "1-fixed")]),
        Some(&admin),
    ))
    .unwrap();
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("b"));

    // Reopening the current step, or a step that has not run, is refused.
    assert!(tokio_block(process_lane_service::reopen_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        "b",
        None,
        Some(&admin),
    ))
    .is_err());
    assert!(tokio_block(process_lane_service::reopen_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        "c",
        None,
        Some(&admin),
    ))
    .is_err());
}

// --------------------------------------------------------------------------
// Reopen never strands a governed write that has already posted.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-012 (WP-0720 AC-2)
#[test]
fn reopen_is_refused_across_a_posted_write() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    create_set_tracking_action(&conn, &ws, &admin);
    // Stage: form a, record post (posts Set Tracking), form c.
    publish(
        &conn,
        &ws,
        &admin,
        &["va", "vc"],
        json!([{"id": "capture", "name": "Capture", "steps": [
            form_step("a", "va"),
            {"id": "post", "name": "Post", "kind": "record", "role": "r", "required": true,
             "writes": [{"action_type": "Set Tracking", "inputs": {"val": "T-1"}}]},
            form_step("c", "vc")
        ]}]),
    );
    let run_id = start(&conn, &ws, &admin, &record);

    // Walk a -> the record step auto-posts -> the run waits on c.
    tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("va", "1")]),
        Some(&admin),
    ))
    .unwrap();
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("c"));
    assert_eq!(
        custom_field_service::get_entity_values(&conn, &record)
            .unwrap()
            .get("tracking")
            .map(String::as_str),
        Some("T-1"),
        "the record step posted its write",
    );

    // Reopening step a would re-walk across the posted write, so it is refused.
    let err = tokio_block(process_lane_service::reopen_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        "a",
        None,
        Some(&admin),
    ));
    assert!(err.is_err(), "reopen across a posted write must be refused");
}

// --------------------------------------------------------------------------
// Only the step's assignee or an administrator may pause or reopen.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-012 (WP-0720 AC-2), REQ-LANE-011
#[test]
fn only_the_assignee_or_admin_can_pause_and_reopen() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    publish(
        &conn,
        &ws,
        &admin,
        &["va", "vb"],
        json!([{"id": "capture", "name": "Capture",
                "steps": [form_step("a", "va"), form_step("b", "vb")]}]),
    );
    // A plain user who does not hold role "r" and is not an administrator.
    let outsider = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "mallory".into(),
            display_name: "Mallory".into(),
            password: "correct horse battery staple".into(),
            roles: vec!["ReadOnly".into()],
        },
        Some(&admin),
    )
    .unwrap()
    .id;

    let run_id = start(&conn, &ws, &admin, &record);
    // The outsider cannot pause the run.
    assert!(process_lane_service::pause_run(&conn, &ws, &run_id, None, Some(&outsider)).is_err());

    // Advance to step b (as admin), then the outsider cannot reopen step a.
    tokio_block(process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        &values(&[("va", "1")]),
        Some(&admin),
    ))
    .unwrap();
    assert!(tokio_block(process_lane_service::reopen_step(
        &conn,
        &ws,
        &master_key(),
        &run_id,
        "a",
        None,
        Some(&outsider),
    ))
    .is_err());
    // The run is untouched: still waiting on b.
    assert_eq!(current_step_id(&conn, &ws, &run_id).as_deref(), Some("b"));
}
