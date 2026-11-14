//! Phase 7, WP-0704: human steps and run data (ADR-0023, ADR-0024 §4).
//!
//! A lane run reaches a `human_step` (form) node and waits in
//! `waiting_person` until a person submits it; the submission is validated
//! against the step's fields and its validation, stored (run data on the
//! run, record-bound fields through the governed write path), and the run
//! advances. These tests cover:
//!   AC-1  the run waits at a form step and a submit advances it
//!   AC-2  every entered value is stored; required / unknown / invalid is blocked and writes nothing
//!   AC-3  a viewer without the step's role cannot complete it and record-bound values are withheld
//!   AC-4  the human step's row carries its wait interval (started_at -> finished_at)
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::access_role::AccessRoleInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, custom_field_service, custom_object_service, custom_record_service,
    process_lane_service, user_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Test Co".into(),
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
    [83u8; 32]
}

/// Creates the `widget` object with a `tracking` custom field and one
/// record, returning the record id.
fn seed_object_and_record(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
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

/// A one-stage lane whose single form step collects `carrier` (run data,
/// required) and `tracking` (a record field), assigned to `role` (every lane
/// step declares one), optionally with a validation that `carrier` equals
/// `ACME`.
fn lane_json(role: &str, require_acme: bool) -> String {
    let mut step = json!({
        "id": "enter-details",
        "name": "Enter details",
        "kind": "form",
        "role": role,
        "required": true,
        "fields": [
            {"key": "carrier", "label": "Carrier", "type": "string",
             "binds_to": "run_data.carrier", "required": true},
            {"key": "tracking", "label": "Tracking", "type": "string",
             "binds_to": "record.tracking", "required": false}
        ]
    });
    if require_acme {
        step["validation"] = json!({
            "condition": {"field": "$run_data.carrier", "operator": "equals", "value": "ACME"},
            "message": "Carrier must be ACME"
        });
    }
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-widget-intake",
            "name": "Widget Intake",
            "object_key": "widget",
            "version": 1,
            "status": "published",
            "run_data": {"fields": [
                {"key": "carrier", "type": "string", "required": true}
            ]},
            "stages": [
                {"id": "capture", "name": "Capture",
                 "steps": [step]}
            ]
        }
    });
    serde_json::to_string(&lane).unwrap()
}

fn publish(conn: &rusqlite::Connection, ws: &str, admin: &str, json: &str) {
    process_lane_service::import_lane_definition(conn, ws, json, Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-widget-intake", Some(admin)).unwrap();
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect()
}

// --------------------------------------------------------------------------
// AC-1 + AC-2 + AC-4: the run waits, a submit stores every value and advances.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-004 (WP-0704 AC-1), REQ-LANE-005 (WP-0704 AC-2)
#[tokio::test]
async fn run_waits_for_a_person_then_a_submit_stores_values_and_advances() {
    let (conn, ws, admin) = setup();
    let record = seed_object_and_record(&conn, &ws, &admin);
    publish(&conn, &ws, &admin, &lane_json("role-receiver", false));

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-widget-intake",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // AC-1: the run is durably waiting on a person - not running, holding no
    // worker lease, with the human step as its current node.
    assert_eq!(run.status, "waiting_person");
    assert!(run.current_node_id.is_some());
    assert!(
        run.lease_owner.is_none(),
        "a run waiting on a person holds no lease"
    );

    // The step's row exists, waiting, with no finish time yet (AC-4: its
    // open interval is the wait).
    let waiting = lanesra_core::repositories::graph_run_repo::list_run_nodes(&conn, &run.id)
        .unwrap()
        .into_iter()
        .find(|n| n.node_type == "human_step")
        .expect("a human_step row");
    assert_eq!(waiting.status, "waiting_person");
    assert!(waiting.finished_at.is_none());

    let done = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME"), ("tracking", "TRK-99")]),
        Some(&admin),
    )
    .await
    .unwrap();

    // AC-1: a submit advances the run to the end.
    assert_eq!(done.status, "completed");

    // AC-2: the run-data field is stored on the run, the record field on the
    // record - nothing dropped.
    let run_data: Value =
        serde_json::from_str(done.run_data_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(run_data["carrier"], json!("ACME"));
    let fields = custom_field_service::get_entity_values(&conn, &record).unwrap();
    assert_eq!(fields.get("tracking").map(String::as_str), Some("TRK-99"));

    // AC-4: the step's row now has a finish time at or after its start - the
    // wait interval the runner and Object 360 read.
    let completed = lanesra_core::repositories::graph_run_repo::list_run_nodes(&conn, &run.id)
        .unwrap()
        .into_iter()
        .find(|n| n.node_type == "human_step")
        .unwrap();
    assert_eq!(completed.status, "completed");
    let finished = completed.finished_at.expect("a finish time");
    assert!(finished >= completed.started_at);
}

// --------------------------------------------------------------------------
// AC-2: missing required, unknown field, and a failed validation each block
// and write nothing.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-005 (WP-0704 AC-2)
#[tokio::test]
async fn a_missing_required_field_blocks_and_writes_nothing() {
    let (conn, ws, admin) = setup();
    let record = seed_object_and_record(&conn, &ws, &admin);
    publish(&conn, &ws, &admin, &lane_json("role-receiver", false));
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-widget-intake",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // Required `carrier` omitted.
    let err = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("tracking", "TRK-1")]),
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(format!("{err}").contains("required"));

    // Nothing was written and the run is still waiting.
    let after = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run.id)
        .unwrap()
        .unwrap();
    assert_eq!(after.status, "waiting_person");
    assert_eq!(after.run_data_json.as_deref().unwrap_or("{}"), "{}");
    assert!(custom_field_service::get_entity_values(&conn, &record)
        .unwrap()
        .get("tracking")
        .is_none());
}

// Covers: REQ-LANE-005 (WP-0704 AC-2)
#[tokio::test]
async fn an_unknown_field_is_rejected() {
    let (conn, ws, admin) = setup();
    let record = seed_object_and_record(&conn, &ws, &admin);
    publish(&conn, &ws, &admin, &lane_json("role-receiver", false));
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-widget-intake",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    let err = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME"), ("mystery", "x")]),
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(format!("{err}").contains("mystery"));
}

// Covers: REQ-LANE-005 (WP-0704 AC-2)
#[tokio::test]
async fn a_failed_validation_blocks_with_its_message_and_writes_nothing() {
    let (conn, ws, admin) = setup();
    let record = seed_object_and_record(&conn, &ws, &admin);
    publish(&conn, &ws, &admin, &lane_json("role-receiver", true)); // carrier must be ACME
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-widget-intake",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    let err = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "OTHER"), ("tracking", "TRK-7")]),
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(format!("{err}").contains("Carrier must be ACME"));

    // Blocked before any write: no run data, no record change.
    let after = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run.id)
        .unwrap()
        .unwrap();
    assert_eq!(after.status, "waiting_person");
    assert_eq!(after.run_data_json.as_deref().unwrap_or("{}"), "{}");
    assert!(custom_field_service::get_entity_values(&conn, &record)
        .unwrap()
        .get("tracking")
        .is_none());

    // The correct value passes and advances the run.
    let done = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME"), ("tracking", "TRK-7")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");
}

// --------------------------------------------------------------------------
// AC-3: a viewer without the step's role cannot complete it, and a viewer
// who cannot read the record does not receive its record-bound values.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-011 (WP-0704 AC-3)
#[tokio::test]
async fn role_gates_completion_and_record_values_are_withheld() {
    let (conn, ws, admin) = setup();
    let record = seed_object_and_record(&conn, &ws, &admin);

    // A role the step is assigned to.
    let receiver_role = access_role_service::create(
        &conn,
        &ws,
        &AccessRoleInput {
            name: "Receiver".into(),
            description: "Receives widgets".into(),
        },
        Some(&admin),
    )
    .unwrap();
    publish(&conn, &ws, &admin, &lane_json(&receiver_role.id, false));

    // A user who holds the Receiver role.
    let receiver = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "rcv".into(),
            display_name: "Receiver".into(),
            password: "receiver-password".into(),
            roles: vec!["Manager".into()],
        },
        Some(&admin),
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &receiver.id, &receiver_role.id, Some(&admin))
        .unwrap();

    // An outsider who holds only a grantless "Observer" role - so they can
    // neither complete the step (they lack Receiver) nor read the record
    // (Observer grants nothing). Holding one role keeps the access layer
    // from bootstrapping them a default read-granting role.
    let observer_role = access_role_service::create(
        &conn,
        &ws,
        &AccessRoleInput {
            name: "Observer".into(),
            description: "Sees nothing by default".into(),
        },
        Some(&admin),
    )
    .unwrap();
    let outsider = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "out".into(),
            display_name: "Outsider".into(),
            password: "outsider-password".into(),
            roles: vec!["ReadOnly".into()],
        },
        Some(&admin),
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &outsider.id, &observer_role.id, Some(&admin))
        .unwrap();
    for role in access_role_service::list_roles_for_user(&conn, &outsider.id).unwrap() {
        if role.id != observer_role.id {
            access_role_service::remove_from_user(&conn, &outsider.id, &role.id, Some(&admin))
                .unwrap();
        }
    }

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-widget-intake",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // The role-holder may complete it; its content is visible to them only
    // as far as their record access allows (checked for the outsider below).
    let for_receiver =
        process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&receiver.id)).unwrap();
    let step = for_receiver.current.expect("a current step");
    assert!(step.can_complete, "the role-holder may complete the step");
    assert_eq!(step.waiting_on.as_deref(), Some(receiver_role.id.as_str()));

    // The outsider sees the step and who it waits on, but cannot complete it,
    // and the record-bound field's value is withheld (run-data fields are
    // still described).
    let for_outsider =
        process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&outsider.id)).unwrap();
    let step = for_outsider.current.expect("a current step");
    assert!(!step.can_complete, "the outsider cannot complete the step");
    assert_eq!(step.waiting_on.as_deref(), Some(receiver_role.id.as_str()));
    let tracking = step
        .fields
        .iter()
        .find(|f| f.binds_to == "record.tracking")
        .unwrap();
    assert!(
        !tracking.readable,
        "a record field the outsider cannot read"
    );
    assert!(tracking.value.is_none());
    let carrier = step
        .fields
        .iter()
        .find(|f| f.binds_to == "run_data.carrier")
        .unwrap();
    assert!(carrier.readable, "run-data fields have no per-record ACL");

    // The outsider's submit is refused and writes nothing.
    let err = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values(&[("carrier", "ACME")]),
        Some(&outsider.id),
    )
    .await
    .unwrap_err();
    assert!(format!("{err}").to_lowercase().contains("cannot complete"));
    let after = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run.id)
        .unwrap()
        .unwrap();
    assert_eq!(after.status, "waiting_person");
    assert_eq!(after.run_data_json.as_deref().unwrap_or("{}"), "{}");
}
