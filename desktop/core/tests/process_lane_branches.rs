//! Phase 7, WP-0706: approvals, branches with exits, pause, handoff and
//! cancel (ADR-0023; REQ-LANE-008, -009, -010, -012).
//!
//! Covers:
//!   AC-1  an approval step waits in the inbox and each response routes the
//!         run along its declared branch
//!   AC-2  a lane whose hold/rejection branch has no way out cannot be
//!         published, and the message names the step
//!   AC-3  a second run of the same lane on the same record is refused with
//!         the active run; a different lane may start unless excluded
//!   AC-4  cancelling after writes leaves them in place and records it in the
//!         audit trail
//!   AC-5  handoff changes who can act and is audited
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
        business_name: "Lane Branch Co".into(),
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

fn publish(conn: &rusqlite::Connection, ws: &str, admin: &str, id: &str, stages: Value) {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": "Lane", "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": []}, "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin)).unwrap();
    process_lane_service::publish_lane(conn, ws, id, Some(admin)).unwrap();
}

fn try_publish(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    id: &str,
    stages: Value,
) -> Result<(), String> {
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": id, "name": "Lane", "object_key": "widget",
            "version": 1, "status": "published",
            "run_data": {"fields": []}, "stages": stages
        }
    });
    process_lane_service::import_lane_definition(conn, ws, &lane.to_string(), Some(admin))
        .map_err(|e| e.to_string())?;
    process_lane_service::publish_lane(conn, ws, id, Some(admin)).map_err(|e| e.to_string())?;
    Ok(())
}

// --------------------------------------------------------------------------
// AC-1: an approval waits in the inbox and each response routes the run.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-008 (WP-0706 AC-1)
#[tokio::test]
async fn an_approval_routes_the_run_along_its_declared_branch() {
    for (approve, want_step) in [(true, "approved-path"), (false, "rejected-path")] {
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
        publish(&conn, &ws, &admin, "lane-approval", stages);

        let run = process_lane_service::start_lane_run(
            &conn,
            &ws,
            &master_key(),
            "lane-approval",
            &record,
            Some(&admin),
        )
        .await
        .unwrap();
        // The approval step holds the run in the inbox.
        assert_eq!(run.status, "waiting_approval");

        let routed = process_lane_service::resolve_lane_approval(
            &conn,
            &ws,
            &master_key(),
            &run.id,
            approve,
            Some("decision"),
            Some(&admin),
        )
        .await
        .unwrap();

        // The run is now waiting on the step the chosen branch names.
        assert_eq!(routed.status, "waiting_person");
        let node_id = routed.current_node_id.clone().unwrap();
        let graph = lanesra_core::repositories::execution_graph_repo::get(&conn, &routed.graph_id)
            .unwrap()
            .unwrap();
        let current = graph.nodes.iter().find(|n| n.id == node_id).unwrap();
        let step: Value = serde_json::from_str(&current.config_json).unwrap();
        assert_eq!(
            step["lane_step_id"],
            json!(want_step),
            "approve={approve} routed to the wrong branch"
        );
    }
}

// --------------------------------------------------------------------------
// AC-2: a dead-end hold/rejection branch cannot be published; names the step.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-012 (WP-0706 AC-2)
#[tokio::test]
async fn a_dead_end_branch_blocks_publish_and_names_the_step() {
    let (conn, ws, admin) = setup();
    seed(&conn, &ws, &admin);

    // The rejection branch leads to a hold check that only ever loops back to
    // itself - no path from it reaches the end of the lane.
    let stages = json!([
        {"id": "decide", "name": "Decide", "steps": [{
            "id": "decide", "name": "Decide", "kind": "approval", "role": "r",
            "required": true,
            "writes": [{"action_type": "noop", "inputs": {}}],
            "branches": [
                {"on": "approved", "end": {"outcome": "done"}},
                {"on": "rejected", "target": {"stage": "hold", "step": "stuck"}}
            ]
        }]},
        {"id": "hold", "name": "Hold", "steps": [{
            "id": "stuck", "name": "Stuck", "kind": "check", "required": true,
            "check": {"scope": "anchor", "message": "stuck",
                      "condition": {"field": "$record.tracking", "operator": "equals", "value": "x"}},
            "branches": [
                {"on": "pass", "target": {"stage": "hold", "step": "stuck"}},
                {"on": "fail", "target": {"stage": "hold", "step": "stuck"}}
            ]
        }]}
    ]);

    let err = try_publish(&conn, &ws, &admin, "lane-deadend", stages).unwrap_err();
    assert!(
        err.contains("stuck"),
        "the message names the dead-end step; got: {err}"
    );
    assert!(err.contains("no way out"), "got: {err}");
}

// --------------------------------------------------------------------------
// AC-3: one active run per lane per record; a different lane may start unless
// excluded.
// --------------------------------------------------------------------------

fn one_form_stage() -> Value {
    json!([{"id": "s", "name": "S", "steps": [{
        "id": "f", "name": "F", "kind": "form", "role": "r", "required": true,
        "fields": [{"key": "n", "label": "N", "type": "string", "binds_to": "run_data.n", "required": false}]
    }]}])
}

// Covers: REQ-LANE-012 (WP-0706 AC-3)
#[tokio::test]
async fn one_active_run_per_lane_per_record_and_exclusions() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    publish(&conn, &ws, &admin, "lane-a", one_form_stage());

    // Lane B excludes lane A.
    let b_lane = json!({
        "format_version": "1.0.0",
        "lane": {"id": "lane-b", "name": "B", "object_key": "widget", "version": 1,
                 "status": "published", "run_data": {"fields": []},
                 "excludes": ["lane-a"], "stages": one_form_stage()}
    });
    process_lane_service::import_lane_definition(&conn, &ws, &b_lane.to_string(), Some(&admin))
        .unwrap();
    process_lane_service::publish_lane(&conn, &ws, "lane-b", Some(&admin)).unwrap();

    let run_a = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-a",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // A second run of lane A on the same record is refused, naming the active one.
    let err = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-a",
        &record,
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err}").contains(&run_a.id),
        "the refusal links the active run; got: {err}"
    );

    // Lane B excludes lane A, so it cannot start on the same record either.
    let err = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-b",
        &record,
        Some(&admin),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err}").to_lowercase().contains("exclud"),
        "got: {err}"
    );

    // A different, non-excluded lane C may start on the same record.
    publish(&conn, &ws, &admin, "lane-c", one_form_stage());
    let run_c = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-c",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run_c.status, "waiting_person");
}

// --------------------------------------------------------------------------
// AC-4: cancelling after a write leaves it in place and records it in audit.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-010 (WP-0706 AC-4)
#[tokio::test]
async fn cancel_leaves_writes_and_records_it_in_the_audit_trail() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);

    // f1 writes a record field, then the run waits at f2.
    let stages = json!([
        {"id": "s1", "name": "S1", "steps": [{
            "id": "f1", "name": "F1", "kind": "form", "role": "r", "required": true,
            "fields": [{"key": "trk", "label": "Trk", "type": "string",
                        "binds_to": "record.tracking", "required": true}]
        }]},
        {"id": "s2", "name": "S2", "steps": [{
            "id": "f2", "name": "F2", "kind": "form", "role": "r", "required": true, "fields": []
        }]}
    ]);
    publish(&conn, &ws, &admin, "lane-cancel", stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-cancel",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();
    let mut values: Map<String, Value> = Map::new();
    values.insert("trk".into(), json!("TRK-1"));
    let run = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &values,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_person"); // now at f2, after the write

    let cancelled = process_lane_service::cancel_lane_run(
        &conn,
        &ws,
        &run.id,
        "no longer needed",
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(cancelled.status, "cancelled");

    // The write is left in place - a cancel never rolls a posted write back.
    assert_eq!(
        custom_field_service::get_entity_values(&conn, &record)
            .unwrap()
            .get("tracking")
            .map(String::as_str),
        Some("TRK-1")
    );
    // The cancellation is in the audit trail.
    let audit =
        lanesra_core::repositories::audit_repo::list_for_entity(&conn, "widget", &record).unwrap();
    assert!(
        audit.iter().any(|e| e.event_type == "lane_run_cancelled"),
        "the cancellation is recorded in the audit trail"
    );
}

// --------------------------------------------------------------------------
// AC-5: handoff changes who can act, and is audited.
// --------------------------------------------------------------------------

/// Makes a role plus a user who holds only it (so the access layer does not
/// bootstrap them a default role).
fn role_and_holder(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    name: &str,
    username: &str,
) -> (String, String) {
    let role = access_role_service::create(
        conn,
        ws,
        &AccessRoleInput {
            name: name.into(),
            description: name.into(),
        },
        Some(admin),
    )
    .unwrap();
    let user = user_service::create(
        conn,
        ws,
        &NewUser {
            username: username.into(),
            display_name: name.into(),
            password: "a-strong-password".into(),
            roles: vec!["Manager".into()],
        },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, &user.id, &role.id, Some(admin)).unwrap();
    for r in access_role_service::list_roles_for_user(conn, &user.id).unwrap() {
        if r.id != role.id {
            access_role_service::remove_from_user(conn, &user.id, &r.id, Some(admin)).unwrap();
        }
    }
    (role.id, user.id)
}

// Covers: REQ-LANE-009 (WP-0706 AC-5)
#[tokio::test]
async fn handoff_changes_who_can_act_and_is_audited() {
    let (conn, ws, admin) = setup();
    let record = seed(&conn, &ws, &admin);
    let (role_a, user_a) = role_and_holder(&conn, &ws, &admin, "Role A", "usera");
    let (role_b, user_b) = role_and_holder(&conn, &ws, &admin, "Role B", "userb");

    let stages = json!([{"id": "s", "name": "S", "steps": [{
        "id": "approve", "name": "Approve", "kind": "form", "role": role_a,
        "required": true, "fields": []
    }]}]);
    publish(&conn, &ws, &admin, "lane-handoff", stages);

    let run = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-handoff",
        &record,
        Some(&admin),
    )
    .await
    .unwrap();

    // Before handoff: A can act, B cannot.
    let view_a = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&user_a)).unwrap();
    assert!(view_a.current.unwrap().can_complete);
    let view_b = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&user_b)).unwrap();
    assert!(!view_b.current.unwrap().can_complete);

    // Hand the step from A to B (A is the current assignee and may hand off).
    process_lane_service::handoff_step(
        &conn,
        &ws,
        &run.id,
        &role_b,
        "please take this",
        Some(&user_a),
    )
    .unwrap();

    // After handoff: B can act, A cannot; the step now waits on B.
    let view_a = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&user_a)).unwrap();
    let step_a = view_a.current.unwrap();
    assert!(!step_a.can_complete, "A no longer may act after handoff");
    assert_eq!(step_a.waiting_on.as_deref(), Some(role_b.as_str()));
    let view_b = process_lane_service::get_run_view(&conn, &ws, &run.id, Some(&user_b)).unwrap();
    assert!(
        view_b.current.unwrap().can_complete,
        "B may act after handoff"
    );

    // A's submit is now refused; B's succeeds.
    let err = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &Map::new(),
        Some(&user_a),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{err}").to_lowercase().contains("cannot complete"),
        "got: {err}"
    );
    let done = process_lane_service::submit_person_step(
        &conn,
        &ws,
        &master_key(),
        &run.id,
        &Map::new(),
        Some(&user_b),
    )
    .await
    .unwrap();
    assert_eq!(done.status, "completed");

    // The handoff is audited.
    let audit =
        lanesra_core::repositories::audit_repo::list_for_entity(&conn, "widget", &record).unwrap();
    assert!(audit.iter().any(|e| e.event_type == "lane_step_handoff"));
}
