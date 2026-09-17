use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::voice_repo;
use lanesra_core::services::voice_entity_resolver::ResolutionOutcome;
use lanesra_core::services::voice_planner_service::PlanOutcome;
use lanesra_core::services::{
    access_role_service, company_service, opportunity_service, task_service, user_service, voice_entity_resolver, voice_execution_service,
    voice_planner_service, voice_policy_service, voice_session_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn company_input(name: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: "Prospect".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        phone: None,
        email: None,
        website: None,
        annual_revenue_cents: None,
        employee_count: None,
        preferred_contact_method: None,
    }
}

fn opportunity_input(company_id: &str, name: &str) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(),
        primary_contact_id: None,
        name: name.into(),
        stage: "Discovery".into(),
        status: "Open".into(),
        value_cents: 10_000_00,
        currency_code: "USD".into(),
        probability_bp: 4000,
        expected_close_date: None,
        owner_user_id: None,
        lost_reason: None,
        next_step: None,
    }
}

/// Every non-admin test needs a real Access Role with real object
/// capabilities *and* its own Voice policy binding - two independent gates,
/// exactly as the feature is designed. Full CRUD/Organization scope on the
/// object side keeps that half a non-issue so each test can focus on the
/// Voice-specific gate it's actually exercising.
fn make_voice_user(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str, voice: VoicePolicyBindingInput) -> String {
    let user = user_service::create(
        conn,
        ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] },
        Some(admin),
    )
    .unwrap();
    let role = access_role_service::create(conn, ws, &AccessRoleInput { name: format!("{username}-role"), description: "".into() }, Some(admin)).unwrap();
    access_role_service::upsert_grant(
        conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: true, record_scope: "ORGANIZATION".into() },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, &user.id, &role.id, Some(admin)).unwrap();
    voice_policy_service::upsert_policy_binding(conn, ws, Some(admin), &VoicePolicyBindingInput { access_role_id: Some(role.id), ..voice }).unwrap();
    user.id
}

fn full_voice_access() -> VoicePolicyBindingInput {
    VoicePolicyBindingInput {
        access_role_id: None,
        can_use_voice: true,
        can_search: true,
        can_create: true,
        can_update: true,
        can_act: true,
        can_bulk_act: true,
        can_external_act: true,
        can_use_agents: true,
        max_action_level: "act_with_confirmation".into(),
        processing_boundary: "cloud".into(),
        max_unlock_minutes: 30,
    }
}

// ---- PIN + session lifecycle -----------------------------------------------

#[test]
fn pin_must_be_exactly_four_digits() {
    let (conn, _ws, admin) = setup_workspace();
    assert!(voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "123".into() }).is_err());
    assert!(voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "12345".into() }).is_err());
    assert!(voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "12ab".into() }).is_err());
    let settings = voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    assert!(settings.pin_set);
}

#[test]
fn wrong_pin_locks_out_after_five_attempts() {
    let (conn, _ws, admin) = setup_workspace();
    voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "1234".into() }).unwrap();

    for _ in 0..4 {
        assert!(voice_session_service::unlock(&conn, &admin, "9999").is_err());
    }
    // The 5th wrong attempt trips the lockout.
    assert!(voice_session_service::unlock(&conn, &admin, "9999").is_err());
    // Even the *correct* PIN is now rejected until the lockout clears.
    let err = voice_session_service::unlock(&conn, &admin, "1234").unwrap_err().to_string();
    assert!(err.contains("locked"), "expected a lockout message, got: {err}");
}

#[test]
fn unlock_requires_voice_capability_from_policy() {
    let (conn, ws, admin) = setup_workspace();
    // A brand-new company record bootstraps Access Control v1's system
    // roles (and, via this PR's fix, "Full Access"'s Voice policy binding)
    // the first time any capability check runs.
    company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    // A Standard-User-equivalent role has no Voice policy binding of its
    // own, so it falls back to the workspace default: can_use_voice=false.
    let no_voice_user = make_voice_user(&conn, &ws, &admin, "novoice", VoicePolicyBindingInput { can_use_voice: false, ..full_voice_access() });
    voice_session_service::set_pin(&conn, &no_voice_user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let err = voice_session_service::unlock(&conn, &no_voice_user, "1234").unwrap_err().to_string();
    assert!(err.contains("not enabled"), "expected a Voice-not-enabled message, got: {err}");

    // Full Access (the Administrator's own role) is real and unlock succeeds.
    voice_session_service::set_pin(&conn, &admin, &SetVoicePinInput { pin: "4321".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &admin, "4321").unwrap();
    assert_eq!(session.user_id, admin);
    assert_eq!(session.state, "idle", "a freshly unlocked session is idle - waiting for its first command, not yet listening");
}

#[test]
fn session_ttl_is_capped_by_the_broadest_applicable_policy_ceiling_and_expires() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();
    // Policy ceilings fold "broadest wins" across a user's roles, same as
    // the capability booleans (only `processing_boundary` is inverted) -
    // starting from the workspace default's own 15-minute ceiling, a role
    // binding can only raise `max_unlock_minutes` further, never lower it.
    let user = make_voice_user(&conn, &ws, &admin, "capped", VoicePolicyBindingInput { max_unlock_minutes: 20, ..full_voice_access() });
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    // Ask for a much longer unlock than the role's own ceiling allows - the
    // user's own preference is capped down to that ceiling, never past it.
    voice_session_service::update_preferences(
        &conn,
        &user,
        &lanesra_core::models::voice::VoicePreferencesInput {
            response_channel: "voice_and_text".into(),
            spoken_detail: "normal".into(),
            auto_speak_confirmations: true,
            quiet_mode: false,
            unlock_duration_minutes: 60,
        },
    )
    .unwrap();

    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();
    let minutes = (chrono::DateTime::parse_from_rfc3339(&session.expires_at).unwrap() - chrono::DateTime::parse_from_rfc3339(&session.unlocked_at).unwrap()).num_minutes();
    assert!((19..=20).contains(&minutes), "expected the session capped at the role's 20-minute ceiling, got {minutes}");

    voice_session_service::expire(&conn, &session.id, &user).unwrap();
    let err = voice_session_service::require_active_session(&conn, &session.id, &user).unwrap_err().to_string();
    assert!(err.contains("expired"));
}

// ---- Entity resolution ------------------------------------------------------

#[test]
fn resolves_exact_name_uniquely_and_flags_ambiguous_or_missing() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Acme Robotics"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Acme Logistics"), Some(&admin)).unwrap();

    match voice_entity_resolver::resolve_by_reference(&conn, &ws, Some("Company"), "Northern Star", None, None).unwrap() {
        ResolutionOutcome::Resolved { object_key, confidence, .. } => {
            assert_eq!(object_key, "Company");
            assert!(confidence >= 0.9);
        }
        other => panic!("expected Resolved, got {other:?}"),
    }

    match voice_entity_resolver::resolve_by_reference(&conn, &ws, Some("Company"), "Acme", None, None).unwrap() {
        ResolutionOutcome::NeedsClarification { candidates } => assert_eq!(candidates.len(), 2),
        other => panic!("expected NeedsClarification for an ambiguous name, got {other:?}"),
    }

    match voice_entity_resolver::resolve_by_reference(&conn, &ws, Some("Company"), "Nonexistent Corp", None, None).unwrap() {
        ResolutionOutcome::NotFound => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn context_reference_short_circuits_to_the_record_in_view() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();

    match voice_entity_resolver::resolve_by_reference(&conn, &ws, None, "this", Some("Company"), Some(&company.id)).unwrap() {
        ResolutionOutcome::Resolved { record_id, confidence, .. } => {
            assert_eq!(record_id, company.id);
            assert_eq!(confidence, 1.0);
        }
        other => panic!("expected a context short-circuit, got {other:?}"),
    }
}

// ---- Planner ----------------------------------------------------------------

#[test]
fn plan_create_task_extracts_title_and_due_date() {
    let (conn, ws, _admin) = setup_workspace();
    match voice_planner_service::plan(&conn, &ws, "create a task to follow up with Acme tomorrow", None, None).unwrap() {
        PlanOutcome::Ready { plan, intent, .. } => {
            assert_eq!(intent, "CREATE");
            let step = &plan.steps[0];
            assert_eq!(step.action, "create_task");
            assert!(step.fields.get("title").unwrap().contains("follow up with Acme"));
            assert!(step.fields.contains_key("due_date"), "expected a due_date extracted from \"tomorrow\"");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

#[test]
fn plan_update_status_matches_object_noun_and_status_value() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "CRM Modernization"), Some(&admin)).unwrap();

    match voice_planner_service::plan(&conn, &ws, "mark CRM Modernization opportunity as Won", None, None).unwrap() {
        PlanOutcome::Ready { plan, intent, object_key, resolved_record_id, .. } => {
            assert_eq!(intent, "UPDATE");
            assert_eq!(object_key.as_deref(), Some("Opportunity"));
            assert_eq!(resolved_record_id.as_deref(), Some(opp.id.as_str()));
            let step = &plan.steps[0];
            assert_eq!(step.action, "update_status");
            assert_eq!(step.fields.get("status").unwrap(), "Won");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

#[test]
fn plan_update_status_falls_back_to_session_context_object() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "CRM Modernization"), Some(&admin)).unwrap();

    // No object noun spoken at all ("mark this Won") - the object type
    // comes entirely from the session's current-record context (spec §7),
    // and the reference resolves via that same context.
    match voice_planner_service::plan(&conn, &ws, "mark this Won", Some("Opportunity"), Some(&opp.id)).unwrap() {
        PlanOutcome::Ready { object_key, resolved_record_id, .. } => {
            assert_eq!(object_key.as_deref(), Some("Opportunity"));
            assert_eq!(resolved_record_id.as_deref(), Some(opp.id.as_str()));
        }
        other => panic!("expected Ready via context fallback, got {other:?}"),
    }
}

#[test]
fn plan_capture_logs_an_interaction_against_the_named_record() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();

    match voice_planner_service::plan(&conn, &ws, "add an interaction to Northern Star that renewal call happened", None, None).unwrap() {
        PlanOutcome::Ready { plan, intent, .. } => {
            assert_eq!(intent, "CAPTURE");
            let step = &plan.steps[0];
            assert_eq!(step.action, "log_activity");
            assert!(step.fields.get("body").unwrap().contains("renewal call happened"));
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

#[test]
fn unrecognized_transcript_is_honestly_unsupported() {
    let (conn, ws, _admin) = setup_workspace();
    match voice_planner_service::plan(&conn, &ws, "what's the weather like today", None, None).unwrap() {
        PlanOutcome::Unsupported { .. } => {}
        other => panic!("expected Unsupported for gibberish, got {other:?}"),
    }
}

// ---- End-to-end: submit_command / confirm_plan / undo, through the real
// entity services (Access Control v1 + status transitions + audit all fire
// exactly as a UI save would) --------------------------------------------

#[test]
fn navigate_executes_immediately_with_no_confirmation() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "navigator", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, &format!("open {}", company.name), "en-US", None).unwrap();
    let plan = outcome.plan.expect("a NAVIGATE command should produce a plan");
    assert_eq!(plan.status, "succeeded", "read-only NAVIGATE should never need confirmation");
}

#[test]
fn medium_risk_update_requires_confirmation_then_executes_and_can_be_undone() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "updater", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Northern Star Company as Inactive", "en-US", None).unwrap();
    let plan = outcome.plan.expect("expected a plan");
    assert_eq!(plan.status, "awaiting_confirmation", "a status change is Medium risk and must always confirm");

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id.clone(), method: "tap".into(), edited_plan: None }).unwrap();
    assert_eq!(result.status, "succeeded");
    assert_eq!(result.executions.len(), 1);
    let execution = &result.executions[0];
    assert_eq!(execution.result, "ok");
    assert!(execution.undo_token.is_some(), "a status change must be undoable");

    let updated = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(updated.status, "Inactive");

    voice_execution_service::undo(&conn, &execution.id, &admin).unwrap();
    let reverted = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(reverted.status, "Prospect", "undo must restore the pre-command value through a real compensating write");
}

#[test]
fn rejecting_a_plan_never_executes_it() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "rejecter", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Northern Star Company as Inactive", "en-US", None).unwrap();
    let plan = outcome.plan.unwrap();

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "reject".into(), edited_plan: None }).unwrap();
    assert_eq!(result.status, "rejected");
    assert!(result.executions.is_empty());

    let unchanged = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(unchanged.status, "Prospect");
}

#[test]
fn act_level_skips_confirmation_on_low_risk_task_creation_and_undo_archives_it() {
    let (conn, ws, admin) = setup_workspace();
    let user = make_voice_user(&conn, &ws, &admin, "actlevel", VoicePolicyBindingInput { max_action_level: "act".into(), ..full_voice_access() });
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "create a task to call the customer tomorrow", "en-US", None).unwrap();
    let plan = outcome.plan.expect("expected a plan");
    assert_eq!(plan.status, "succeeded", "Act level should skip confirmation for Low-risk task creation");

    let executions = voice_repo::list_executions_for_plan(&conn, &plan.id).unwrap();
    assert_eq!(executions.len(), 1);
    let execution = &executions[0];
    let task_id = execution.entity_id.clone().expect("create_task should record the new task's id");
    let task = task_service::get(&conn, &task_id).unwrap();
    assert!(task.title.contains("call the customer"));
    assert!(task.archived_at.is_none());

    voice_execution_service::undo(&conn, &execution.id, &admin).unwrap();
    let archived = task_service::get(&conn, &task_id).unwrap();
    assert!(archived.archived_at.is_some(), "undo of a created task should archive it");
}

#[test]
fn required_capability_is_enforced_even_when_max_action_level_would_otherwise_allow_it() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    // "act" is the most permissive Max Action Level, but can_update is off -
    // Voice's own narrower capability gate must still block an UPDATE_STATUS
    // command regardless of how high max_action_level is set.
    let user = make_voice_user(&conn, &ws, &admin, "noupdate", VoicePolicyBindingInput { max_action_level: "act".into(), can_update: false, ..full_voice_access() });
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Northern Star Company as Inactive", "en-US", None).unwrap();
    assert!(outcome.plan.is_none(), "a capability-blocked command must not produce an executable plan");
    let reason = outcome.unsupported_reason.expect("expected a blocked reason");
    assert!(reason.contains("Voice Update"), "expected the reason to name the missing capability, got: {reason}");

    let unchanged = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(unchanged.status, "Prospect", "a blocked command must never touch the record");
}

#[test]
fn ambiguous_reference_asks_for_clarification_instead_of_guessing() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Acme Robotics"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Acme Logistics"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "clarifier", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Acme Company as Inactive", "en-US", None).unwrap();
    assert!(outcome.plan.is_none());
    assert!(outcome.clarification_question.is_some());
    assert_eq!(outcome.candidates.len(), 2);
}
