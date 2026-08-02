use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::invoice::PaymentInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::quote::{QuoteInput, QuoteLineInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    company_service, contact_service, invoice_service, opportunity_service, order_service,
    quote_service, workspace_service,
};

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

#[test]
fn foreign_keys_are_enforced_on_direct_inserts() {
    let (conn, _admin) = setup_workspace();
    // Bypassing the service layer's validation: inserting a contact whose
    // company_id does not exist must fail at the database level (BR-016).
    let result = conn.execute(
        "INSERT INTO contacts (id, workspace_id, contact_number, company_id, first_name, last_name, status, created_at, updated_at)
         VALUES ('c1', 'nonexistent-workspace', 'CON-000001', 'nonexistent-company', 'Jane', 'Doe', 'Active', '2026-01-01', '2026-01-01')",
        [],
    );
    assert!(result.is_err(), "expected a foreign key violation");
}

#[test]
fn opportunity_primary_contact_must_belong_to_selected_company() {
    let (conn, admin) = setup_workspace();
    let company_a = company_service::create(
        &conn,
        &workspace_id(&conn),
        &company_input("Company A"),
        Some(&admin),
    )
    .unwrap();
    let company_b = company_service::create(
        &conn,
        &workspace_id(&conn),
        &company_input("Company B"),
        Some(&admin),
    )
    .unwrap();
    let contact_b = contact_service::create(&conn, &contact_input(&company_b.id), Some(&admin)).unwrap();

    let result = opportunity_service::create(
        &conn,
        &OpportunityInput {
            company_id: company_a.id.clone(),
            primary_contact_id: Some(contact_b.id.clone()),
            name: "Cross-company deal".into(),
            stage: "New".into(),
            status: "Open".into(),
            value_cents: 10000,
            currency_code: "USD".into(),
            probability_bp: 1000,
            expected_close_date: None,
            owner_user_id: None,
            lost_reason: None,
            next_step: None,
        },
        Some(&admin),
    );

    assert!(result.is_err(), "contact from a different company must be rejected");
}

#[test]
fn full_sales_lifecycle_company_to_paid_invoice() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);

    let company = company_service::create(&conn, &ws, &company_input("Globex"), Some(&admin)).unwrap();
    let contact = contact_service::create(&conn, &contact_input(&company.id), Some(&admin)).unwrap();

    let opportunity = opportunity_service::create(
        &conn,
        &OpportunityInput {
            company_id: company.id.clone(),
            primary_contact_id: Some(contact.id.clone()),
            name: "Globex Engagement".into(),
            stage: "Proposal".into(),
            status: "Open".into(),
            value_cents: 500000,
            currency_code: "USD".into(),
            probability_bp: 5000,
            expected_close_date: None,
            owner_user_id: Some(admin.clone()),
            lost_reason: None,
            next_step: None,
        },
        Some(&admin),
    )
    .unwrap();

    let quote = quote_service::create(
        &conn,
        &QuoteInput {
            company_id: company.id.clone(),
            contact_id: Some(contact.id.clone()),
            opportunity_id: Some(opportunity.id.clone()),
            currency_code: "USD".into(),
            issue_date: None,
            expiry_date: None,
            notes: None,
            terms: None,
            lines: vec![QuoteLineInput {
                product_id: None,
                description: "Consulting".into(),
                quantity_milli: 2000,
                unit_price_cents: 50000,
                discount_bp: 1000,
                tax_rate_bp: 800,
            }],
        },
        Some(&admin),
    )
    .unwrap();

    assert_eq!(quote.quote.status, "Draft");
    // 2 units @ $500 = $1000 gross, 10% discount = $900 net, 8% tax = $72 -> total $972
    assert_eq!(quote.quote.subtotal_cents, 100000);
    assert_eq!(quote.quote.discount_cents, 10000);
    assert_eq!(quote.quote.total_cents, 97200);

    // Converting before acceptance must be rejected.
    assert!(quote_service::convert_to_order(&conn, &quote.quote.id, Some(&admin)).is_err());

    quote_service::set_status(&conn, &quote.quote.id, "Sent", Some(&admin)).unwrap();
    let accepted = quote_service::set_status(&conn, &quote.quote.id, "Accepted", Some(&admin)).unwrap();

    let order = quote_service::convert_to_order(&conn, &accepted.quote.id, Some(&admin)).unwrap();
    assert_eq!(order.order.source_quote_id.as_deref(), Some(quote.quote.id.as_str()));
    assert_eq!(order.order.total_cents, quote.quote.total_cents);
    assert_eq!(order.lines.len(), 1);

    // Source quote must remain completely untouched by the conversion.
    let quote_after_conversion = quote_service::get(&conn, &quote.quote.id).unwrap();
    assert_eq!(quote_after_conversion.quote.status, "Accepted");
    assert_eq!(quote_after_conversion.quote.total_cents, 97200);

    let confirmed_order = order_service::set_status(&conn, &order.order.id, "Confirmed", Some(&admin)).unwrap();
    let invoice = order_service::convert_to_invoice(&conn, &confirmed_order.order.id, Some(&admin)).unwrap();
    assert_eq!(invoice.invoice.source_order_id.as_deref(), Some(order.order.id.as_str()));
    assert_eq!(invoice.invoice.total_cents, 97200);
    assert_eq!(invoice.invoice.balance_cents, 97200);
    assert_eq!(invoice.invoice.status, "Draft");

    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();

    let partially_paid = invoice_service::record_payment(
        &conn,
        &invoice.invoice.id,
        &PaymentInput {
            amount_cents: 40000,
            paid_at: "2026-01-15".into(),
            method: Some("Bank transfer".into()),
            reference: None,
        },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(partially_paid.invoice.status, "Partially Paid");
    assert_eq!(partially_paid.invoice.amount_paid_cents, 40000);
    assert_eq!(partially_paid.invoice.balance_cents, 57200);

    let paid = invoice_service::record_payment(
        &conn,
        &invoice.invoice.id,
        &PaymentInput {
            amount_cents: 57200,
            paid_at: "2026-01-20".into(),
            method: Some("Bank transfer".into()),
            reference: None,
        },
        Some(&admin),
    )
    .unwrap();
    assert_eq!(paid.invoice.status, "Paid");
    assert_eq!(paid.invoice.balance_cents, 0);
    assert_eq!(paid.payments.len(), 2);

    // Order was never mutated by the invoice conversion either.
    let order_after_conversion = order_service::get(&conn, &order.order.id).unwrap();
    assert_eq!(order_after_conversion.order.status, "Confirmed");
}

#[test]
fn company_duplicate_names_are_flagged_but_not_blocked() {
    let (conn, admin) = setup_workspace();
    let ws = workspace_id(&conn);
    let first = company_service::create(&conn, &ws, &company_input("Duplicate Inc"), Some(&admin)).unwrap();
    let second = company_service::create(&conn, &ws, &company_input("duplicate inc"), Some(&admin)).unwrap();

    let duplicates = company_service::check_duplicates(&conn, &ws, "Duplicate Inc", None).unwrap();
    assert_eq!(duplicates.len(), 2);
    assert!(duplicates.iter().any(|c| c.id == first.id));
    assert!(duplicates.iter().any(|c| c.id == second.id));
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
