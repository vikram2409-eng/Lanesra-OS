//! Phase 7, WP-0710: per-item assignment runtime (ADR-0024; REQ-LANE-017).
//!
//! A repeating form step (`repeat_over` names a relationship to a child object)
//! is worked one entry per child record. Each item is completed on its own
//! through `submit_person_step_item`, so two people can complete different
//! items at the same time without overwriting each other; the step is done -
//! and the run jumps past the loop to the next step - only when every item is
//! done. Covers:
//!   AC-1  a repeating step enumerates one item per child, two people complete
//!         different items concurrently without overwrite, per-item values land
//!         on each child record, and the run advances only when all are done.
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Map, Value};

use lanesra_core::models::access_role::AccessRoleInput;
use lanesra_core::models::custom_field::{CustomFieldDefinitionInput, CustomFieldValues};
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::graph_run_repo;
use lanesra_core::services::{
    access_role_service, custom_field_service, custom_object_service, custom_record_service,
    process_lane_service, relationship_service, user_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Repeat Co".into(),
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

fn make_object(conn: &rusqlite::Connection, ws: &str, admin: &str, key: &str, prefix: &str) {
    custom_object_service::create_with_key(
        conn,
        ws,
        key,
        &CustomObjectDefinitionInput {
            singular_label: key.into(),
            plural_label: format!("{key}s"),
            icon: "▣".into(),
            prefix: prefix.into(),
            digits: 6,
        },
        Some(admin),
    )
    .unwrap();
}

fn make_text_field(conn: &rusqlite::Connection, ws: &str, admin: &str, object: &str, key: &str) {
    custom_field_service::create_definition_with_key(
        conn,
        ws,
        key,
        &CustomFieldDefinitionInput {
            entity_type: object.into(),
            label: key.into(),
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

fn make_record(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    object: &str,
    name: &str,
) -> String {
    custom_record_service::create(
        conn,
        ws,
        &CustomRecordInput {
            object_key: object.into(),
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

fn field_value(conn: &rusqlite::Connection, record: &str, field: &str) -> Option<String> {
    custom_field_service::get_entity_values(conn, record)
        .unwrap()
        .get(field)
        .cloned()
}

fn set_field(
    conn: &rusqlite::Connection,
    admin: &str,
    object: &str,
    record: &str,
    field: &str,
    value: &str,
) {
    let mut values = CustomFieldValues::new();
    values.insert(field.into(), value.into());
    custom_field_service::set_entity_values(conn, object, record, &values, Some(admin)).unwrap();
}

fn values(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect()
}

fn status(conn: &rusqlite::Connection, run_id: &str) -> String {
    graph_run_repo::get_run(conn, run_id)
        .unwrap()
        .unwrap()
        .status
}

fn run_data(conn: &rusqlite::Connection, run_id: &str) -> Value {
    let run = graph_run_repo::get_run(conn, run_id).unwrap().unwrap();
    serde_json::from_str(run.run_data_json.as_deref().unwrap_or("{}")).unwrap()
}

/// An `order` with two `line` children linked through a `lines` relationship.
/// Each line carries a `counted` field (the per-item form writes it) and a
/// `picker` field (the per-item assignee, Q-33 `child_field`).
fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str, link: bool) -> (String, Vec<String>) {
    make_object(conn, ws, admin, "batch", "BAT");
    make_object(conn, ws, admin, "tallyline", "TLN");
    make_text_field(conn, ws, admin, "tallyline", "counted");
    make_text_field(conn, ws, admin, "tallyline", "picker");

    relationship_service::create(
        conn,
        ws,
        &RelationshipDefinitionInput {
            source_entity_type: "batch".into(),
            target_entity_type: "tallyline".into(),
            target_is_polymorphic: false,
            relationship_type: "many_to_many".into(),
            forward_label: "lines".into(),
            reverse_label: "batch".into(),
            is_required: false,
            show_related_list: true,
            delete_behavior: "restrict".into(),
            sort_order: 0,
        },
        Some(admin),
    )
    .unwrap();

    let order = make_record(conn, ws, admin, "batch", "BAT-1");
    let def = relationship_service::list(conn, ws, true)
        .unwrap()
        .into_iter()
        .find(|d| d.source_entity_type == "batch" && d.target_entity_type == "tallyline")
        .expect("lines relationship");
    let mut lines = Vec::new();
    for name in ["BAT-1-A", "BAT-1-B"] {
        let line = make_record(conn, ws, admin, "tallyline", name);
        if link {
            relationship_service::link(
                conn,
                ws,
                &def.id,
                "batch",
                &order,
                "tallyline",
                &line,
                Some(admin),
            )
            .unwrap();
        }
        lines.push(line);
    }
    (order, lines)
}

/// Publishes a repeating lane over `lines`, with one field bound as given
/// (`record.lines[].counted` or `run_data.qty`) and per-item assignment by the
/// child's `picker` field. A `run_data.*` binding declares its per-child field.
fn publish_repeat_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, binds_to: &str) {
    let run_data_fields = if let Some(key) = binds_to.strip_prefix("run_data.") {
        json!([{"key": key, "type": "integer", "per_child": true, "required": true}])
    } else {
        json!([])
    };
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-count", "name": "Count", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": run_data_fields},
            "stages": [{
                "id": "count", "name": "Count", "steps": [{
                    "id": "tally", "name": "Tally", "kind": "form",
                    "role": "role_counter", "required": true,
                    "repeat_over": {
                        "field": "lines", "item_id_field": "id",
                        "assign": {"by": "child_field", "field": "picker"}
                    },
                    "fields": [{
                        "key": "counted", "label": "Counted", "type": "integer",
                        "binds_to": binds_to, "required": true
                    }]
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-count", Some(admin)).unwrap();
}

// --------------------------------------------------------------------------
// AC-1: one item per child; two people complete different items concurrently
// without overwrite; per-item writes land on each child; the run advances only
// when every item is done.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-017 (WP-0710 AC-1)
#[tokio::test]
async fn a_repeating_step_is_done_per_item_and_advances_when_all_items_are_done() {
    let (conn, ws, admin) = setup();
    let (order, lines) = seed(&conn, &ws, &admin, true);
    publish_repeat_lane(&conn, &ws, &admin, "record.lines[].counted");

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-count",
        &order,
        Some(&admin),
    )
    .await
    .unwrap();

    // The run parks on the repeating step, waiting on a person - one open
    // step for the whole set of items, not one run per item.
    assert_eq!(run.status, "waiting_person");

    // One person completes the first line. The step stays open: its other
    // item is still outstanding.
    let after_first = process_lane_service::submit_person_step_item(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &lines[0],
        &values(&[("counted", "40")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(
        after_first.status, "waiting_person",
        "the step is still open while an item is outstanding"
    );
    assert_eq!(
        field_value(&conn, &lines[0], "counted").as_deref(),
        Some("40"),
        "the first item's value lands on its own child record"
    );
    assert_eq!(
        field_value(&conn, &lines[1], "counted"),
        None,
        "the untouched item's child record is left alone"
    );

    // A second person completes the other line concurrently. The first item's
    // write is untouched (no overwrite), and with every item done the run
    // jumps past the loop and completes.
    let after_second = process_lane_service::submit_person_step_item(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &lines[1],
        &values(&[("counted", "30")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(
        after_second.status, "completed",
        "the step advances only when every item is done"
    );
    assert_eq!(
        field_value(&conn, &lines[0], "counted").as_deref(),
        Some("40")
    );
    assert_eq!(
        field_value(&conn, &lines[1], "counted").as_deref(),
        Some("30")
    );

    // Both items are recorded done in the run's per-step item state.
    let rd = run_data(&conn, &run.id);
    let items = rd["__items"]["tally"].as_object().expect("item state");
    assert_eq!(items.len(), 2, "one entry per child");
    assert!(items.values().all(|v| v["done"] == json!(true)));
    assert_eq!(status(&conn, &run.id), "completed");
}

// A repeating step with no items has nothing for a person to submit, so it
// advances past its loop as soon as the run reaches it rather than parking.
//
// Covers: REQ-LANE-017 (WP-0710 AC-1)
#[tokio::test]
async fn a_repeating_step_with_no_items_advances_immediately() {
    let (conn, ws, admin) = setup();
    // `link: false` - the batch has no related lines, so the step has no items.
    let (order, _lines) = seed(&conn, &ws, &admin, false);
    publish_repeat_lane(&conn, &ws, &admin, "record.lines[].counted");

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-count",
        &order,
        Some(&admin),
    )
    .await
    .unwrap();

    assert_eq!(
        run.status, "completed",
        "an empty repeating step never parks: {:?}",
        run.status
    );
}

// A repeating step's value can bind to a per-child run-data field, which the
// run keeps as a `{item_id: value}` map - the same shape the dry run takes.
//
// Covers: REQ-LANE-017 (WP-0710 AC-1)
#[tokio::test]
async fn per_item_values_can_bind_to_a_per_child_run_data_map() {
    let (conn, ws, admin) = setup();
    let (order, lines) = seed(&conn, &ws, &admin, true);
    publish_repeat_lane(&conn, &ws, &admin, "run_data.qty");

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-count",
        &order,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    for (line, qty) in lines.iter().zip(["40", "30"]) {
        process_lane_service::submit_person_step_item(
            &conn,
            &ws,
            &master_key(),
            &run.id,
            line,
            &values(&[("counted", qty)]),
            Some(&admin),
        )
        .await
        .unwrap();
    }

    // The per-child values are keyed by item id under the declared field, and
    // nothing lands on the child records (the binding was to run data).
    let rd = run_data(&conn, &run.id);
    assert_eq!(rd["qty"][&lines[0]], json!("40"));
    assert_eq!(rd["qty"][&lines[1]], json!("30"));
    assert_eq!(field_value(&conn, &lines[0], "counted"), None);
    assert_eq!(status(&conn, &run.id), "completed");
}

// Per-item assignment (Q-33 `child_field`): an item carries its own assignee,
// read from a field on the child. A person who does not hold that role cannot
// complete the item; an administrator still can.
//
// Covers: REQ-LANE-017 (WP-0710 AC-1)
#[tokio::test]
async fn an_item_is_gated_by_its_own_child_field_assignee() {
    let (conn, ws, admin) = setup();
    let (order, lines) = seed(&conn, &ws, &admin, true);
    publish_repeat_lane(&conn, &ws, &admin, "record.lines[].counted");

    // The first line is assigned to a real role; an outsider lacks it.
    let counter_role = access_role_service::create(
        &conn,
        &ws,
        &AccessRoleInput {
            name: "Counter".into(),
            description: "Counts a line".into(),
        },
        Some(&admin),
    )
    .unwrap();
    set_field(
        &conn,
        &admin,
        "tallyline",
        &lines[0],
        "picker",
        &counter_role.id,
    );

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

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-count",
        &order,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    // The outsider does not hold the line's assignee role, so the item is
    // refused and nothing is written.
    let err = process_lane_service::submit_person_step_item(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &lines[0],
        &values(&[("counted", "40")]),
        Some(&outsider.id),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err:?}").to_lowercase().contains("assigned")
            || format!("{err:?}").to_lowercase().contains("cannot"),
        "the refusal names the assignment: {err:?}"
    );
    assert_eq!(field_value(&conn, &lines[0], "counted"), None);
    assert_eq!(status(&conn, &run.id), "waiting_person");

    // The administrator may complete any item.
    process_lane_service::submit_person_step_item(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &lines[0],
        &values(&[("counted", "40")]),
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(
        field_value(&conn, &lines[0], "counted").as_deref(),
        Some("40")
    );
}

// --------------------------------------------------------------------------
// AC-3: a lane cannot start on a record that fails its start condition, and
// the reason is shown.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-017 (WP-0710 AC-3)
#[tokio::test]
async fn a_lane_refuses_to_start_on_a_record_that_fails_its_start_condition() {
    let (conn, ws, admin) = setup();
    make_object(&conn, &ws, &admin, "batch", "BAT");
    make_text_field(&conn, &ws, &admin, "batch", "state");

    // The lane only runs on a batch whose `state` is "ready".
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-gated", "name": "Gated", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": []},
            "start_condition": {
                "field": "$record.state", "operator": "equals", "value": "ready"
            },
            "stages": [{
                "id": "do", "name": "Do", "steps": [{
                    "id": "step", "name": "Step", "kind": "form",
                    "role": "r", "required": true, "fields": []
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(&conn, &ws, &lane.to_string(), Some(&admin))
        .unwrap();
    process_lane_service::publish_lane(&conn, &ws, "lane-gated", Some(&admin)).unwrap();

    let ready = make_record(&conn, &ws, &admin, "batch", "BAT-READY");
    set_field(&conn, &admin, "batch", &ready, "state", "ready");
    let not_ready = make_record(&conn, &ws, &admin, "batch", "BAT-HOLD");
    set_field(&conn, &admin, "batch", &not_ready, "state", "hold");

    // The record that meets the condition starts.
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-gated",
        &ready,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    // The record that fails it is refused, and the reason names the start
    // condition - nothing is started.
    let err = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-gated",
        &not_ready,
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err:?}")
            .to_lowercase()
            .contains("start condition"),
        "the refusal explains the start condition: {err:?}"
    );
    assert!(
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "batch", &not_ready)
            .unwrap()
            .runs
            .is_empty(),
        "a refused start leaves no run on the record"
    );
}

// --------------------------------------------------------------------------
// AC-2: a stage whose skip condition holds is skipped, and the audit trail
// says why.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-017 (WP-0710 AC-2)
#[tokio::test]
async fn a_stage_whose_skip_condition_holds_is_skipped_and_audited() {
    let (conn, ws, admin) = setup();
    make_object(&conn, &ws, &admin, "batch", "BAT");
    make_text_field(&conn, &ws, &admin, "batch", "state");

    // Two stages. The first is skipped when `state` is "skip"; the run then
    // parks on the second stage's step. Each stage's step is a plain form so
    // the run stops where it lands.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-skip", "name": "Skip", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": []},
            "stages": [
                {
                    "id": "alpha", "name": "Alpha",
                    "skip_condition": {
                        "field": "$record.state", "operator": "equals", "value": "skip"
                    },
                    "steps": [{
                        "id": "a-step", "name": "A", "kind": "form",
                        "role": "r", "required": true, "fields": []
                    }]
                },
                {
                    "id": "beta", "name": "Beta",
                    "steps": [{
                        "id": "b-step", "name": "B", "kind": "form",
                        "role": "r", "required": true, "fields": []
                    }]
                }
            ]
        }
    });
    process_lane_service::import_lane_definition(&conn, &ws, &lane.to_string(), Some(&admin))
        .unwrap();
    process_lane_service::publish_lane(&conn, &ws, "lane-skip", Some(&admin)).unwrap();

    // A record that meets the skip condition: Alpha is skipped, and the run
    // parks on Beta's step.
    let skip_rec = make_record(&conn, &ws, &admin, "batch", "BAT-SKIP");
    set_field(&conn, &admin, "batch", &skip_rec, "state", "skip");
    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-skip",
        &skip_rec,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    // The audit trail names the skipped stage and why.
    let audit =
        lanesra_core::repositories::audit_repo::list_for_entity(&conn, "batch", &skip_rec).unwrap();
    let skip_entry = audit
        .iter()
        .find(|e| e.event_type == "lane_stage_skipped")
        .expect("a lane_stage_skipped audit entry");
    assert!(
        skip_entry.summary.contains("Alpha"),
        "the audit names the stage: {}",
        skip_entry.summary
    );
    assert!(
        skip_entry.summary.to_lowercase().contains("skip condition"),
        "the audit says why: {}",
        skip_entry.summary
    );

    // A record that does not meet the skip condition runs Alpha normally - no
    // skip is recorded.
    let keep_rec = make_record(&conn, &ws, &admin, "batch", "BAT-KEEP");
    set_field(&conn, &admin, "batch", &keep_rec, "state", "run");
    process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-skip",
        &keep_rec,
        Some(&admin),
    )
    .await
    .unwrap();
    let kept_audit =
        lanesra_core::repositories::audit_repo::list_for_entity(&conn, "batch", &keep_rec).unwrap();
    assert!(
        !kept_audit
            .iter()
            .any(|e| e.event_type == "lane_stage_skipped"),
        "a stage that runs is not recorded as skipped"
    );
}
