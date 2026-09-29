//! AI Agent Platform v2, Phase 2: the Tool Registry's risk classification
//! (`tool_registry_service`) and the Policy Engine's Tool-Call Firewall
//! decision (`policy_engine_service`) - plus, at the bottom, the real
//! wiring into `chat_service::execute_tool` this whole phase exists to
//! gate, through the identical stub-listener pattern
//! `ai_agent_guardrails.rs` already established.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_policy::{AiAgentPolicyInput, PolicyDecision};
use lanesra_core::models::ai_tool_registry::{AiToolRegistryOverrideInput, RiskLevel};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, ai_service, approval_service, chat_service, policy_engine_service, tool_registry_service, workspace_service};

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
    [61u8; 32]
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
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    port
}

// --- Tool Registry: default classification + overrides -----------------

#[test]
fn default_risk_classification_follows_the_documented_convention() {
    // Read: list_*/get_*/*search_*.
    assert_eq!(tool_registry_service::default_risk_for("list_objects", Some("record")), RiskLevel::Read);
    assert_eq!(tool_registry_service::default_risk_for("get_record", Some("record")), RiskLevel::Read);
    assert_eq!(tool_registry_service::default_risk_for("search_records", Some("record")), RiskLevel::Read);
    // Hardcoded overrides pulling create_record/update_record down from
    // the plain create_*/update_* -> Write default.
    assert_eq!(tool_registry_service::default_risk_for("create_record", Some("record")), RiskLevel::LowWrite);
    assert_eq!(tool_registry_service::default_risk_for("update_record", Some("record")), RiskLevel::LowWrite);
    // Destructive.
    assert_eq!(tool_registry_service::default_risk_for("archive_record", Some("record")), RiskLevel::Destructive);
    // Privileged: identity/credentials/triggering another run.
    assert_eq!(tool_registry_service::default_risk_for("create_user", Some("admin")), RiskLevel::Privileged);
    assert_eq!(tool_registry_service::default_risk_for("create_api_client", Some("admin")), RiskLevel::Privileged);
    assert_eq!(tool_registry_service::default_risk_for("run_ai_agent", Some("admin")), RiskLevel::Privileged);
    // An ordinary workspace-configuration change - the safe Write middle
    // default for anything the prefix convention doesn't special-case.
    assert_eq!(tool_registry_service::default_risk_for("create_business_rule", Some("admin")), RiskLevel::Write);
    // Connector-derived: read vs. write source.
    assert_eq!(tool_registry_service::default_risk_for("some_connector_action", Some("connector_read")), RiskLevel::Read);
    assert_eq!(tool_registry_service::default_risk_for("some_connector_action", Some("connector_write")), RiskLevel::ExternalAction);
}

#[test]
fn risk_level_ordering_supports_at_or_above_threshold_checks() {
    assert!(RiskLevel::Write >= RiskLevel::LowWrite);
    assert!(RiskLevel::Privileged >= RiskLevel::Destructive);
    assert!(!(RiskLevel::Read >= RiskLevel::Write));
}

#[test]
fn a_workspace_override_changes_effective_risk_until_cleared() {
    let (conn, ws, admin) = setup_workspace("Tool Registry Test Co");

    // No override yet - the built-in default applies.
    assert_eq!(tool_registry_service::effective_risk(&conn, &ws, "list_objects", Some("record")).unwrap(), RiskLevel::Read);

    tool_registry_service::set_override(&conn, &ws, &AiToolRegistryOverrideInput { tool_name: "list_objects".into(), risk_level: RiskLevel::Privileged }, Some(&admin)).unwrap();
    assert_eq!(tool_registry_service::effective_risk(&conn, &ws, "list_objects", Some("record")).unwrap(), RiskLevel::Privileged);

    let overrides = tool_registry_service::list_overrides(&conn, &ws, Some(&admin)).unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].tool_name, "list_objects");

    tool_registry_service::clear_override(&conn, &ws, "list_objects", Some(&admin)).unwrap();
    assert_eq!(tool_registry_service::effective_risk(&conn, &ws, "list_objects", Some("record")).unwrap(), RiskLevel::Read);
    assert!(tool_registry_service::list_overrides(&conn, &ws, Some(&admin)).unwrap().is_empty());
}

#[test]
fn setting_a_tool_override_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace("Tool Registry Auth Test Co");
    // No actor at all - rejected before any role check even runs.
    assert!(tool_registry_service::set_override(&conn, &ws, &AiToolRegistryOverrideInput { tool_name: "list_objects".into(), risk_level: RiskLevel::Write }, None).is_err());

    // A real, non-Administrator user - rejected on the role check itself.
    let non_admin = lanesra_core::services::user_service::create(
        &conn, &ws,
        &lanesra_core::models::user::NewUser { username: "regular".into(), display_name: "Regular User".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();
    let err = tool_registry_service::set_override(&conn, &ws, &AiToolRegistryOverrideInput { tool_name: "list_objects".into(), risk_level: RiskLevel::Write }, Some(&non_admin.id)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("administrator"), "unexpected error: {err}");

    // The real Administrator can, of course.
    assert!(tool_registry_service::set_override(&conn, &ws, &AiToolRegistryOverrideInput { tool_name: "list_objects".into(), risk_level: RiskLevel::Write }, Some(&admin)).is_ok());
}

// --- Policy Engine: resolution + evaluation -----------------------------

#[test]
fn a_policy_free_workspace_allows_every_tool_call_unchanged() {
    let (conn, ws, _admin) = setup_workspace("Policy-Free Test Co");
    for name in ["list_objects", "create_record", "archive_record", "create_user"] {
        let decision = policy_engine_service::evaluate(&conn, &ws, None, name, Some("record")).unwrap();
        assert_eq!(decision, PolicyDecision::Allow, "expected Allow for {name} with no policy configured");
    }
}

#[test]
fn a_blocklisted_tool_is_denied_regardless_of_its_own_risk_level() {
    let (conn, ws, admin) = setup_workspace("Blocklist Test Co");
    policy_engine_service::upsert_policy(
        &conn, &ws, None,
        &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec!["list_objects".into()] },
        Some(&admin),
    )
    .unwrap();

    // list_objects is Read risk - the lowest tier - yet still denied,
    // since a blocklist entry is checked before any risk threshold.
    let decision = policy_engine_service::evaluate(&conn, &ws, None, "list_objects", Some("record")).unwrap();
    assert_eq!(decision, PolicyDecision::Deny { risk_level: RiskLevel::Read });

    // An unlisted tool under the same policy is unaffected.
    let decision = policy_engine_service::evaluate(&conn, &ws, None, "create_record", Some("record")).unwrap();
    assert_eq!(decision, PolicyDecision::Allow);
}

#[test]
fn a_risk_threshold_queues_a_durable_approval_instead_of_denying() {
    let (conn, ws, admin) = setup_workspace("Risk Threshold Test Co");
    policy_engine_service::upsert_policy(
        &conn, &ws, None,
        &AiAgentPolicyInput { require_approval_at_or_above: Some(RiskLevel::Write), blocked_tool_names: vec![] },
        Some(&admin),
    )
    .unwrap();

    // Below the threshold - allowed outright.
    assert_eq!(policy_engine_service::evaluate(&conn, &ws, None, "list_objects", Some("record")).unwrap(), PolicyDecision::Allow);
    assert_eq!(policy_engine_service::evaluate(&conn, &ws, None, "create_record", Some("record")).unwrap(), PolicyDecision::Allow); // low_write < write

    // At the threshold - queued for approval, never denied outright.
    let decision = policy_engine_service::evaluate(&conn, &ws, None, "create_business_rule", Some("admin")).unwrap();
    assert_eq!(decision, PolicyDecision::RequireApproval { risk_level: RiskLevel::Write });

    // Above the threshold too.
    let decision = policy_engine_service::evaluate(&conn, &ws, None, "archive_record", Some("record")).unwrap();
    assert_eq!(decision, PolicyDecision::RequireApproval { risk_level: RiskLevel::Destructive });

    // The durable paper trail: recording a pending call creates a real
    // ai_approvals row an administrator can later resolve.
    assert!(approval_service::list(&conn, &ws, Some("pending"), Some(&admin)).unwrap().is_empty());
    policy_engine_service::record_pending_tool_call(&conn, &ws, None, "archive_record", &serde_json::json!({"object_key": "Company", "id": "cmp_1"}), RiskLevel::Destructive, Some(&admin)).unwrap();
    let pending = approval_service::list(&conn, &ws, Some("pending"), Some(&admin)).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].subject_type, "tool_call");
    assert_eq!(pending[0].subject_id, "archive_record");
}

#[test]
fn an_agent_specific_policy_takes_precedence_over_the_workspace_default() {
    let (conn, ws, admin) = setup_workspace("Precedence Test Co");
    let agent = ai_agent_service::create(
        &conn, &ws,
        &AiAgentInput { name: "Unrestricted Reporter".into(), description: None, icon: "📊".into(), system_prompt: "You report on records.".into(), action_names: vec!["list_objects".into()], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    // A strict workspace-wide default: even a Read-level lookup needs
    // approval.
    policy_engine_service::upsert_policy(&conn, &ws, None, &AiAgentPolicyInput { require_approval_at_or_above: Some(RiskLevel::Read), blocked_tool_names: vec![] }, Some(&admin)).unwrap();

    // A different, unconfigured agent falls through to that strict
    // default.
    let unconfigured_decision = policy_engine_service::evaluate(&conn, &ws, Some("some-other-agent-id"), "list_objects", Some("record")).unwrap();
    assert_eq!(unconfigured_decision, PolicyDecision::RequireApproval { risk_level: RiskLevel::Read });

    // This agent gets its own, deliberately looser policy - no threshold
    // at all - which wins over the workspace default entirely, not just
    // for the fields it happens to set.
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec![] }, Some(&admin)).unwrap();
    let this_agent_decision = policy_engine_service::evaluate(&conn, &ws, Some(&agent.id), "list_objects", Some("record")).unwrap();
    assert_eq!(this_agent_decision, PolicyDecision::Allow);

    // The fixed "records"/"admin" chat assistants have no agent identity
    // of their own (agent_id = None) - only the workspace default can
    // ever govern those, confirmed by re-checking it directly.
    assert_eq!(policy_engine_service::evaluate(&conn, &ws, None, "list_objects", Some("record")).unwrap(), PolicyDecision::RequireApproval { risk_level: RiskLevel::Read });
}

#[test]
fn getting_or_listing_policies_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace("Policy Auth Test Co");
    policy_engine_service::upsert_policy(&conn, &ws, None, &AiAgentPolicyInput { require_approval_at_or_above: Some(RiskLevel::Write), blocked_tool_names: vec![] }, Some(&admin)).unwrap();

    assert!(policy_engine_service::get_policy(&conn, &ws, None, None).is_err());
    assert!(policy_engine_service::list_policies(&conn, &ws, None).is_err());
    assert!(policy_engine_service::get_policy(&conn, &ws, None, Some(&admin)).unwrap().is_some());
}

// --- Real wiring: chat_service::execute_tool's Tool-Call Firewall -------

#[tokio::test]
async fn a_blocked_tool_call_never_reaches_its_real_dispatcher() {
    let (conn, ws, admin) = setup_workspace("Firewall Deny Test Co");
    policy_engine_service::upsert_policy(&conn, &ws, None, &AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec!["list_objects".into()] }, Some(&admin)).unwrap();

    let port = spawn_sequence_stub(vec![anthropic_tool_use_body("t1", "list_objects", serde_json::json!({})), anthropic_text_body("Understood, I won't do that.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "list every object").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(content.contains("is blocked by this workspace's agent policy"), "unexpected tool message: {content}");
    assert!(content.contains("read"), "expected the risk level to be named: {content}");
}

#[tokio::test]
async fn a_risk_gated_tool_call_is_queued_for_approval_and_never_dispatched() {
    let (conn, ws, admin) = setup_workspace("Firewall Approval Test Co");
    policy_engine_service::upsert_policy(&conn, &ws, None, &AiAgentPolicyInput { require_approval_at_or_above: Some(RiskLevel::Read), blocked_tool_names: vec![] }, Some(&admin)).unwrap();

    let port = spawn_sequence_stub(vec![anthropic_tool_use_body("t1", "list_objects", serde_json::json!({})), anthropic_text_body("Noted - awaiting approval.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    assert!(approval_service::list(&conn, &ws, Some("pending"), Some(&admin)).unwrap().is_empty());
    let messages = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "list every object").await.unwrap();

    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(content.contains("requires administrator approval"), "unexpected tool message: {content}");

    // The durable side effect the message promises really happened.
    let pending = approval_service::list(&conn, &ws, Some("pending"), Some(&admin)).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].subject_type, "tool_call");
    assert_eq!(pending[0].subject_id, "list_objects");
}

#[tokio::test]
async fn an_allowed_tool_call_dispatches_exactly_as_it_always_has() {
    let (conn, ws, admin) = setup_workspace("Firewall Allow Test Co");
    // No policy configured anywhere - the pre-Phase-2 behavior every
    // existing workspace keeps.
    let port = spawn_sequence_stub(vec![anthropic_tool_use_body("t1", "list_objects", serde_json::json!({})), anthropic_text_body("Here they are.")]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_message(&conn, &ws, &master_key(), &admin, "records", "list every object").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(!content.starts_with("Error:"), "an unpolicied workspace's tool call should dispatch normally, got: {content}");
    assert!(messages.iter().any(|m| m.content.as_deref() == Some("Here they are.")));
}
