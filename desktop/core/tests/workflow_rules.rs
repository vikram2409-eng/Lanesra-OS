use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::invoice::{InvoiceInput, InvoiceLineInput};
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workflow_rule::WorkflowRuleInput;
use lanesra_core::models::workspace::WorkspaceSetup;
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
        name: name.into(),
        status: "Active Customer".into(),
        owner_user_id: owner_user_id.map(String::from),
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
    }
}

fn opportunity_input(company_id: &str, stage: &str, owner_user_id: Option<&str>) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(),
        primary_contact_id: None,
        name: "Big Deal".into(),
        stage: stage.into(),
        status: if stage == "Won" { "Won" } else { "Open" }.into(),
        value_cents: 500000,
        currency_code: "USD".into(),
        probability_bp: 0,
        expected_close_date: None,
        owner_user_id: owner_user_id.map(String::from),
        lost_reason: None,
        next_step: None,
    }
}

fn win_rule() -> WorkflowRuleInput {
    WorkflowRuleInput {
        entity_type: "Opportunity".into(),
        trigger_status: "Won".into(),
        task_title: "Send onboarding kit".into(),
        task_description: Some("Kick off the new customer's onboarding".into()),
        due_in_days: 3,
        assignee_user_id: None,
    }
}

#[test]
fn rule_creates_a_follow_up_task_when_opportunity_stage_matches() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &win_rule(), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", None), Some(&admin)).unwrap();

    // Moving through an intermediate stage must not fire the "Won" rule.
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
fn re_saving_without_changing_stage_does_not_refire_the_rule() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &win_rule(), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();
    // create() never fires workflow rules - only a transition (via update) does.
    assert!(task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap().is_empty());

    // Saving again with the same stage must not create a second task.
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();
    assert!(task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap().is_empty());
}

#[test]
fn task_is_assigned_to_the_opportunity_owner_when_the_rule_has_no_explicit_assignee() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();
    workflow_service::create_rule(&conn, &ws, &win_rule(), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", Some(&rep.id)), Some(&admin)).unwrap();
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", Some(&rep.id)), Some(&admin)).unwrap();

    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks[0].owner_user_id.as_deref(), Some(rep.id.as_str()));
}

#[test]
fn non_administrator_cannot_define_a_workflow_rule() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = workflow_service::create_rule(&conn, &ws, &win_rule(), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn invalid_trigger_status_for_the_entity_type_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let mut rule = win_rule();
    rule.entity_type = "Invoice".into();
    rule.trigger_status = "Won".into(); // not a valid invoice status
    let err = workflow_service::create_rule(&conn, &ws, &rule, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("not a valid Invoice status"));
}

#[test]
fn multiple_matching_rules_are_additive_not_conflict_resolved() {
    let (conn, ws, admin) = setup_workspace();
    workflow_service::create_rule(&conn, &ws, &win_rule(), Some(&admin)).unwrap();
    let mut second = win_rule();
    second.task_title = "Notify finance".into();
    workflow_service::create_rule(&conn, &ws, &second, Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", None), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", None), Some(&admin)).unwrap();
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", None), Some(&admin)).unwrap();

    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks.len(), 2, "both active matching rules should each create their own task");
}

#[test]
fn invoice_overdue_transition_fires_a_rule_assigned_via_the_company_owner() {
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "fin".into(), display_name: "Fin".into(), password: "anothersecretpw".into(), roles: vec!["Finance".into()] },
        Some(&admin),
    )
    .unwrap();
    workflow_service::create_rule(
        &conn,
        &ws,
        &WorkflowRuleInput {
            entity_type: "Invoice".into(),
            trigger_status: "Overdue".into(),
            task_title: "Follow up on payment".into(),
            task_description: None,
            due_in_days: 0,
            assignee_user_id: None,
        },
        Some(&admin),
    )
    .unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", Some(&rep.id)), Some(&admin)).unwrap();
    let invoice = invoice_service::create(
        &conn,
        &InvoiceInput {
            company_id: company.id.clone(),
            contact_id: None,
            currency_code: "USD".into(),
            issue_date: Some("2020-01-01".into()),
            due_date: Some("2020-01-15".into()), // long past due
            payment_terms: None,
            notes: None,
            lines: vec![InvoiceLineInput {
                product_id: None,
                description: "Consulting".into(),
                quantity_milli: 1000,
                unit_price_cents: 100000,
                discount_bp: 0,
                tax_rate_bp: 0,
            }],
        },
        Some(&admin),
    )
    .unwrap();
    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();

    let updated_count = invoice_service::refresh_overdue(&conn, &ws).unwrap();
    assert_eq!(updated_count, 1);

    let tasks = task_service::list_by_related(&conn, "Invoice", &invoice.invoice.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Follow up on payment");
    assert_eq!(tasks[0].owner_user_id.as_deref(), Some(rep.id.as_str()));
}
