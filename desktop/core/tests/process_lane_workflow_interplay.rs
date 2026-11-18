//! Phase 7, WP-0709: lanes and workflows - declared replacement
//! (ADR-0023; REQ-LANE-013).
//!
//! A lane step can declare the workflow actions it replaces. When the step
//! makes its change, the workflow still fires - but the declared actions are
//! skipped, because the lane did that work itself (no duplicate work), and
//! the skip is recorded on the run, not hidden. The same change made outside
//! a lane fires the workflow in full.
//!
//! Covers:
//!   AC-1  a lane-made change fires the workflow except the actions the step
//!         declares it replaces, and the skip is visible on the run
//!   AC-2  the same change made outside a lane fires the workflow in full
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use serde_json::{json, Value};

use lanesra_core::models::custom_field::{CustomFieldDefinitionInput, CustomFieldValues};
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    custom_field_service, custom_object_service, custom_record_service, graph_runtime_service,
    process_lane_service, task_service, workflow_service, workspace_service,
};

fn setup() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Lane Workflow Co".into(),
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

fn text_field(conn: &rusqlite::Connection, ws: &str, admin: &str, object: &str, key: &str) {
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

/// A `widget` object with a `stage` field (what the workflow watches and the
/// lane writes) and a `note` field (what the workflow's second action sets).
fn seed(conn: &rusqlite::Connection, ws: &str, admin: &str) {
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
    text_field(conn, ws, admin, "widget", "stage");
    text_field(conn, ws, admin, "widget", "note");
}

fn make_widget(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
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

/// A `field_changed` workflow on `widget.stage` with two actions in order:
/// `create_task` (index 0) then `update_field note = "touched"` (index 1).
/// Returns the workflow id.
fn make_workflow(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    workflow_service::create_rule(
        conn,
        ws,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "widget".into(),
            name: "On stage change".into(),
            description: None,
            trigger_type: "field_changed".into(),
            trigger_status: None,
            trigger_field_key: Some("stage".into()),
            trigger_field_source: "custom".into(),
            trigger_offset_days: 0,
            match_type: "all".into(),
            priority: 0,
            conditions: vec![],
            actions: vec![
                WorkflowActionInput {
                    action_type: "create_task".into(),
                    params_json: json!({"title": "Review stage", "due_in_days": 1}).to_string(),
                },
                WorkflowActionInput {
                    action_type: "update_field".into(),
                    params_json: json!({
                        "target_field_key": "note",
                        "target_field_source": "custom",
                        "value": "touched"
                    })
                    .to_string(),
                },
            ],
        },
        Some(admin),
    )
    .unwrap()
    .id
}

/// Publishes a one-step lane whose form step writes `record.stage` and
/// declares it replaces the given `(workflow_id, action_index)`.
fn publish_lane(conn: &rusqlite::Connection, ws: &str, admin: &str, replaces: Value) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-stage", "name": "Set stage", "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": []},
            "stages": [{
                "id": "do", "name": "Do", "steps": [{
                    "id": "set", "name": "Set stage", "kind": "form",
                    "role": "r", "required": true,
                    "replaces": replaces,
                    "fields": [{
                        "key": "stage_in", "label": "Stage", "type": "string",
                        "binds_to": "record.stage", "required": true
                    }]
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-stage", Some(admin)).unwrap();
}

fn stage(conn: &rusqlite::Connection, record: &str) -> Option<String> {
    custom_field_service::get_entity_values(conn, record)
        .unwrap()
        .get("stage")
        .cloned()
}

fn note(conn: &rusqlite::Connection, record: &str) -> Option<String> {
    custom_field_service::get_entity_values(conn, record)
        .unwrap()
        .get("note")
        .cloned()
}

fn task_count(conn: &rusqlite::Connection, record: &str) -> usize {
    task_service::list_by_related(conn, "widget", record)
        .unwrap()
        .len()
}

fn set_stage_directly(conn: &rusqlite::Connection, admin: &str, record: &str, value: &str) {
    let mut values = CustomFieldValues::new();
    values.insert("stage".into(), value.into());
    custom_field_service::set_entity_values(conn, "widget", record, &values, Some(admin)).unwrap();
}

// --------------------------------------------------------------------------
// AC-1: a lane-made change fires the workflow except the actions the step
// declares it replaces, and the skip is visible on the run.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-013 (WP-0709 AC-1)
#[tokio::test]
async fn a_lane_write_skips_only_the_actions_its_step_declares_it_replaces() {
    let (conn, ws, admin) = setup();
    seed(&conn, &ws, &admin);
    let widget = make_widget(&conn, &ws, &admin, "W1");
    let wf = make_workflow(&conn, &ws, &admin);
    // The step replaces action 0 (create_task) but not action 1 (update_field).
    publish_lane(
        &conn,
        &ws,
        &admin,
        json!([{"workflow_id": wf, "action_index": 0}]),
    );

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-stage",
        &widget,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person");

    let mut values = serde_json::Map::new();
    values.insert("stage_in".into(), json!("Review"));
    process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values,
        Some(&admin),
    )
    .await
    .unwrap();

    // The lane's own write landed.
    assert_eq!(stage(&conn, &widget).as_deref(), Some("Review"));
    // The replaced action (create_task) was skipped - no duplicate work.
    assert_eq!(
        task_count(&conn, &widget),
        0,
        "the create_task action the step replaces must not fire"
    );
    // The action the step did NOT replace still ran.
    assert_eq!(
        note(&conn, &widget).as_deref(),
        Some("touched"),
        "a non-replaced action still fires"
    );

    // The skip is recorded on the workflow run, not hidden.
    let runs = workflow_service::list_runs(&conn, &ws, &wf, Some(&admin)).unwrap();
    assert!(
        runs.iter().any(|r| {
            let s = r.actions_summary.as_deref().unwrap_or("").to_lowercase();
            s.contains("skipped") && s.contains("process lane")
        }),
        "a run summary names the skipped, lane-replaced action: {:?}",
        runs.iter()
            .map(|r| r.actions_summary.clone())
            .collect::<Vec<_>>()
    );
}

// --------------------------------------------------------------------------
// AC-2: the same change made outside a lane fires the workflow in full.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-013 (WP-0709 AC-2)
#[tokio::test]
async fn the_same_change_outside_a_lane_fires_the_workflow_in_full() {
    let (conn, ws, admin) = setup();
    seed(&conn, &ws, &admin);
    let widget = make_widget(&conn, &ws, &admin, "W2");
    let wf = make_workflow(&conn, &ws, &admin);
    // A published lane that replaces the action still exists in the
    // workspace, to prove suppression is scoped to the lane's own write.
    publish_lane(
        &conn,
        &ws,
        &admin,
        json!([{"workflow_id": wf, "action_index": 0}]),
    );

    // Change stage directly, not through the lane.
    set_stage_directly(&conn, &admin, &widget, "Review");

    // Every action fires: the task is created AND the note is set.
    assert_eq!(
        task_count(&conn, &widget),
        1,
        "a non-lane change fires every workflow action, including the one a lane step would replace"
    );
    assert_eq!(note(&conn, &widget).as_deref(), Some("touched"));
}

/// Publishes a minimal published lane `lane-followup` on `widget`: one form
/// step a person submits, writing nothing to the record (so it cannot loop
/// with the workflow that starts it).
fn publish_followup_lane(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-followup", "name": "Follow up", "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": [{"key": "outcome", "type": "string", "required": false}]},
            "stages": [{
                "id": "do", "name": "Do", "steps": [{
                    "id": "review", "name": "Review", "kind": "form",
                    "role": "r", "required": true,
                    "fields": [{
                        "key": "outcome", "label": "Outcome", "type": "string",
                        "binds_to": "run_data.outcome", "required": false
                    }]
                }]
            }]
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, "lane-followup", Some(admin)).unwrap();
}

// --------------------------------------------------------------------------
// AC-4: a workflow starts a lane run on the triggering record.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-013 (WP-0709 AC-4)
#[tokio::test]
async fn a_workflow_action_starts_a_lane_run_on_the_triggering_record() {
    let (conn, ws, admin) = setup();
    seed(&conn, &ws, &admin);
    publish_followup_lane(&conn, &ws, &admin);
    let widget = make_widget(&conn, &ws, &admin, "W3");

    // A workflow on widget: when `stage` changes, start the follow-up lane on
    // the record.
    workflow_service::create_rule(
        &conn,
        &ws,
        &WorkflowDefinitionInput {
            app_id: None,
            entity_type: "widget".into(),
            name: "Stage change starts follow-up".into(),
            description: None,
            trigger_type: "field_changed".into(),
            trigger_status: None,
            trigger_field_key: Some("stage".into()),
            trigger_field_source: "custom".into(),
            trigger_offset_days: 0,
            match_type: "all".into(),
            priority: 0,
            conditions: vec![],
            actions: vec![WorkflowActionInput {
                action_type: "start_lane".into(),
                params_json: json!({"lane_key": "lane-followup"}).to_string(),
            }],
        },
        Some(&admin),
    )
    .unwrap();

    // No lane run yet.
    assert!(
        process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &widget)
            .unwrap()
            .runs
            .is_empty()
    );

    // Change stage - the workflow fires and enqueues the lane start.
    set_stage_directly(&conn, &admin, &widget, "Done");

    // Draining the pending queue starts the lane as a real lane run on the
    // record (anchor + run data), parked on its first person step.
    let drained = graph_runtime_service::drain_pending_graph_runs(&conn, &ws, &master_key(), 10)
        .await
        .unwrap();
    assert_eq!(drained, 1, "the enqueued lane start is drained");

    let runs = process_lane_service::list_lane_runs_for_record(&conn, &ws, "widget", &widget)
        .unwrap()
        .runs;
    assert_eq!(
        runs.len(),
        1,
        "a lane run was started on the triggering record"
    );
    assert_eq!(runs[0].lane_id, "lane-followup");
    assert_eq!(runs[0].status, "waiting_person");
}
