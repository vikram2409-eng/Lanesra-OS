use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_report::CustomReportInput;
use lanesra_core::models::field_rule::FieldRuleInput;
use lanesra_core::models::numbering_override::NumberingOverrideInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::{DashboardKpiPrefs, WorkspaceUpdate};
use lanesra_core::services::{
    company_service, custom_field_service, custom_report_service, field_rule_service, numbering_service,
    opportunity_service, task_service, user_service, workflow_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = lanesra_core::models::workspace::WorkspaceSetup {
        business_name: "Admin Flex Co".into(),
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

fn company_input(name: &str, status: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: status.into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
    }
}

fn opportunity_input(company_id: &str, stage: &str, value_cents: i64) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(),
        primary_contact_id: None,
        name: "Deal".into(),
        stage: stage.into(),
        status: if stage == "Won" { "Won" } else { "Open" }.into(),
        value_cents,
        currency_code: "USD".into(),
        probability_bp: 0,
        expected_close_date: None,
        owner_user_id: None,
        lost_reason: None,
        next_step: None,
    }
}

// --- FR-CFG / FR-RUL / FR-WFL generalized to a non-Company entity ---

#[test]
fn custom_fields_business_rules_and_workflow_automation_all_work_on_opportunity() {
    let (conn, ws, admin) = setup_workspace();

    // Custom field on Opportunity (previously Company/Contact only).
    let source_field = custom_field_service::create_definition(
        &conn,
        &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Opportunity".into(),
            label: "Lead Source".into(),
            field_type: "text".into(),
            options: vec![],
            required: false,
            show_in_list: false,
            sort_order: 0,
        },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(source_field.entity_type, "Opportunity");

    // Business rule on Opportunity (previously Company/Contact only) -
    // require Lead Source once status is Won.
    field_rule_service::create_rule(
        &conn,
        &ws,
        &FieldRuleInput {
            entity_type: "Opportunity".into(),
            trigger_field_source: "builtin".into(),
            trigger_field_key: "status".into(),
            operator: "equals".into(),
            trigger_value: "Won".into(),
            target_field_key: source_field.key.clone(),
            effect: "require".into(),
            sort_order: 0,
        },
        Some(&admin),
    )
    .unwrap();

    // Workflow automation on Opportunity, keyed off stage (already
    // supported before this phase, kept here to prove it still composes
    // correctly alongside the newly generalized field/rule).
    workflow_service::create_rule(
        &conn,
        &ws,
        &lanesra_core::models::workflow_rule::WorkflowRuleInput {
            entity_type: "Opportunity".into(),
            trigger_status: "Won".into(),
            task_title: "Kick off onboarding".into(),
            task_description: None,
            due_in_days: 2,
            assignee_user_id: None,
        },
        Some(&admin),
    )
    .unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Active Customer"), Some(&admin)).unwrap();
    let opp = opportunity_service::create(&conn, &opportunity_input(&company.id, "New", 100000), Some(&admin)).unwrap();

    // While the opportunity is still New/Open, the rule doesn't apply -
    // empty custom field values save without complaint.
    custom_field_service::set_entity_values(&conn, "Opportunity", &opp.id, &HashMap::new(), Some(&admin)).unwrap();

    // Workflow automation: moving the opportunity's stage to Won fires
    // the workflow rule and creates a follow-up task.
    opportunity_service::update(&conn, &opp.id, &opportunity_input(&company.id, "Won", 100000), Some(&admin)).unwrap();
    let tasks = task_service::list_by_related(&conn, "Opportunity", &opp.id).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Kick off onboarding");

    // Business rule enforcement: now that status is Won (opportunity_input
    // sets status = Won whenever stage = Won), the custom field is required.
    let err = custom_field_service::set_entity_values(&conn, "Opportunity", &opp.id, &HashMap::new(), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Lead Source is required"));

    let mut values = HashMap::new();
    values.insert("lead_source".to_string(), "Referral".to_string());
    custom_field_service::set_entity_values(&conn, "Opportunity", &opp.id, &values, Some(&admin)).unwrap();
}

// --- Configurable numbering format ---

#[test]
fn admin_can_override_a_number_format_and_it_applies_immediately() {
    let (conn, ws, admin) = setup_workspace();

    let before = numbering_service::list_effective(&conn, &ws, Some(&admin)).unwrap();
    let company_default = before.iter().find(|e| e.entity_type == "Company").unwrap();
    assert!(!company_default.is_custom);
    assert_eq!(company_default.prefix, "CUS");

    let company1 = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    assert_eq!(company1.customer_number, "CUS-000001");

    numbering_service::set_override(
        &conn,
        &ws,
        &NumberingOverrideInput { entity_type: "Company".into(), prefix: "ACC".into(), digits: 4 },
        Some(&admin),
    )
    .unwrap();

    // Sequence continues (2), just reformatted with the new prefix/width.
    let company2 = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    assert_eq!(company2.customer_number, "ACC-0002");

    let after = numbering_service::list_effective(&conn, &ws, Some(&admin)).unwrap();
    let company_after = after.iter().find(|e| e.entity_type == "Company").unwrap();
    assert!(company_after.is_custom);
    assert_eq!(company_after.prefix, "ACC");
    assert_eq!(company_after.digits, 4);

    let reset = numbering_service::reset_override(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(!reset.is_custom);
    let company3 = company_service::create(&conn, &ws, &company_input("Initech", "Prospect"), Some(&admin)).unwrap();
    assert_eq!(company3.customer_number, "CUS-000003");
}

#[test]
fn non_administrator_cannot_change_a_number_format() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = numbering_service::set_override(
        &conn,
        &ws,
        &NumberingOverrideInput { entity_type: "Company".into(), prefix: "ACC".into(), digits: 4 },
        Some(&sales_user.id),
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

// --- Simple report builder ---

#[test]
fn custom_report_counts_companies_grouped_by_status() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    company_service::create(&conn, &ws, &company_input("Initech", "Active Customer"), Some(&admin)).unwrap();

    let report = custom_report_service::create(
        &conn,
        &ws,
        &CustomReportInput {
            name: "Companies by status".into(),
            entity_type: "Company".into(),
            group_by_source: "builtin".into(),
            group_by_field: "status".into(),
            aggregate: "count".into(),
            sum_field_key: None,
        },
        Some(&admin),
    )
    .unwrap();

    let rows = custom_report_service::run(&conn, &report).unwrap();
    let by_group: HashMap<_, _> = rows.into_iter().map(|r| (r.group, r.value)).collect();
    assert_eq!(by_group.get("Prospect"), Some(&2.0));
    assert_eq!(by_group.get("Active Customer"), Some(&1.0));
}

#[test]
fn custom_report_sums_a_numeric_custom_field_grouped_by_another_custom_field() {
    let (conn, ws, admin) = setup_workspace();
    let region = custom_field_service::create_definition(
        &conn,
        &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Region".into(), field_type: "text".into(),
            options: vec![], required: false, show_in_list: false, sort_order: 0,
        },
        Some(&admin),
    )
    .unwrap();
    let headcount = custom_field_service::create_definition(
        &conn,
        &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Headcount".into(), field_type: "number".into(),
            options: vec![], required: false, show_in_list: false, sort_order: 0,
        },
        Some(&admin),
    )
    .unwrap();

    let acme = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let mut acme_values = HashMap::new();
    acme_values.insert(region.key.clone(), "EMEA".to_string());
    acme_values.insert(headcount.key.clone(), "50".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &acme.id, &acme_values, Some(&admin)).unwrap();

    let globex = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    let mut globex_values = HashMap::new();
    globex_values.insert(region.key.clone(), "EMEA".to_string());
    globex_values.insert(headcount.key.clone(), "30".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &globex.id, &globex_values, Some(&admin)).unwrap();

    let report = custom_report_service::create(
        &conn,
        &ws,
        &CustomReportInput {
            name: "Headcount by region".into(),
            entity_type: "Company".into(),
            group_by_source: "custom".into(),
            group_by_field: region.key.clone(),
            aggregate: "sum".into(),
            sum_field_key: Some(headcount.key.clone()),
        },
        Some(&admin),
    )
    .unwrap();

    let rows = custom_report_service::run(&conn, &report).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].group, "EMEA");
    assert_eq!(rows[0].value, 80.0);
}

#[test]
fn a_sum_report_must_target_an_active_numeric_custom_field() {
    let (conn, ws, admin) = setup_workspace();
    let text_field = custom_field_service::create_definition(
        &conn,
        &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Notes field".into(), field_type: "text".into(),
            options: vec![], required: false, show_in_list: false, sort_order: 0,
        },
        Some(&admin),
    )
    .unwrap();

    let err = custom_report_service::create(
        &conn,
        &ws,
        &CustomReportInput {
            name: "Bad report".into(),
            entity_type: "Company".into(),
            group_by_source: "builtin".into(),
            group_by_field: "status".into(),
            aggregate: "sum".into(),
            sum_field_key: Some(text_field.key.clone()),
        },
        Some(&admin),
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("not an active numeric custom field"));
}

// --- Dashboard KPI picker ---

#[test]
fn admin_can_choose_which_dashboard_kpis_show() {
    let (conn, ws, admin) = setup_workspace();
    let _ = ws;

    let updated = workspace_service::set_dashboard_kpis(
        &conn,
        &DashboardKpiPrefs { keys: vec!["open_pipeline".into(), "overdue_invoices".into()] },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.dashboard_kpi_prefs.as_deref(), Some(r#"["open_pipeline","overdue_invoices"]"#));

    // An empty list resets to "show every KPI, default order" (NULL).
    let reset = workspace_service::set_dashboard_kpis(&conn, &DashboardKpiPrefs { keys: vec![] }, Some(&admin)).unwrap();
    assert_eq!(reset.dashboard_kpi_prefs, None);
}

#[test]
fn non_administrator_cannot_change_dashboard_kpi_prefs() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn,
        &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = workspace_service::set_dashboard_kpis(
        &conn,
        &DashboardKpiPrefs { keys: vec!["open_pipeline".into()] },
        Some(&sales_user.id),
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

// --- Workspace phone field ---

#[test]
fn workspace_profile_can_store_a_phone_number() {
    let (conn, ws, admin) = setup_workspace();
    let _ = ws;
    let updated = workspace_service::update(
        &conn,
        &WorkspaceUpdate {
            business_name: "Admin Flex Co".into(),
            legal_name: None,
            business_address: None,
            phone: Some("+1 555-0100".into()),
            currency_code: "USD".into(),
            locale: "en-US".into(),
            timezone: "UTC".into(),
            default_tax_rate_bp: 0,
        },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(updated.phone.as_deref(), Some("+1 555-0100"));
}
