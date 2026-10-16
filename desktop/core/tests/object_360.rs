//! Object 360 (issue #352): the record-level counterpart to the Ontology
//! Explorer (#339) - given one real record, shows every record directly
//! linked to it. Covers the two genuinely new pieces this phase added:
//! `relationship_service::native_related_records_for` (the built-in CRM's
//! own FK-based relationships, which completely bypass the generic
//! `relationship_definitions`/`relationship_instances` system
//! `related_records_for` already covers) and
//! `ai_agent_service::list_scoped_to_object_type` (which agents are
//! relevant to a given object type, for the inspector's "AI Agents scoped
//! to this object type" group).

use lanesra_core::models::ai_agent::{AiAgentInput, ObjectTypeScope};
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    ai_agent_service, company_service, contact_service, opportunity_service, relationship_service,
    task_service, workspace_service,
};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn make_company(conn: &rusqlite::Connection, ws: &str, admin: &str, name: &str) -> String {
    company_service::create(
        conn,
        ws,
        &CompanyInput {
            name: name.into(),
            status: "Prospect".into(),
            ..Default::default()
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn agent_input(object_type_scopes: Vec<ObjectTypeScope>) -> AiAgentInput {
    AiAgentInput {
        name: "Test Agent".into(),
        description: None,
        icon: "🤖".into(),
        system_prompt: "You are a test agent.".into(),
        action_names: vec![],
        delegate_agent_ids: vec![],
        skill_ids: vec![],
        object_type_scopes,
        action_type_ids: vec![],
    }
}

#[test]
fn native_related_records_for_a_company_includes_its_contact_and_opportunity() {
    let (conn, ws, admin) = setup_workspace("Object 360 Co");
    let company = make_company(&conn, &ws, &admin, "BrightPath Logistics");
    let other_company = make_company(&conn, &ws, &admin, "Unrelated Co");

    let contact = contact_service::create(
        &conn,
        &ContactInput {
            company_id: company.clone(),
            first_name: "Ava".into(),
            last_name: "Martin".into(),
            status: "Active".into(),
            ..Default::default()
        },
        Some(&admin),
    )
    .unwrap();
    let opportunity = opportunity_service::create(
        &conn,
        &OpportunityInput {
            company_id: company.clone(),
            primary_contact_id: Some(contact.id.clone()),
            name: "Platform Modernization".into(),
            stage: "Proposal".into(),
            status: "Open".into(),
            value_cents: 42_000_00,
            currency_code: "USD".into(),
            probability_bp: 7000,
            expected_close_date: None,
            owner_user_id: None,
            lost_reason: None,
            next_step: None,
        },
        Some(&admin),
    )
    .unwrap();

    let related =
        relationship_service::native_related_records_for(&conn, &ws, "Company", &company).unwrap();
    assert!(
        related
            .iter()
            .any(|r| r.entity_type == "Contact" && r.entity_id == contact.id),
        "expected the company's Contact in its native related records"
    );
    assert!(
        related
            .iter()
            .any(|r| r.entity_type == "Opportunity" && r.entity_id == opportunity.id),
        "expected the company's Opportunity in its native related records"
    );

    // A different company's related records must not leak the first
    // company's Contact/Opportunity - these are plain FK filters, not a
    // workspace-wide scan returned unfiltered.
    let other_related =
        relationship_service::native_related_records_for(&conn, &ws, "Company", &other_company)
            .unwrap();
    assert!(
        other_related.is_empty(),
        "an unrelated company should have no native related records here"
    );
}

#[test]
fn native_related_records_for_a_task_resolves_its_own_related_record() {
    let (conn, ws, admin) = setup_workspace("Task Related Co");
    let company = make_company(&conn, &ws, &admin, "BrightPath Logistics");
    let task = task_service::create(
        &conn,
        &ws,
        &TaskInput {
            title: "Follow up".into(),
            description: None,
            owner_user_id: None,
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: None,
            reminder_at: None,
            related_type: Some("Company".into()),
            related_id: Some(company.clone()),
        },
        Some(&admin),
    )
    .unwrap();

    let related =
        relationship_service::native_related_records_for(&conn, &ws, "Task", &task.id).unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].entity_type, "Company");
    assert_eq!(related[0].entity_id, company);
}

#[test]
fn agents_scoped_to_an_object_type_includes_unscoped_and_matching_scoped_agents_only() {
    let (conn, ws, admin) = setup_workspace("Scoped Agents Co");

    let unscoped =
        ai_agent_service::create(&conn, &ws, &agent_input(vec![]), Some(&admin)).unwrap();
    let scoped_to_company = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(vec![ObjectTypeScope {
            object_type: "Company".into(),
            capabilities: vec!["read".into()],
        }]),
        Some(&admin),
    )
    .unwrap();
    let scoped_to_contact = ai_agent_service::create(
        &conn,
        &ws,
        &agent_input(vec![ObjectTypeScope {
            object_type: "Contact".into(),
            capabilities: vec!["read".into()],
        }]),
        Some(&admin),
    )
    .unwrap();

    let for_company =
        ai_agent_service::list_scoped_to_object_type(&conn, &ws, "Company", true).unwrap();
    let ids: Vec<&str> = for_company.iter().map(|a| a.id.as_str()).collect();
    assert!(
        ids.contains(&unscoped.id.as_str()),
        "an unscoped agent should be scoped to every object type"
    );
    assert!(
        ids.contains(&scoped_to_company.id.as_str()),
        "the Company-scoped agent should be scoped to Company"
    );
    assert!(
        !ids.contains(&scoped_to_contact.id.as_str()),
        "the Contact-scoped agent should not be scoped to Company"
    );
}

#[test]
fn a_deactivated_agent_is_excluded_when_active_only() {
    let (conn, ws, admin) = setup_workspace("Deactivated Agent Co");
    let agent = ai_agent_service::create(&conn, &ws, &agent_input(vec![]), Some(&admin)).unwrap();
    ai_agent_service::set_active(&conn, &agent.id, false, Some(&admin)).unwrap();

    let active_only =
        ai_agent_service::list_scoped_to_object_type(&conn, &ws, "Company", true).unwrap();
    assert!(
        !active_only.iter().any(|a| a.id == agent.id),
        "a deactivated agent should be excluded when active_only is true"
    );

    let including_inactive =
        ai_agent_service::list_scoped_to_object_type(&conn, &ws, "Company", false).unwrap();
    assert!(
        including_inactive.iter().any(|a| a.id == agent.id),
        "a deactivated agent should still appear when active_only is false"
    );
}
