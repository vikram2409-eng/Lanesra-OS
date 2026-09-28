//! AI Agent Platform v2, Phase 1 follow-up: `agent_version_service`'s
//! Draft -> Test -> Published -> Deprecated -> Disabled lifecycle
//! (immutability once Published, auto-deprecation of the prior Published
//! version on publish) and `approval_service`'s create/resolve round
//! trip - through the real service layer this time, unlike
//! `ai_agent_versioning.rs`'s migration-only, direct-SQL scope.

use lanesra_core::models::ai_agent::{validate_output_schema, AiAgentInput, AiAgentVersionInput};
use lanesra_core::models::ai_approval::AiApprovalInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{agent_version_service, ai_agent_service, approval_service, workspace_service};

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
