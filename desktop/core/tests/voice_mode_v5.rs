//! Voice-First Mode PR 2 (part 3): guided step-by-step record creation
//! (GitHub issue #160). Covers the one-turn happy path (all info given in
//! the initial utterance), the multi-turn guided loop (missing name, then a
//! missing required custom field, asked one at a time), cancel-word
//! abandonment mid-flow, the honest `CREATE_UNSUPPORTED_CORE_OBJECTS` gap
//! for objects that need a related Company voice doesn't resolve yet, and
//! Custom Object parity (VOICE-AC-06: no new code needed for an
//! admin-defined object).

use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::custom_field_repo;
use lanesra_core::services::{
    access_role_service, company_service, custom_field_service, custom_object_service, custom_record_service, user_service, voice_execution_service,
    voice_policy_service, voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [31u8; 32]
}

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
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

fn unlocked_session(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str) -> (String, String) {
    let user = make_voice_user(conn, ws, admin, username, full_voice_access());
    voice_session_service::set_pin(conn, &user, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(conn, &user, "1234").unwrap();
    (user, session.id)
}

fn select_field_def(entity_type: &str, label: &str, options: Vec<String>, required: bool, sort_order: i64) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: entity_type.into(),
        label: label.into(),
        field_type: "select".into(),
        options,
        required,
        show_in_list: true,
        sort_order,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable: false,
        is_filterable: true,
        is_reportable: true,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

#[tokio::test]
async fn guided_create_company_single_utterance_with_all_info_succeeds_immediately() {
    let (conn, ws, admin) = setup_workspace();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "single-turn-creator");

    let outcome = voice_execution_service::submit_command(&conn, &session_id, &user, "create a company Acme Corp", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("Company has no required relationship, so one utterance with a name should be enough to plan");
    assert_eq!(plan.status, "awaiting_confirmation", "create_record is Low risk, but act_with_confirmation still confirms once risk is above None");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let companies = company_service::list(&conn, &ws).unwrap();
    assert!(companies.iter().any(|c| c.name == "Acme Corp"), "the real company_service::create must have actually run");
}

#[tokio::test]
async fn guided_create_asks_for_missing_name_then_succeeds_on_the_next_turn() {
    let (conn, ws, admin) = setup_workspace();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "guided-name-asker");

    let first = voice_execution_service::submit_command(&conn, &session_id, &user, "create a company", "en-US", None, &master_key()).await.unwrap();
    assert!(first.plan.is_none(), "no name was given yet, so this must not plan a create with a blank name");
    let question = first.clarification_question.expect("guided create must ask a concrete question, never just fail");
    assert!(question.to_lowercase().contains("name"), "expected the question to ask for the company name, got: {question}");

    let second = voice_execution_service::submit_command(&conn, &session_id, &user, "Acme Corp", "en-US", None, &master_key()).await.unwrap();
    let plan = second.plan.expect("the follow-up answer should be read as the missing name, not a brand-new unrelated command");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let companies = company_service::list(&conn, &ws).unwrap();
    assert!(companies.iter().any(|c| c.name == "Acme Corp"));
}

#[tokio::test]
async fn guided_create_asks_for_a_missing_required_custom_field_before_finishing() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field_def("Company", "Industry", vec!["Manufacturing".into(), "Retail".into()], true, 0), Some(&admin)).unwrap();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "guided-required-field");

    let first = voice_execution_service::submit_command(&conn, &session_id, &user, "create a company Acme Corp", "en-US", None, &master_key()).await.unwrap();
    assert!(first.plan.is_none(), "the required Industry field is still unanswered, so this must not plan yet");
    let question = first.clarification_question.expect("must ask about the missing required field");
    assert!(question.to_lowercase().contains("industry"), "expected the question to name the Industry field, got: {question}");

    let second = voice_execution_service::submit_command(&conn, &session_id, &user, "Manufacturing", "en-US", None, &master_key()).await.unwrap();
    let plan = second.plan.expect("both the name and the one required custom field are now answered");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let company = company_service::list(&conn, &ws).unwrap().into_iter().find(|c| c.name == "Acme Corp").expect("company must actually exist");
    let values = custom_field_repo::get_values(&conn, &company.id).unwrap();
    assert_eq!(values.get("industry").map(|s| s.as_str()), Some("Manufacturing"), "the guided answer must be applied through custom_field_service::set_entity_values, same as any other save");
}

#[tokio::test]
async fn guided_create_cancel_word_abandons_the_create_and_clears_pending_state() {
    let (conn, ws, admin) = setup_workspace();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "guided-canceler");

    let first = voice_execution_service::submit_command(&conn, &session_id, &user, "create a company", "en-US", None, &master_key()).await.unwrap();
    assert!(first.clarification_question.is_some());

    let cancel = voice_execution_service::submit_command(&conn, &session_id, &user, "never mind", "en-US", None, &master_key()).await.unwrap();
    assert!(cancel.plan.is_none());
    assert!(cancel.unsupported_reason.is_some(), "a cancel word must end the guided flow honestly, not silently create a blank record");

    // The pending state must be cleared - an unrelated command right after
    // must be treated as a fresh command, not another guided-create answer.
    let next = voice_execution_service::submit_command(&conn, &session_id, &user, "what's the weather like today", "en-US", None, &master_key()).await.unwrap();
    assert!(next.plan.is_none());
    assert!(next.unsupported_reason.is_some(), "this nonsense command must be honestly unsupported on its own terms, not misread as another create-flow answer");
    let companies = company_service::list(&conn, &ws).unwrap();
    assert!(companies.is_empty(), "cancelling must never leave a partially-created record behind");
}

#[tokio::test]
async fn guided_create_names_the_gap_honestly_for_unsupported_core_objects() {
    let (conn, ws, admin) = setup_workspace();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "guided-unsupported");

    let outcome = voice_execution_service::submit_command(&conn, &session_id, &user, "create a contact John Smith", "en-US", None, &master_key()).await.unwrap();
    assert!(outcome.plan.is_none());
    let reason = outcome.unsupported_reason.expect("Contact needs a related Company voice doesn't resolve yet, so this must be an honest Unsupported, not a broken create");
    assert!(reason.contains("Contact"), "the gap must be named by object, got: {reason}");
}

#[tokio::test]
async fn guided_create_works_identically_for_a_custom_object() {
    let (conn, ws, admin) = setup_workspace();
    let def = custom_object_service::create(
        &conn,
        &ws,
        &CustomObjectDefinitionInput { singular_label: "Unit".into(), plural_label: "Units".into(), icon: "🏠".into(), prefix: "UNIT".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "guided-custom-object");

    let outcome = voice_execution_service::submit_command(&conn, &session_id, &user, "create a Unit East Wing", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("a Custom Object needs zero new code to be guided-createable (VOICE-AC-06)");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
    let records = custom_record_service::list(&conn, &ws, &def.key).unwrap();
    assert!(records.iter().any(|r| r.primary_name == "East Wing"));
}
