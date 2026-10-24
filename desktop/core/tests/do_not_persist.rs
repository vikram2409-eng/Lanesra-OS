//! WP-0127 (REQ-LOOP-021): a node marked `do_not_persist` produces a value
//! the run can use but never stores. A planted secret is searched for in
//! every column of every table after a run (AC-1), and a run resumed after
//! a wait fetches the value again rather than reading it back (AC-2).
//!
//! Runs go through the scripted provider (WP-0005), with trace bodies set
//! to `full` so the strictest storage setting is the one tested.

use lanesra_core::models::ai::{AiAgentModelRouting, AiProviderInput};
use lanesra_core::models::ai_agent::{AiAgentDefinition, AiAgentInput};
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::run_trace::TraceBodiesInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::ai_service::{RequestedToolCall, TokenUsage};
use lanesra_core::services::scripted_provider::{self, ScriptedResponse};
use lanesra_core::services::{
    ai_agent_service, ai_provider_service, execution_graph_service, graph_runtime_service,
    trace_service, workspace_service,
};
use serde_json::{json, Value};

const KEY: [u8; 32] = [127u8; 32];
const SECRET: &str = "SECRET-TOKEN-7731-ZQ";

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Private Co".into(),
            legal_name: None,
            currency_code: "USD".into(),
            locale: "en-US".into(),
            timezone: "UTC".into(),
            default_tax_rate_bp: 0,
            admin_username: "admin".into(),
            admin_display_name: "Admin".into(),
            admin_password: "supersecretpassword".into(),
            load_sample_data: false,
        },
    )
    .unwrap();
    trace_service::set_trace_bodies(
        &conn,
        &workspace.id,
        &TraceBodiesInput {
            trace_bodies: "full".into(),
        },
        Some(&admin.id),
    )
    .unwrap();
    (conn, workspace.id, admin.id)
}

fn agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> AiAgentDefinition {
    let provider = ai_provider_service::create(
        conn,
        ws,
        &KEY,
        &AiProviderInput {
            name: format!("{name} provider"),
            provider: "anthropic".into(),
            base_url: None,
            model: "scripted-model".into(),
            api_key: Some("k".into()),
        },
        Some(admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_providers SET provider = 'scripted' WHERE id = ?1",
        [&provider.id],
    )
    .unwrap();
    let created = ai_agent_service::create(
        conn,
        ws,
        &AiAgentInput {
            name: name.into(),
            description: None,
            icon: "🤖".into(),
            system_prompt: format!("You are {name}."),
            action_names: vec!["list_objects".into()],
            delegate_agent_ids: vec![],
            skill_ids: vec![],
            object_type_scopes: vec![],
            action_type_ids: vec![],
        },
        Some(admin),
    )
    .unwrap();
    ai_agent_service::set_model_routing(
        conn,
        &created.id,
        ws,
        Some(AiAgentModelRouting {
            primary_provider_id: Some(provider.id),
            ..Default::default()
        }),
        Some(admin),
    )
    .unwrap()
}

fn node(key: &str, node_type: &str, config: Value) -> GraphNodeInput {
    GraphNodeInput {
        node_key: key.into(),
        node_type: node_type.into(),
        config_json: config.to_string(),
        position_x: None,
        position_y: None,
        sort_order: 0,
    }
}

fn edge(from: &str, to: &str) -> GraphEdgeInput {
    GraphEdgeInput {
        from_node_key: from.into(),
        to_node_key: to.into(),
        branch_label: None,
        sort_order: 0,
    }
}

fn text(t: &str) -> ScriptedResponse {
    ScriptedResponse::Text {
        text: t.into(),
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
        },
    }
}

fn tool_with_secret() -> ScriptedResponse {
    ScriptedResponse::ToolCalls {
        calls: vec![RequestedToolCall {
            id: "c1".into(),
            name: "list_objects".into(),
            arguments: json!({"note": format!("using {SECRET}")}),
            malformed: None,
        }],
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
        },
    }
}

/// Every table and column whose stored text contains `needle`.
fn tables_containing(conn: &rusqlite::Connection, needle: &str) -> Vec<String> {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut found = Vec::new();
    for table in tables {
        let mut stmt = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for (i, column) in columns.iter().enumerate() {
                if let Ok(rusqlite::types::ValueRef::Text(bytes)) = row.get_ref(i) {
                    if String::from_utf8_lossy(bytes).contains(needle) {
                        found.push(format!("{table}.{column}"));
                    }
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

fn publish(
    conn: &rusqlite::Connection,
    ws: &str,
    admin: &str,
    nodes: Vec<GraphNodeInput>,
    edges: Vec<GraphEdgeInput>,
) -> lanesra_core::models::execution_graph::ExecutionGraph {
    let graph = execution_graph_service::create(
        conn,
        ws,
        &ExecutionGraphInput {
            name: "Private".into(),
            description: None,
            nodes,
            edges,
        },
        Some(admin),
    )
    .unwrap();
    execution_graph_service::publish(conn, &graph.id, ws, Some(admin)).unwrap()
}

#[tokio::test]
async fn ac1_a_do_not_persist_value_is_used_but_never_stored_anywhere() {
    // Covers: REQ-LOOP-021 (WP-0127 AC-1)
    let (conn, ws, admin) = setup_workspace();
    let fetcher = agent(&conn, &ws, &admin, "Fetcher");
    let user = agent(&conn, &ws, &admin, "User");
    let graph = publish(
        &conn,
        &ws,
        &admin,
        vec![
            node("trigger", "trigger", json!({})),
            node(
                "fetch",
                "agent",
                json!({"agent_id": fetcher.id, "input_template": "Fetch the access token", "do_not_persist": true}),
            ),
            node(
                "use",
                "agent",
                json!({"agent_id": user.id, "input_template": "Call the service with {{fetch.output}}"}),
            ),
            node("end", "end", json!({})),
        ],
        vec![
            edge("trigger", "fetch"),
            edge("fetch", "use"),
            edge("use", "end"),
        ],
    );
    scripted_provider::reset();
    scripted_provider::script_responses(vec![
        text(SECRET),
        tool_with_secret(),
        text(&format!("Finished with {SECRET}.")),
    ]);
    let run = graph_runtime_service::start_run(
        &conn,
        &ws,
        &KEY,
        &graph.id,
        "Go",
        Some(&admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    // The value was used: the next step's model call was sent it.
    let captured = scripted_provider::take_captured();
    let sent_to_use = captured[1].history.last().unwrap().content.clone().unwrap();
    assert!(sent_to_use.contains(SECRET), "{sent_to_use}");
    // ...and stored nowhere: not in checkpoints, steps, traces, bodies or audit.
    assert_eq!(tables_containing(&conn, SECRET), Vec::<String>::new());
    // What was stored says it was withheld.
    assert!(!tables_containing(&conn, lanesra_core::do_not_persist::PLACEHOLDER).is_empty());
}

#[tokio::test]
async fn ac2_a_resumed_run_fetches_the_value_again() {
    // Covers: REQ-LOOP-021 (WP-0127 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let fetcher = agent(&conn, &ws, &admin, "Fetcher");
    let user = agent(&conn, &ws, &admin, "User");
    let graph = publish(
        &conn,
        &ws,
        &admin,
        vec![
            node("trigger", "trigger", json!({})),
            node(
                "fetch",
                "agent",
                json!({"agent_id": fetcher.id, "input_template": "Fetch the access token", "do_not_persist": true}),
            ),
            node("wait", "delay", json!({"delay_seconds": 0})),
            node(
                "use",
                "agent",
                json!({"agent_id": user.id, "input_template": "Call the service with {{fetch.output}}"}),
            ),
            node("end", "end", json!({})),
        ],
        vec![
            edge("trigger", "fetch"),
            edge("fetch", "wait"),
            edge("wait", "use"),
            edge("use", "end"),
        ],
    );
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text(SECRET)]);
    let run = graph_runtime_service::start_run(
        &conn,
        &ws,
        &KEY,
        &graph.id,
        "Go",
        Some(&admin),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(run.status, "waiting_scheduled");
    assert_eq!(
        tables_containing(&conn, SECRET),
        Vec::<String>::new(),
        "the waiting run's checkpoint holds no value"
    );

    // Resuming (in a fresh process the value is gone from memory, and it
    // never reached storage) runs the fetch node again.
    scripted_provider::reset();
    scripted_provider::script_responses(vec![text(SECRET), text("Done.")]);
    graph_runtime_service::resume_due_delays(&conn, &ws, &KEY)
        .await
        .unwrap();
    let run = graph_runtime_service::get_run(&conn, &run.id, &ws, Some(&admin)).unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    let captured = scripted_provider::take_captured();
    assert_eq!(captured.len(), 2, "fetch again, then use");
    let sent_to_use = captured[1].history.last().unwrap().content.clone().unwrap();
    assert!(
        sent_to_use.contains(SECRET),
        "the re-fetched value reached the next step: {sent_to_use}"
    );
    let spans = trace_service::spans_for_run(&conn, &ws, "graph", &run.id).unwrap();
    assert!(spans
        .iter()
        .any(|s| s.name == "fetch"
            && s.attributes.get("lanesra.node.refetched") == Some(&json!(true))));
    assert_eq!(tables_containing(&conn, SECRET), Vec::<String>::new());
}

#[test]
fn only_agent_and_transform_nodes_can_be_marked() {
    // Covers: REQ-LOOP-021 (WP-0127): a node the runtime cannot safely run
    // again cannot keep its output out of storage.
    let (conn, ws, admin) = setup_workspace();
    let graph = execution_graph_service::create(
        &conn,
        &ws,
        &ExecutionGraphInput {
            name: "Bad".into(),
            description: None,
            nodes: vec![
                node("trigger", "trigger", json!({})),
                node(
                    "wait",
                    "delay",
                    json!({"delay_seconds": 0, "do_not_persist": true}),
                ),
                node("end", "end", json!({})),
            ],
            edges: vec![edge("trigger", "wait"), edge("wait", "end")],
        },
        Some(&admin),
    )
    .unwrap();
    let err = execution_graph_service::publish(&conn, &graph.id, &ws, Some(&admin)).unwrap_err();
    assert!(
        err.to_string().contains("cannot be marked do-not-persist"),
        "{err}"
    );
}

#[test]
fn scrubbing_finds_raw_and_json_escaped_forms_and_ends_with_its_scope() {
    // Covers: REQ-LOOP-021 (WP-0127)
    use lanesra_core::do_not_persist::{mark, scope, scrub, PLACEHOLDER};
    {
        let _scope = scope();
        mark(&json!({"output": "line one\nsk-\"quoted\"-value"}));
        mark(&json!("ok"));
        let stored = json!({"copy": "line one\nsk-\"quoted\"-value"}).to_string();
        assert_eq!(scrub(&stored), format!("{{\"copy\":\"{PLACEHOLDER}\"}}"));
        assert_eq!(scrub("all ok here"), "all ok here", "too short to scrub");
    }
    assert_eq!(
        scrub("line one\nsk-\"quoted\"-value"),
        "line one\nsk-\"quoted\"-value",
        "the scope ended"
    );
}
