//! AI Agent Platform v2, Phase 7: Process Lane definitions in the core
//! (WP-0703) - import, publish, compile, version pinning, upgrade and
//! import validation of the lane model (ADR-0023, ADR-0024).
//!
//! Untested locally - see CI (no Rust toolchain in this VM).

use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    custom_object_service, custom_record_service, execution_graph_service, process_lane_service,
    workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
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

fn example_path() -> String {
    format!(
        "{}/../../program/spec/schemas/examples/lane-wh-receiving.json",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn example_json() -> String {
    std::fs::read_to_string(example_path()).expect("demo lane example exists")
}

fn table_columns(conn: &rusqlite::Connection, table: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

/// Covers: REQ-LANE-001 (WP-0703 AC-1), REQ-LANE-002 (WP-0703 AC-1)
#[test]
fn published_lane_compiles_to_a_graph_that_passes_validate_for_publish() {
    let (conn, ws, admin) = setup_workspace();
    let json = example_json();

    // AC-3: the demo lane imports unchanged - the stored definition is the
    // file's bytes, same schema, no rewriting.
    let lane =
        process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).unwrap();
    assert_eq!(lane.lane_key, "lane-wh-receiving");
    assert_eq!(lane.object_key, "purchase_order");
    assert_eq!(lane.versions.len(), 1);
    assert_eq!(lane.versions[0].version, 1);
    assert_eq!(lane.versions[0].status, "draft");
    assert_eq!(lane.versions[0].definition_json, json);

    let published =
        process_lane_service::publish_lane(&conn, &ws, "lane-wh-receiving", Some(&admin)).unwrap();
    assert_eq!(published.status, "published");
    let graph_id = published
        .graph_id
        .clone()
        .expect("publish compiles a graph");
    let graph = execution_graph_service::get(&conn, &graph_id, &ws, Some(&admin)).unwrap();
    assert_eq!(graph.source_kind.as_deref(), Some("lane"));
    assert_eq!(graph.source_id.as_deref(), Some("lane-wh-receiving@v1"));

    // The publish itself already ran validate_for_publish (see
    // `create_from_source`); run it again on the stored graph directly.
    execution_graph_service::validate_for_publish(&graph).unwrap();

    // 12 steps + 5 repeating steps wrapped in a loop node + trigger + end.
    assert_eq!(graph.nodes.len(), 19);
    assert_eq!(graph.edges.len(), 25);

    let node_by_key = |key: &str| {
        graph
            .nodes
            .iter()
            .find(|n| n.node_key == key)
            .unwrap_or_else(|| panic!("node {key} exists"))
    };
    // Step -> node mapping (ADR-0024 section 3).
    assert_eq!(
        node_by_key("step:verify:confirm-arrival").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:verify:match-delivery-note").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:record-receipt:enter-quantities").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:record-receipt:discrepancy-check").node_type,
        "condition"
    );
    assert_eq!(
        node_by_key("step:hold-review:review-discrepancy").node_type,
        "approval"
    );
    assert_eq!(
        node_by_key("step:hold-review:record-hold").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:stage-put-away:assign-staging-bin").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:stage-put-away:post-receipt-moves").node_type,
        "action"
    );
    assert_eq!(
        node_by_key("step:stage-put-away:confirm-putaway-bins").node_type,
        "human_step"
    );
    assert_eq!(
        node_by_key("step:stage-put-away:post-putaway-moves").node_type,
        "action"
    );
    assert_eq!(
        node_by_key("step:confirm:review-summary").node_type,
        "human_step"
    );
    assert_eq!(node_by_key("step:confirm:close-po").node_type, "action");

    // Every step node carries its lane position; stages are grouping only.
    for node in &graph.nodes {
        if node.node_key.starts_with("step:") || node.node_key.starts_with("loop:") {
            let config: serde_json::Value = serde_json::from_str(&node.config_json).unwrap();
            assert!(config.get("lane_stage_id").is_some(), "{}", node.node_key);
            assert!(config.get("lane_step_id").is_some(), "{}", node.node_key);
        }
    }
    let stage_nodes = |stage: &str| {
        graph
            .nodes
            .iter()
            .filter(|n| {
                serde_json::from_str::<serde_json::Value>(&n.config_json)
                    .ok()
                    .and_then(|c| c.get("lane_stage_id").cloned())
                    .and_then(|v| v.as_str().map(str::to_string))
                    .as_deref()
                    == Some(stage)
            })
            .count()
    };
    assert_eq!(stage_nodes("verify"), 3); // 2 steps + 1 loop
    assert_eq!(stage_nodes("record-receipt"), 3);
    assert_eq!(stage_nodes("hold-review"), 2);
    assert_eq!(stage_nodes("stage-put-away"), 7); // 4 steps + 3 loops
    assert_eq!(stage_nodes("confirm"), 2);

    // Repeat steps are wrapped in a loop with the safety bound.
    for loop_key in [
        "loop:verify:match-delivery-note",
        "loop:record-receipt:enter-quantities",
        "loop:stage-put-away:post-receipt-moves",
        "loop:stage-put-away:confirm-putaway-bins",
        "loop:stage-put-away:post-putaway-moves",
    ] {
        let loop_node = node_by_key(loop_key);
        assert_eq!(loop_node.node_type, "loop");
        let config: serde_json::Value = serde_json::from_str(&loop_node.config_json).unwrap();
        assert_eq!(config["max_iterations"], 1000);
        assert!(config.get("repeat_over").is_some());
    }

    // The check's pass/fail outcomes are the condition node's true/false
    // edges; the approval's outcomes are its approved/rejected edges.
    let edge_to = |from: &str, label: Option<&str>| {
        let edge = graph
            .edges
            .iter()
            .find(|e| {
                graph
                    .nodes
                    .iter()
                    .any(|n| n.id == e.from_node_id && n.node_key == from)
                    && e.branch_label.as_deref() == label
            })
            .unwrap_or_else(|| panic!("edge {from} --{label:?}--> ? exists"));
        graph
            .nodes
            .iter()
            .find(|n| n.id == edge.to_node_id)
            .unwrap()
            .node_key
            .clone()
    };
    assert_eq!(
        edge_to("step:record-receipt:discrepancy-check", Some("true")),
        "step:stage-put-away:assign-staging-bin"
    );
    assert_eq!(
        edge_to("step:record-receipt:discrepancy-check", Some("false")),
        "step:hold-review:review-discrepancy"
    );
    assert_eq!(
        edge_to("step:hold-review:review-discrepancy", Some("approved")),
        "step:stage-put-away:assign-staging-bin"
    );
    assert_eq!(
        edge_to("step:hold-review:review-discrepancy", Some("rejected")),
        "step:hold-review:record-hold"
    );
    // The hold step ends the run; the last step ends it too.
    assert_eq!(edge_to("step:hold-review:record-hold", None), "end");
    assert_eq!(edge_to("step:confirm:close-po", None), "end");
}

/// Covers: REQ-LANE-001 (WP-0703 AC-1), REQ-LANE-002 (WP-0703 AC-1)
#[test]
fn agent_step_compiles_to_an_agent_node() {
    let (conn, ws, admin) = setup_workspace();
    let json = serde_json::json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-agent-check",
            "name": "Agent check",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "stages": [{
                "id": "verify",
                "name": "Verify",
                "steps": [{
                    "id": "suggest-bin",
                    "name": "Suggest a bin",
                    "kind": "agent",
                    "assistance": {"autonomy": "propose", "agent": "wh-agt-receiving"},
                    "required": true,
                    "role": "role_wh_receiver"
                }]
            }]
        }
    })
    .to_string();
    let lane =
        process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).unwrap();
    assert_eq!(lane.lane_key, "lane-agent-check");
    let published =
        process_lane_service::publish_lane(&conn, &ws, "lane-agent-check", Some(&admin)).unwrap();
    let graph =
        execution_graph_service::get(&conn, &published.graph_id.unwrap(), &ws, Some(&admin))
            .unwrap();
    execution_graph_service::validate_for_publish(&graph).unwrap();
    let agent = graph
        .nodes
        .iter()
        .find(|n| n.node_key == "step:verify:suggest-bin")
        .unwrap();
    assert_eq!(agent.node_type, "agent");
    let config: serde_json::Value = serde_json::from_str(&agent.config_json).unwrap();
    assert_eq!(config["lane_stage_id"], "verify");
    assert_eq!(config["lane_step_id"], "suggest-bin");
}

/// A lane with one check step and no human steps, so the run can actually
/// finish on the graph runtime (human-step execution is WP-0704's).
fn check_only_lane_json(version: i64, step_name: &str) -> String {
    serde_json::json!({
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
                    "name": step_name,
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

/// Covers: REQ-LANE-001 (WP-0703 AC-2), REQ-LANE-002 (WP-0703 AC-2)
#[tokio::test]
async fn run_started_before_a_new_version_finishes_on_its_own_version() {
    let (conn, ws, admin) = setup_workspace();
    let object = custom_object_service::create_with_key(
        &conn,
        &ws,
        "purchase_order",
        &CustomObjectDefinitionInput {
            singular_label: "Purchase Order".into(),
            plural_label: "Purchase Orders".into(),
            icon: "▣".into(),
            prefix: "PO".into(),
            digits: 6,
        },
        Some(&admin),
    )
    .unwrap();
    let record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput {
            object_key: object.key.clone(),
            primary_name: "PO-2026-001".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(&admin),
    )
    .unwrap();

    process_lane_service::import_lane_definition(
        &conn,
        &ws,
        &check_only_lane_json(1, "Status check"),
        Some(&admin),
    )
    .unwrap();
    let v1 =
        process_lane_service::publish_lane(&conn, &ws, "lane-check-only", Some(&admin)).unwrap();
    let graph_v1 = v1.graph_id.clone().unwrap();

    // Started before v2 exists; finishes on v1.
    let run1 = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-check-only",
        &record.id,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run1.graph_id, graph_v1);
    assert_eq!(run1.status, "completed");

    // Publish v2 (same lane, changed step name) - the v1 run keeps v1.
    process_lane_service::create_version(
        &conn,
        &ws,
        "lane-check-only",
        &check_only_lane_json(2, "Status check v2"),
        Some(&admin),
    )
    .unwrap();
    let v2 =
        process_lane_service::publish_lane(&conn, &ws, "lane-check-only", Some(&admin)).unwrap();
    let graph_v2 = v2.graph_id.clone().unwrap();
    assert_ne!(graph_v1, graph_v2);
    let run1_after = lanesra_core::repositories::graph_run_repo::get_run(&conn, &run1.id)
        .unwrap()
        .unwrap();
    assert_eq!(run1_after.graph_id, graph_v1);
    assert_eq!(run1_after.status, "completed");

    // A new run starts on v2.
    let run2 = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-check-only",
        &record.id,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(run2.graph_id, graph_v2);
    assert_eq!(run2.status, "completed");

    // The run carries its anchor and its (empty) run data.
    assert_eq!(run2.anchor_object_key.as_deref(), Some("purchase_order"));
    assert_eq!(run2.anchor_record_id.as_deref(), Some(record.id.as_str()));
    assert_eq!(run2.run_data_json.as_deref(), Some("{}"));
}

/// Covers: REQ-LANE-001 (WP-0703 AC-2)
#[tokio::test]
async fn start_lane_run_refuses_a_second_active_run_on_the_same_record() {
    let (conn, ws, admin) = setup_workspace();
    let object = custom_object_service::create_with_key(
        &conn,
        &ws,
        "purchase_order",
        &CustomObjectDefinitionInput {
            singular_label: "Purchase Order".into(),
            plural_label: "Purchase Orders".into(),
            icon: "▣".into(),
            prefix: "PO".into(),
            digits: 6,
        },
        Some(&admin),
    )
    .unwrap();
    let record = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput {
            object_key: object.key.clone(),
            primary_name: "PO-2026-001".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(&admin),
    )
    .unwrap();
    // A lane whose only step is an approval: the run waits for the manager
    // (status `waiting_approval`), so it stays active - the one-active-run
    // rule (ADR-0024 section 6) can be proven end to end.
    let json = serde_json::json!({
        "format_version": "1.0.0",
        "lane": {
            "id": "lane-approval-first",
            "name": "Approval first",
            "object_key": "purchase_order",
            "version": 1,
            "status": "draft",
            "stages": [{
                "id": "verify",
                "name": "Verify",
                "steps": [{
                    "id": "review-discrepancy",
                    "name": "Review discrepancy",
                    "kind": "approval",
                    "required": true,
                    "role": "role_wh_warehouse_manager",
                    "writes": [{"action_type": "purchase_order.set_status", "inputs": {"status": "PARTIALLY_RECEIVED"}}],
                    "branches": [
                        {"on": "approved", "end": {"outcome": "done"}},
                        {"on": "rejected", "end": {"outcome": "held"}}
                    ]
                }]
            }]
        }
    })
    .to_string();
    process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).unwrap();
    process_lane_service::publish_lane(&conn, &ws, "lane-approval-first", Some(&admin)).unwrap();
    let first = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-approval-first",
        &record.id,
        Some(&admin),
    )
    .await
    .unwrap();
    assert_eq!(first.status, "waiting_approval");
    let second = process_lane_service::start_lane_run(
        &conn,
        &ws,
        &master_key(),
        "lane-approval-first",
        &record.id,
        Some(&admin),
    )
    .await;
    let err = second.expect_err("a second active run on the same record is refused");
    assert!(
        matches!(err, lanesra_core::domain::AppError::Conflict(_)),
        "expected a conflict, got {err:?}"
    );
}

/// Covers: REQ-LANE-001 (WP-0703 AC-4)
#[test]
fn upgrade_creates_lane_tables_and_anchor_columns() {
    let (conn, _, _) = setup_workspace();
    let lane_columns = table_columns(&conn, "process_lanes");
    for expected in [
        "id",
        "workspace_id",
        "lane_key",
        "name",
        "object_key",
        "created_at",
    ] {
        assert!(lane_columns.contains(&expected.to_string()), "{expected}");
    }
    let version_columns = table_columns(&conn, "process_lane_versions");
    for expected in [
        "id",
        "lane_id",
        "workspace_id",
        "version",
        "status",
        "definition_json",
        "graph_id",
    ] {
        assert!(
            version_columns.contains(&expected.to_string()),
            "{expected}"
        );
    }
    let run_columns = table_columns(&conn, "ai_runs");
    for expected in ["anchor_object_key", "anchor_record_id", "run_data_json"] {
        assert!(run_columns.contains(&expected.to_string()), "{expected}");
    }
    let run_node_columns = table_columns(&conn, "ai_run_nodes");
    assert!(run_node_columns.contains(&"stage_id".to_string()));
}

fn mutated_example(mutate: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut value: serde_json::Value =
        serde_json::from_str(&example_json()).expect("example parses");
    mutate(&mut value);
    serde_json::to_string(&value).unwrap()
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_check_step_with_a_role() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][1]["steps"][1]["role"] = serde_json::json!("role_wh_receiver");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_form_step_with_writes() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][0]["steps"][0]["writes"] =
            serde_json::json!([{"action_type": "warehouse.post_stock_move", "inputs": {}}]);
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_branch_with_no_exit() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][1]["steps"][1]["branches"][0]
            .as_object_mut()
            .unwrap()
            .remove("target");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_subtraction_expression() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][1]["steps"][0]["validation"]["expression"] =
            serde_json::json!("$item.qty_received - $item.qty_rejected <= $item.qty_ordered");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_an_approval_step_without_branches() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][2]["steps"][0]
            .as_object_mut()
            .unwrap()
            .remove("branches");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_record_step_without_writes() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][3]["steps"][1]
            .as_object_mut()
            .unwrap()
            .remove("writes");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_an_approval_step_without_writes() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["stages"][2]["steps"][0]
            .as_object_mut()
            .unwrap()
            .remove("writes");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}

/// Covers: REQ-LANE-001 (WP-0703 AC-3)
#[test]
fn import_refuses_a_lane_id_without_the_lane_prefix() {
    let (conn, ws, admin) = setup_workspace();
    let json = mutated_example(|v| {
        v["lane"]["id"] = serde_json::json!("receiving");
    });
    assert!(process_lane_service::import_lane_definition(&conn, &ws, &json, Some(&admin)).is_err());
}
