//! AI & Agentic Layer, Phase 7e: Human-in-the-loop approval gates on a
//! sequential Pipeline (pause, approve - with or without editing the
//! output - and reject), plus the OTLP trace exporter. Phase 7f adds the
//! collector push on top: the settings dial and the real outbound POST.
//! Reuses `ai_agent_orchestration.rs`'s own stub-listener pattern.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::{AiObservabilitySettingsInput, AiSettingsInput};
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_orchestration_service, ai_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "HITL Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [11u8; 32]
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

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

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

fn two_step_pipeline(step1: &str, step2: &str, requires_approval: bool) -> AiAgentPipelineInput {
    AiAgentPipelineInput {
        name: "Gate then finish".into(), description: None, topology: "sequential".into(),
        steps: vec![
            PipelineStepInput { agent_id: step1.into(), input_template: "{{trigger_input}}".into(), requires_approval },
            PipelineStepInput { agent_id: step2.into(), input_template: "Finalize: {{previous_output}}".into(), requires_approval: false },
        ],
    }
}

#[tokio::test]
async fn a_requires_approval_step_pauses_the_run_and_a_non_admin_cannot_act_on_it() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, true), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft output")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "awaiting_approval", "{run:?}");
    assert_eq!(run.steps.len(), 1, "should stop right after the gated step");
    assert_eq!(run.steps[0].output_text.as_deref(), Some("draft output"));
    assert_eq!(run.paused_at_step_order, Some(1));
    assert_eq!(run.resume_previous_output.as_deref(), Some("draft output"));
    assert_eq!(run.trigger_input, "hello");

    let denied = ai_orchestration_service::approve_pending_step(&conn, &ws, &master_key(), &run.id, None, Some(&rep)).await;
    assert!(denied.unwrap_err().to_string().contains("Administrator"));
    let denied_reject = ai_orchestration_service::reject_pending_run(&conn, &run.id, "no", Some(&rep));
    assert!(denied_reject.unwrap_err().to_string().contains("Administrator"));
}

#[tokio::test]
async fn approving_unchanged_resumes_with_the_gated_steps_real_output() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, true), Some(&admin)).unwrap();

    let (port, captured) = spawn_sequence_stub(vec![anthropic_text_body("draft output"), anthropic_text_body("Finalized: draft output")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let paused = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    assert_eq!(paused.status, "awaiting_approval");

    let resumed = ai_orchestration_service::approve_pending_step(&conn, &ws, &master_key(), &paused.id, None, Some(&admin)).await.unwrap();
    assert_eq!(resumed.status, "succeeded", "{resumed:?}");
    assert_eq!(resumed.steps.len(), 2);
    assert_eq!(resumed.steps[1].input_text, "Finalize: draft output");
    assert_eq!(resumed.steps[1].output_text.as_deref(), Some("Finalized: draft output"));
    assert!(resumed.paused_at_step_order.is_none(), "a finished run clears its pause state");

    // The real outbound request for step 2 genuinely carried the gated
    // step's real output.
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[1].contains("Finalize: draft output"), "{}", requests[1]);
}

#[tokio::test]
async fn approving_with_an_edit_overrides_the_gated_steps_output() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, true), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft output"), anthropic_text_body("Finalized: corrected text")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let paused = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    let resumed = ai_orchestration_service::approve_pending_step(&conn, &ws, &master_key(), &paused.id, Some("corrected text"), Some(&admin)).await.unwrap();
    assert_eq!(resumed.status, "succeeded", "{resumed:?}");
    assert_eq!(resumed.steps[1].input_text, "Finalize: corrected text", "the edited output, not the original draft, should feed step 2");
}

#[tokio::test]
async fn rejecting_a_paused_run_stops_it_and_it_cannot_be_acted_on_again() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, true), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft output")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let paused = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    let rejected = ai_orchestration_service::reject_pending_run(&conn, &paused.id, "not good enough", Some(&admin)).unwrap();
    assert_eq!(rejected.status, "rejected");
    assert_eq!(rejected.error.as_deref(), Some("not good enough"));
    assert_eq!(rejected.steps.len(), 1, "no further step should ever run on a rejected run");

    let second_reject = ai_orchestration_service::reject_pending_run(&conn, &paused.id, "again", Some(&admin));
    assert!(second_reject.unwrap_err().to_string().contains("not awaiting approval"));
    let approve_after_reject = ai_orchestration_service::approve_pending_step(&conn, &ws, &master_key(), &paused.id, None, Some(&admin)).await;
    assert!(approve_after_reject.unwrap_err().to_string().contains("not awaiting approval"));
}

#[tokio::test]
async fn requires_approval_is_rejected_outside_the_sequential_topology() {
    let (conn, ws, admin) = setup_workspace();
    let a1 = make_agent(&conn, &ws, &admin, "A1");
    let a2 = make_agent(&conn, &ws, &admin, "A2");
    let bad = AiAgentPipelineInput {
        name: "Bad".into(), description: None, topology: "peer_review".into(),
        steps: vec![
            PipelineStepInput { agent_id: a1.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: true },
            PipelineStepInput { agent_id: a2.id.clone(), input_template: "{{previous_output}}".into(), requires_approval: false },
        ],
    };
    let err = ai_orchestration_service::create_pipeline(&conn, &ws, &bad, Some(&admin)).unwrap_err();
    assert!(err.to_string().contains("sequential"), "{err}");
}

#[tokio::test]
async fn the_otlp_export_is_a_deterministic_two_span_trace_for_a_two_step_run() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, false), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft output"), anthropic_text_body("Finalized: draft output")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "succeeded");

    let trace_a = ai_orchestration_service::export_run_as_otlp(&conn, &run.id).unwrap();
    let trace_b = ai_orchestration_service::export_run_as_otlp(&conn, &run.id).unwrap();
    assert_eq!(trace_a, trace_b, "exporting the same finished run twice should be byte-identical");

    let spans = trace_a["resourceSpans"][0]["scopeSpans"][0]["spans"].as_array().unwrap();
    assert_eq!(spans.len(), 3, "1 root span + 2 step spans");
    let root_trace_id = spans[0]["traceId"].as_str().unwrap().to_string();
    assert!(spans[0].get("parentSpanId").is_none(), "the root span has no parent");
    for step_span in &spans[1..] {
        assert_eq!(step_span["traceId"].as_str().unwrap(), root_trace_id, "every step span shares the run's trace id");
        assert_eq!(step_span["parentSpanId"].as_str().unwrap(), spans[0]["spanId"].as_str().unwrap());
    }
}

// --- Phase 7f: pushing a run's OTLP trace to a configured collector ------

/// A plain HTTP stub (unlike `spawn_sequence_stub`, which wraps every
/// body in an Anthropic-shaped `{"content": [...]}`) - returns a fixed
/// status/body for every request, capturing each raw request body
/// verbatim, so a test can assert exactly what `push_run_trace_to_otlp`
/// actually sent.
fn spawn_status_stub(status_line: &'static str, response_body: &'static str) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
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
            let response = format!("HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response_body.len(), response_body);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

#[tokio::test]
async fn otlp_endpoint_setting_is_administrator_gated_and_blank_clears_it() {
    let (conn, ws, admin) = setup_workspace();
    let rep = non_admin_user(&conn, &ws, &admin);

    let denied = ai_service::set_otlp_endpoint(&conn, &ws, &AiObservabilitySettingsInput { otlp_endpoint: Some("http://example.com".into()) }, Some(&rep));
    assert!(denied.unwrap_err().to_string().contains("Administrator"));

    let saved = ai_service::set_otlp_endpoint(&conn, &ws, &AiObservabilitySettingsInput { otlp_endpoint: Some("http://collector.example.com/v1/traces".into()) }, Some(&admin)).unwrap();
    assert_eq!(saved.otlp_endpoint.as_deref(), Some("http://collector.example.com/v1/traces"));

    let cleared = ai_service::set_otlp_endpoint(&conn, &ws, &AiObservabilitySettingsInput { otlp_endpoint: Some("   ".into()) }, Some(&admin)).unwrap();
    assert_eq!(cleared.otlp_endpoint, None, "blank/whitespace-only should clear it, not store an empty string");
}

#[tokio::test]
async fn pushing_a_trace_with_no_endpoint_configured_fails_clearly() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, false), Some(&admin)).unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft"), anthropic_text_body("final")]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();

    let err = ai_orchestration_service::push_run_trace_to_otlp(&conn, &ws, &run.id, Some(&admin)).await.unwrap_err();
    assert!(err.to_string().contains("No OTLP collector endpoint"), "{err}");
}

#[tokio::test]
async fn pushing_a_trace_posts_the_real_otlp_json_and_surfaces_a_collector_failure() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let finisher = make_agent(&conn, &ws, &admin, "Finisher");
    let pipeline = ai_orchestration_service::create_pipeline(&conn, &ws, &two_step_pipeline(&drafter.id, &finisher.id, false), Some(&admin)).unwrap();

    let (agent_port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("draft"), anthropic_text_body("final")]);
    configure_anthropic_key(&conn, &ws, &admin, agent_port);
    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "hello", Some(&admin)).await.unwrap();
    let expected_trace = ai_orchestration_service::export_run_as_otlp(&conn, &run.id).unwrap();

    let (collector_port, collector_captured) = spawn_status_stub("200 OK", "ok");
    ai_service::set_otlp_endpoint(&conn, &ws, &AiObservabilitySettingsInput { otlp_endpoint: Some(format!("http://127.0.0.1:{collector_port}/v1/traces")) }, Some(&admin)).unwrap();

    let ok_message = ai_orchestration_service::push_run_trace_to_otlp(&conn, &ws, &run.id, Some(&admin)).await.unwrap();
    assert!(ok_message.contains("200"), "{ok_message}");
    let requests = collector_captured.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let sent: serde_json::Value = serde_json::from_str(&requests[0]).unwrap();
    assert_eq!(sent, expected_trace, "the collector should receive exactly the same trace run_to_otlp_json/export_run_as_otlp produces");
    drop(requests);

    let (failing_port, _failing_captured) = spawn_status_stub("500 Internal Server Error", "collector on fire");
    ai_service::set_otlp_endpoint(&conn, &ws, &AiObservabilitySettingsInput { otlp_endpoint: Some(format!("http://127.0.0.1:{failing_port}/v1/traces")) }, Some(&admin)).unwrap();
    let err = ai_orchestration_service::push_run_trace_to_otlp(&conn, &ws, &run.id, Some(&admin)).await.unwrap_err();
    assert!(err.to_string().contains("500"), "{err}");
}
