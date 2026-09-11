//! AI & Agentic Layer, Phase 7d: the two new orchestration topologies on
//! top of Phase 6b's Pipeline - consensus (candidates run independently,
//! a synthesizer combines them) and peer_review (a drafter/reviewer loop
//! bounded at 3 rounds). Reuses `ai_agent_orchestration.rs`'s own
//! stub-listener pattern.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_pipeline::{AiAgentPipelineInput, PipelineStepInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_orchestration_service, ai_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Topologies Test Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn master_key() -> [u8; 32] {
    [77u8; 32]
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

/// Same raw-socket stub `ai_agent_orchestration.rs`/`ai_agent_guardrails.rs`
/// already use - serves one canned response body per request in order
/// (repeating the last once exhausted), capturing each request's raw
/// JSON body so a test can assert exactly what was sent.
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
async fn pipeline_crud_rejects_the_wrong_step_count_for_consensus_and_peer_review() {
    let (conn, ws, admin) = setup_workspace();
    let a1 = make_agent(&conn, &ws, &admin, "Solo");

    let one_step = |topology: &str| AiAgentPipelineInput {
        name: "Bad".into(), description: None, topology: topology.into(),
        steps: vec![PipelineStepInput { agent_id: a1.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false }],
    };
    let consensus_err = ai_orchestration_service::create_pipeline(&conn, &ws, &one_step("consensus"), Some(&admin)).unwrap_err();
    assert!(consensus_err.to_string().contains("consensus"), "{consensus_err}");
    let peer_review_err = ai_orchestration_service::create_pipeline(&conn, &ws, &one_step("peer_review"), Some(&admin)).unwrap_err();
    assert!(peer_review_err.to_string().contains("peer-review"), "{peer_review_err}");

    let bad_topology = AiAgentPipelineInput { name: "Bad".into(), description: None, topology: "made_up".into(), steps: one_step("sequential").steps };
    assert!(ai_orchestration_service::create_pipeline(&conn, &ws, &bad_topology, Some(&admin)).is_err());
}

#[tokio::test]
async fn a_consensus_run_gives_every_candidate_the_same_trigger_input_and_synthesizes_them() {
    let (conn, ws, admin) = setup_workspace();
    let candidate_a = make_agent(&conn, &ws, &admin, "Candidate A");
    let candidate_b = make_agent(&conn, &ws, &admin, "Candidate B");
    let synthesizer = make_agent(&conn, &ws, &admin, "Synthesizer");

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Consensus vote".into(), description: None, topology: "consensus".into(),
            steps: vec![
                PipelineStepInput { agent_id: candidate_a.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: candidate_b.id.clone(), input_template: "{{trigger_input}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: synthesizer.id.clone(), input_template: "Question: {{trigger_input}}\n\nAnswers:\n{{candidate_outputs}}".into(), requires_approval: false },
            ],
        },
        Some(&admin),
    )
    .unwrap();

    let (port, captured) = spawn_sequence_stub(vec![
        anthropic_text_body("42"),
        anthropic_text_body("forty-two"),
        anthropic_text_body("The answers agree: the number is 42."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "What is the answer?", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(run.steps.len(), 3);

    // Both candidates saw the trigger input directly - never chained to
    // each other (neither one's input contains the other's output).
    assert_eq!(run.steps[0].input_text, "What is the answer?");
    assert_eq!(run.steps[1].input_text, "What is the answer?");
    assert_eq!(run.steps[0].output_text.as_deref(), Some("42"));
    assert_eq!(run.steps[1].output_text.as_deref(), Some("forty-two"));

    // The synthesizer's real resolved input carried both candidates'
    // real answers, numbered.
    let synth_input = &run.steps[2].input_text;
    assert!(synth_input.contains("Candidate 1: 42"), "{synth_input}");
    assert!(synth_input.contains("Candidate 2: forty-two"), "{synth_input}");
    assert_eq!(run.steps[2].output_text.as_deref(), Some("The answers agree: the number is 42."));

    // Not just the recorded text - the actual outbound request too.
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests[2].contains("Candidate 1: 42"), "{}", requests[2]);
    assert!(requests[2].contains("Candidate 2: forty-two"), "{}", requests[2]);
}

#[tokio::test]
async fn peer_review_approves_on_the_first_round_when_the_reviewer_says_so() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let reviewer = make_agent(&conn, &ws, &admin, "Reviewer");

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Draft and review".into(), description: None, topology: "peer_review".into(),
            steps: vec![
                PipelineStepInput { agent_id: drafter.id.clone(), input_template: "Draft: {{trigger_input}}. Feedback so far: {{previous_output}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: reviewer.id.clone(), input_template: "Review this draft: {{previous_output}}".into(), requires_approval: false },
            ],
        },
        Some(&admin),
    )
    .unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![anthropic_text_body("Here is my first draft."), anthropic_text_body("APPROVED - clean and correct.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "write a haiku", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "succeeded", "{run:?}");
    assert_eq!(run.steps.len(), 2, "should stop after exactly one draft+review round once approved");
    assert_eq!(run.steps[0].input_text, "Draft: write a haiku. Feedback so far: ");
    assert_eq!(run.steps[0].output_text.as_deref(), Some("Here is my first draft."));
    assert_eq!(run.steps[1].input_text, "Review this draft: Here is my first draft.");
    assert_eq!(run.steps[1].output_text.as_deref(), Some("APPROVED - clean and correct."));
}

#[tokio::test]
async fn peer_review_feeds_the_reviewers_feedback_back_into_the_next_draft_and_gives_up_after_3_rounds() {
    let (conn, ws, admin) = setup_workspace();
    let drafter = make_agent(&conn, &ws, &admin, "Drafter");
    let reviewer = make_agent(&conn, &ws, &admin, "Reviewer");

    let pipeline = ai_orchestration_service::create_pipeline(
        &conn, &ws,
        &AiAgentPipelineInput {
            name: "Never satisfied".into(), description: None, topology: "peer_review".into(),
            steps: vec![
                PipelineStepInput { agent_id: drafter.id.clone(), input_template: "{{trigger_input}} | prior feedback: {{previous_output}}".into(), requires_approval: false },
                PipelineStepInput { agent_id: reviewer.id.clone(), input_template: "critique: {{previous_output}}".into(), requires_approval: false },
            ],
        },
        Some(&admin),
    )
    .unwrap();

    let (port, _captured) = spawn_sequence_stub(vec![
        anthropic_text_body("draft v1"),
        anthropic_text_body("Needs more detail."),
        anthropic_text_body("draft v2"),
        anthropic_text_body("Still needs more detail."),
        anthropic_text_body("draft v3"),
        anthropic_text_body("Not there yet."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let run = ai_orchestration_service::run_manual(&conn, &ws, &master_key(), "pipeline", &pipeline.id, "write the spec", Some(&admin)).await.unwrap();
    assert_eq!(run.status, "failed", "{run:?}");
    assert_eq!(run.steps.len(), 6, "3 rounds of draft+review, never approved");
    assert!(run.error.as_deref().unwrap_or("").contains("3 rounds"), "{:?}", run.error);

    // Round 2's draft genuinely carried round 1's real review feedback.
    assert_eq!(run.steps[2].input_text, "write the spec | prior feedback: Needs more detail.");
    assert_eq!(run.steps[4].input_text, "write the spec | prior feedback: Still needs more detail.");
}
