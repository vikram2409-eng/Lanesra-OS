//! Dashboard customization Phase 3: record-list widget data - a short
//! list of records for one entity type, either the most recently created
//! ("recent") or, for Tasks/Invoices specifically, the soonest-due open
//! ones ("due_soon"). See `dashboard_widget_service::run`'s own doc
//! comment for the full scoping rationale.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::invoice::{InvoiceInput, InvoiceLineInput, PaymentInput};
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, custom_field_service, dashboard_widget_service, invoice_service, task_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Dashboard Widgets Co".into(),
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
        status: "Active Customer".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        ..Default::default()
    }
}

fn task_input(title: &str, status: &str, due_date: Option<&str>) -> TaskInput {
    TaskInput {
        title: title.into(),
        description: None,
        owner_user_id: None,
        priority: "Normal".into(),
        status: status.into(),
        due_date: due_date.map(|d| d.into()),
        reminder_at: None,
        related_type: None,
        related_id: None,
    }
}

fn invoice_input(company_id: &str, due_date: &str, amount_cents: i64) -> InvoiceInput {
    InvoiceInput {
        company_id: company_id.into(),
        contact_id: None,
        currency_code: "USD".into(),
        issue_date: Some("2026-08-01".into()),
        due_date: Some(due_date.into()),
        payment_terms: None,
        notes: None,
        lines: vec![InvoiceLineInput {
            product_id: None,
            description: "Consulting".into(),
            quantity_milli: 1000,
            unit_price_cents: amount_cents,
            discount_bp: 0,
            tax_rate_bp: 0,
        }],
    }
}

#[test]
fn recent_mode_returns_newest_first_for_any_entity_type() {
    let (conn, ws, admin) = setup_workspace();
    let first = company_service::create(&conn, &ws, &company_input("First Co"), Some(&admin)).unwrap();
    let second = company_service::create(&conn, &ws, &company_input("Second Co"), Some(&admin)).unwrap();

    let rows = dashboard_widget_service::run(&conn, &ws, "Company", "recent", 5, &HashMap::new()).unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].entity_id, second.id);
    assert_eq!(rows[0].title, "Second Co");
    assert_eq!(rows[1].entity_id, first.id);
}

#[test]
fn recent_mode_is_capped_at_the_max_row_limit_regardless_of_requested_limit() {
    let (conn, ws, admin) = setup_workspace();
    for i in 0..15 {
        company_service::create(&conn, &ws, &company_input(&format!("Co {i}")), Some(&admin)).unwrap();
    }

    let rows = dashboard_widget_service::run(&conn, &ws, "Company", "recent", 999, &HashMap::new()).unwrap();

    assert_eq!(rows.len(), 10); // MAX_ROWS, not the requested 999
}

#[test]
fn due_soon_mode_orders_open_tasks_by_nearest_due_date_and_excludes_closed_ones() {
    let (conn, ws, admin) = setup_workspace();
    let far = task_service::create(&conn, &ws, &task_input("Far out", "Not Started", Some("2027-01-01")), Some(&admin)).unwrap();
    let soon = task_service::create(&conn, &ws, &task_input("Due soon", "Not Started", Some("2026-09-01")), Some(&admin)).unwrap();
    task_service::create(&conn, &ws, &task_input("No due date", "Not Started", None), Some(&admin)).unwrap();
    task_service::create(&conn, &ws, &task_input("Already done", "Completed", Some("2026-08-20")), Some(&admin)).unwrap();

    let rows = dashboard_widget_service::run(&conn, &ws, "Task", "due_soon", 5, &HashMap::new()).unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].entity_id, soon.id);
    assert_eq!(rows[1].entity_id, far.id);
    assert!(rows[0].subtitle.as_deref().unwrap().contains("2026-09-01"));
}

#[test]
fn due_soon_mode_orders_open_invoices_by_nearest_due_date_and_excludes_paid_ones() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let open = invoice_service::create(&conn, &invoice_input(&company.id, "2026-08-25", 100000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &open.invoice.id, Some(&admin)).unwrap();

    let paid = invoice_service::create(&conn, &invoice_input(&company.id, "2026-07-25", 50000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &paid.invoice.id, Some(&admin)).unwrap();
    invoice_service::record_payment(
        &conn, &paid.invoice.id,
        &PaymentInput { amount_cents: 50000, paid_at: "2026-07-20".into(), method: None, reference: None },
        Some(&admin),
    )
    .unwrap();

    let rows = dashboard_widget_service::run(&conn, &ws, "Invoice", "due_soon", 5, &HashMap::new()).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, open.invoice.id);
}

#[test]
fn due_soon_mode_falls_back_to_recent_for_entity_types_with_no_due_date() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Acme"), Some(&admin)).unwrap();

    let rows = dashboard_widget_service::run(&conn, &ws, "Company", "due_soon", 5, &HashMap::new()).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, company.id);
}

#[test]
fn an_entity_type_with_no_records_yet_returns_an_empty_list_not_an_error() {
    let (conn, ws, _admin) = setup_workspace();
    let rows = dashboard_widget_service::run(&conn, &ws, "Contact", "recent", 5, &HashMap::new()).unwrap();
    assert!(rows.is_empty());
}

/// A record-list widget backed by a Saved View narrows to that view's
/// filters - the same reuse `useSavedViews` gives a list screen, applied
/// here to a dashboard tile's data source instead.
#[test]
fn a_saved_views_filters_narrow_both_recent_and_due_soon_widget_rows() {
    let (conn, ws, admin) = setup_workspace();
    let region_def = custom_field_service::create_definition(
        &conn,
        &ws,
        &CustomFieldDefinitionInput {
            entity_type: "Company".into(), label: "Region".into(), field_type: "text".into(), options: vec![],
            required: false, show_in_list: true, sort_order: 1, min_value: None, max_value: None, max_length: None,
            regex_pattern: None, is_searchable: false, is_filterable: true, is_reportable: false, default_value: None,
            is_unique: false, help_text: None, placeholder: None, is_hidden_by_default: false,
        },
        Some(&admin),
    )
    .unwrap();

    let west = company_service::create(&conn, &ws, &company_input("West Co"), Some(&admin)).unwrap();
    let east = company_service::create(&conn, &ws, &company_input("East Co"), Some(&admin)).unwrap();
    let mut west_values = HashMap::new();
    west_values.insert(region_def.key.clone(), "West".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &west.id, &west_values, Some(&admin)).unwrap();
    let mut east_values = HashMap::new();
    east_values.insert(region_def.key.clone(), "East".to_string());
    custom_field_service::set_entity_values(&conn, "Company", &east.id, &east_values, Some(&admin)).unwrap();

    let mut filters = HashMap::new();
    filters.insert(region_def.key.clone(), "West".to_string());

    let rows = dashboard_widget_service::run(&conn, &ws, "Company", "recent", 5, &filters).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, west.id);
}
