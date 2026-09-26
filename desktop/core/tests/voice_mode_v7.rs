//! Voice-First Mode PR 3 (GitHub issue #161): real response-detail levels.
//! `VoicePlanStep` now carries `brief_description`/`detail_note` alongside
//! the existing `description`, so a user's `spoken_detail` preference
//! ("short"/"normal"/"detailed") can pick real, planner-authored text
//! instead of being read by nothing (the exact gap issue #161 names).
//! These tests cover the planner side of that: `description` (normal) is
//! never changed, `brief_description` drops the non-essential detail
//! `description` includes, and `detail_note` states something concrete the
//! brief/normal tiers leave out - never fabricated filler.

use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::voice_planner_service::PlanOutcome;
use lanesra_core::services::{
    access_role_service, company_service, custom_field_service, opportunity_service, user_service, voice_execution_service, voice_planner_service,
    voice_policy_service, voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [61u8; 32]
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

fn unlocked_session(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str) -> (String, String) {
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
    voice_policy_service::upsert_policy_binding(conn, ws, Some(admin), &VoicePolicyBindingInput { access_role_id: Some(role.id), ..full_voice_access() }).unwrap();
    voice_session_service::set_pin(conn, &user.id, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(conn, &user.id, "1234").unwrap();
    (user.id, session.id)
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

#[test]
fn update_status_short_tier_is_terser_and_detailed_tier_names_the_specific_field() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Northern Star"), Some(&admin)).unwrap();
    opportunity_service::create(&conn, &opportunity_input(&company.id, "CRM Modernization"), Some(&admin)).unwrap();

    match voice_planner_service::plan(&conn, &ws, "mark CRM Modernization opportunity as Won", None, None, None, None).unwrap() {
        PlanOutcome::Ready { plan, .. } => {
            let step = &plan.steps[0];
            assert_eq!(step.description, "Set Opportunity status to Won", "normal tier must stay exactly what it always was");
            assert_eq!(step.brief_description, "Status: Won", "short tier must drop the object-key preamble the normal tier states in full");
            let note = step.detail_note.as_deref().expect("update_status must always have something concrete to add at the detailed tier");
            assert!(note.contains("status") && note.contains("Opportunity"), "detailed tier must name the specific field and record type changed, got: {note}");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

#[test]
fn create_task_short_tier_omits_the_due_date_the_detailed_tier_states() {
    let (conn, ws, _admin) = setup_workspace();
    match voice_planner_service::plan(&conn, &ws, "create a task to follow up with Acme tomorrow", None, None, None, None).unwrap() {
        PlanOutcome::Ready { plan, .. } => {
            let step = &plan.steps[0];
            assert!(step.description.to_lowercase().contains("due"), "normal tier already states the due date");
            assert!(!step.brief_description.to_lowercase().contains("due"), "short tier must drop the due date, got: {}", step.brief_description);
            assert!(step.brief_description.contains("follow up with Acme"), "short tier must still say what the task is, got: {}", step.brief_description);
            let note = step.detail_note.as_deref().expect("a task with a due date must have a detail_note stating it");
            assert!(note.to_lowercase().contains("due"), "detailed tier must state the due date, got: {note}");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

#[tokio::test]
async fn create_record_detail_note_lists_the_extra_field_the_brief_and_normal_tiers_omit() {
    let (conn, ws, admin) = setup_workspace();
    custom_field_service::create_definition(&conn, &ws, &select_field_def("Company", "Industry", vec!["Manufacturing".into(), "Retail".into()], true, 0), Some(&admin)).unwrap();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "detail-note-creator");

    let first = voice_execution_service::submit_command(&conn, &session_id, &user, "create a company Acme Corp", "en-US", None, &master_key()).await.unwrap();
    assert!(first.plan.is_none(), "the required Industry field is still unanswered");
    let second = voice_execution_service::submit_command(&conn, &session_id, &user, "Manufacturing", "en-US", None, &master_key()).await.unwrap();
    let plan = second.plan.expect("both the name and the required custom field are now answered");

    let step = &plan.plan.steps[0];
    assert_eq!(step.description, "Create Company \"Acme Corp\"");
    assert_eq!(step.brief_description, "New Company: Acme Corp", "short tier must not mention the Industry field");
    assert!(!step.description.to_lowercase().contains("industry"), "normal tier never mentioned Industry either - it only ever named the record");
    let note = step.detail_note.as_deref().expect("a create with an extra field set must have a detail_note listing it");
    assert!(note.to_lowercase().contains("industry") && note.contains("Manufacturing"), "detailed tier must list the extra field and its value, got: {note}");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");
}
