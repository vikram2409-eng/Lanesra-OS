//! AI & Agentic Layer, Phase 7c: an agent's Guardrails (prompted, via
//! `guardrails_md`), the loop-detection guard `run_agent_once` now
//! enforces in code, and declarative export/import of Agents/Skills via
//! the Solution manifest mechanism. Reuses `ai_context_layer.rs`'s own
//! stub-listener/captured-request-body pattern.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::{AiAgentInput, AiSkillInput};
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::solution::{SolutionInput, SolutionMemberInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_service, chat_service, industry_package_service, solution_service, workspace_service};

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
    [93u8; 32]
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}).to_string()
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
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
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (port, captured)
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str, action_names: Vec<String>) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput {
            name: name.into(), description: None, icon: "🤖".into(), system_prompt: format!("You are {name}."),
            action_names, delegate_agent_ids: vec![], skill_ids: vec![],
        },
        Some(admin),
    )
    .unwrap()
}

// --- Guardrails: prompted, not enforced -------------------------------

#[test]
fn guardrails_md_persists() {
    let (conn, ws, admin) = setup_workspace("Guardrails Test Co");
    let agent = make_agent(&conn, &ws, &admin, "Compliance Bot", vec![]);
    assert_eq!(agent.guardrails_md, "");

    let updated = ai_agent_service::set_guardrails(&conn, &agent.id, "Never approve a payout over $10,000 without human sign-off.", Some(&admin)).unwrap();
    assert_eq!(updated.guardrails_md, "Never approve a payout over $10,000 without human sign-off.");
}

#[tokio::test]
async fn guardrails_md_appears_in_the_system_prompt_sent_to_the_model() {
    let (conn, ws, admin) = setup_workspace("Guardrails Test Co");
    let agent = make_agent(&conn, &ws, &admin, "Compliance Bot", vec![]);
    ai_agent_service::set_guardrails(&conn, &agent.id, "Never mark a claim Paid above $10,000 without executive approval.", Some(&admin)).unwrap();

    let (port, captured) = spawn_sequence_stub(vec![anthropic_text_body("Understood.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);
    chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "hello").await.unwrap();

    let requests = captured.lock().unwrap();
    assert!(
        requests[0].contains("Never mark a claim Paid above $10,000 without executive approval."),
        "guardrails missing from request: {}",
        requests[0]
    );
}

// --- Loop detection: code-enforced -------------------------------------

#[tokio::test]
async fn three_identical_consecutive_tool_calls_trip_the_loop_guard() {
    let (conn, ws, admin) = setup_workspace("Loop Guard Test Co");
    let agent = make_agent(&conn, &ws, &admin, "Looper", vec!["list_objects".into()]);

    let same_call = anthropic_tool_use_body("t", "list_objects", serde_json::json!({}));
    let (port, _captured) = spawn_sequence_stub(vec![same_call.clone(), same_call.clone(), same_call]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let err = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "list everything forever").await.unwrap_err();
    assert!(err.to_string().contains("same input 3 times in a row"), "unexpected error: {err}");
}

#[tokio::test]
async fn varying_arguments_never_trips_the_loop_guard() {
    let (conn, ws, admin) = setup_workspace("Loop Guard Test Co");
    let agent = make_agent(&conn, &ws, &admin, "Careful Lister", vec!["get_object_metadata".into()]);

    // Three calls to the same tool, but with different arguments each
    // time - never counts as a repeat.
    let (port, _captured) = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "get_object_metadata", serde_json::json!({"object_key": "Company"})),
        anthropic_tool_use_body("t2", "get_object_metadata", serde_json::json!({"object_key": "Contact"})),
        anthropic_tool_use_body("t3", "get_object_metadata", serde_json::json!({"object_key": "Opportunity"})),
        anthropic_text_body("Done."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "describe three objects").await.unwrap();
    assert!(messages.iter().any(|m| m.content.as_deref() == Some("Done.")));
}

// --- Declarative export/import via a Solution --------------------------

#[test]
fn a_solution_export_round_trips_an_agent_its_skill_and_its_delegate_into_a_second_workspace() {
    let (conn_a, ws_a, admin_a) = setup_workspace("Source Workspace Co");

    let skill = ai_agent_service::create_skill(
        &conn_a, &ws_a,
        &AiSkillInput { name: "Fraud Heuristics".into(), description: "Rules of thumb for spotting a suspicious claim".into(), instructions_md: "Flag any claim over $50,000 filed within 48 hours of the policy's start date.".into() },
        Some(&admin_a),
    )
    .unwrap();

    let sub_agent = ai_agent_service::create(
        &conn_a, &ws_a,
        &AiAgentInput { name: "Policy Lookup Agent".into(), description: None, icon: "📄".into(), system_prompt: "You look up policy details.".into(), action_names: vec!["get_record".into()], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin_a),
    )
    .unwrap();

    let lead_agent = ai_agent_service::create(
        &conn_a, &ws_a,
        &AiAgentInput {
            name: "Claims Auditor".into(), description: Some("Audits incoming claims".into()), icon: "🕵️".into(),
            system_prompt: "You audit insurance claims against policy.".into(), action_names: vec!["list_records".into()],
            delegate_agent_ids: vec![sub_agent.id.clone()], skill_ids: vec![skill.id.clone()],
        },
        Some(&admin_a),
    )
    .unwrap();
    ai_agent_service::set_memory(&conn_a, &lead_agent.id, "Tier 1 claims (<= $1,000) auto-approve.".into(), Some(&admin_a)).unwrap();
    ai_agent_service::set_guardrails(&conn_a, &lead_agent.id, "Never mark a claim Paid above $10,000 without executive approval.", Some(&admin_a)).unwrap();

    let solution = solution_service::create(&conn_a, &ws_a, &SolutionInput { name: "Claims AI Bundle".into(), description: None, version: None, publisher_id: None }, Some(&admin_a)).unwrap();
    for (artifact_type, metadata_id) in [("ai_skill", skill.id.as_str()), ("ai_agent", sub_agent.id.as_str()), ("ai_agent", lead_agent.id.as_str())] {
        solution_service::add_component(&conn_a, &ws_a, &solution.id, &SolutionMemberInput { artifact_type: artifact_type.into(), metadata_id: metadata_id.into() }, Some(&admin_a)).unwrap();
    }

    let manifest_json = industry_package_service::export_solution(&conn_a, &ws_a, &solution.id, Some(&admin_a)).unwrap();
    assert!(manifest_json.contains("Claims Auditor"));
    assert!(manifest_json.contains("Fraud Heuristics"));

    // Import into a completely separate workspace/database.
    let (conn_b, ws_b, admin_b) = setup_workspace("Destination Workspace Co");
    let package = industry_package_service::import_package(&conn_b, &ws_b, &ImportPackageInput { manifest_json }, Some(&admin_b)).unwrap();
    industry_package_service::install(&conn_b, &ws_b, &package.id, Some(&admin_b)).unwrap();

    let imported_agents = ai_agent_service::list(&conn_b, &ws_b, true).unwrap();
    let imported_lead = imported_agents.iter().find(|a| a.name == "Claims Auditor").expect("lead agent imported");
    let imported_sub = imported_agents.iter().find(|a| a.name == "Policy Lookup Agent").expect("delegate imported");
    let imported_skills = ai_agent_service::list_skills(&conn_b, &ws_b, true).unwrap();
    let imported_skill = imported_skills.iter().find(|s| s.name == "Fraud Heuristics").expect("skill imported");

    assert_eq!(imported_lead.memory_md, "Tier 1 claims (<= $1,000) auto-approve.");
    assert_eq!(imported_lead.guardrails_md, "Never mark a claim Paid above $10,000 without executive approval.");
    assert_eq!(imported_lead.action_names, vec!["list_records".to_string()]);
    assert_eq!(imported_lead.skill_ids, vec![imported_skill.id.clone()]);
    assert_eq!(imported_lead.delegate_agent_ids, vec![imported_sub.id.clone()]);
    assert_eq!(imported_skill.instructions_md, "Flag any claim over $50,000 filed within 48 hours of the policy's start date.");
}
