//! Admin extensibility Phase D (spec §23/ADM-WF): the richer Trigger ->
//! Conditions -> Actions workflow engine that replaced the original
//! single-trigger (status transition) / single-action (create task)
//! workflow_rules.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::invoice::{InvoiceInput, InvoiceLineInput};
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow::{WorkflowActionInput, WorkflowDefinitionInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::notification_repo;
use lanesra_core::services::{
    company_service, invoice_service, opportunity_service, task_service, user_service, workflow_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Workflow Test Co".into(),
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

fn company_input(name: &str, owner_user_id: Option<&str>) -> CompanyInput {
    CompanyInput {
        name: name.into(), status: "Active Customer".into(), owner_user_id: owner_user_id.map(String::from),
        tax_number: None, billing_address: None, shipping_address: None, tags: None, notes: None,
    }
}

fn opportunity_input(company_id: &str, stage: &str, owner_user_id: Option<&str>) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(), primary_contact_id: None, name: "Big Deal".into(), stage: stage.into(),
        status: if stage == "Won" { "Won" } else { "Open" }.into(), value_cents: 500000, currency_code: "USD".into(),
        probability_bp: 0, expected_close_date: None, owner_user_id: owner_user_id.map(String::from), lost_reason: None, next_step: None,
    }
}

fn create_task_action(title: &str, due_in_days: i64, assignee_user_id: Option<&str>) -> WorkflowActionInput {
    WorkflowActionInput {
        action_type: "create_task".into(),
        params_json: serde_json::json!({
            "title": title, "description": "Kick off the new customer's onboarding",
            "due_in_days": due_in_days, "assignee_user_id": assignee_user_id,
        }).to_string(),
    }
}

fn status_changed_workflow(entity_type: &str, trigger_status: &str, action: WorkflowActionInput) -> WorkflowDefinitionInput {
    WorkflowDefinitionInput {
        entity_type: entity_type.into(), name: format!("{entity_type} -> {trigger_status}"), description: None,
        trigger_type: "status_changed".into(), trigger_status: Some(trigger_status.into()), trigger_field_key: None,
        trigger_offset_days: 0, match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![action],
    }
}

#[test]
fn workflow_creates_a_follow_up_task_when_opportunity_stage_matches() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("Send onboarding kit", 3, None)), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", None), Some(&admin)).unwrap();

    // Moving through an intermediate stage must not fire the "Won" workflow.
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Proposal", None), Some(&admin)).unwrap();
    assert!(task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap().is_empty());

    // Reaching Won fires it exactly once.
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();
    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Send onboarding kit");
    assert_eq!(tasks[0].related_type.as_deref(), Some("Opportunity"));
    assert_eq!(tasks[0].related_id.as_deref(), Some(opp.id.as_str()));
}

#[test]
fn re_saving_without_changing_stage_does_not_refire_the_workflow() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("Send onboarding kit", 3, None)), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    // create() fires record_created, not status_changed - no task yet.
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();
    assert!(task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap().is_empty());

    // Saving again with the same stage must not create a second task.
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();
    assert!(task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap().is_empty());
}

#[test]
fn task_is_assigned_to_the_opportunity_owner_when_the_action_has_no_explicit_assignee() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("Send onboarding kit", 3, None)), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", Some(&rep.id)), Some(&admin)).unwrap();
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", Some(&rep.id)), Some(&admin)).unwrap();

    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks[0].owner_user_id.as_deref(), Some(rep.id.as_str()));
}

#[test]
fn non_administrator_cannot_define_a_workflow() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(&conn, &ws, &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();

    let err = workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("X", 0, None)), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn invalid_trigger_status_for_the_entity_type_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    // "Won" is a valid Opportunity stage but not a valid Invoice status.
    let wf = status_changed_workflow("Invoice", "Won", create_task_action("X", 0, None));
    let err = workflow_service::create_rule(&conn, &ws, &wf, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("not a valid Invoice status"));
}

#[test]
fn multiple_matching_workflows_are_additive_not_conflict_resolved() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("Send onboarding kit", 3, None)), Some(&admin)).unwrap();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Opportunity", "Won", create_task_action("Notify finance", 0, None)), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", None), Some(&admin)).unwrap();
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();

    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks.len(), 2, "both active matching workflows should each create their own task");
}

#[test]
fn invoice_overdue_transition_fires_a_workflow_assigned_via_the_company_owner() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &NewUser { username: "fin".into(), display_name: "Fin".into(), password: "anothersecretpw".into(), roles: vec!["Finance".into()] }, Some(&admin)).unwrap();
    workflow_service::create_rule(&conn, &ws, &status_changed_workflow("Invoice", "Overdue", create_task_action("Follow up on payment", 0, None)), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", Some(&rep.id)), Some(&admin)).unwrap();
    let invoice = invoice_service::create(
        &conn,
        &InvoiceInput {
            company_id: company.id.clone(), contact_id: None, currency_code: "USD".into(),
            issue_date: Some("2020-01-01".into()), due_date: Some("2020-01-15".into()), payment_terms: None, notes: None,
            lines: vec![InvoiceLineInput { product_id: None, description: "Consulting".into(), quantity_milli: 1000, unit_price_cents: 100000, discount_bp: 0, tax_rate_bp: 0 }],
        },
        Some(&admin),
    ).unwrap();
    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();

    let updated_count = invoice_service::refresh_overdue(&conn, &ws).unwrap();
    assert_eq!(updated_count, 1);

    let tasks = task_service::list_by_related(&conn, "Invoice", &invoice.invoice.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Follow up on payment");
    assert_eq!(tasks[0].owner_user_id.as_deref(), Some(rep.id.as_str()));
}

#[test]
fn record_created_trigger_fires_on_creation_but_not_on_update() {
    let (conn, ws, admin) = setup_workspace();
    let wf = WorkflowDefinitionInput {
        entity_type: "Company".into(), name: "Welcome new company".into(), description: None,
        trigger_type: "record_created".into(), trigger_status: None, trigger_field_key: None, trigger_offset_days: 0,
        match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![create_task_action("Send welcome email", 0, None)],
    };
    workflow_service::create_rule(&conn, &ws, &wf, Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let tasks = task_service::list_by_related(&conn, "Company", &company.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Send welcome email");

    // Updating the same company again must not re-fire record_created.
    company_service::update(&conn, &company.id, &company_input("Acme Renamed", None), Some(&admin)).unwrap();
    assert_eq!(task_service::list_by_related(&conn, "Company", &company.id).unwrap().len(), 1);
}

#[test]
fn add_notification_action_notifies_the_records_owner() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(&conn, &ws, &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] }, Some(&admin)).unwrap();
    let wf = WorkflowDefinitionInput {
        entity_type: "Company".into(), name: "Notify owner on Prospect".into(), description: None,
        trigger_type: "status_changed".into(), trigger_status: Some("Prospect".into()), trigger_field_key: None, trigger_offset_days: 0,
        match_type: "all".into(), priority: 0, conditions: vec![],
        actions: vec![WorkflowActionInput { action_type: "add_notification".into(), params_json: serde_json::json!({"message": "New prospect assigned", "audience": "owner"}).to_string() }],
    };
    workflow_service::create_rule(&conn, &ws, &wf, Some(&admin)).unwrap();

    let mut input = company_input("Acme", Some(&rep.id));
    input.status = "Active Customer".into();
    let company = company_service::create(&conn, &ws, &input, Some(&admin)).unwrap();
    input.status = "Prospect".into();
    company_service::update(&conn, &company.id, &input, Some(&admin)).unwrap();

    let notifications = notification_repo::list_for_user(&conn, &ws, &rep.id, true).unwrap();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].message, "New prospect assigned");
}

#[test]
fn test_business_rules_style_recursion_guard_bounds_a_self_referential_workflow_chain() {
    // A record_updated workflow on Task that itself creates another Task
    // (which fires record_created -> nothing further, since this workflow
    // only listens for record_updated) exercises the depth guard without
    // an infinite loop; this mainly proves the engine terminates.
    let (conn, ws, admin) = setup_workspace();
    let wf = WorkflowDefinitionInput {
        entity_type: "Task".into(), name: "On task update, log a follow-up".into(), description: None,
        trigger_type: "record_updated".into(), trigger_status: None, trigger_field_key: None, trigger_offset_days: 0,
        match_type: "all".into(), priority: 0, conditions: vec![], actions: vec![create_task_action("Follow-up", 0, None)],
    };
    workflow_service::create_rule(&conn, &ws, &wf, Some(&admin)).unwrap();

    let task = task_service::create(
        &conn, &ws,
        &lanesra_core::models::task::TaskInput { title: "Original".into(), description: None, owner_user_id: None, priority: "Normal".into(), status: "Not Started".into(), due_date: None, reminder_at: None, related_type: None, related_id: None },
        Some(&admin),
    ).unwrap();
    task_service::update(
        &conn, &task.id, &ws,
        &lanesra_core::models::task::TaskInput { title: "Original".into(), description: None, owner_user_id: None, priority: "Normal".into(), status: "In Progress".into(), due_date: None, reminder_at: None, related_type: None, related_id: None },
        Some(&admin),
    ).unwrap();

    // Exactly one follow-up task was created by the workflow (the new
    // task's own creation is record_created, which this workflow doesn't
    // listen for, so no infinite chain).
    let all_tasks = task_service::list(&conn, &ws).unwrap();
    assert_eq!(all_tasks.len(), 2);
}
