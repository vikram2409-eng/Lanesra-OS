//! AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime -
//! CRUD/publish-validation (`execution_graph_service`) and the
//! step-by-step executor (`graph_runtime_service`), through every node
//! type, checkpoint/resume durability, and a parity check proving the
//! compatibility-shape mapping (`execution_graph_service::
//! graph_from_pipeline`) produces the same real output as the pre-existing
//! `ai_orchestration_service::run_manual` path for an equivalent sequential
//! Pipeline. Reuses the raw-socket stub-listener pattern already
//! established in `ai_orchestration_topologies.rs`/`ai_agent_policy_engine.rs`.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_orchestration_service, ai_service, company_service, execution_graph_service, graph_runtime_service, workspace_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [83u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

/// Same raw-socket stub as `ai_orchestration_topologies.rs` - serves one
/// canned response body per request in order (repeating the last once
/// exhausted).
fn spawn_sequence_stub(bodies: Vec<String>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line);
            let mut content_length: usize = 0;
            loop {
                let mut l = String::new();
                match reader.read_line(&mut l) {
                    Ok(0) => break,
                    Ok(_) => {
                        if l == "\r\n" || l.trim().is_empty() {
                            break;
                        }
                        if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    Err(_) => break,
                }
            }
            let mut body_buf = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body_buf);
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

fn node(key: &str, node_type: &str, config: serde_json::Value) -> GraphNodeInput {
    GraphNodeInput { node_key: key.into(), node_type: node_type.into(), config_json: config.to_string(), position_x: None, position_y: None, sort_order: 0 }
}

fn edge(from: &str, to: &str, label: Option<&str>) -> GraphEdgeInput {
    GraphEdgeInput { from_node_key: from.into(), to_node_key: to.into(), branch_label: label.map(String::from), sort_order: 0 }
}

fn publish(conn: &rusqlite::Connection, ws: &str, admin: &str, input: ExecutionGraphInput) -> lanesra_core::models::execution_graph::ExecutionGraph {
    let graph = execution_graph_service::create(conn, ws, &input, Some(admin)).unwrap();
    execution_graph_service::publish(conn, &graph.id, ws, Some(admin)).unwrap()
}

// --- CRUD + publish-lifecycle --------------------------------------------

#[test]
fn execution_graph_crud_round_trips_nodes_and_edges_and_locks_after_publish() {
    let (conn, ws, admin) = setup_workspace("Graph CRUD Co");
    let input = ExecutionGraphInput {
        name: "Simple".into(), description: Some("a trigger straight to end".into()),
        nodes: vec![node("trigger", "trigger", serde_json::json!({})), node("end", "end", serde_json::json!({}))],
        edges: vec![edge("trigger", "end", None)],
    };
    let created = execution_graph_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(created.status, "draft");
    assert_eq!(created.nodes.len(), 2);
    assert_eq!(created.edges.len(), 1);
    let trigger_id = created.nodes.iter().find(|n| n.node_key == "trigger").unwrap().id.clone();
    let end_id = created.nodes.iter().find(|n| n.node_key == "end").unwrap().id.clone();
    assert_eq!(created.edges[0].from_node_id, trigger_id);
    assert_eq!(created.edges[0].to_node_id, end_id);

    // A draft can be edited freely.
    let updated_input = ExecutionGraphInput { name: "Simple v2".into(), description: None, nodes: input.nodes.clone(), edges: input.edges.clone() };
    let updated = execution_graph_service::update(&conn, &created.id, &ws, &updated_input, Some(&admin)).unwrap();
    assert_eq!(updated.name, "Simple v2");

    let published = execution_graph_service::publish(&conn, &created.id, &ws, Some(&admin)).unwrap();
    assert_eq!(published.status, "published");

    // Once published, editing is rejected - the same immutable-once-
    // published rule `agent_version_service` enforces.
    let err = execution_graph_service::update(&conn, &created.id, &ws, &updated_input, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("draft"), "{err}");
}

#[test]
fn validate_for_publish_rejects_a_graph_with_no_end_node_reachable() {
    let (conn, ws, admin) = setup_workspace("Graph Missing End Co");
    // trigger -> agent1 -> trigger: no `end` node exists at all.
    let input = ExecutionGraphInput {
        name: "No end".into(), description: None,
        nodes: vec![node("trigger", "trigger", serde_json::json!({})), node("agent1", "agent", serde_json::json!({"agent_id": "x", "input_template": ""}))],
        edges: vec![edge("trigger", "agent1", None), edge("agent1", "trigger", None)],
    };
    let created = execution_graph_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = execution_graph_service::publish(&conn, &created.id, &ws, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("End node"), "{err}");
}

#[test]
fn validate_for_publish_rejects_an_unbounded_cycle_not_passing_through_a_loop_node() {
    let (conn, ws, admin) = setup_workspace("Graph Cycle Co");
    // trigger -> cond -[true]-> end, cond -[false]-> agent1 -> cond (a
    // genuine cycle back into a non-`loop` node).
    let input = ExecutionGraphInput {
        name: "Bad cycle".into(), description: None,
        nodes: vec![
            node("trigger", "trigger", serde_json::json!({})),
            node("cond", "condition", serde_json::json!({"match_type": "all", "conditions": []})),
            node("agent1", "agent", serde_json::json!({"agent_id": "x", "input_template": ""})),
            node("end", "end", serde_json::json!({})),
        ],
        edges: vec![edge("trigger", "cond", None), edge("cond", "end", Some("true")), edge("cond", "agent1", Some("false")), edge("agent1", "cond", None)],
    };
    let created = execution_graph_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = execution_graph_service::publish(&conn, &created.id, &ws, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("unbounded cycle"), "{err}");
}

#[test]
fn validate_for_publish_rejects_a_loop_node_without_a_positive_max_iterations() {
    let (conn, ws, admin) = setup_workspace("Graph Loop Config Co");
    let input = ExecutionGraphInput {
        name: "Unbounded loop".into(), description: None,
        nodes: vec![
            node("trigger", "trigger", serde_json::json!({})),
            node("loop", "loop", serde_json::json!({})), // no max_iterations
            node("end", "end", serde_json::json!({})),
        ],
        edges: vec![edge("trigger", "loop", None), edge("loop", "end", Some("body")), edge("loop", "end", Some("exit"))],
    };
    let created = execution_graph_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = execution_graph_service::publish(&conn, &created.id, &ws, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("max_iterations"), "{err}");
}

// --- Node type execution ---------------------------------------------------

#[tokio::test]
async fn agent_node_resolves_trigger_input_and_a_prior_nodes_recorded_output() {
    let (conn, ws, admin) = setup_workspace("Graph Agent Co");
    let a1 = make_agent(&conn, &ws, &admin, "First");
    let a2 = make_agent(&conn, &ws, &admin, "Second");
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Agent chain".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("agent1", "agent", serde_json::json!({"agent_id": a1.id, "input_template": "{{trigger_input}}"})),
                node("agent2", "agent", serde_json::json!({"agent_id": a2.id, "input_template": "prior: {{agent1.output}} orig: {{trigger_input}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![edge("trigger", "agent1", None), edge("agent1", "agent2", None), edge("agent2", "end", None)],
        },
    );

    let port = spawn_sequence_stub(vec![anthropic_text_body("Hello!"), anthropic_text_body("Got it.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "Say hi", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");

    let agent2_node = run.nodes.iter().find(|n| n.node_key == "agent2").expect("agent2 ran");
    let input_json = agent2_node.input_json.as_deref().unwrap_or_default();
    assert_eq!(input_json, "prior: Hello! orig: Say hi");
    let output_json = agent2_node.output_json.as_deref().unwrap_or_default();
    assert!(output_json.contains("Got it."), "{output_json}");
}

#[tokio::test]
async fn condition_node_takes_the_true_or_false_branch_based_on_trigger_input() {
    let (conn, ws, admin) = setup_workspace("Graph Condition Co");
    let urgent_agent = make_agent(&conn, &ws, &admin, "Urgent handler");
    let normal_agent = make_agent(&conn, &ws, &admin, "Normal handler");
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Branch by urgency".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("cond", "condition", serde_json::json!({"match_type": "all", "conditions": [{"group_id": null, "field_key": "trigger_input", "operator": "equals", "value": "urgent"}]})),
                node("agent_urgent", "agent", serde_json::json!({"agent_id": urgent_agent.id, "input_template": "{{trigger_input}}"})),
                node("agent_normal", "agent", serde_json::json!({"agent_id": normal_agent.id, "input_template": "{{trigger_input}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![
                edge("trigger", "cond", None),
                edge("cond", "agent_urgent", Some("true")),
                edge("cond", "agent_normal", Some("false")),
                edge("agent_urgent", "end", None),
                edge("agent_normal", "end", None),
            ],
        },
    );

    let port = spawn_sequence_stub(vec![anthropic_text_body("handled")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let urgent_run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "urgent", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(urgent_run.status, "completed", "{urgent_run:?}");
    assert!(urgent_run.nodes.iter().any(|n| n.node_key == "agent_urgent"));
    assert!(!urgent_run.nodes.iter().any(|n| n.node_key == "agent_normal"));

    let normal_run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "just checking in", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(normal_run.status, "completed", "{normal_run:?}");
    assert!(normal_run.nodes.iter().any(|n| n.node_key == "agent_normal"));
    assert!(!normal_run.nodes.iter().any(|n| n.node_key == "agent_urgent"));
}

#[tokio::test]
async fn action_node_invokes_the_same_dispatch_a_workflow_action_already_uses() {
    let (conn, ws, admin) = setup_workspace("Graph Action Co");
    let company = company_service::create(
        &conn, &ws,
        &CompanyInput {
            name: "Acme Corp".into(), status: "Prospect".into(), owner_user_id: None, tax_number: None, billing_address: None,
            shipping_address: None, tags: None, notes: None, phone: None, email: None, website: None, annual_revenue_cents: None,
            employee_count: None, preferred_contact_method: None,
        },
        Some(&admin),
    )
    .unwrap();

    let params = serde_json::json!({"target_field_key": "notes", "target_field_source": "builtin", "value": "updated by graph"}).to_string();
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Update company notes".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("set_notes", "action", serde_json::json!({"action_type": "update_field", "params_json": params})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![edge("trigger", "set_notes", None), edge("set_notes", "end", None)],
        },
    );

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "", Some(&admin), None, Some("Company"), Some(&company.id)).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");

    let reloaded = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(reloaded.notes.as_deref(), Some("updated by graph"));
}

#[tokio::test]
async fn approval_node_pauses_the_run_and_resolve_approval_continues_down_the_matching_branch() {
    let (conn, ws, admin) = setup_workspace("Graph Approval Co");
    let a1 = make_agent(&conn, &ws, &admin, "Drafter");
    let after_agent = make_agent(&conn, &ws, &admin, "After approval");
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Gate on approval".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("draft", "agent", serde_json::json!({"agent_id": a1.id, "input_template": "{{trigger_input}}"})),
                node("gate", "approval", serde_json::json!({})),
                node("after", "agent", serde_json::json!({"agent_id": after_agent.id, "input_template": "{{draft.output}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![
                edge("trigger", "draft", None),
                edge("draft", "gate", None),
                edge("gate", "after", Some("approved")),
                edge("gate", "end", Some("rejected")),
                edge("after", "end", None),
            ],
        },
    );

    let port = spawn_sequence_stub(vec![anthropic_text_body("draft text"), anthropic_text_body("finalized")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let paused = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write something", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(paused.status, "waiting_approval", "{paused:?}");
    assert!(paused.pending_approval_id.is_some());

    let approved_run = graph_runtime_service::resolve_approval(&conn, &ws, &master_key(), &paused.id, true, Some("looks good"), Some(&admin)).await.unwrap();
    assert_eq!(approved_run.status, "completed", "{approved_run:?}");
    assert!(approved_run.nodes.iter().any(|n| n.node_key == "after"));

    // A second run, rejected this time, never reaches the "after" agent.
    let paused2 = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write something else", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(paused2.status, "waiting_approval");
    let rejected_run = graph_runtime_service::resolve_approval(&conn, &ws, &master_key(), &paused2.id, false, None, Some(&admin)).await.unwrap();
    assert_eq!(rejected_run.status, "completed", "{rejected_run:?}");
    assert!(!rejected_run.nodes.iter().any(|n| n.node_key == "after"));
}

#[tokio::test]
async fn delay_node_pauses_the_run_and_resume_due_delays_continues_it_once_past_resume_at() {
    let (conn, ws, admin) = setup_workspace("Graph Delay Co");
    let after_agent = make_agent(&conn, &ws, &admin, "After delay");
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Delay then agent".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("wait", "delay", serde_json::json!({"delay_seconds": 0})),
                node("after", "agent", serde_json::json!({"agent_id": after_agent.id, "input_template": "{{trigger_input}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![edge("trigger", "wait", None), edge("wait", "after", None), edge("after", "end", None)],
        },
    );

    let port = spawn_sequence_stub(vec![anthropic_text_body("done waiting")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let paused = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "start", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(paused.status, "waiting_scheduled", "{paused:?}");

    // Force `resume_at` safely into the past rather than relying on real
    // time passing, to avoid any timing flakiness.
    conn.execute("UPDATE ai_runs SET resume_at = '2020-01-01T00:00:00Z' WHERE id = ?1", [&paused.id]).unwrap();

    let resumed_count = graph_runtime_service::resume_due_delays(&conn, &ws, &master_key()).await.unwrap();
    assert_eq!(resumed_count, 1);

    let run = graph_runtime_service::get_run(&conn, &paused.id, &ws, Some(&admin)).unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    assert!(run.nodes.iter().any(|n| n.node_key == "after"));
}

#[tokio::test]
async fn loop_node_bounds_iteration_and_exits_via_its_own_exit_edge_after_max_iterations() {
    let (conn, ws, admin) = setup_workspace("Graph Loop Co");
    let reviewer = make_agent(&conn, &ws, &admin, "Never satisfied reviewer");
    // trigger -> loop -[body]-> reviewer -> review_check -[true]-> end
    //                                                     -[false]-> loop (legitimate re-entry)
    //            loop -[exit]-> end
    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Bounded review loop".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("loop", "loop", serde_json::json!({"max_iterations": 3})),
                node("reviewer", "agent", serde_json::json!({"agent_id": reviewer.id, "input_template": "{{trigger_input}}"})),
                node("review_check", "condition", serde_json::json!({"match_type": "all", "conditions": [{"group_id": null, "field_key": "reviewer.output", "operator": "starts_with", "value": "APPROVED"}]})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![
                edge("trigger", "loop", None),
                edge("loop", "reviewer", Some("body")),
                edge("loop", "end", Some("exit")),
                edge("reviewer", "review_check", None),
                edge("review_check", "end", Some("true")),
                edge("review_check", "loop", Some("false")),
            ],
        },
    );

    let port = spawn_sequence_stub(vec![anthropic_text_body("Not good enough."), anthropic_text_body("Still not good enough."), anthropic_text_body("Nope.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "draft this", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");

    let loop_visits = run.nodes.iter().filter(|n| n.node_key == "loop").count();
    let reviewer_runs = run.nodes.iter().filter(|n| n.node_key == "reviewer").count();
    // 3 bounded rounds (reviewer never approves) then the loop node's own
    // 4th visit exceeds max_iterations and takes the `exit` edge instead.
    assert_eq!(reviewer_runs, 3, "{run:?}");
    assert_eq!(loop_visits, 4, "{run:?}");
    let last_loop_output = run.nodes.iter().filter(|n| n.node_key == "loop").last().unwrap().output_json.clone().unwrap_or_default();
    assert!(last_loop_output.contains("max_iterations_exceeded"), "{last_loop_output}");
}

// --- Parity: the graph engine agrees with the pre-existing orchestration
// engine for an equivalent sequential Pipeline, proving the compatibility-
// shape mapping (`graph_from_pipeline`) is faithful, not just plausible. --

#[tokio::test]
async fn a_sequential_pipeline_mapped_to_a_graph_produces_the_same_output_as_the_original_orchestration_engine() {
    let (conn, ws, admin) = setup_workspace("Graph Parity Co");
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let polisher = make_agent(&conn, &ws, &admin, "Polisher");

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Draft then polish".into(), description: None, topology: "sequential".into(),
            steps: vec![
                PipelineStepInput { agent_id: drafter.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: polisher.id.clone(), input_template: "polish: {{previous_output}}".into(), requires_approval: false },
            ],
        },
        Some(&admin),
    )
    .unwrap();

    // The old engine's run consumes the first two canned responses, the
    // new engine's run against the same agents/templates consumes the
    // next two - both given the identical trigger input, so any
    // divergence between the two engines' template resolution shows up
    // as a mismatched assertion below.
    let port = spawn_sequence_stub(vec![
        anthropic_text_body("A rough draft."),
        anthropic_text_body("A polished draft."),
        anthropic_text_body("A rough draft."),
        anthropic_text_body("A polished draft."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let old_run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "write the intro", Some(&admin)).await.unwrap();
    assert_eq!(old_run.status, "succeeded", "{old_run:?}");
    assert_eq!(old_run.steps[1].input_text, "polish: A rough draft.");
    assert_eq!(old_run.steps[1].output_text.as_deref(), Some("A polished draft."));

    let graph_input = execution_graph_service::graph_from_pipeline(&pipeline).unwrap();
    let graph = execution_graph_service::create_from_source(&conn, &ws, graph_input, "pipeline", &pipeline.id, Some(&admin)).unwrap();
    assert_eq!(graph.status, "published");

    let new_run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write the intro", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(new_run.status, "completed", "{new_run:?}");

    let agent1_node = new_run.nodes.iter().find(|n| n.node_key == "agent_1").expect("agent_1 (the polisher) ran");
    // The exact same resolved input the old engine's `{{previous_output}}`
    // produced - proving `resolve_previous_output_token`'s rewrite to
    // `{{agent_0.output}}` is a faithful translation, not an approximation.
    assert_eq!(agent1_node.input_json.as_deref(), Some("polish: A rough draft."));
    assert_eq!(agent1_node.output_json.as_deref().unwrap_or_default(), serde_json::json!({"output": "A polished draft."}).to_string());
}
