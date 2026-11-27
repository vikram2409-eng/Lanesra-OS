//! AI Agent Platform v2, Phase 7: Process Lane commands over HTTP
//! (WP-0703 AC-5) - the Team Workspace server exposes the same lane
//! operations as the desktop Tauri commands, over the same service code.
//! (WP-0710 AC-4) - the per-item submit runs over the admin route too, so
//! the desktop and Team Workspace editions behave the same from the same code.
//! (WP-0711 AC-4) - notes, attachments and reassign run over the admin
//! routes too, and the run view over HTTP carries them.
//! (WP-0713 AC-5) - the board, my-steps, timing and ageing reads run over
//! the admin routes too, from the same service code as the desktop
//! Tauri commands.
//! Import/publish/list/get go through the sync `/api/invoke` dispatcher;
//! starting a run goes through the async admin route, which opens its own
//! connection (hence the file-backed DB, as in `agent_v1.rs`).
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::models::access_role::AccessRoleInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::services::{
    access_role_service, custom_field_service, custom_object_service, custom_record_service,
    relationship_service, user_service,
};
use lanesra_server::state::SharedState;
use lanesra_server::{build_router, SecurityConfig, ServerState};
use serde_json::{json, Value};

async fn spawn_file_backed_server() -> (SocketAddr, SharedState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let conn = open_workspace_db(&db_path).unwrap();
    let state = ServerState::new(conn, db_path, SecurityConfig::default());
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state.clone(), frontend_dir);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state, dir)
}

fn client_with_cookies() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn invoke(client: &reqwest::Client, addr: SocketAddr, command: &str, args: Value) -> Value {
    client
        .post(format!("http://{addr}/api/invoke/{command}"))
        .json(&args)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn first_run(client: &reqwest::Client, addr: SocketAddr) -> Value {
    invoke(
        client,
        addr,
        "first_run_setup",
        json!({
            "setup": {
                "business_name": "Lane HTTP Co", "legal_name": null, "currency_code": "USD", "locale": "en-US",
                "timezone": "UTC", "default_tax_rate_bp": 0, "admin_username": "admin",
                "admin_display_name": "Admin", "admin_password": "supersecretpw", "load_sample_data": false
            }
        }),
    )
    .await
}

fn check_only_lane_json(version: i64) -> String {
    json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-check-only",
            "name": "Check only",
            "object_key": "purchase_order",
            "version": version,
            "status": "draft",
            "start_condition": {"field": "$record.status", "operator": "equals", "value": "Active"},
            "stages": [{
                "id": "verify",
                "name": "Verify",
                "steps": [{
                    "id": "status-check",
                    "name": "Status check",
                    "kind": "check",
                    "check": {
                        "scope": "anchor",
                        "message": "must be active",
                        "condition": {"field": "$record.status", "operator": "equals", "value": "Active"}
                    },
                    "branches": [
                        {"on": "pass", "end": {"outcome": "done"}},
                        {"on": "fail", "end": {"outcome": "held"}}
                    ]
                }]
            }]
        }
    })
    .to_string()
}

/// Covers: REQ-LANE-001 (WP-0703 AC-5), REQ-LANE-002 (WP-0703 AC-5)
#[tokio::test]
async fn lane_commands_work_over_http_the_same_as_on_desktop() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // The demo lane imports unchanged over HTTP, exactly as in the core test.
    let example = std::fs::read_to_string(format!(
        "{}/../../program/spec/schemas/examples/lane-wh-receiving.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("demo lane example exists");
    let imported = invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": example}),
    )
    .await;
    assert_eq!(imported["data"]["lane_key"], "lane-wh-receiving");
    assert_eq!(imported["data"]["versions"][0]["definition_json"], example);

    // WP-0714 P-12: the demo Receiving lane's Action Types are not
    // adopted, so publish refuses over HTTP exactly as on desktop - the
    // error names the check. The check-only lane below still covers the
    // publish-success path.
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(published["ok"], false);
    assert!(
        published["error"].to_string().contains("P-12"),
        "the publish refusal names P-12: {}",
        published["error"]
    );

    let listed = invoke(&client, addr, "list_process_lanes", json!({})).await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    let got = invoke(
        &client,
        addr,
        "get_process_lane",
        json!({"laneKey": "lane-wh-receiving"}),
    )
    .await;
    assert_eq!(got["data"]["lane_key"], "lane-wh-receiving");

    // A run starts over the async admin route and finishes on its own
    // version, the same result the core test proves directly.
    let record_id = {
        let conn = state.conn.lock().unwrap();
        let object = custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "purchase_order",
            &CustomObjectDefinitionInput {
                singular_label: "Purchase Order".into(),
                plural_label: "Purchase Orders".into(),
                icon: "▣".into(),
                prefix: "PO".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: object.key.clone(),
                primary_name: "PO-2026-001".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap()
        .id
    };
    invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": check_only_lane_json(1)}),
    )
    .await;
    let v1 = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-check-only"}),
    )
    .await;
    let graph_v1 = v1["data"]["graph_id"].as_str().unwrap().to_string();
    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-check-only/runs"
        ))
        .json(&json!({"anchor_record_id": record_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true);
    assert_eq!(run["data"]["graph_id"], graph_v1);
    assert_eq!(run["data"]["status"], "completed");
    assert_eq!(run["data"]["anchor_object_key"], "purchase_order");
    assert_eq!(run["data"]["anchor_record_id"], record_id);
}

// --------------------------------------------------------------------------
// WP-0710 AC-4: the per-item submit works over the Team Workspace admin route
// exactly as the desktop Tauri command does - one item at a time, the run
// stays open until every item is done, and each item's value lands on its
// own child record. The fixture mirrors the core test's batch/tallyline
// pair (process_lane_repeat_conditions.rs).
// --------------------------------------------------------------------------

// Covers: REQ-LANE-017 (WP-0710 AC-4)
#[tokio::test]
async fn per_item_submit_works_over_http_the_same_as_on_desktop() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // A batch with two tally lines, linked through the `lines` relationship.
    // Each line carries a `counted` field (the per-item form writes it) and
    // a `picker` field (the per-item assignee, Q-33 `child_field`).
    let (batch_id, line_ids) = {
        let conn = state.conn.lock().unwrap();
        custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "batch",
            &CustomObjectDefinitionInput {
                singular_label: "Batch".into(),
                plural_label: "Batches".into(),
                icon: "▣".into(),
                prefix: "BAT".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "tallyline",
            &CustomObjectDefinitionInput {
                singular_label: "Tally line".into(),
                plural_label: "Tally lines".into(),
                icon: "▣".into(),
                prefix: "TLN".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        for key in ["counted", "picker"] {
            custom_field_service::create_definition_with_key(
                &conn,
                &workspace_id,
                key,
                &CustomFieldDefinitionInput {
                    entity_type: "tallyline".into(),
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
                Some(&admin_id),
            )
            .unwrap();
        }
        relationship_service::create(
            &conn,
            &workspace_id,
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
            Some(&admin_id),
        )
        .unwrap();
        let batch = custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: "batch".into(),
                primary_name: "BAT-1".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap();
        let def = relationship_service::list(&conn, &workspace_id, true)
            .unwrap()
            .into_iter()
            .find(|d| d.source_entity_type == "batch" && d.target_entity_type == "tallyline")
            .expect("lines relationship");
        let mut lines = Vec::new();
        for name in ["BAT-1-A", "BAT-1-B"] {
            let line = custom_record_service::create(
                &conn,
                &workspace_id,
                &CustomRecordInput {
                    object_key: "tallyline".into(),
                    primary_name: name.into(),
                    status: "Active".into(),
                    owner_user_id: None,
                    notes: None,
                },
                Some(&admin_id),
            )
            .unwrap();
            relationship_service::link(
                &conn,
                &workspace_id,
                &def.id,
                "batch",
                &batch.id,
                "tallyline",
                &line.id,
                Some(&admin_id),
            )
            .unwrap();
            lines.push(line.id);
        }
        (batch.id, lines)
    };

    // A repeating form step over `lines`, one field bound to each child's
    // `counted`, assigned per item from the child's `picker` field.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-count", "name": "Count", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": []},
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
                        "binds_to": "record.lines[].counted", "required": true
                    }]
                }]
            }]
        }
    });
    let imported = invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": lane.to_string()}),
    )
    .await;
    assert_eq!(imported["data"]["lane_key"], "lane-count");
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-count"}),
    )
    .await;
    assert_eq!(
        published["ok"], true,
        "the form-only lane publishes: {}",
        published
    );

    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-count/runs"
        ))
        .json(&json!({"anchor_record_id": batch_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true, "the run starts over HTTP: {}", run);
    let run_id = run["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(run["data"]["status"], "waiting_person");

    // One person completes the first line over HTTP. The step stays open:
    // its other item is still outstanding.
    let after_first = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/submit-item"
        ))
        .json(&json!({"item_id": line_ids[0], "values": {"counted": "40"}}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        after_first["ok"], true,
        "the first item submits: {}",
        after_first
    );
    assert_eq!(
        after_first["data"]["status"], "waiting_person",
        "the step is still open while an item is outstanding"
    );

    // The second line completes the step, and the run finishes.
    let after_second = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/submit-item"
        ))
        .json(&json!({"item_id": line_ids[1], "values": {"counted": "55"}}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        after_second["ok"], true,
        "the second item submits: {}",
        after_second
    );
    assert_eq!(after_second["data"]["status"], "completed");

    // Each item's value landed on its own child record.
    let conn = state.conn.lock().unwrap();
    for (line, expected) in [(&line_ids[0], "40"), (&line_ids[1], "55")] {
        let got = custom_field_service::get_entity_values(&conn, line)
            .unwrap()
            .get("counted")
            .cloned();
        assert_eq!(got.as_deref(), Some(expected));
    }
}

// --------------------------------------------------------------------------
// WP-0711 AC-4: notes, attachments and reassign work over the Team
// Workspace admin routes exactly as the desktop Tauri commands do - the
// same service code, the same run view. The fixture mirrors the core test
// (process_lane_anytime_actions.rs): one form step waiting on a role held
// by one user.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-018 (WP-0711 AC-4)
#[tokio::test]
async fn anytime_actions_work_over_http_the_same_as_on_desktop() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    // A batch record, a Counter role held by Casey, and an outsider (Ollie).
    let (batch_id, role_id, casey_id, ollie_id) = {
        let conn = state.conn.lock().unwrap();
        custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "batch",
            &CustomObjectDefinitionInput {
                singular_label: "Batch".into(),
                plural_label: "Batches".into(),
                icon: "▣".into(),
                prefix: "BAT".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        let batch = custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: "batch".into(),
                primary_name: "BAT-1".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap();
        let role = access_role_service::create(
            &conn,
            &workspace_id,
            &AccessRoleInput {
                name: "Counter".into(),
                description: "Counter role".into(),
            },
            Some(&admin_id),
        )
        .unwrap();
        let make_user = |username: &str, display_name: &str| {
            user_service::create(
                &conn,
                &workspace_id,
                &NewUser {
                    username: username.into(),
                    display_name: display_name.into(),
                    password: "correct horse battery staple".into(),
                    roles: vec!["Sales".into()],
                },
                Some(&admin_id),
            )
            .unwrap()
            .id
        };
        let casey = make_user("casey", "Casey Counter");
        let ollie = make_user("ollie", "Ollie Outsider");
        access_role_service::assign_to_user(&conn, &casey, &role.id, Some(&admin_id)).unwrap();
        (batch.id, role.id, casey, ollie)
    };

    // One form step waiting on the Counter role, writing one run-data field.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-simple", "name": "Simple", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": [{"key": "note_text", "type": "string", "required": false}]},
            "stages": [{
                "id": "s1", "name": "Step one", "steps": [{
                    "id": "fill", "name": "Fill", "kind": "form",
                    "role": role_id, "required": true,
                    "fields": [{
                        "key": "note_text", "label": "Note text", "type": "string",
                        "binds_to": "run_data.note_text", "required": true
                    }]
                }]
            }]
        }
    });
    let imported = invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": lane.to_string()}),
    )
    .await;
    assert_eq!(imported["data"]["lane_key"], "lane-simple");
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-simple"}),
    )
    .await;
    assert_eq!(
        published["ok"], true,
        "the form-only lane publishes: {}",
        published
    );

    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-simple/runs"
        ))
        .json(&json!({"anchor_record_id": batch_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true, "the run starts over HTTP: {}", run);
    let run_id = run["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(run["data"]["status"], "waiting_person");

    // A note over HTTP lands on the run and in its audit trail.
    let noted = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/note"
        ))
        .json(&json!({"body": "Checked the count against the pick list."}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(noted["ok"], true, "the note posts: {}", noted);
    assert_eq!(
        noted["data"]["body"],
        "Checked the count against the pick list."
    );

    // An attachment over HTTP lands with its bytes, downloadable through
    // the run.
    let attached = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/attachment"
        ))
        .json(&json!({
            "file_name": "count-sheet.txt",
            "mime_type": "text/plain",
            "content_base64": "Y291bnQgc2hlZXQ=",
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(attached["ok"], true, "the attachment posts: {}", attached);
    let attachment_id = attached["data"]["id"].as_str().unwrap().to_string();
    let downloaded = client
        .get(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/attachments/{attachment_id}"
        ))
        .send()
        .await
        .unwrap();
    assert!(downloaded.status().is_success());
    assert_eq!(downloaded.bytes().await.unwrap().as_ref(), b"count sheet");

    // Reassign over HTTP narrows the step to Casey; reassign to the
    // outsider is refused.
    let refused = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/reassign"
        ))
        .json(&json!({"to_user_id": ollie_id, "reason": "Ollie should cover"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(refused["ok"], false, "the outsider cannot take the step");
    let reassigned = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/reassign"
        ))
        .json(&json!({"to_user_id": casey_id, "reason": "Casey is on shift"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(reassigned["ok"], true, "the reassign posts: {}", reassigned);
    assert_eq!(reassigned["data"]["status"], "waiting_person");

    // The run view over HTTP shows the note, the attachment, the audit
    // events and the reassigned waiter - the same view the desktop shows.
    let view = invoke(
        &client,
        addr,
        "get_process_lane_run_view",
        json!({"runId": run_id}),
    )
    .await;
    assert_eq!(view["data"]["notes"].as_array().unwrap().len(), 1);
    assert_eq!(view["data"]["attachments"].as_array().unwrap().len(), 1);
    assert_eq!(
        view["data"]["current"]["waiting_on"], "Casey Counter",
        "the view names the assignee: {}",
        view["data"]["current"]
    );
    let event_types: Vec<&str> = view["data"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["event_type"].as_str())
        .collect();
    for expected in [
        "lane_run_note_added",
        "lane_run_attachment_added",
        "lane_step_reassigned",
    ] {
        assert!(
            event_types.contains(&expected),
            "the audit trail carries {expected}: {event_types:?}"
        );
    }
}

// --------------------------------------------------------------------------
// WP-0720 AC-2/AC-5: pause, resume and reopen run over the Team Workspace
// transport exactly as the desktop Tauri commands do - pause and resume
// through the sync `/api/invoke` dispatcher, reopen through its async
// admin route. The run parks on the same step while paused, and reopening
// walks it back to the earlier completed step in the same stage; every
// action is audited in the run view.
// --------------------------------------------------------------------------

/// Covers: REQ-LANE-009 (WP-0720 AC-2, AC-5)
#[tokio::test]
async fn pause_resume_reopen_work_over_http_the_same_as_on_desktop() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    let (batch_id, role_id) = {
        let conn = state.conn.lock().unwrap();
        custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "batch",
            &CustomObjectDefinitionInput {
                singular_label: "Batch".into(),
                plural_label: "Batches".into(),
                icon: "▣".into(),
                prefix: "BAT".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        let batch = custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: "batch".into(),
                primary_name: "BAT-1".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap();
        let role = access_role_service::create(
            &conn,
            &workspace_id,
            &AccessRoleInput {
                name: "Counter".into(),
                description: "Counter role".into(),
            },
            Some(&admin_id),
        )
        .unwrap();
        (batch.id, role.id)
    };

    // Two form steps in one stage, so the second can be reopened back to
    // the first without leaving the stage.
    let lane = json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-pause-reopen", "name": "Pause Reopen", "object_key": "batch",
            "version": 1, "status": "published",
            "run_data": {"fields": [{"key": "note_text", "type": "string", "required": false}]},
            "stages": [{
                "id": "s1", "name": "Step one", "steps": [
                    {
                        "id": "fill-first", "name": "Fill first", "kind": "form",
                        "role": role_id, "required": true,
                        "fields": [{
                            "key": "note_text", "label": "Note text", "type": "string",
                            "binds_to": "run_data.note_text", "required": true
                        }]
                    },
                    {
                        "id": "fill-second", "name": "Fill second", "kind": "form",
                        "role": role_id, "required": true,
                        "fields": [{
                            "key": "note_text", "label": "Note text", "type": "string",
                            "binds_to": "run_data.note_text", "required": true
                        }]
                    }
                ]
            }]
        }
    });
    let imported = invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": lane.to_string()}),
    )
    .await;
    assert_eq!(imported["data"]["lane_key"], "lane-pause-reopen");
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-pause-reopen"}),
    )
    .await;
    assert_eq!(
        published["ok"], true,
        "the two-step lane publishes: {}",
        published
    );

    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-pause-reopen/runs"
        ))
        .json(&json!({"anchor_record_id": batch_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true, "the run starts over HTTP: {}", run);
    let run_id = run["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(run["data"]["status"], "waiting_person");

    // Complete the first step, so the run waits on the second.
    let submitted = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/submit-step"
        ))
        .json(&json!({"values": {"note_text": "first"}}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        submitted["ok"], true,
        "the first step submits: {}",
        submitted
    );
    assert_eq!(submitted["data"]["status"], "waiting_person");

    // Pause through the sync dispatcher: the run parks on the same step.
    let paused = invoke(
        &client,
        addr,
        "pause_process_lane_run",
        json!({"runId": run_id, "reason": "Waiting for the manager"}),
    )
    .await;
    assert_eq!(paused["ok"], true, "the run pauses: {}", paused);
    assert_eq!(paused["data"]["status"], "paused");

    // A second pause is refused: the run is not waiting on a person.
    let paused_again = invoke(
        &client,
        addr,
        "pause_process_lane_run",
        json!({"runId": run_id, "reason": null}),
    )
    .await;
    assert_eq!(paused_again["ok"], false, "pausing a paused run is refused");

    // Resume through the sync dispatcher: back to the step it paused on.
    let resumed = invoke(
        &client,
        addr,
        "resume_process_lane_run",
        json!({"runId": run_id}),
    )
    .await;
    assert_eq!(resumed["ok"], true, "the run resumes: {}", resumed);
    assert_eq!(resumed["data"]["status"], "waiting_person");

    // Reopen the first step through its async admin route: the run walks
    // back to it, in the same stage.
    let reopened = client
        .post(format!(
            "http://{addr}/api/admin/process-lane-runs/{run_id}/reopen-step"
        ))
        .json(&json!({"step_id": "fill-first", "reason": "Wrong note entered"}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(reopened["ok"], true, "the first step reopens: {}", reopened);
    assert_eq!(reopened["data"]["status"], "waiting_person");

    // The run view shows the run back on the first step, and the audit
    // trail carries all three actions.
    let view = invoke(
        &client,
        addr,
        "get_process_lane_run_view",
        json!({"runId": run_id}),
    )
    .await;
    assert_eq!(
        view["data"]["current"]["step_id"], "fill-first",
        "the run is back on the first step: {}",
        view["data"]["current"]
    );
    let event_types: Vec<&str> = view["data"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["event_type"].as_str())
        .collect();
    for expected in [
        "lane_run_paused",
        "lane_run_resumed",
        "lane_run_step_reopened",
    ] {
        assert!(
            event_types.contains(&expected),
            "the audit trail carries {expected}: {event_types:?}"
        );
    }
}

// --------------------------------------------------------------------------
// WP-0713 AC-5: the board, my-steps, timing and ageing reads run over the
// Team Workspace admin routes exactly as the desktop Tauri commands do -
// the same service code, the same figures. The check-only lane's run
// completes immediately, so the board and ageing views are empty by
// design (they list active runs only) while timing counts the run.
// --------------------------------------------------------------------------

// Covers: REQ-LANE-019 (WP-0713 AC-5)
#[tokio::test]
async fn lane_board_my_steps_timing_ageing_work_over_http() {
    let (addr, state, _dir) = spawn_file_backed_server().await;
    let client = client_with_cookies();
    let setup = first_run(&client, addr).await;
    let workspace_id = setup["data"][0]["id"].as_str().unwrap().to_string();
    let admin_id = setup["data"][1]["id"].as_str().unwrap().to_string();

    let record_id = {
        let conn = state.conn.lock().unwrap();
        let object = custom_object_service::create_with_key(
            &conn,
            &workspace_id,
            "purchase_order",
            &CustomObjectDefinitionInput {
                singular_label: "Purchase Order".into(),
                plural_label: "Purchase Orders".into(),
                icon: "▣".into(),
                prefix: "PO".into(),
                digits: 6,
            },
            Some(&admin_id),
        )
        .unwrap();
        custom_record_service::create(
            &conn,
            &workspace_id,
            &CustomRecordInput {
                object_key: object.key.clone(),
                primary_name: "PO-2026-001".into(),
                status: "Active".into(),
                owner_user_id: None,
                notes: None,
            },
            Some(&admin_id),
        )
        .unwrap()
        .id
    };
    invoke(
        &client,
        addr,
        "import_process_lane",
        json!({"definitionJson": check_only_lane_json(1)}),
    )
    .await;
    let published = invoke(
        &client,
        addr,
        "publish_process_lane",
        json!({"laneKey": "lane-check-only"}),
    )
    .await;
    assert_eq!(published["ok"], true);
    let run = client
        .post(format!(
            "http://{addr}/api/admin/process-lanes/lane-check-only/runs"
        ))
        .json(&json!({"anchor_record_id": record_id}))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(run["ok"], true);
    assert_eq!(run["data"]["status"], "completed");

    async fn get_admin(client: &reqwest::Client, addr: SocketAddr, path: &str) -> Value {
        client
            .get(format!("http://{addr}{path}"))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
    }

    // Board: the lane's one stage, no active runs - the completed run is
    // excluded by design.
    let board = get_admin(
        &client,
        addr,
        "/api/admin/process-lanes/lane-check-only/board",
    )
    .await;
    assert_eq!(board["ok"], true);
    assert_eq!(board["data"]["lane_id"], "lane-check-only");
    assert_eq!(board["data"]["stages"].as_array().unwrap().len(), 1);
    assert_eq!(
        board["data"]["stages"][0]["stage_id"], "verify",
        "board stage: {board}"
    );
    assert_eq!(
        board["data"]["stages"][0]["runs"].as_array().unwrap().len(),
        0
    );

    // My steps: nothing waiting on the completed run.
    let mine = get_admin(&client, addr, "/api/admin/process-lanes/my-steps").await;
    assert_eq!(mine["ok"], true);
    assert_eq!(mine["data"]["items"].as_array().unwrap().len(), 0);

    // Timing: the one run completed, and the verify stage carries it.
    let timing = get_admin(
        &client,
        addr,
        "/api/admin/process-lanes/lane-check-only/timing",
    )
    .await;
    assert_eq!(timing["ok"], true);
    assert_eq!(timing["data"]["total_runs"], 1);
    assert_eq!(timing["data"]["completed_runs"], 1);
    assert_eq!(timing["data"]["completion_rate"], 1.0);
    assert_eq!(timing["data"]["stages"].as_array().unwrap().len(), 1);
    assert_eq!(timing["data"]["stages"][0]["stage_id"], "verify");
    assert_eq!(timing["data"]["stages"][0]["runs_completed"], 1);
    assert!(
        timing["data"]["stages"][0]["avg_seconds"].is_number(),
        "avg_seconds present: {timing}"
    );

    // Ageing: no active runs to age.
    let ageing = get_admin(
        &client,
        addr,
        "/api/admin/process-lanes/lane-check-only/ageing",
    )
    .await;
    assert_eq!(ageing["ok"], true);
    assert_eq!(ageing["data"].as_array().unwrap().len(), 0);
}
