//! AI Agent Platform v2, Phase 1 follow-up: `agent_version_service`'s
//! Draft -> Test -> Published -> Deprecated -> Disabled lifecycle
//! (immutability once Published, auto-deprecation of the prior Published
//! version on publish) and `approval_service`'s create/resolve round
//! trip - through the real service layer this time, unlike
//! `ai_agent_versioning.rs`'s migration-only, direct-SQL scope.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::{validate_output_schema, AiAgentInput, AiAgentVersionInput};
use lanesra_core::models::ai_agent_policy::AiAgentPolicyInput;
use lanesra_core::models::ai_approval::AiApprovalInput;
use lanesra_core::models::ai_eval::{AiEvalCaseInput, AiEvalSuiteInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{agent_version_service, ai_agent_service, ai_eval_service, ai_service, approval_service, policy_engine_service, workspace_service};

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

fn version_input(name: &str) -> AiAgentVersionInput {
    AiAgentVersionInput {
        name: name.into(), description: None, icon: "🤖".into(), system_prompt: "Updated persona.".into(),
        action_names: vec!["get_record".into()], delegate_agent_ids: vec![], skill_ids: vec![],
        model_routing: None, output_schema: None,
    }
}

#[test]
fn publishing_a_new_version_deprecates_the_previous_published_one() {
    let (conn, workspace_id, admin) = setup_workspace("Version Lifecycle Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Auditor".into(), description: None, icon: "🕵".into(), system_prompt: "You audit.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();
    let v1_id = agent.current_version_id.clone().expect("agent should have a v1 on creation");

    let draft = agent_version_service::create_draft(&conn, &agent.id, &workspace_id, &version_input("Auditor v2"), Some(&admin)).unwrap();
    assert_eq!(draft.version_number, 2);
    assert_eq!(draft.status, "draft");

    agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "test", Some(&admin)).unwrap();
    let published = agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "published", Some(&admin)).unwrap();
    assert_eq!(published.status, "published");
    assert!(published.published_at.is_some());

    let versions = agent_version_service::list_versions(&conn, &agent.id, &workspace_id, Some(&admin)).unwrap();
    let v1 = versions.iter().find(|v| v.id == v1_id).unwrap();
    assert_eq!(v1.status, "deprecated", "publishing v2 should have auto-deprecated v1");

    let refreshed_agent = ai_agent_service::get(&conn, &agent.id).unwrap().unwrap();
    assert_eq!(refreshed_agent.current_version_id.as_deref(), Some(draft.id.as_str()));
}

#[test]
fn illegal_transitions_are_rejected() {
    let (conn, workspace_id, admin) = setup_workspace("Illegal Transition Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Agent".into(), description: None, icon: "🤖".into(), system_prompt: "P.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();
    let v1_id = agent.current_version_id.unwrap();

    // v1 is already 'published' - publish -> published is not a listed transition.
    let result = agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &v1_id, "published", Some(&admin));
    assert!(result.is_err(), "publishing an already-Published version should be rejected");

    // draft -> disabled is legal, but disabled -> test is not.
    let draft = agent_version_service::create_draft(&conn, &agent.id, &workspace_id, &version_input("v2"), Some(&admin)).unwrap();
    agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "disabled", Some(&admin)).unwrap();
    let result = agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "test", Some(&admin));
    assert!(result.is_err(), "a Disabled version should be a dead end, not resurrectable to Test");
}

#[test]
fn a_published_version_is_immutable() {
    let (conn, workspace_id, admin) = setup_workspace("Immutability Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Agent".into(), description: None, icon: "🤖".into(), system_prompt: "P.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();
    let v1_id = agent.current_version_id.unwrap();

    let result = agent_version_service::update_draft(&conn, &agent.id, &workspace_id, &v1_id, &version_input("Renamed"), Some(&admin));
    assert!(result.is_err(), "a Published version's content must not be editable");

    // A Draft/Test version, by contrast, can still be edited freely.
    let draft = agent_version_service::create_draft(&conn, &agent.id, &workspace_id, &version_input("v2 draft"), Some(&admin)).unwrap();
    let edited = agent_version_service::update_draft(&conn, &agent.id, &workspace_id, &draft.id, &version_input("v2 draft, edited"), Some(&admin)).unwrap();
    assert_eq!(edited.name, "v2 draft, edited");
}

#[test]
fn approval_create_resolve_and_double_resolve_rejection() {
    let (conn, workspace_id, admin) = setup_workspace("Approvals Round Trip Co");
    let created = approval_service::create(
        &conn, &workspace_id,
        &AiApprovalInput { subject_type: "agent_version_publish".into(), subject_id: "some-version-id".into(), proposal: serde_json::json!({"proposed_status": "published"}) },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(created.status, "pending");

    let pending = approval_service::list(&conn, &workspace_id, Some("pending"), Some(&admin)).unwrap();
    assert_eq!(pending.len(), 1);

    let resolved = approval_service::resolve(
        &conn, &created.id, &workspace_id,
        &lanesra_core::models::ai_approval::AiApprovalResolution { approve: true, resolution_notes: Some("looks fine".into()) },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(resolved.status, "approved");
    assert_eq!(resolved.resolved_by.as_deref(), Some(admin.as_str()));

    let result = approval_service::resolve(
        &conn, &created.id, &workspace_id,
        &lanesra_core::models::ai_approval::AiApprovalResolution { approve: false, resolution_notes: None },
        Some(&admin),
    );
    assert!(result.is_err(), "an already-resolved approval must not be resolvable a second time");
}

#[test]
fn output_schema_validation_reports_type_and_required_violations() {
    let schema = serde_json::json!({
        "type": "object",
        "required": ["answer", "confidence"],
        "properties": {
            "answer": {"type": "string"},
            "confidence": {"type": "number"},
        },
    });

    let valid = serde_json::json!({"answer": "42", "confidence": 0.9});
    assert!(validate_output_schema(&schema, &valid).is_empty());

    let missing_field = serde_json::json!({"answer": "42"});
    let errors = validate_output_schema(&schema, &missing_field);
    assert!(errors.iter().any(|e| e.contains("confidence")));

    let wrong_type = serde_json::json!({"answer": 42, "confidence": "high"});
    let errors = validate_output_schema(&schema, &wrong_type);
    assert!(errors.iter().any(|e| e.contains(".answer")));
    assert!(errors.iter().any(|e| e.contains(".confidence")));
}

fn master_key() -> [u8; 32] {
    [91u8; 32]
}

fn anthropic_text_body(text: &str) -> String {
    serde_json::json!({"content": [{"type": "text", "text": text}]}).to_string()
}

fn anthropic_tool_use_body(id: &str, name: &str, input: serde_json::Value) -> String {
    serde_json::json!({"content": [{"type": "tool_use", "id": id, "name": name, "input": input}]}).to_string()
}

fn configure_anthropic_key(conn: &rusqlite::Connection, workspace_id: &str, admin: &str, port: u16) {
    ai_service::save_settings(
        conn, workspace_id, &master_key(),
        &AiSettingsInput { provider: "anthropic".into(), base_url: Some(format!("http://127.0.0.1:{port}")), model: "claude-haiku-4-5-20251001".into(), api_key: Some("sk-ant-test".into()) },
        Some(admin),
    )
    .unwrap();
}

/// Same raw-socket stub every other AI Foundry test file uses.
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

// AI Agent Platform v2, Phase 6a (AI-AC-12): a policy_compliance Eval
// Suite's most recent failing run blocks publishing a new version for
// its target agent - `agent_version_service::transition_status`'s own
// real gate, not just the online demo mirror's.
#[tokio::test]
async fn publishing_is_blocked_by_a_failing_policy_compliance_eval_run() {
    let (conn, workspace_id, admin) = setup_workspace("Publish Gate Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Lister".into(), description: None, icon: "🤖".into(), system_prompt: "You list objects.".into(), action_names: vec!["list_objects".into()], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    let suite = ai_eval_service::create_suite(
        &conn, &workspace_id,
        &AiEvalSuiteInput {
            name: "Policy gate check".into(), description: None, target_type: "agent".into(), target_id: agent.id.clone(),
            evaluator_type: "policy_compliance".into(),
            cases: vec![AiEvalCaseInput { input_text: "list every object".into(), success_criteria: String::new() }],
        },
        Some(&admin),
    )
    .unwrap();

    policy_engine_service::upsert_policy(&conn, &workspace_id, Some(&agent.id), &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec!["list_objects".into()], exclude_restricted_memory: true }, Some(&admin)).unwrap();
    let port = spawn_sequence_stub(vec![anthropic_tool_use_body("t1", "list_objects", serde_json::json!({})), anthropic_text_body("Understood, I won't do that.")]);
    configure_anthropic_key(&conn, &workspace_id, &admin, port);
    let run = ai_eval_service::run_suite(&conn, &workspace_id, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run.failed_count, 1, "the policy_compliance run should have failed");

    let draft = agent_version_service::create_draft(&conn, &agent.id, &workspace_id, &version_input("v2"), Some(&admin)).unwrap();
    agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "test", Some(&admin)).unwrap();
    let blocked = agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "published", Some(&admin));
    assert!(blocked.is_err(), "publish should be blocked by the failing policy_compliance run");
    assert!(blocked.unwrap_err().to_string().contains("Policy gate check"));

    // Resolve the violation (remove the tool from the policy's blocklist)
    // and re-run - now it passes, and publish is no longer blocked.
    policy_engine_service::upsert_policy(&conn, &workspace_id, Some(&agent.id), &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec![], exclude_restricted_memory: true }, Some(&admin)).unwrap();
    let port2 = spawn_sequence_stub(vec![anthropic_tool_use_body("t2", "list_objects", serde_json::json!({})), anthropic_text_body("Here they are.")]);
    configure_anthropic_key(&conn, &workspace_id, &admin, port2);
    let run2 = ai_eval_service::run_suite(&conn, &workspace_id, &master_key(), &suite.id, Some(&admin)).await.unwrap();
    assert_eq!(run2.failed_count, 0);

    let published = agent_version_service::transition_status(&conn, &agent.id, &workspace_id, &draft.id, "published", Some(&admin));
    assert!(published.is_ok(), "publish should succeed once the suite's most recent run passes: {:?}", published.err());
}
