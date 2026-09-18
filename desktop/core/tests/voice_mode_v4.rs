//! Voice-First Mode follow-up fixes: Custom Objects (including ones from an
//! installed Industry App) are voice-addressable the same way a built-in
//! record already is (spec's VOICE-AC-06 - `global_search` already covered
//! this generically; this test locks it in as a real regression test, not
//! just an assumption), and a bounded edit-distance fuzzy fallback tier
//! (spec §8.1's own named-but-previously-unbuilt "fuzzy match") resolves a
//! typo'd name instead of an honest "not found" - still never silently
//! guessing when more than one candidate is plausible (VOICE-AC-05).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    access_role_service, company_service, custom_object_service, custom_record_service, user_service, voice_execution_service, voice_policy_service,
    voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [31u8; 32]
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
async fn custom_object_records_are_voice_addressable_like_a_built_in_record() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput { singular_label: "Unit".into(), plural_label: "Units".into(), icon: "🏠".into(), prefix: "UNIT".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: def.key.clone(), primary_name: "Unit 200".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();

    let user = make_voice_user(&conn, &ws, &admin, "propmanager", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "open Unit 200", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("a Custom Object record should be just as voice-findable as a built-in one");
    assert_eq!(plan.status, "succeeded", "read-only NAVIGATE should never need confirmation");
}

#[tokio::test]
async fn fuzzy_match_resolves_a_single_close_typo() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Atlas Construction"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "typo-tolerant", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    // A real, plausible typo - "Construcion" is missing a "t" - one
    // character off in an 11-letter word, well inside the fuzzy tolerance.
    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Atlas Construcion Company as Inactive", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("a single close-typo match should resolve via the fuzzy fallback tier, not an honest miss");
    assert_eq!(plan.status, "awaiting_confirmation");

    let result = voice_execution_service::confirm_plan(&conn, &session.id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let updated = company_service::get(&conn, &company.id).unwrap();
    assert_eq!(updated.status, "Inactive");
}

#[tokio::test]
async fn fuzzy_match_with_multiple_plausible_typos_asks_rather_than_guesses() {
    let (conn, ws, admin) = setup_workspace();
    // Two real names, each one character off from the spoken query below
    // ("Nordic Robotics") but in different words - neither is a substring
    // match of the other or of the query, so both only surface via the
    // fuzzy tier, and both are close enough that neither should be
    // silently preferred over the other.
    company_service::create(&conn, &ws, &company_input("Nordik Robotics"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Nordic Robotecs"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "asks-not-guesses", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Nordic Robotics Company as Inactive", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none());
    assert_eq!(outcome.candidates.len(), 2, "two plausible fuzzy matches must ask, never silently pick one (VOICE-AC-05)");
}

#[tokio::test]
async fn nonsense_text_is_still_honestly_not_found_even_with_fuzzy_fallback() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Atlas Construction"), Some(&admin)).unwrap();
    let user = make_voice_user(&conn, &ws, &admin, "no-false-positive", full_voice_access());
    voice_session_service::set_pin(&conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(&conn, &user, "1234").unwrap();

    let outcome = voice_execution_service::submit_command(&conn, &session.id, &user, "mark Zorblatt Enterprises Company as Inactive", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none(), "a genuinely unrelated name must not fuzzy-match onto an unrelated real record");
    assert!(outcome.unsupported_reason.is_some());
}
