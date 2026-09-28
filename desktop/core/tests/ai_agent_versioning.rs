//! AI Agent Platform v2, Phase 1: migration `0059_ai_agent_versioning.sql`
//! - the v1 backfill onto `ai_agent_versions` for an agent that already
//! existed before this migration ran, and basic round-trips for the two
//! new supporting tables (`ai_agent_model_refs`, `ai_approvals`). No
//! service layer exists yet (`agent_version_service`/`approval_service`
//! are later work), so this exercises the schema directly - the same
//! "prove the migration/backfill itself is correct" scope every other
//! migration-only test file in this codebase has.

use lanesra_core::models::ai_agent::AiAgentInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_agent_service, workspace_service};

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

#[test]
fn creating_an_agent_backfills_a_published_v1_version() {
    let (conn, workspace_id, admin) = setup_workspace("Versioning Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput {
            name: "Claims Auditor".into(), description: Some("Audits claims".into()), icon: "🕵".into(),
            system_prompt: "You are a claims auditor.".into(),
            action_names: vec!["get_record".into(), "list_records".into()],
            delegate_agent_ids: vec![], skill_ids: vec![],
        },
        Some(&admin),
    )
    .unwrap();

    let current_version_id: Option<String> = conn
        .query_row("SELECT current_version_id FROM ai_agents WHERE id = ?1", [&agent.id], |r| r.get(0))
        .unwrap();
    let current_version_id = current_version_id.expect("backfill should have set current_version_id");
    assert_eq!(current_version_id, format!("v1-{}", agent.id));

    let (version_number, status, name, system_prompt, action_names_json): (i64, String, String, String, String) = conn
        .query_row(
            "SELECT version_number, status, name, system_prompt, action_names_json FROM ai_agent_versions WHERE id = ?1",
            [&current_version_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();
    assert_eq!(version_number, 1);
    assert_eq!(status, "published");
    assert_eq!(name, "Claims Auditor");
    assert_eq!(system_prompt, "You are a claims auditor.");
    let action_names: Vec<String> = serde_json::from_str(&action_names_json).unwrap();
    assert_eq!(action_names, vec!["get_record".to_string(), "list_records".to_string()]);
}

#[test]
fn a_second_agent_gets_its_own_independent_v1() {
    let (conn, workspace_id, admin) = setup_workspace("Versioning Co 2");
    let a = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Agent A".into(), description: None, icon: "🤖".into(), system_prompt: "A.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();
    let b = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Agent B".into(), description: None, icon: "🤖".into(), system_prompt: "B.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM ai_agent_versions", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 2, "each agent gets exactly one backfilled version, never shared");

    let a_version: String = conn.query_row("SELECT current_version_id FROM ai_agents WHERE id = ?1", [&a.id], |r| r.get(0)).unwrap();
    let b_version: String = conn.query_row("SELECT current_version_id FROM ai_agents WHERE id = ?1", [&b.id], |r| r.get(0)).unwrap();
    assert_ne!(a_version, b_version);
}

#[test]
fn deleting_an_agent_cascades_its_versions() {
    let (conn, workspace_id, admin) = setup_workspace("Versioning Co 3");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Temp Agent".into(), description: None, icon: "🤖".into(), system_prompt: "Temp.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();

    conn.execute("DELETE FROM ai_agents WHERE id = ?1", [&agent.id]).unwrap();

    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM ai_agent_versions WHERE agent_id = ?1", [&agent.id], |r| r.get(0)).unwrap();
    assert_eq!(remaining, 0, "ON DELETE CASCADE should remove the agent's version history with it");
}

#[test]
fn agent_model_ref_resolves_to_a_provider_and_is_unique_per_workspace_name() {
    let (conn, workspace_id, admin) = setup_workspace("Model Ref Co");
    let now = lanesra_core::domain::ids::now_iso();
    let provider_id = lanesra_core::domain::ids::new_uuid();
    conn.execute(
        "INSERT INTO ai_providers (id, workspace_id, name, provider, base_url, model, is_active, created_at, created_by, updated_at, updated_by) \
         VALUES (?1, ?2, 'Primary Anthropic', 'anthropic', NULL, 'claude-haiku-4-5', 1, ?3, ?4, ?3, ?4)",
        rusqlite::params![provider_id, workspace_id, now, admin],
    )
    .unwrap();

    let ref_id = lanesra_core::domain::ids::new_uuid();
    conn.execute(
        "INSERT INTO ai_agent_model_refs (id, workspace_id, name, provider_id, created_at, created_by, updated_at, updated_by) \
         VALUES (?1, ?2, 'primary', ?3, ?4, ?5, ?4, ?5)",
        rusqlite::params![ref_id, workspace_id, provider_id, now, admin],
    )
    .unwrap();

    let resolved_provider_name: String = conn
        .query_row(
            "SELECT p.name FROM ai_agent_model_refs r JOIN ai_providers p ON p.id = r.provider_id WHERE r.workspace_id = ?1 AND r.name = 'primary'",
            [&workspace_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(resolved_provider_name, "Primary Anthropic");

    // A second "primary" in the same workspace violates the UNIQUE(workspace_id, name) constraint.
    let dup = conn.execute(
        "INSERT INTO ai_agent_model_refs (id, workspace_id, name, provider_id, created_at, created_by, updated_at, updated_by) \
         VALUES (?1, ?2, 'primary', ?3, ?4, ?5, ?4, ?5)",
        rusqlite::params![lanesra_core::domain::ids::new_uuid(), workspace_id, provider_id, now, admin],
    );
    assert!(dup.is_err(), "a second model ref named 'primary' in the same workspace should be rejected");
}

#[test]
fn approval_round_trips_pending_to_resolved() {
    let (conn, workspace_id, admin) = setup_workspace("Approvals Co");
    let agent = ai_agent_service::create(
        &conn, &workspace_id,
        &AiAgentInput { name: "Reviewed Agent".into(), description: None, icon: "🤖".into(), system_prompt: "Reviewed.".into(), action_names: vec![], delegate_agent_ids: vec![], skill_ids: vec![] },
        Some(&admin),
    )
    .unwrap();
    let version_id: String = conn.query_row("SELECT current_version_id FROM ai_agents WHERE id = ?1", [&agent.id], |r| r.get(0)).unwrap();

    let approval_id = lanesra_core::domain::ids::new_uuid();
    let now = lanesra_core::domain::ids::now_iso();
    let proposal = serde_json::json!({"proposed_status": "published", "diff": "no-op v1 -> v1"}).to_string();
    conn.execute(
        "INSERT INTO ai_approvals (id, workspace_id, subject_type, subject_id, proposal_json, status, requested_by, created_at) \
         VALUES (?1, ?2, 'agent_version_publish', ?3, ?4, 'pending', ?5, ?6)",
        rusqlite::params![approval_id, workspace_id, version_id, proposal, admin, now],
    )
    .unwrap();

    let pending_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ai_approvals WHERE workspace_id = ?1 AND status = 'pending'", [&workspace_id], |r| r.get(0))
        .unwrap();
    assert_eq!(pending_count, 1);

    conn.execute(
        "UPDATE ai_approvals SET status = 'approved', resolved_by = ?1, resolved_at = ?2, resolution_notes = 'looks fine' WHERE id = ?3",
        rusqlite::params![admin, lanesra_core::domain::ids::now_iso(), approval_id],
    )
    .unwrap();

    let (status, resolved_by): (String, Option<String>) = conn
        .query_row("SELECT status, resolved_by FROM ai_approvals WHERE id = ?1", [&approval_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert_eq!(status, "approved");
    assert_eq!(resolved_by.as_deref(), Some(admin.as_str()));
}
