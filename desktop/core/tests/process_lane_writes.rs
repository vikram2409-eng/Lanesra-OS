//! Phase 7, WP-0705: record and check steps - real, atomic, repeat-safe
//! writes (ADR-0023, REQ-LANE-006, REQ-LANE-007).
//!
//! A `record` step posts its writes through the same governed path a direct
//! edit uses (`workflow_service::apply_action` -> the object's real write
//! path, Business Rules and access enforced), all-or-nothing and idempotent
//! per step attempt (WP-0141). A `check` step evaluates its rule and carries
//! a human-readable reason when it does not pass. A required step cannot be
//! skipped; an optional one can, and the skip records who, when and why.
//!
//! These tests cover:
//!   AC-1  a step's several writes are all-or-nothing (failure mid-step)
//!   AC-2  completing/resuming does not repeat a write
//!   AC-3  a Business Rule that blocks the write blocks the step with its message
//!   AC-4  a failed check carries its reason; required cannot skip; optional skip recorded
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Value};

use lanesra_core::models::action_type::{ActionParameterSpec, ActionTypeDefinitionInput};
use lanesra_core::models::business_rule::{
    BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput,
};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    action_type_service, business_rule_service, custom_field_service, custom_object_service,
    custom_record_service, idempotency_service, process_lane_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Writes Co".into(),
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

/// The `widget` object with a `tracking` text field and one record whose
/// tracking starts as `start`. Returns the record id.
fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str, start: &str) -> String {
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
    let record = custom_record_service::create(
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
    .id;
    if !start.is_empty() {
        let mut values = lanesra_core::models::custom_field::CustomFieldValues::new();
        values.insert("tracking".into(), start.into());
        custom_field_service::set_entity_values(conn, "widget", &record, &values, Some(admin))
            .unwrap();
    }
    record
}

/// Defines the `Set Tracking` Action Type on `widget`: a `set_fields` effect
/// that writes its `val` parameter into the `tracking` field, through the
/// governed write path (so Business Rules and access apply).
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

/// A record-step write that invokes the `Set Tracking` Action Type with a
/// literal or token value.
fn set_tracking(value: &str) -> Value {
    json!({
        "action_type": "Set Tracking",
        "inputs": { "val": value }
    })
}

fn tracking(conn: &rusqlite::Connection, record: &str) -> Option<String> {
    custom_field_service::get_entity_values(conn, record)
        .unwrap()
        .get("tracking")
        .cloned()
}

/// Publishes a one-object lane from a `stages` array and returns its key.
fn publish_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, stages: Value) -> &'static str {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-widget",
            "name": "Widget Lane",
            "object_key": "widget",
            "version": 1,
            "status": "published",
            "run_data": {"fields": []},
            "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-widget", Some(admin)).unwrap();
    "lane-widget"
}

fn action_node_id(conn: &rusqlite::Connection, run_graph_id: &str) -> String {
    lanesra_core::repositories::execution_graph_repo::get(conn, run_graph_id)
        .unwrap()
        .unwrap()
        .nodes
        .into_iter()
        .find(|n| n.node_type == "action")
        .expect("an action node")
        .id
}

// --------------------------------------------------------------------------
// AC-1: several writes are all-or-nothing - a failure mid-step applies none.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-006 (WP-0705 AC-1)
#[tokio::test]
async fn several_writes_are_all_or_nothing() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "START");
    create_set_tracking_action(&conn, &ws, &admin);

    // A record step with two writes: the first would set tracking, the second
    // names an action that does not exist and fails. Neither must apply.
    let stages = json!([{
        "id": "post", "name": "Post",
        "steps": [{
            "id": "post-both", "name": "Post both", "kind": "record", "role": "r",
            "required": true,
            "writes": [
                set_tracking("CHANGED"),
                {"action_type": "not_a_real_action", "inputs": {}}
            ]
        }]
    }]);
    let lane = publish_lane(&conn, &ws, &admin, stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // The run failed on the bad write, and the first write was rolled back:
    // tracking is still its starting value, not "CHANGED".
    assert_eq!(run.status, "failed");
    assert_eq!(
        tracking(&conn, &record).as_deref(),
        Some("START"),
        "a failure mid-step applies none of the step's writes"
    );
}

// --------------------------------------------------------------------------
// AC-2: completing/resuming the same step attempt does not repeat a write.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-006 (WP-0705 AC-2)
#[tokio::test]
async fn a_write_is_not_repeated_on_resume() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "START");
    create_set_tracking_action(&conn, &ws, &admin);

    let stages = json!([{
        "id": "post", "name": "Post",
        "steps": [{
            "id": "post-one", "name": "Post one", "kind": "record", "role": "r",
            "required": true,
            "writes": [set_tracking("DONE")]
        }]
    }]);
    let lane = publish_lane(&conn, &ws, &admin, stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // The write happened exactly once on the first run through the node.
    assert_eq!(run.status, "completed");
    assert_eq!(tracking(&conn, &record).as_deref(), Some("DONE"));

    // The write is recorded under its idempotency key (run, node, attempt 1).
    let node_id = action_node_id(&conn, &run.graph_id);
    assert!(
        idempotency_service::already_done(&conn, &run.id, &node_id, 1, "").unwrap(),
        "the step's write is recorded in the ledger"
    );

    // A resume re-enters the node at the same attempt. `run_once` with that
    // key must return the stored result WITHOUT calling the write again - so
    // a crash-resume or a double submit never repeats the write (REQ-LANE-006).
    let mut called = false;
    idempotency_service::run_once(&conn, &ws, &run.id, &node_id, 1, "", || {
        called = true;
        Ok(json!("second time"))
    })
    .unwrap();
    assert!(
        !called,
        "re-running the step attempt does not apply the write again"
    );
    assert_eq!(tracking(&conn, &record).as_deref(), Some("DONE"));
}

// --------------------------------------------------------------------------
// AC-3: a Business Rule that blocks the write blocks the step with its message.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-006 (WP-0705 AC-3)
#[tokio::test]
async fn a_business_rule_blocks_the_step_with_its_message() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "OK");
    create_set_tracking_action(&conn, &ws, &admin);

    // A rule on widget blocks any save that sets tracking to "BAD".
    business_rule_service::create_rule(
        &conn,
        &ws,
        &BusinessRuleInput {
            entity_type: "widget".into(),
            name: "No bad tracking".into(),
            description: None,
            match_type: "all".into(),
            priority: 0,
            effective_start_date: None,
            effective_end_date: None,
            app_id: None,
            conditions: vec![BusinessRuleConditionInput {
                field_source: "custom".into(),
                field_key: "tracking".into(),
                operator: "equals".into(),
                value: "BAD".into(),
                compare_field_source: None,
                compare_field_key: None,
                group_id: None,
                relationship_definition_id: None,
            }],
            actions: vec![BusinessRuleActionInput {
                action_type: "block_save".into(),
                target_field_key: None,
                target_field_source: "custom".into(),
                action_value: None,
                message: Some("Tracking cannot be BAD".into()),
            }],
        },
        Some(&admin),
    )
    .unwrap();

    let stages = json!([{
        "id": "post", "name": "Post",
        "steps": [{
            "id": "post-bad", "name": "Post bad", "kind": "record", "role": "r",
            "required": true,
            "writes": [set_tracking("BAD")]
        }]
    }]);
    let lane = publish_lane(&conn, &ws, &admin, stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // The rule blocked the write: the run failed with the rule's message, and
    // the record is unchanged (the governed path refused it, same as a direct
    // edit would).
    assert_eq!(run.status, "failed");
    assert!(
        run.error_message
            .as_deref()
            .unwrap_or_default()
            .contains("Tracking cannot be BAD"),
        "the step is blocked with the Business Rule's own message, got: {:?}",
        run.error_message
    );
    assert_eq!(tracking(&conn, &record).as_deref(), Some("OK"));
}

// --------------------------------------------------------------------------
// AC-4a: a failed check blocks advance with the reason shown.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-007 (WP-0705 AC-4)
#[tokio::test]
async fn a_failed_check_carries_its_reason() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "no"); // tracking = "no"

    // A check requires tracking == "yes"; it does not, so the check fails and
    // the run takes the fail branch, which ends the run. The check's reason is
    // recorded on its run node.
    let stages = json!([{
        "id": "verify", "name": "Verify",
        "steps": [{
            "id": "is-approved", "name": "Is approved", "kind": "check",
            "required": true,
            "check": {
                "scope": "anchor",
                "message": "Tracking is not yes yet",
                "condition": {"field": "$record.tracking", "operator": "equals", "value": "yes"}
            },
            "branches": [
                {"on": "pass", "end": {"outcome": "done"}},
                {"on": "fail", "end": {"outcome": "held"}}
            ]
        }]
    }]);
    let lane = publish_lane(&conn, &ws, &admin, stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // The check node recorded its reason on the fail path.
    let check_row = lanesra_core::repositories::graph_run_repo::list_run_nodes(&conn, &run.id)
        .unwrap()
        .into_iter()
        .find(|n| n.node_type == "condition")
        .expect("a condition (check) run node");
    let output: Value =
        serde_json::from_str(check_row.output_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(output["result"], json!(false), "the check did not pass");
    assert_eq!(
        output["reason"],
        json!("Tracking is not yes yet"),
        "the failed check carries its human-readable reason"
    );
}

// --------------------------------------------------------------------------
// AC-4b/c: a required step cannot be skipped; an optional skip is recorded.
// --------------------------------------------------------------------------

/// A one-stage lane whose single form step is required or optional.
fn form_lane(required: bool) -> Value {
    json!([{
        "id": "capture", "name": "Capture",
        "steps": [{
            "id": "maybe", "name": "Maybe do it", "kind": "form", "role": "r",
            "required": required,
            "fields": [
                {"key": "note", "label": "Note", "type": "string",
                 "binds_to": "run_data.note", "required": false}
            ]
        }]
    }])
}

// Covers: REQ-LANE-007 (WP-0705 AC-4)
#[tokio::test]
async fn a_required_step_cannot_be_skipped() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "");
    let lane = publish_lane(&conn, &ws, &admin, form_lane(true));

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    let err = process_lane_service::skip_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        "not needed",
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err}").contains("required step cannot be skipped"),
        "got: {err}"
    );

    // The run is untouched: still waiting on the person.
    let after = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run.id)
        .unwrap()
        .unwrap();
    assert_eq!(after.status, "waiting_person");
}

// Covers: REQ-LANE-007 (WP-0705 AC-4)
#[tokio::test]
async fn an_optional_step_skip_is_recorded_and_advances() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin, "");
    let lane = publish_lane(&conn, &ws, &admin, form_lane(false));

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        lane,
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");
    let node_id = run.current_node_id.clone().unwrap();

    // A reason is required to skip.
    let err =
        process_lane_service::skip_step(&conn, &ws, &master_key(), &run.id, "   ", Some(&admin))
            .await
            .unwrap_err();
    assert!(
        format!("{err}").contains("reason is required"),
        "got: {err}"
    );

    // Skipping the optional step advances the run and records who/when/why.
    let done = process_lane_service::skip_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        "nothing to note",
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");

    let skipped = lanesra_core::repositories::graph_run_repo::list_run_nodes(&conn, &run.id)
        .unwrap()
        .into_iter()
        .find(|n| n.node_id == node_id && n.status == "skipped")
        .expect("a skipped run node");
    let output: Value =
        serde_json::from_str(skipped.output_json.as_deref().unwrap_or("{}")).unwrap();
    assert_eq!(output["reason"], json!("nothing to note"));
    assert_eq!(output["skipped_by"], json!(admin));
    assert!(output.get("at").is_some(), "the skip records when");
}
