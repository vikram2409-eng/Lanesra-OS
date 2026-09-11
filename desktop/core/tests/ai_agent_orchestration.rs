//! AI & Agentic Layer, Phase 6b: orchestration on top of Phase 6a's
//! Agents - Pipeline CRUD, a manual multi-step Pipeline run (proving step
//! 2's resolved input genuinely contains step 1's real output, not just
//! that the final answer looks right), the new `run_ai_agent` Workflow
//! Automation action (proving it only ever *enqueues* - a workflow fires
//! synchronously inside a record save, so it must never call out to a
//! provider inline), `drain_pending_runs` actually processing that queued
//! row, and `describe_action_for_test`-equivalent coverage via
//! `workflow_service::test_workflows`. The inbound webhook Trigger route
//! (`server/src/agent_v1.rs`) is covered separately in the server crate's
//! own `tests/agent_v1.rs`, alongside every other `/api/v1` route.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, AiAgentTriggerInput, PipelineStepInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::ai_agent_pending_run_repo;
use lanesra_core::services::{ai_agent_service, ai_orchestration_service, ai_service, company_service, user_service, workflow_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Orchestration Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [31u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn non_admin_user(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "rep".into(), display_name: "Sales Rep".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(admin),
    )
    .unwrap()
    .id
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

fn company_input(name: &str) -> CompanyInput {
    CompanyInput { name: name.into(), status: "Active Customer".into(), owner_user_id: None, tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None, ..Default::default() }
}

fn run_ai_agent_action(target_type: &str, target_id: &str, input_template: &str) -> WorkflowActionInput {
    WorkflowActionInput {
        action_type: "run_ai_agent".into(),
        params_json: serde_json::json!({"target_type": target_type, "target_id": target_id, "input_template": input_template}).to_string(),
    }
}

fn record_created_workflow(entity_type: &str, action: WorkflowActionInput) -> WorkflowDefinitionInput {
    WorkflowDefinitionInput {
        app_id: None,
        entity_type: entity_type.into(), name: format!("{entity_type} created -> run agent"), description: None,
        trigger_type: "record_created".into(), trigger_status: None, trigger_field_key: None, trigger_field_source: "custom".into(),
        trigger_offset_days: 0, match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![action],
    }
}

/// Same raw-socket stub as `ai_agent_foundry.rs`'s own - serves one canned
/// response body per request (repeating the last once exhausted) and
/// captures each request's raw JSON body, so a test can assert what was
/// actually sent (here: that step 2's real request contained step 1's
/// real output, not just that the final answer happens to look right).
fn spawn_sequence_stub(bodies: Vec<String>) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let queue = Arc::new(Mutex::new(VecDeque::from(bodies)));
    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
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
            captured_clone.lock().unwrap().push(String::from_utf8_lossy(&body_buf).to_string());
            let body = {
                let mut q = queue.lock().unwrap();
                if q.len() > 1 { q.pop_front().unwrap() } else { q.front().cloned().unwrap_or_default() }
            };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

#[tokio::test]
async fn pipeline_crud_is_administrator_gated_and_validates_its_steps() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let a1 = make_agent(&conn, &ws, &admin, "Step One");
    let a2 = make_agent(&conn, &ws, &admin, "Step Two");

    let input = AiAgentPipelineInput {
        name: "Two Step".into(), description: Some("A demo pipeline".into()), topology: "sequential".into(),
        steps: vec![
            PipelineStepInput { agent_id: a1.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false },
            PipelineStepInput { agent_id: a2.id.clone(), input_template: "{{previous_output}}".into(), requires_approval: false },
        ],
    };
    let denied = ai_orchestration_service::create_pipeline(&conn, &ws, &input, Some(&rep));
    assert!(denied.unwrap_err().to_string().contains("Administrator"));

    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(pipeline.steps.len(), 2);

    // A step naming a nonexistent agent is rejected.
    let bad = AiAgentPipelineInput { name: "Bad".into(), description: None, topology: "sequential".into(), steps: vec![PipelineStepInput { agent_id: "not-a-real-id".into(), input_template: "hi".into(), requires_approval: false }] };
    assert!(ai_orchestration_service::create_pipeline(&conn, &ws, &bad, Some(&admin)).is_err());

    let renamed = AiAgentPipelineInput { name: "Two Step (renamed)".into(), ..input };
    let updated = ai_orchestration_service::update_pipeline(&conn, &pipeline.id, &ws, &renamed, Some(&admin)).unwrap();
    assert_eq!(updated.name, "Two Step (renamed)");

    let deactivated = ai_orchestration_service::set_pipeline_active(&conn, &pipeline.id, false, Some(&admin)).unwrap();
    assert!(!deactivated.is_active);
    let listed_active_only = ai_orchestration_service::list_pipelines(&conn, &ws, true).unwrap();
    assert!(!listed_active_only.iter().any(|p| p.id == pipeline.id));
}

#[tokio::test]
async fn a_manual_pipeline_run_feeds_step_ones_real_output_into_step_two() {
    let (conn, ws, admin) = setup_workspace();
    let summarizer = make_agent(&conn, &ws, &admin, "Summarizer");
    let translator = make_agent(&conn, &ws, &admin, "Translator");

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Summarize then translate".into(), description: None, topology: "sequential".into(),
            steps: vec![
                PipelineStepInput { agent_id: summarizer.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: translator.id.clone(), input_template: "Translate: {{previous_output}}".into(), requires_approval: false },
            ],
        },
        Some(&admin),
    )
    .unwrap();

    let (port, captured) = spawn_sequence_stub(vec![anthropic_text_body("SUMMARY: hello world"), anthropic_text_body("TRANSLATED: hola mundo")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello world", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(run.steps.len(), 2);
    assert_eq!(run.steps[0].input_text, "hello world");
    assert_eq!(run.steps[0].output_text.as_deref(), Some("SUMMARY: hello world"));
    assert_eq!(run.steps[1].input_text, "Translate: SUMMARY: hello world");
    assert_eq!(run.steps[1].output_text.as_deref(), Some("TRANSLATED: hola mundo"));

    // Not just the recorded step text - the *actual outbound request* for
    // step 2 really carried step 1's real output.
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[1].contains("SUMMARY: hello world"), "step 2's request should carry step 1's real output: {}", requests[1]);
}

#[tokio::test]
async fn the_run_ai_agent_workflow_action_only_enqueues_never_runs_inline() {
    let (conn, ws, admin) = setup_workspace();
    // Deliberately no AI provider configured at all - if this action ever
    // called out synchronously, resolving the provider settings would
    // itself fail (or hang trying to reach an unconfigured endpoint);
    // getting a clean "record created" with nothing but a queued row
    // proves it never tried.
    let agent = make_agent(&conn, &ws, &admin, "Workflow Agent");
    workflow_service::create_rule(&conn, &ws, &record_created_workflow("Company", run_ai_agent_action("agent", &agent.id, "hello from workflow")), Some(&admin)).unwrap();

    company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let pending = ai_agent_pending_run_repo::list_batch(&conn, &ws, 10).unwrap();
    assert_eq!(pending.len(), 1, "expected exactly one enqueued pending run");
    assert_eq!(pending[0].target_type, "agent");
    assert_eq!(pending[0].target_id, agent.id);
    assert_eq!(pending[0].resolved_input_text, "hello from workflow");
    assert_eq!(pending[0].triggered_by.as_deref(), Some("workflow"));

    // No run was recorded yet - proves it truly didn't run inline.
    assert!(ai_orchestration_service::list_runs(&conn, "agent", &agent.id, 10).unwrap().is_empty());

    // Now drain it - this is the only place a real provider call happens.
    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("Handled the workflow trigger.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let drained = ai_orchestration_service::drain_pending_runs(&conn, &ws, &master_key(), 10).await.unwrap();
    assert_eq!(drained, 1);
    assert!(ai_agent_pending_run_repo::list_batch(&conn, &ws, 10).unwrap().is_empty());

    let runs = ai_orchestration_service::list_runs(&conn, "agent", &agent.id, 10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "succeeded", "{:?}", runs[0]);
    assert_eq!(runs[0].triggered_by.as_deref(), Some("workflow"));
    assert_eq!(runs[0].steps[0].output_text.as_deref(), Some("Handled the workflow trigger."));
}

#[tokio::test]
async fn test_workflows_describes_the_run_ai_agent_action_without_running_it() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Describable Agent");
    // Deliberately no AI provider configured - test mode must never call out.
    workflow_service::create_rule(&conn, &ws, &record_created_workflow("Company", run_ai_agent_action("agent", &agent.id, "hi there")), Some(&admin)).unwrap();

    let result = workflow_service::test_workflows(&conn, &ws, "Company", &HashMap::new(), Some(&admin)).unwrap();
    assert_eq!(result.matches.len(), 1);
    let description = &result.matches[0].action_descriptions[0];
    assert!(description.contains("Describable Agent"), "{description}");
    assert!(description.contains("not actually run in test mode"), "{description}");

    // Never actually ran - no pending row, no run.
    assert!(ai_agent_pending_run_repo::list_batch(&conn, &ws, 10).unwrap().is_empty());
    assert!(ai_orchestration_service::list_runs(&conn, "agent", &agent.id, 10).unwrap().is_empty());
}

#[tokio::test]
async fn trigger_crud_validates_target_and_schedule_interval() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Triggerable");

    let bad_target = AiAgentTriggerInput { target_type: "agent".into(), target_id: "not-real".into(), trigger_type: "webhook".into(), interval_minutes: None };
    assert!(ai_orchestration_service::create_trigger(&conn, &ws, &bad_target, Some(&admin)).is_err());

    let bad_interval = AiAgentTriggerInput { target_type: "agent".into(), target_id: agent.id.clone(), trigger_type: "schedule".into(), interval_minutes: Some(0) };
    assert!(ai_orchestration_service::create_trigger(&conn, &ws, &bad_interval, Some(&admin)).is_err());

    let trigger = ai_orchestration_service::create_trigger(
        &conn, &ws,
        &AiAgentTriggerInput { target_type: "agent".into(), target_id: agent.id.clone(), trigger_type: "schedule".into(), interval_minutes: Some(30) },
        Some(&admin),
    )
    .unwrap();
    let listed = ai_orchestration_service::list_triggers(&conn, "agent", &agent.id).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, trigger.id);

    ai_orchestration_service::set_trigger_active(&conn, &trigger.id, false, Some(&admin)).unwrap();
    ai_orchestration_service::delete_trigger(&conn, &trigger.id, Some(&admin)).unwrap();
    assert!(ai_orchestration_service::list_triggers(&conn, "agent", &agent.id).unwrap().is_empty());
}

#[tokio::test]
async fn a_due_schedule_trigger_enqueues_exactly_once_per_interval() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Scheduled Agent");
    ai_orchestration_service::create_trigger(
        &conn, &ws,
        &AiAgentTriggerInput { target_type: "agent".into(), target_id: agent.id.clone(), trigger_type: "schedule".into(), interval_minutes: Some(30) },
        Some(&admin),
    )
    .unwrap();

    // last_run_at starts NULL - due immediately.
    let enqueued = ai_orchestration_service::enqueue_due_schedules(&conn, &ws).unwrap();
    assert_eq!(enqueued, 1);
    let pending = ai_agent_pending_run_repo::list_batch(&conn, &ws, 10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].target_type, "agent");
    assert_eq!(pending[0].target_id, agent.id);
    assert_eq!(pending[0].triggered_by.as_deref(), Some("schedule"));

    // Just marked as run - not due again this soon.
    let enqueued_again = ai_orchestration_service::enqueue_due_schedules(&conn, &ws).unwrap();
    assert_eq!(enqueued_again, 0);
}
