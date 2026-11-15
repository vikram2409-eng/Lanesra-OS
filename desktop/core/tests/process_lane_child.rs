//! Phase 7, WP-0716: child lanes (ADR-0024 §6, §9; REQ-LANE-023).
//!
//! A `child_lane` field on a form step starts a lane on a related record and
//! waits for it. The runtime is untouched; the lane service drives the child
//! from its own advance hooks. Covers:
//!   AC-1  a parent waits in `waiting_child`, shows the child's stage, and
//!         continues on the child's outcome; a child that finishes at once
//!         never parks the parent
//!   AC-2  the depth limit (3) is refused with a readable reason, named and
//!         recorded, leaving the parent waiting on the step
//!   AC-3  cancelling the parent cancels or keeps the child; a cancelled
//!         child resumes the parent with outcome `cancelled`
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::{audit_repo, graph_run_repo, process_lane_repo};
use lanesra_core::services::{
    custom_field_service, custom_object_service, custom_record_service, process_lane_service,
    workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Child Lane Co".into(),
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

/// The `widget` object with a `related` text field (holds another widget's id,
/// the child anchor).
fn seed_object(conn: &rusqlite::Connection, ws: &str, admin: &str) {
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
        "related",
        &CustomFieldDefinitionInput {
            entity_type: "widget".into(),
            label: "Related".into(),
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
}

fn make_record(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: "widget".into(),
            primary_name: name.into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn set_related(conn: &rusqlite::Connection, admin: &str, record: &str, related: &str) {
    let mut values = lanesra_core::models::custom_field::CustomFieldValues::new();
    values.insert("related".into(), related.into());
    custom_field_service::set_entity_values(conn, "widget", record, &values, Some(admin)).unwrap();
}

fn publish(conn: &rusqlite::Connection, ws: &str, admin: &str, id: &str, stages: Value) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": id, "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": []}, "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, id, Some(admin)).unwrap();
}

/// A child-lane step: a form step with no fields carrying a `child_lane`.
fn child_step(id: &str, lane_id: &str) -> Value {
    json!({
        "id": id, "name": id, "kind": "form", "role": "r", "required": true,
        "child_lane": {"lane_id": lane_id, "anchor_record": "$record.related"}
    })
}

fn run_data(conn: &rusqlite::Connection, run_id: &str) -> Value {
    let run = graph_run_repo::get_run(conn, run_id).unwrap().unwrap();
    serde_json::from_str(run.run_data_json.as_deref().unwrap_or("{}")).unwrap()
}

fn status(conn: &rusqlite::Connection, run_id: &str) -> String {
    graph_run_repo::get_run(conn, run_id)
        .unwrap()
        .unwrap()
        .status
}

// --------------------------------------------------------------------------
// AC-1: a parent waits for its child, shows the child's stage, and continues
// on the child's outcome.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-023 (WP-0716 AC-1)
#[tokio::test]
async fn parent_waits_for_a_child_then_continues_on_its_outcome() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    let child_rec = make_record(&conn, &ws, &admin, "child");
    let parent_rec = make_record(&conn, &ws, &admin, "parent");
    set_related(&conn, &admin, &parent_rec, &child_rec);

    // Child lane: one form step that waits for a person.
    publish(
        &conn,
        &ws,
        &admin,
        "lane-child",
        json!([{"id": "do", "name": "Do", "steps": [
            {"id": "fill", "name": "Fill", "kind": "form", "role": "r", "required": true, "fields": []}
        ]}]),
    );
    // Parent lane: a child step, then a plain wrap-up form step.
    publish(
        &conn,
        &ws,
        &admin,
        "lane-parent",
        json!([
            {"id": "delegate", "name": "Delegate", "steps": [child_step("hand-off", "lane-child")]},
            {"id": "finish", "name": "Finish", "steps": [
                {"id": "wrap", "name": "Wrap", "kind": "form", "role": "r", "required": true, "fields": []}
            ]}
        ]),
    );

    let parent = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-parent",
        &parent_rec,
        Some(&admin),
    )
    .await
    .unwrap();

    // The parent is parked waiting on its child; the child is running.
    assert_eq!(parent.status, "waiting_child");
    let child_id = run_data(&conn, &parent.id)["__child"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(status(&conn, &child_id), "waiting_person");

    // The run view surfaces the child it waits on and the child's stage (AC-1).
    let view = process_lane_service::get_run_view(&conn, &ws, &parent.id, Some(&admin)).unwrap();
    let child_view = view.child.expect("a child view while waiting_child");
    assert_eq!(child_view.child_run_id, child_id);
    assert_eq!(child_view.lane_id.as_deref(), Some("lane-child"));
    assert_eq!(child_view.stage.as_deref(), Some("Do"));

    // Finishing the child resumes the parent, which records the outcome and
    // advances past the child step to its wrap-up step.
    let done_child = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &child_id,
        &Map::new(),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(done_child.status, "completed");

    assert_eq!(status(&conn, &parent.id), "waiting_person"); // now at "wrap"
    assert_eq!(
        run_data(&conn, &parent.id)["__child_outcome"],
        json!("completed")
    );
    let view = process_lane_service::get_run_view(&conn, &ws, &parent.id, Some(&admin)).unwrap();
    assert_eq!(view.current.unwrap().step_id, "wrap");
}

// Covers: REQ-LANE-023 (WP-0716 AC-1)
#[tokio::test]
async fn a_child_that_finishes_at_once_never_parks_the_parent() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    let child_rec = make_record(&conn, &ws, &admin, "child");
    let parent_rec = make_record(&conn, &ws, &admin, "parent");
    set_related(&conn, &admin, &parent_rec, &child_rec);

    // Child lane with only a check that passes at once (no waiting step).
    publish(
        &conn,
        &ws,
        &admin,
        "lane-instant",
        json!([{"id": "c", "name": "Check", "steps": [
            {"id": "ok", "name": "OK", "kind": "check", "required": true,
             "check": {"scope": "anchor", "message": "no",
                       "condition": {"field": "$record.status", "operator": "equals", "value": "Active"}},
             "branches": [{"on": "pass", "end": {"outcome": "done"}},
                          {"on": "fail", "end": {"outcome": "held"}}]}
        ]}]),
    );
    publish(
        &conn,
        &ws,
        &admin,
        "lane-parent",
        json!([
            {"id": "delegate", "name": "Delegate", "steps": [child_step("hand-off", "lane-instant")]},
            {"id": "finish", "name": "Finish", "steps": [
                {"id": "wrap", "name": "Wrap", "kind": "form", "role": "r", "required": true, "fields": []}
            ]}
        ]),
    );

    let parent = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-parent",
        &parent_rec,
        Some(&admin),
    )
    .await
    .unwrap();

    // The child finished during the same call, so the parent was never parked
    // in `waiting_child` - it advanced straight to its wrap-up step.
    assert_eq!(parent.status, "waiting_person");
    assert_eq!(
        run_data(&conn, &parent.id)["__child_outcome"],
        json!("completed")
    );
    let view = process_lane_service::get_run_view(&conn, &ws, &parent.id, Some(&admin)).unwrap();
    assert_eq!(view.current.unwrap().step_id, "wrap");
    assert!(view.child.is_none(), "not waiting on a child any more");
}

// --------------------------------------------------------------------------
// AC-2: the depth limit is refused with a readable, recorded reason.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-023 (WP-0716 AC-2)
#[tokio::test]
async fn the_depth_limit_is_enforced_with_a_readable_refusal() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    // A chain of records r0 -> r1 -> r2 -> r3 -> r4 via `related`.
    let ids: Vec<String> = (0..5)
        .map(|i| make_record(&conn, &ws, &admin, &format!("r{i}")))
        .collect();
    for i in 0..4 {
        set_related(&conn, &admin, &ids[i], &ids[i + 1]);
    }
    // One lane that delegates to itself on the related record.
    publish(
        &conn,
        &ws,
        &admin,
        "lane-chain",
        json!([{"id": "s", "name": "S", "steps": [child_step("deeper", "lane-chain")]}]),
    );

    // Start at r0 (depth 0); it recurses r1 (1), r2 (2), r3 (3). The run at r3
    // is at the limit and cannot start a child on r4.
    process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-chain",
        &ids[0],
        Some(&admin),
    )
    .await
    .unwrap();

    // The depth-3 run (on r3) refused to start its child and stays waiting.
    let r3_run =
        process_lane_repo::find_active_run_for_lane_record(&conn, &ws, "lane-chain", &ids[3])
            .unwrap()
            .expect("a run on r3");
    assert_eq!(
        status(&conn, &r3_run),
        "waiting_person",
        "refused run stays waiting, retryable"
    );
    let err = run_data(&conn, &r3_run)["__child_error"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(err.contains('3'), "the refusal names the limit (3): {err}");
    assert!(
        err.contains("lane-chain"),
        "the refusal names the lane chain: {err}"
    );

    // The refusal is audited on the record.
    let audit = audit_repo::list_for_entity(&conn, "widget", &ids[3]).unwrap();
    assert!(audit.iter().any(|e| e.event_type == "lane_child_refused"));

    // No run was started on r4 (the depth-5 record).
    assert!(
        process_lane_repo::find_active_run_for_lane_record(&conn, &ws, "lane-chain", &ids[4])
            .unwrap()
            .is_none(),
        "the fourth level never started"
    );
}

// --------------------------------------------------------------------------
// AC-3: cancelling the parent cancels or keeps the child; a cancelled child
// resumes its parent with outcome `cancelled`.
// --------------------------------------------------------------------------

async fn start_parent_waiting_on_child(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    parent_name: &str,
) -> (String, String) {
    let child_rec = make_record(conn, ws, admin, &format!("{parent_name}-child"));
    let parent_rec = make_record(conn, ws, admin, parent_name);
    set_related(conn, admin, &parent_rec, &child_rec);
    let parent = process_lane_service::start_lane_run(
        conn,
        ws,
        &master_key(),
        "lane-parent",
        &parent_rec,
        Some(admin),
    )
    .await
    .unwrap();
    assert_eq!(parent.status, "waiting_child");
    let child_id = run_data(conn, &parent.id)["__child"]
        .as_str()
        .unwrap()
        .to_string();
    (parent.id, child_id)
}

// Covers: REQ-LANE-023 (WP-0716 AC-3)
#[tokio::test]
async fn cancelling_the_parent_cancels_or_keeps_the_child() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    publish(
        &conn,
        &ws,
        &admin,
        "lane-child",
        json!([{"id": "do", "name": "Do", "steps": [
            {"id": "fill", "name": "Fill", "kind": "form", "role": "r", "required": true, "fields": []}
        ]}]),
    );
    publish(
        &conn,
        &ws,
        &admin,
        "lane-parent",
        json!([{"id": "delegate", "name": "Delegate", "steps": [child_step("hand-off", "lane-child")]}]),
    );

    // Cancel with "cancel child": both end cancelled.
    let (p1, c1) = start_parent_waiting_on_child(&conn, &ws, &admin, "p-cancel").await;
    process_lane_service::cancel_lane_run(
        &conn,
        &ws,
        &master_key(),
        &p1,
        "done",
        true,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &p1), "cancelled");
    assert_eq!(status(&conn, &c1), "cancelled");

    // Cancel with "keep child": the child keeps running, and its later finish
    // is a no-op for the cancelled parent.
    let (p2, c2) = start_parent_waiting_on_child(&conn, &ws, &admin, "p-keep").await;
    process_lane_service::cancel_lane_run(
        &conn,
        &ws,
        &master_key(),
        &p2,
        "done",
        false,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &p2), "cancelled");
    assert_eq!(
        status(&conn, &c2),
        "waiting_person",
        "the kept child is still running"
    );
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &c2,
        &Map::new(),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &c2), "completed");
    assert_eq!(
        status(&conn, &p2),
        "cancelled",
        "the kept child's finish does not touch the cancelled parent"
    );
}

// Covers: REQ-LANE-023 (WP-0716 AC-3)
#[tokio::test]
async fn a_cancelled_child_resumes_the_parent_with_outcome_cancelled() {
    let (conn, ws, admin) = setup();
    seed_object(&conn, &ws, &admin);
    publish(
        &conn,
        &ws,
        &admin,
        "lane-child",
        json!([{"id": "do", "name": "Do", "steps": [
            {"id": "fill", "name": "Fill", "kind": "form", "role": "r", "required": true, "fields": []}
        ]}]),
    );
    // Parent has a wrap-up step so we can see it advance past the child step.
    publish(
        &conn,
        &ws,
        &admin,
        "lane-parent",
        json!([
            {"id": "delegate", "name": "Delegate", "steps": [child_step("hand-off", "lane-child")]},
            {"id": "finish", "name": "Finish", "steps": [
                {"id": "wrap", "name": "Wrap", "kind": "form", "role": "r", "required": true, "fields": []}
            ]}
        ]),
    );

    let (parent_id, child_id) =
        start_parent_waiting_on_child(&conn, &ws, &admin, "p-childcancel").await;

    // Cancel the child directly - the parent resumes with outcome cancelled.
    process_lane_service::cancel_lane_run(
        &conn,
        &ws,
        &master_key(),
        &child_id,
        "stop",
        false,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(status(&conn, &child_id), "cancelled");
    assert_eq!(
        status(&conn, &parent_id),
        "waiting_person",
        "parent resumed past the child step"
    );
    assert_eq!(
        run_data(&conn, &parent_id)["__child_outcome"],
        json!("cancelled")
    );
    let view = process_lane_service::get_run_view(&conn, &ws, &parent_id, Some(&admin)).unwrap();
    assert_eq!(view.current.unwrap().step_id, "wrap");
}
