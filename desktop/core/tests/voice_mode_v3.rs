//! Voice-First Mode, PR 2 (part 2): multi-turn conversational context (spec
//! §14) - a session's own bounded history of the last few records a voice
//! command actually resolved to, consulted as a second, lower-priority
//! fallback for "it"/"that"/"this" behind the record currently on screen
//! (never a replacement for it), so a reference survives navigating away
//! from the record it originally named.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{access_role_service, company_service, user_service, voice_execution_service, voice_policy_service, voice_session_service, workspace_service};

fn master_key() -> [u8; 32] {
    [23u8; 32]
}

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

#[tokio::test]
async fn conversation_history_resolves_it_when_nothing_is_on_screen() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "conversant", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    // "open Northern Star" resolves and records a conversation turn - but,
    // unlike the desktop UI's own VoiceContext reporting on a detail page
    // mount, executing a NAVIGATE command never itself sets the session's
    // screen context. So nothing is "on screen" for the next command below;
    // it can only resolve through the bounded conversation history.
    let nav = voice_execution_service::submit_command(&conn, &session.id, &user, "open Northern Star", "en-US", None, &master_key()).await.unwrap();
    assert_eq!(nav.plan.unwrap().status, "succeeded");
    let mid_session = voice_session_service::current_session(&conn, &user).unwrap().unwrap();
    assert!(mid_session.context_record_id.is_none(), "NAVIGATE must never silently set screen context on its own");

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark this company as inactive", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("expected \"this\" to resolve via this session's own conversation history");
    assert_eq!(plan.status, "awaiting_confirmation");

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let updated = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(updated.status, "Inactive");
}

#[tokio::test]
async fn screen_context_still_wins_over_conversation_history() {
    let (conn, ws, admin) = setup_workspace();
    let company_a = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let company_b = company_service::create(&conn, &ws, &company_input("Acme Robotics"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "onscreen", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    // "open Acme Robotics" resolves it and records a conversation turn for B.
    voice_execution_service::submit_command(&conn, &session.id, &user, "open Acme Robotics", "en-US", None, &master_key()).await.unwrap();

    // But the user is actually looking at Company A's detail page right now
    // (set the way the desktop UI's own VoiceContext reports it on mount) -
    // the live screen context must still win over the more recent
    // conversation turn, exactly per voice_entity_resolver's priority order.
    voice_session_service::set_context(&conn, &session.id, &user, Some("Company"), Some(&company_a.id)).unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark this company as inactive", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("expected a plan");
    voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();

    let updated_a = company_service::get(&conn, &company_a.id).unwrap();
    let updated_b = company_service::get(&conn, &company_b.id).unwrap();
    assert_eq!(updated_a.status, "Inactive", "the record actually on screen must win over conversation history");
    assert_eq!(updated_b.status, "Prospect", "conversation history must never override what's actually on screen");
}

#[tokio::test]
async fn reset_conversation_clears_the_fallback_reference() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "resetter", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    voice_execution_service::submit_command(&conn, &session.id, &user, "open Northern Star", "en-US", None, &master_key()).await.unwrap();
    // Spec §14's own "Reset voice context" action - a fresh empty history,
    // same session, no logout required.
    voice_session_service::reset_conversation(&conn, &session.id, &user).unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark this company as inactive", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none(), "with no screen context and a reset conversation, \"this\" has nothing left to mean");
    let reason = outcome.unsupported_reason.expect("expected an honest not-found reason");
    assert!(reason.to_lowercase().contains("company"), "expected the reason to name the object type it couldn't find, got: {reason}");
}
