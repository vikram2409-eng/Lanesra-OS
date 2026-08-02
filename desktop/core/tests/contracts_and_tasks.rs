use chrono::{Duration, Utc};

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::contract::ContractInput;
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::task_repo;
use lanesra_core::services::{company_service, contact_service, contract_service, task_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String) {
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
    let (_workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, admin.id)
}

fn workspace_id(conn: &rusqlite::Connection) -> String {
    lanesra_core::repositories::workspace_repo::get_current(conn)
        .unwrap()
        .unwrap()
        .id
}

fn company_input(name: &str) -> CompanyInput {
    CompanyInput {
        name: name.to_string(),
        status: "Prospect".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
    }
}

fn contact_input(company_id: &str) -> ContactInput {
    ContactInput {
        company_id: company_id.to_string(),
        first_name: "Jane".into(),
        last_name: "Doe".into(),
        job_title: None,
        email: Some("jane.doe@example.com".into()),
        phone: None,
        mobile: None,
        is_primary: true,
        status: "Active".into(),
        tags: None,
        notes: None,
    }
}

fn contract_input(company_id: &str) -> ContractInput {
    ContractInput {
        company_id: company_id.to_string(),
        contact_id: None,
        source_quote_id: None,
        title: "Master Services Agreement".into(),
        r#type: Some("MSA".into()),
        value_cents: 100000,
        currency_code: "USD".into(),
        owner_user_id: None,
        start_date: None,
        end_date: None,
        renewal_date: None,
        notice_period_days: Some(30),
        status: "Active".into(),
        notes: None,
    }
}

#[test]
fn contract_requires_an_existing_company() {
    let (conn, admin) = setup_workspace();
    let mut input = contract_input("nonexistent-company");
    input.company_id = "nonexistent-company".into();
    let result = contract_service::create(&conn, &input, Some(&admin));
    assert!(result.is_err(), "contract with a bogus company must be rejected");
}

#[test]
fn contract_number_follows_appendix_b_pattern() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let contract = contract_service::create(&conn, &contract_input(&company.id), Some(&admin)).unwrap();

    let year = Utc::now().format("%Y").to_string();
    assert_eq!(contract.contract_number, format!("CTR-{year}-000001"));
    assert_eq!(contract.status, "Active");
}

#[test]
fn contract_contact_must_belong_to_the_selected_company() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);
    let company_a = company_service::create(&conn, &ws, &company_input("Company A"), Some(&admin)).unwrap();
    let company_b = company_service::create(&conn, &ws, &company_input("Company B"), Some(&admin)).unwrap();
    let contact_b = contact_service::create(&conn, &contact_input(&company_b.id), Some(&admin)).unwrap();

    let mut input = contract_input(&company_a.id);
    input.contact_id = Some(contact_b.id);
    let result = contract_service::create(&conn, &input, Some(&admin));

    assert!(result.is_err(), "contact from a different company must be rejected");
}

#[test]
fn contract_renewal_alerts_are_cumulative_across_windows() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let mut renewing_in_20_days = contract_input(&company.id);
    renewing_in_20_days.title = "Renews in 20 days".into();
    renewing_in_20_days.renewal_date = Some((Utc::now() + Duration::days(20)).format("%Y-%m-%d").to_string());
    contract_service::create(&conn, &renewing_in_20_days, Some(&admin)).unwrap();

    let mut renewing_in_50_days = contract_input(&company.id);
    renewing_in_50_days.title = "Renews in 50 days".into();
    renewing_in_50_days.renewal_date = Some((Utc::now() + Duration::days(50)).format("%Y-%m-%d").to_string());
    contract_service::create(&conn, &renewing_in_50_days, Some(&admin)).unwrap();

    let mut renewing_in_120_days = contract_input(&company.id);
    renewing_in_120_days.title = "Renews in 120 days".into();
    renewing_in_120_days.renewal_date = Some((Utc::now() + Duration::days(120)).format("%Y-%m-%d").to_string());
    contract_service::create(&conn, &renewing_in_120_days, Some(&admin)).unwrap();

    let alerts = contract_service::renewal_alerts(&conn, &ws).unwrap();
    assert_eq!(alerts.within_30_days, 1);
    assert_eq!(alerts.within_60_days, 2);
    assert_eq!(alerts.within_90_days, 2);
}

#[test]
fn general_task_needs_no_relationship() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);

    let task = task_service::create(
        &conn,
        &ws,
        &TaskInput {
            title: "General follow-up".into(),
            description: None,
            owner_user_id: None,
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: None,
            reminder_at: None,
            related_type: None,
            related_id: None,
        },
        Some(&admin),
    )
    .unwrap();

    assert!(task.related_type.is_none());
    assert!(task.task_number.starts_with("TSK-"));
}

#[test]
fn task_rejects_a_relationship_type_without_a_related_id() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);

    let result = task_service::create(
        &conn,
        &ws,
        &TaskInput {
            title: "Incomplete relation".into(),
            description: None,
            owner_user_id: None,
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: None,
            reminder_at: None,
            related_type: Some("Company".into()),
            related_id: None,
        },
        Some(&admin),
    );

    assert!(result.is_err(), "a relationship type without an id must be rejected");
}

#[test]
fn task_related_record_must_actually_exist() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);

    let result = task_service::create(
        &conn,
        &ws,
        &TaskInput {
            title: "Dangling relation".into(),
            description: None,
            owner_user_id: None,
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: None,
            reminder_at: None,
            related_type: Some("Company".into()),
            related_id: Some("nonexistent-company".into()),
        },
        Some(&admin),
    );

    assert!(result.is_err(), "a relation pointing at a nonexistent record must be rejected");
}

#[test]
fn task_linked_to_a_company_round_trips_and_is_listed_by_relation() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let task = task_service::create(
        &conn,
        &ws,
        &TaskInput {
            title: "Send welcome package".into(),
            description: None,
            owner_user_id: None,
            priority: "Low".into(),
            status: "Not Started".into(),
            due_date: None,
            reminder_at: None,
            related_type: Some("Company".into()),
            related_id: Some(company.id.clone()),
        },
        Some(&admin),
    )
    .unwrap();

    assert_eq!(task.related_type.as_deref(), Some("Company"));
    assert_eq!(task.related_id.as_deref(), Some(company.id.as_str()));

    let related = task_service::list_by_related(&conn, "Company", &company.id).unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].id, task.id);
}

#[test]
fn open_and_overdue_task_counts_exclude_completed_and_cancelled() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);

    let make_task = |title: &str, status: &str, due_offset_days: i64| TaskInput {
        title: title.into(),
        description: None,
        owner_user_id: None,
        priority: "Normal".into(),
        status: status.into(),
        due_date: Some((Utc::now() + Duration::days(due_offset_days)).format("%Y-%m-%d").to_string()),
        reminder_at: None,
        related_type: None,
        related_id: None,
    };

    task_service::create(&conn, &ws, &make_task("Overdue task", "Not Started", -3), Some(&admin)).unwrap();
    task_service::create(&conn, &ws, &make_task("Upcoming task", "Not Started", 5), Some(&admin)).unwrap();
    task_service::create(&conn, &ws, &make_task("Completed task", "Completed", -10), Some(&admin)).unwrap();
    task_service::create(&conn, &ws, &make_task("Cancelled task", "Cancelled", -10), Some(&admin)).unwrap();

    let (open, overdue) = task_repo::count_open_and_overdue(&conn, &ws).unwrap();
    assert_eq!(open, 2, "only the two non-completed/cancelled tasks are open");
    assert_eq!(overdue, 1, "only the task with a past due date is overdue");
}
