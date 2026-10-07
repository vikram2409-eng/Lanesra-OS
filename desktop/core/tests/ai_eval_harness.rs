//! AI & Agentic Layer, Phase 7d: the Evaluation Harness - Suite CRUD
//! (admin-gated, target validated) and `run_suite`'s real grading pass,
//! proving each case's actual target response and the judge's real
//! verdict both land correctly on the stored result. Reuses
//! `ai_agent_orchestration.rs`'s own stub-listener pattern.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_policy::AiAgentPolicyInput;
use lanesra_core::models::ai_eval::{AiEvalCaseInput, AiEvalSuiteInput};
use lanesra_core::models::ai_tool_registry::RiskLevel;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{agent_version_service, ai_agent_service, ai_eval_service, ai_service, policy_engine_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Eval Harness Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [64u8; 32]
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

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}).to_string()
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

/// Same raw-socket stub every other AI Foundry test file uses - serves
/// one canned response body per request in order (repeating the last
/// once exhausted).
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

fn two_case_suite_input(agent_id: &str) -> AiEvalSuiteInput {
    AiEvalSuiteInput {
        name: "Arithmetic & geography".into(), description: Some("Two simple golden cases".into()), target_type: "agent".into(), target_id: agent_id.into(),
        evaluator_type: "task_completion".into(),
        cases: vec![
            AiEvalCaseInput { input_text: "What is 2+2?".into(), success_criteria: "The response states the number 4.".into() },
            AiEvalCaseInput { input_text: "What is the capital of France?".into(), success_criteria: "The response names Paris.".into() },
        ],
    }
}

#[tokio::test]
async fn suite_crud_is_administrator_gated_and_validates_its_target_and_cases() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let agent = make_agent(&conn, &ws, &admin, "Answerer");

    let input = two_case_suite_input(&agent.id);
    let denied = ai_eval_service::create_suite(&conn, &ws, &input, Some(&rep));
    assert!(denied.unwrap_err().to_string().contains("Administrator"));

    // An unknown target is rejected.
    let bad_target = AiEvalSuiteInput { target_id: "not-a-real-id".into(), ..two_case_suite_input(&agent.id) };
    assert!(ai_eval_service::create_suite(&conn, &ws, &bad_target, Some(&admin)).is_err());

    // No cases is rejected.
    let no_cases = AiEvalSuiteInput { cases: vec![], ..two_case_suite_input(&agent.id) };
    assert!(ai_eval_service::create_suite(&conn, &ws, &no_cases, Some(&admin)).is_err());

    // A case missing its criteria is rejected.
    let blank_criteria = AiEvalSuiteInput {
        cases: vec![AiEvalCaseInput { input_text: "hi".into(), success_criteria: "".into() }],
        ..two_case_suite_input(&agent.id)
    };
    assert!(ai_eval_service::create_suite(&conn, &ws, &blank_criteria, Some(&admin)).is_err());

    let suite = ai_eval_service::create_suite(&conn, &ws, &input, Some(&admin)).unwrap();
    assert_eq!(suite.cases.len(), 2);

    let renamed = AiEvalSuiteInput { name: "Renamed suite".into(), ..input };
    let updated = ai_eval_service::update_suite(&conn, &suite.id, &ws, &renamed, Some(&admin)).unwrap();
    assert_eq!(updated.name, "Renamed suite");

    ai_eval_service::delete_suite(&conn, &suite.id, Some(&admin)).unwrap();
    assert!(ai_eval_service::get_suite(&conn, &suite.id).unwrap().is_none());
}

#[tokio::test]
async fn running_a_suite_grades_each_case_against_the_targets_real_response() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Answerer");
    let suite = ai_eval_service::create_suite(&conn, &ws, &two_case_suite_input(&agent.id), Some(&admin)).unwrap();

    // Order: case 1's target response, case 1's judge verdict, case 2's
    // target response, case 2's judge verdict.
    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_text_body("The answer is 4."),
        anthropic_text_body("PASS\nThe response states 4, matching the criteria."),
        anthropic_text_body("I'm not sure."),
        anthropic_text_body("FAIL\nThe response never mentions Paris."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run.status, "completed");
    assert_eq!(run.passed_count, 1);
    assert_eq!(run.failed_count, 1);
    assert_eq!(run.results.len(), 2);

    assert_eq!(run.results[0].input_text, "What is 2+2?");
    assert_eq!(run.results[0].actual_output.as_deref(), Some("The answer is 4."));
    assert!(run.results[0].passed);
    assert_eq!(run.results[0].judge_reasoning.as_deref(), Some("The response states 4, matching the criteria."));
    assert!(run.results[0].error.is_none());

    assert_eq!(run.results[1].input_text, "What is the capital of France?");
    assert_eq!(run.results[1].actual_output.as_deref(), Some("I'm not sure."));
    assert!(!run.results[1].passed);
    assert_eq!(run.results[1].judge_reasoning.as_deref(), Some("The response never mentions Paris."));

    // The real judge call's request genuinely carried the case's real
    // input, criteria, and the target's real answer - not just that the
    // stored result looks right.
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 4);
    assert!(requests[1].contains("What is 2+2?"), "{}", requests[1]);
    assert!(requests[1].contains("The response states the number 4."), "{}", requests[1]);
    assert!(requests[1].contains("The answer is 4."), "{}", requests[1]);

    // The run also shows up in this suite's history.
    let history = ai_eval_service::list_runs(&conn, &suite.id, 10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, run.id);
}

// --- AI Agent Platform v2, Phase 6a: evaluator type expansion ----------

#[tokio::test]
async fn structured_output_evaluator_validates_against_the_agents_declared_schema_with_no_judge_call() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Summarizer");
    let schema = serde_json::json!({"type": "object", "required": ["answer"], "properties": {"answer": {"type": "string"}}});
    let draft = agent_version_service::create_draft(
        &conn, &agent.id, &ws,
        &lanesra_core::models::ai_agent::AiAgentVersionInput {
            name: "Summarizer".into(), description: None, icon: "🤖".into(), system_prompt: "You are Summarizer.".into(),
            action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![], model_routing: None, output_schema: Some(schema),
        },
        Some(&admin),
    )
    .unwrap();
    agent_version_service::transition_status(&conn, &agent.id, &ws, &draft.id, "test", Some(&admin)).unwrap();
    agent_version_service::transition_status(&conn, &agent.id, &ws, &draft.id, "published", Some(&admin)).unwrap();

    let suite = ai_eval_service::create_suite(
        &conn, &ws,
        &AiEvalSuiteInput {
            name: "Structured output check".into(), description: None, target_type: "agent".into(), target_id: agent.id.clone(),
            evaluator_type: "structured_output".into(),
            cases: vec![AiEvalCaseInput { input_text: "Summarize this".into(), success_criteria: String::new() }],
        },
        Some(&admin),
    )
    .unwrap();

    // Only one provider call (the target's own reply) - no judge call for
    // this evaluator type, the whole point of it being deterministic.
    let (port, captured) = spawn_sequence_stub(vec![anthropic_text_body(r#"{"answer": "Paris"}"#)]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let run = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(captured.lock().unwrap().len(), 1, "expected no judge call for structured_output");
    assert_eq!(run.passed_count, 1);
    assert_eq!(run.failed_count, 0);
    assert!(run.results[0].judge_reasoning.as_deref().unwrap_or_default().contains("conforms"));

    // A response missing the required property fails. This agent's own
    // Structured Output repair-retry (chat_service::run_agent_once, Phase
    // 1) already makes one extra attempt before giving up - genuine
    // existing behavior, not an eval judge call - so two provider calls
    // are expected here, still no third (judge) call.
    let (port2, captured2) = spawn_sequence_stub(vec![anthropic_text_body(r#"{"wrong_key": "oops"}"#)]);
    configure_anthropic_key(&conn, &ws, &admin, port2);
    let run2 = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(captured2.lock().unwrap().len(), 2, "expected the repair-retry's two calls, no third judge call");
    assert_eq!(run2.failed_count, 1);
    assert!(run2.results[0].judge_reasoning.as_deref().unwrap_or_default().contains("missing required property"));

    // An agent with no declared schema records an error, not a guessed
    // pass/fail.
    let unschema_agent = make_agent(&conn, &ws, &admin, "No Schema Agent");
    let unschema_suite = ai_eval_service::create_suite(
        &conn, &ws,
        &AiEvalSuiteInput {
            name: "No schema".into(), description: None, target_type: "agent".into(), target_id: unschema_agent.id.clone(),
            evaluator_type: "structured_output".into(),
            cases: vec![AiEvalCaseInput { input_text: "hi".into(), success_criteria: String::new() }],
        },
        Some(&admin),
    )
    .unwrap();
    let (port3, _c3) = spawn_sequence_stub(vec![anthropic_text_body("hello there")]);
    configure_anthropic_key(&conn, &ws, &admin, port3);
    let run3 = ai_eval_service::run_suite(&conn, &ws, &master_key(), &unschema_suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run3.failed_count, 1);
    assert!(run3.results[0].error.as_deref().unwrap_or_default().contains("no Structured Output schema declared"));
}

#[tokio::test]
async fn policy_compliance_evaluator_checks_for_a_blocked_tool_call_with_no_judge_call() {
    let (conn, ws, admin) = setup_workspace();
    let agent = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput { name: "Lister".into(), description: None, icon: "🤖".into(), system_prompt: "You list objects.".into(), action_names: vec!["list_objects".into()], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    let suite = ai_eval_service::create_suite(
        &conn, &ws,
        &AiEvalSuiteInput {
            name: "Policy compliance check".into(), description: None, target_type: "agent".into(), target_id: agent.id.clone(),
            evaluator_type: "policy_compliance".into(),
            cases: vec![AiEvalCaseInput { input_text: "list every object".into(), success_criteria: String::new() }],
        },
        Some(&admin),
    )
    .unwrap();

    // No policy configured yet - the tool call dispatches cleanly, so the
    // run passes.
    let (port, captured) = spawn_sequence_stub(vec![anthropic_tool_use_body("t1", "list_objects", serde_json::json!({})), anthropic_text_body("Here they are.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let run = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(captured.lock().unwrap().len(), 2, "target run's own two rounds, no third judge call");
    assert_eq!(run.passed_count, 1);
    assert_eq!(run.failed_count, 0);

    // Block the tool outright under this agent's policy - the same call
    // now trips the Tool-Call Firewall, and the eval records it as a
    // failure with no judge call either.
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec!["list_objects".into()], exclude_restricted_memory: true, enforce_record_access: false }, Some(&admin)).unwrap();
    let (port2, captured2) = spawn_sequence_stub(vec![anthropic_tool_use_body("t2", "list_objects", serde_json::json!({})), anthropic_text_body("Understood, I won't do that.")]);
    configure_anthropic_key(&conn, &ws, &admin, port2);
    let run2 = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(captured2.lock().unwrap().len(), 2);
    assert_eq!(run2.passed_count, 0);
    assert_eq!(run2.failed_count, 1);
    assert!(run2.results[0].judge_reasoning.as_deref().unwrap_or_default().contains("blocked or required approval"));

    // A require-approval threshold at or below the tool's own risk level
    // (read, for list_objects) also counts as a violation - not just an
    // outright blocklist entry.
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { require_approval_at_or_above: Some(RiskLevel::Read), blocked_tool_names: vec![], exclude_restricted_memory: true, enforce_record_access: false }, Some(&admin)).unwrap();
    let (port3, _c3) = spawn_sequence_stub(vec![anthropic_tool_use_body("t3", "list_objects", serde_json::json!({})), anthropic_text_body("Noted - awaiting approval.")]);
    configure_anthropic_key(&conn, &ws, &admin, port3);
    let run3 = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run3.failed_count, 1);
}

#[tokio::test]
async fn a_malformed_judge_reply_is_graded_as_failed_not_silently_passed() {
    let (conn, ws, admin) = setup_workspace();
    let agent = make_agent(&conn, &ws, &admin, "Answerer");
    let suite = ai_eval_service::create_suite(
        &conn, &ws,
        &AiEvalSuiteInput {
            name: "One case".into(), description: None, target_type: "agent".into(), target_id: agent.id.clone(),
            evaluator_type: "task_completion".into(),
            cases: vec![AiEvalCaseInput { input_text: "hello".into(), success_criteria: "says hello back".into() }],
        },
        Some(&admin),
    )
    .unwrap();

    // The "judge" replies with neither PASS nor FAIL - graded as failed,
    // fail-closed, rather than defaulting to a pass.
    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("hello there"), anthropic_text_body("I refuse to grade this.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_eval_service::run_suite(&conn, &ws, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run.passed_count, 0);
    assert_eq!(run.failed_count, 1);
    assert!(!run.results[0].passed);
}
