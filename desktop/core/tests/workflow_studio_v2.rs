//! UX/UI Modernization, Workflow Studio 2.0 (issue #193): rebuilds
//! Workflow Automation onto the Shared Visual Builder Framework (#192)
//! without migrating a single existing saved Workflow's execution path -
//! `graph_id` stays `None` forever unless an admin explicitly opts in
//! (`workflow_service::upgrade_to_graph`), so every Workflow that exists
//! today keeps firing through the exact same flat executor it always has.
//!
//! The first test below is this issue's own mandatory parity gate: a
//! representative, un-upgraded Workflow produces byte-for-byte identical
//! behavior to every pre-existing `workflow_automation.rs` test - proven
//! here by construction (its `graph_id` is never set) rather than by an
//! after-the-fact output comparison, the same reasoning `run_workflow`'s
//! own doc comment gives. The rest cover the two new Execution Graph node
//! types (`run_agent_team`, `evaluate_result`) and the upgrade/publish
//! lifecycle, reusing the raw-socket stub-listener pattern
//! `execution_graph_runtime.rs`/`ai_eval_harness.rs` already established.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::execution_graph::{ExecutionGraphInput, GraphEdgeInput, GraphNodeInput};
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_orchestration_service, ai_service, company_service, execution_graph_service, graph_runtime_service, opportunity_service, task_service, workflow_service, workspace_service};

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
    [91u8; 32]
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

/// Same raw-socket stub as `execution_graph_runtime.rs` - serves one
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
            let mut q = queue.lock().unwrap();
            let body = if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() };
            drop(q);
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
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

fn company_input(name: &str) -> CompanyInput {
    CompanyInput { name: name.into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }
}

// --- Mandatory parity gate ----------------------------------------------

/// This issue's own gate: proves an un-upgraded Workflow runs byte-for-
/// byte unchanged - same assertions `workflow_automation.rs`'s own
/// `workflow_creates_a_follow_up_task_when_opportunity_stage_matches`
/// makes, plus the explicit check that `graph_id` never got set.
#[test]
fn an_un_upgraded_workflow_fires_through_the_exact_same_flat_path_as_before_this_issue() {
    let (conn, ws, admin) = setup_workspace("Parity Co");
    let wf = workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            app_id: None, entity_type: "Opportunity".into(), name: "Opportunity -> Won".into(), description: None,
            trigger_type: "status_changed".into(), trigger_status: Some("Won".into()), trigger_field_key: None, trigger_field_source: "custom".into(),
            trigger_offset_days: 0, match_type: "all".into(), priority: 0, conditions: vec![],
            actions: vec![WorkflowActionInput {
                action_type: "create_task".into(),
                params_json: serde_json::json!({"title": "Send onboarding kit", "description": null, "due_in_days": 3, "assignee_user_id": null}).to_string(),
            }],
        },
        Some(&admin),
    )
    .unwrap();
    assert!(wf.graph_id.is_none(), "a freshly-created workflow must never get a graph_id on its own");

    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    let opp_input = |stage: &str| OpportunityInput {
        company_id: company.id.clone(), primary_contact_id: None, name: "Big Deal".into(), stage: stage.into(),
        status: if stage == "Won" { "Won" } else { "Open" }.into(), value_cents: 500000, currency_code: "USD".into(),
        probability_bp: 0, expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None,
    };
    let opp = opportunity_service::create(&conn, &opp_input("New"), Some(&admin)).unwrap();
    opportunity_service::update(&conn, &opp.id, &opp_input("Won"), Some(&admin)).unwrap();

    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Send onboarding kit");

    let reloaded = workflow_service::get_rule(&conn, &wf.id).unwrap().unwrap();
    assert!(reloaded.graph_id.is_none(), "firing a workflow must never silently set graph_id");
}

// --- Upgrade / publish lifecycle -----------------------------------------

#[tokio::test]
async fn upgrade_to_graph_seeds_a_draft_and_the_workflow_stops_firing_until_its_published() {
    let (conn, ws, admin) = setup_workspace("Upgrade Lifecycle Co");
    let wf = workflow_service::create_rule(
        &conn, &ws,
        &WorkflowDefinitionInput {
            app_id: None, entity_type: "Company".into(), name: "On create".into(), description: None,
            trigger_type: "record_created".into(), trigger_status: None, trigger_field_key: None, trigger_field_source: "custom".into(),
            trigger_offset_days: 0, match_type: "all".into(), priority: 0, conditions: vec![],
            actions: vec![WorkflowActionInput { action_type: "create_task".into(), params_json: serde_json::json!({"title": "Welcome", "description": null, "due_in_days": 1, "assignee_user_id": null}).to_string() }],
        },
        Some(&admin),
    )
    .unwrap();

    let upgraded = workflow_service::upgrade_to_graph(&conn, &ws, &wf.id, Some(&admin)).unwrap();
    let graph_id = upgraded.graph_id.clone().expect("graph_id set by upgrade_to_graph");
    let graph = execution_graph_service::get(&conn, &graph_id, &ws, Some(&admin)).unwrap();
    assert_eq!(graph.status, "draft", "upgrading must leave the graph editable, not auto-publish it");

    // Upgrading again is rejected.
    let err = workflow_service::upgrade_to_graph(&conn, &ws, &wf.id, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("already"), "{err}");

    // Fires while the graph is still a draft: enqueued, but the drain
    // finds the graph unpublished and the run never actually starts - no
    // Task, no GraphRun row, and the pending queue is still drained
    // (not retried forever).
    let company_1 = company_service::create(&conn, &ws, &company_input("Acme One"), Some(&admin)).unwrap();
    let drained = graph_runtime_service::drain_pending_graph_runs(&conn, &ws, &master_key(), 10).await.unwrap();
    assert_eq!(drained, 1);
    assert!(task_service::list_by_related(&conn, "Company", &company_1.id).unwrap().is_empty());
    assert!(graph_runtime_service::list_runs_for_graph(&conn, &graph_id, &ws, Some(&admin)).unwrap().is_empty());

    // Publishing resumes it - the mapped graph (trigger -> action -> end,
    // from `graph_from_workflow`) now actually runs the create_task action.
    execution_graph_service::publish(&conn, &graph_id, &ws, Some(&admin)).unwrap();
    let company_2 = company_service::create(&conn, &ws, &company_input("Acme Two"), Some(&admin)).unwrap();
    let drained_2 = graph_runtime_service::drain_pending_graph_runs(&conn, &ws, &master_key(), 10).await.unwrap();
    assert_eq!(drained_2, 1);
    let tasks = task_service::list_by_related(&conn, "Company", &company_2.id).unwrap();
    assert_eq!(tasks.len(), 1, "the published graph should have run the mapped create_task action");
    assert_eq!(tasks[0].title, "Welcome");
    let runs = graph_runtime_service::list_runs_for_graph(&conn, &graph_id, &ws, Some(&admin)).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "completed", "{:?}", runs[0]);
}

// --- run_agent_team node --------------------------------------------------

#[tokio::test]
async fn run_agent_team_node_invokes_the_pipeline_and_continues_on_success() {
    let (conn, ws, admin) = setup_workspace("Run Agent Team Co");
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

    let port = spawn_sequence_stub(vec![anthropic_text_body("A rough draft."), anthropic_text_body("A polished draft.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Run a team".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("team", "run_agent_team", serde_json::json!({"pipeline_id": pipeline.id, "input_template": "{{trigger_input}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![edge("trigger", "team", None), edge("team", "end", None)],
        },
    );

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write the intro", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    let team_node = run.nodes.iter().find(|n| n.node_key == "team").expect("team node ran");
    assert_eq!(team_node.output_json.as_deref().unwrap_or_default(), serde_json::json!({"output": "A polished draft."}).to_string());
}

#[tokio::test]
async fn run_agent_team_node_fails_the_run_when_the_pipeline_does_not_complete() {
    let (conn, ws, admin) = setup_workspace("Run Agent Team Pause Co");
    let reviewer = make_agent(&conn, &ws, &admin, "Reviewer");
    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Needs approval".into(), description: None, topology: "sequential".into(),
            steps: vec![PipelineStepInput { agent_id: reviewer.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: true }],
        },
        Some(&admin),
    )
    .unwrap();
    let port = spawn_sequence_stub(vec![anthropic_text_body("A draft awaiting approval.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let graph = publish(
        &conn, &ws, &admin,
        ExecutionGraphInput {
            name: "Run a paused team".into(), description: None,
            nodes: vec![
                node("trigger", "trigger", serde_json::json!({})),
                node("team", "run_agent_team", serde_json::json!({"pipeline_id": pipeline.id, "input_template": "{{trigger_input}}"})),
                node("end", "end", serde_json::json!({})),
            ],
            edges: vec![edge("trigger", "team", None), edge("team", "end", None)],
        },
    );

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "review this", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "failed", "{run:?}");
    assert!(run.error_message.as_deref().unwrap_or_default().contains("awaiting_approval"), "{:?}", run.error_message);
}

// --- evaluate_result node -------------------------------------------------

fn evaluate_graph_input() -> ExecutionGraphInput {
    ExecutionGraphInput {
        name: "Draft then grade".into(), description: None,
        nodes: vec![
            node("trigger", "trigger", serde_json::json!({})),
            node("drafter", "agent", serde_json::json!({"agent_id": "", "input_template": "{{trigger_input}}"})),
            node("grade", "evaluate_result", serde_json::json!({"source_node_key": "drafter", "success_criteria": "Mentions a discount"})),
            node("pass_action", "transform", serde_json::json!({"set": {"result": "graded pass"}})),
            node("end", "end", serde_json::json!({})),
        ],
        edges: vec![
            edge("trigger", "drafter", None),
            edge("drafter", "grade", None),
            edge("grade", "pass_action", Some("pass")),
            edge("grade", "end", Some("fail")),
            edge("pass_action", "end", None),
        ],
    }
}

#[tokio::test]
async fn evaluate_result_node_follows_the_pass_edge_when_the_judge_approves() {
    let (conn, ws, admin) = setup_workspace("Evaluate Pass Co");
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let port = spawn_sequence_stub(vec![anthropic_text_body("Enjoy a 10% discount on your order."), anthropic_text_body("PASS\nMentions a discount as required.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let mut input = evaluate_graph_input();
    input.nodes[1] = node("drafter", "agent", serde_json::json!({"agent_id": drafter.id, "input_template": "{{trigger_input}}"}));
    let graph = publish(&conn, &ws, &admin, input);

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write a promo", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    assert!(run.nodes.iter().any(|n| n.node_key == "pass_action"), "should have taken the pass branch: {:?}", run.nodes);
}

#[tokio::test]
async fn evaluate_result_node_follows_the_fail_edge_when_the_judge_rejects() {
    let (conn, ws, admin) = setup_workspace("Evaluate Fail Co");
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let port = spawn_sequence_stub(vec![anthropic_text_body("Thanks for your order."), anthropic_text_body("FAIL\nNever mentions a discount.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let mut input = evaluate_graph_input();
    input.nodes[1] = node("drafter", "agent", serde_json::json!({"agent_id": drafter.id, "input_template": "{{trigger_input}}"}));
    let graph = publish(&conn, &ws, &admin, input);

    let run = graph_runtime_service::start_run(&conn, &ws, &master_key(), &graph.id, "write a promo", Some(&admin), None, None, None).await.unwrap();
    assert_eq!(run.status, "completed", "{run:?}");
    assert!(!run.nodes.iter().any(|n| n.node_key == "pass_action"), "should have taken the fail branch, not pass: {:?}", run.nodes);
}

#[test]
fn validate_for_publish_rejects_an_evaluate_result_node_missing_a_pass_or_fail_edge() {
    let (conn, ws, admin) = setup_workspace("Evaluate Validation Co");
    let input = ExecutionGraphInput {
        name: "Missing fail edge".into(), description: None,
        nodes: vec![
            node("trigger", "trigger", serde_json::json!({})),
            node("grade", "evaluate_result", serde_json::json!({"source_node_key": "trigger", "success_criteria": "anything"})),
            node("end", "end", serde_json::json!({})),
        ],
        // Only a "pass" edge - no "fail" edge, which validate_for_publish must reject.
        edges: vec![edge("trigger", "grade", None), edge("grade", "end", Some("pass"))],
    };
    let graph = execution_graph_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    let err = execution_graph_service::publish(&conn, &graph.id, &ws, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("pass") && err.to_string().contains("fail"), "{err}");
}
