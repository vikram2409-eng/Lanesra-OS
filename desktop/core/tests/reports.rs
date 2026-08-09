use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::invoice::{InvoiceInput, InvoiceLineInput, PaymentInput};
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, invoice_service, opportunity_service, report_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Report Test Co".into(),
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
    }
}

fn opportunity_input(company_id: &str, stage: &str, status: &str, value_cents: i64) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(),
        primary_contact_id: None,
        name: format!("Deal ({stage})"),
        stage: stage.into(),
        status: status.into(),
        value_cents,
        currency_code: "USD".into(),
        probability_bp: 0,
        expected_close_date: None,
        owner_user_id: None,
        lost_reason: if status == "Lost" { Some("Budget cut".into()) } else { None },
        next_step: None,
    }
}

fn invoice_input(company_id: &str, issue_date: &str, due_date: &str, amount_cents: i64) -> InvoiceInput {
    InvoiceInput {
        company_id: company_id.into(),
        contact_id: None,
        currency_code: "USD".into(),
        issue_date: Some(issue_date.into()),
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
fn revenue_by_month_only_counts_issued_invoices_in_range() {
    let (conn, _ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &_ws, &company_input("Acme"), Some(&admin)).unwrap();

    let issued = invoice_service::create(&conn, &invoice_input(&company.id, "2026-03-15", "2026-04-15", 100000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &issued.invoice.id, Some(&admin)).unwrap();

    // A Draft invoice with an issue_date set anyway must not be counted as
    // recognized revenue - only actually-issued invoices should.
    invoice_service::create(&conn, &invoice_input(&company.id, "2026-03-20", "2026-04-20", 999900), Some(&admin)).unwrap();

    let rows = report_service::revenue_by_month(&conn, &_ws, &None, &None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].month, "2026-03");
    assert_eq!(rows[0].invoice_count, 1);
    assert_eq!(rows[0].total_cents, 100000);
}

#[test]
fn win_rate_by_owner_and_lost_reasons_reflect_opportunity_outcomes() {
    let (conn, _ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &_ws, &company_input("Acme"), Some(&admin)).unwrap();

    let won = opportunity_service::create(&conn, &opportunity_input(&company.id, "Won", "Open", 200000), Some(&admin)).unwrap();
    let mut won_input = opportunity_input(&company.id, "Won", "Won", 200000);
    won_input.name = won.name.clone();
    opportunity_service::update(&conn, &won.id, &won_input, Some(&admin)).unwrap();

    let lost = opportunity_service::create(&conn, &opportunity_input(&company.id, "Lost", "Open", 50000), Some(&admin)).unwrap();
    let mut lost_input = opportunity_input(&company.id, "Lost", "Lost", 50000);
    lost_input.name = lost.name.clone();
    opportunity_service::update(&conn, &lost.id, &lost_input, Some(&admin)).unwrap();

    let win_rate = report_service::win_rate_by_owner(&conn, &_ws, &None, &None).unwrap();
    assert_eq!(win_rate.len(), 1); // both unowned -> one "Unassigned" bucket
    assert_eq!(win_rate[0].won_count, 1);
    assert_eq!(win_rate[0].lost_count, 1);
    assert_eq!(win_rate[0].won_value_cents, 200000);

    let lost_reasons = report_service::lost_reason_breakdown(&conn, &_ws, &None, &None).unwrap();
    assert_eq!(lost_reasons.len(), 1);
    assert_eq!(lost_reasons[0].reason, "Budget cut");
    assert_eq!(lost_reasons[0].count, 1);
    assert_eq!(lost_reasons[0].value_cents, 50000);
}

#[test]
fn ar_aging_buckets_by_days_past_due() {
    let (conn, _ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &_ws, &company_input("Acme"), Some(&admin)).unwrap();

    // Overdue by ~45 days as of the fixed as_of_date used below.
    let overdue = invoice_service::create(&conn, &invoice_input(&company.id, "2026-01-01", "2026-01-15", 100000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &overdue.invoice.id, Some(&admin)).unwrap();

    // Not yet due.
    let current = invoice_service::create(&conn, &invoice_input(&company.id, "2026-02-01", "2026-06-01", 50000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &current.invoice.id, Some(&admin)).unwrap();

    let buckets = report_service::ar_aging(&conn, &_ws, &Some("2026-03-01".into())).unwrap();
    let overdue_bucket = buckets.iter().find(|b| b.bucket == "31-60 days overdue").unwrap();
    assert_eq!(overdue_bucket.invoice_count, 1);
    assert_eq!(overdue_bucket.balance_cents, 100000);

    let not_yet_due = buckets.iter().find(|b| b.bucket == "Not yet due").unwrap();
    assert_eq!(not_yet_due.invoice_count, 1);
    assert_eq!(not_yet_due.balance_cents, 50000);
}

#[test]
fn ar_aging_excludes_fully_paid_invoices() {
    let (conn, _ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &_ws, &company_input("Acme"), Some(&admin)).unwrap();

    let invoice = invoice_service::create(&conn, &invoice_input(&company.id, "2026-01-01", "2026-01-15", 100000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();
    invoice_service::record_payment(
        &conn,
        &invoice.invoice.id,
        &PaymentInput { amount_cents: 100000, paid_at: "2026-01-20T00:00:00Z".into(), method: None, reference: None },
        Some(&admin),
    )
    .unwrap();

    let buckets = report_service::ar_aging(&conn, &_ws, &Some("2026-03-01".into())).unwrap();
    assert!(buckets.is_empty(), "a fully paid invoice must not appear in any aging bucket");
}

#[test]
fn sales_by_owner_attributes_revenue_via_company_owner() {
    let (conn, _ws, admin) = setup_workspace();
    let mut input = company_input("Acme");
    input.owner_user_id = Some(admin.clone());
    let company = company_service::create(&conn, &_ws, &input, Some(&admin)).unwrap();

    let invoice = invoice_service::create(&conn, &invoice_input(&company.id, "2026-03-10", "2026-04-10", 75000), Some(&admin)).unwrap();
    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();

    let rows = report_service::sales_by_owner(&conn, &_ws, &None, &None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].owner_name, "Admin User");
    assert_eq!(rows[0].invoice_count, 1);
    assert_eq!(rows[0].total_cents, 75000);
}
