//! Agent Access Governance (issue #245): closes the gap between the
//! Tool-Call Firewall's own claim ("every tool call") and reality (the
//! 6 Foundry-internal tools used to dispatch before the firewall ever
//! saw them), and gives an opt-in `enforce_record_access` policy flag
//! real teeth over Access Control v1 for AI-driven record writes - both
//! purely additive. Plus the Agent Access Inspector that traces exactly
//! this.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::ai::AiSettingsInput;
use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::ai_agent_policy::{AiAgentPolicyInput, PolicyDecision};
use lanesra_core::models::ai_tool_registry::RiskLevel;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, agent_access_inspector_service, ai_agent_service, ai_service, chat_service, policy_engine_service, user_service, workspace_service,
};

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
    [62u8; 32]
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

// Identical stub-listener pattern ai_agent_policy_engine.rs already
// established.
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

fn no_op_policy() -> AiAgentPolicyInput {
    AiAgentPolicyInput { require_approval_at_or_above: None, blocked_tool_names: vec![], exclude_restricted_memory: true, enforce_record_access: false }
}

fn make_agent(conn: &rusqlite::Connection, ws: &str, admin: &str, action_names: Vec<String>) -> lanesra_core::models::ai_agent::AiAgentDefinition {
    ai_agent_service::create(
        conn, ws,
        &AiAgentInput { name: "Records Agent".into(), description: None, icon: "🤖".into(), system_prompt: "You act on records.".into(), action_names, delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(admin),
    )
    .unwrap()
}

/// Every user auto-bootstraps into the system "Standard User" Access
/// Role (org-wide Create/Read/Update/Delete, by design - see
/// `access_role_service::assign_default_role_for_new_user`'s own doc
/// comment) the moment any Access Control v1 check runs for them, even
/// if its assignment is removed first - `access_service::
/// ensure_actor_bootstrapped` silently re-adds it, since "a user with
/// zero Access Roles" isn't a state this product leaves anyone in. A
/// genuinely restricted actor for these tests comes from tightening
/// "Standard User"'s own grant instead, scoped to this one in-memory
/// workspace.
fn deny_default_create_access(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    let standard = access_role_service::list(conn, ws).unwrap().into_iter().find(|r| r.name == "Standard User").expect("Standard User should already be bootstrapped");
    access_role_service::upsert_grant(
        conn, &standard.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: false, can_read: true, can_update: true, can_delete: true, can_assign: false, record_scope: "ORGANIZATION".into() },
        Some(admin),
    )
    .unwrap();
}

// --- Tool-Call Firewall gap closed for Foundry-internal tools ----------

#[tokio::test]
async fn a_blocklisted_foundry_internal_tool_is_denied_before_it_ever_runs() {
    let (conn, ws, admin) = setup_workspace("Foundry Firewall Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["list_objects".into()]);
    policy_engine_service::upsert_policy(
        &conn, &ws, Some(&agent.id),
        &AiAgentPolicyInput { blocked_tool_names: vec!["get_memory".into()], ..no_op_policy() },
        Some(&admin),
    )
    .unwrap();

    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "get_memory", serde_json::json!({"memory_type": "entity"})),
        anthropic_text_body("Understood, I can't check that."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "what do you remember about this record?").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(content.contains("is blocked by this workspace's agent policy"), "unexpected tool message: {content}");
}

// --- enforce_record_access: opt-in, backward-compatible by default -----

#[tokio::test]
async fn enforce_record_access_off_preserves_the_unattributed_default() {
    let (conn, ws, admin) = setup_workspace("Unattributed Default Test Co");
    let zero_role_user = user_service::create(&conn, &ws, &NewUser { username: "zero_role".into(), display_name: "Zero Role".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    ai_agent_service::set_acts_as(&conn, &agent.id, &ws, Some(zero_role_user.id.clone()), Some(&admin)).unwrap();
    // No policy configured at all - enforce_record_access defaults false.
    assert!(policy_engine_service::resolve_policy(&conn, &ws, Some(&agent.id)).unwrap().is_none());

    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "create_record", serde_json::json!({"object_key": "Company", "data": {"name": "Acme", "status": "Prospect"}})),
        anthropic_text_body("Created it."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "create a company named Acme").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(!content.starts_with("Error:"), "a zero-role acts_as user shouldn't block the write while enforcement is off, got: {content}");
}

#[tokio::test]
async fn enforce_record_access_on_actually_blocks_a_write_access_control_v1_would_deny() {
    let (conn, ws, admin) = setup_workspace("Real Enforcement Test Co");
    let zero_role_user = user_service::create(&conn, &ws, &NewUser { username: "zero_role2".into(), display_name: "Zero Role Two".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    deny_default_create_access(&conn, &ws, &admin);
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    ai_agent_service::set_acts_as(&conn, &agent.id, &ws, Some(zero_role_user.id.clone()), Some(&admin)).unwrap();
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { enforce_record_access: true, ..no_op_policy() }, Some(&admin)).unwrap();

    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "create_record", serde_json::json!({"object_key": "Company", "data": {"name": "Acme", "status": "Prospect"}})),
        anthropic_text_body("I couldn't create that."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    // The admin is the one chatting, but this agent's own acts_as wins -
    // proving this isn't just inheriting whoever happens to be chatting.
    let messages = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "create a company named Acme").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(content.starts_with("Error:"), "a zero-role acts_as user under real enforcement should be denied, got: {content}");
}

#[tokio::test]
async fn enforce_record_access_on_still_allows_a_write_a_real_role_grants() {
    let (conn, ws, admin) = setup_workspace("Real Enforcement Allow Test Co");
    let granted_user = user_service::create(&conn, &ws, &NewUser { username: "granted".into(), display_name: "Granted User".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    let role = access_role_service::create(&conn, &ws, &AccessRoleInput { name: "Company Creator".into(), description: "".into() }, Some(&admin)).unwrap();
    access_role_service::upsert_grant(
        &conn, &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: false, can_assign: false, record_scope: "ORGANIZATION".into() },
        Some(&admin),
    )
    .unwrap();
    access_role_service::assign_to_user(&conn, &granted_user.id, &role.id, Some(&admin)).unwrap();

    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    ai_agent_service::set_acts_as(&conn, &agent.id, &ws, Some(granted_user.id.clone()), Some(&admin)).unwrap();
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { enforce_record_access: true, ..no_op_policy() }, Some(&admin)).unwrap();

    let port = spawn_sequence_stub(vec![
        anthropic_tool_use_body("t1", "create_record", serde_json::json!({"object_key": "Company", "data": {"name": "Acme", "status": "Prospect"}})),
        anthropic_text_body("Created it."),
    ]);
    configure_anthropic_key(&conn, &ws, &admin, port);

    let messages = chat_service::send_agent_message(&conn, &ws, &master_key(), &admin, &agent.id, "create a company named Acme").await.unwrap();
    let tool_message = messages.iter().find(|m| m.role == "tool").expect("a tool-result message was appended");
    let content = tool_message.content.as_deref().unwrap_or_default();
    assert!(!content.starts_with("Error:"), "a user with a real Create grant should succeed under real enforcement, got: {content}");
}

// --- set_acts_as validation ----------------------------------------------

#[test]
fn acts_as_must_be_an_active_user_of_this_workspace() {
    let (conn, ws, admin) = setup_workspace("Acts As Validation Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);

    let err = ai_agent_service::set_acts_as(&conn, &agent.id, &ws, Some("not-a-real-user-id".into()), Some(&admin)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("does not exist"), "unexpected error: {err}");

    let other_user = user_service::create(&conn, &ws, &NewUser { username: "real".into(), display_name: "Real User".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    let updated = ai_agent_service::set_acts_as(&conn, &agent.id, &ws, Some(other_user.id.clone()), Some(&admin)).unwrap();
    assert_eq!(updated.acts_as_user_id, Some(other_user.id));

    // Clearing it back to None is always allowed.
    let cleared = ai_agent_service::set_acts_as(&conn, &agent.id, &ws, None, Some(&admin)).unwrap();
    assert_eq!(cleared.acts_as_user_id, None);
}

#[test]
fn setting_acts_as_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace("Acts As Auth Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    assert!(ai_agent_service::set_acts_as(&conn, &agent.id, &ws, None, None).is_err());
}

// --- Agent Access Inspector ----------------------------------------------

#[test]
fn the_inspector_reports_the_same_policy_decision_the_firewall_would_make() {
    let (conn, ws, admin) = setup_workspace("Inspector Decision Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into(), "list_objects".into()]);
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { blocked_tool_names: vec!["list_objects".into()], ..no_op_policy() }, Some(&admin)).unwrap();

    let blocked = agent_access_inspector_service::inspect(&conn, &ws, &agent.id, "list_objects", None, None, None, Some(&admin)).unwrap();
    assert!(blocked.tool_in_action_list);
    assert_eq!(blocked.policy_decision, PolicyDecision::Deny { risk_level: RiskLevel::Read });
    assert_eq!(blocked.policy_source, "agent-specific policy");

    let not_in_list = agent_access_inspector_service::inspect(&conn, &ws, &agent.id, "archive_record", None, None, None, Some(&admin)).unwrap();
    assert!(!not_in_list.tool_in_action_list, "archive_record was never added to this agent's action_names");
}

#[test]
fn the_inspector_honestly_reports_when_record_access_is_not_enforced_yet() {
    let (conn, ws, admin) = setup_workspace("Inspector Unenforced Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);

    let result = agent_access_inspector_service::inspect(&conn, &ws, &agent.id, "create_record", Some("Company"), None, Some(&admin), Some(&admin)).unwrap();
    assert!(!result.record_access_enforced);
    assert!(result.acting_as_user_id.is_none());
    assert!(result.record_access.is_none(), "no trace should be produced while enforcement is off, even with a simulate-as user supplied");
}

#[test]
fn the_inspector_traces_the_real_access_control_decision_once_enforced() {
    let (conn, ws, admin) = setup_workspace("Inspector Enforced Test Co");
    let zero_role_user = user_service::create(&conn, &ws, &NewUser { username: "inspected".into(), display_name: "Inspected User".into(), password: "supersecretpassword".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    deny_default_create_access(&conn, &ws, &admin);
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    policy_engine_service::upsert_policy(&conn, &ws, Some(&agent.id), &AiAgentPolicyInput { enforce_record_access: true, ..no_op_policy() }, Some(&admin)).unwrap();

    let result = agent_access_inspector_service::inspect(&conn, &ws, &agent.id, "create_record", Some("Company"), None, Some(&zero_role_user.id), Some(&admin)).unwrap();
    assert!(result.record_access_enforced);
    assert_eq!(result.acting_as_user_id, Some(zero_role_user.id));
    let trace = result.record_access.expect("a record-write tool with a resolvable actor should produce a real trace");
    assert!(!trace.decision.allowed, "a zero-role user should be denied");
}

#[test]
fn inspecting_agent_access_requires_an_administrator() {
    let (conn, ws, admin) = setup_workspace("Inspector Auth Test Co");
    let agent = make_agent(&conn, &ws, &admin, vec!["create_record".into()]);
    assert!(agent_access_inspector_service::inspect(&conn, &ws, &agent.id, "create_record", None, None, None, None).is_err());
}
