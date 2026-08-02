use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::invoice::PaymentInput;
use crate::models::opportunity::OpportunityInput;
use crate::models::order::OrderInput;
use crate::models::product::ProductInput;
use crate::models::quote::{QuoteInput, QuoteLineInput};
use crate::models::user::User;
use crate::models::workspace::{Workspace, WorkspaceSetup};
use crate::repositories::{audit_repo, user_repo, workspace_repo};
use crate::services::{
    auth_service, company_service, contact_service, invoice_service, opportunity_service,
    order_service, product_service, quote_service,
};

/// Runs the first-run wizard (5.1): creates the single workspace this
/// database holds, the local administrator account, and optionally seeds
/// sample data end to end across the sales lifecycle.
pub fn first_run_setup(conn: &Connection, setup: &WorkspaceSetup) -> AppResult<(Workspace, User)> {
    if workspace_repo::get_current(conn)?.is_some() {
        return Err(AppError::Conflict("A workspace already exists".into()));
    }
    if setup.business_name.trim().is_empty() {
        return Err(AppError::Validation("Business name is required".into()));
    }
    if setup.admin_username.trim().is_empty() || setup.admin_password.len() < 8 {
        return Err(AppError::Validation(
            "Administrator username is required and password must be at least 8 characters".into(),
        ));
    }

    let workspace = workspace_repo::create(conn, setup)?;
    user_repo::ensure_roles_seeded(conn)?;

    let password_hash = auth_service::hash_password(&setup.admin_password)?;
    let admin_record = user_repo::create(
        conn,
        &workspace.id,
        &setup.admin_username,
        &setup.admin_display_name,
        &password_hash,
        &["Administrator".to_string()],
    )?;
    let admin_id = admin_record.id.clone();
    let roles = user_repo::roles_for_user(conn, &admin_id)?;
    let admin = user_repo::to_public(admin_record, roles);

    audit_repo::record(
        conn,
        &workspace.id,
        Some(&admin_id),
        "create",
        Some("workspace"),
        Some(&workspace.id),
        &format!("Workspace '{}' created", workspace.business_name),
        None,
    )?;
    audit_repo::record(
        conn,
        &workspace.id,
        Some(&admin_id),
        "user_admin",
        Some("user"),
        Some(&admin_id),
        &format!("Administrator account '{}' created", admin.username),
        None,
    )?;

    if setup.load_sample_data {
        seed_sample_data(conn, &admin_id, &setup.currency_code)?;
    }

    Ok((workspace, admin))
}

fn seed_sample_data(conn: &Connection, actor_user_id: &str, currency_code: &str) -> AppResult<()> {
    let actor = Some(actor_user_id);

    let acme = company_service::create(
        conn,
        &workspace_repo::get_current(conn)?.unwrap().id,
        &CompanyInput {
            name: "Acme Consulting Ltd".into(),
            status: "Active Customer".into(),
            owner_user_id: Some(actor_user_id.to_string()),
            tax_number: None,
            billing_address: Some("1 Market Street, London".into()),
            shipping_address: None,
            tags: Some("sample".into()),
            notes: Some("Sample company seeded by first-run setup".into()),
        },
        actor,
    )?;

    let northwind = company_service::create(
        conn,
        &acme.workspace_id,
        &CompanyInput {
            name: "Northwind Traders".into(),
            status: "Prospect".into(),
            owner_user_id: Some(actor_user_id.to_string()),
            tax_number: None,
            billing_address: Some("221B Baker Street, London".into()),
            shipping_address: None,
            tags: Some("sample".into()),
            notes: None,
        },
        actor,
    )?;

    let acme_contact = contact_service::create(
        conn,
        &ContactInput {
            company_id: acme.id.clone(),
            first_name: "Jordan".into(),
            last_name: "Blake".into(),
            job_title: Some("Operations Director".into()),
            email: Some("jordan.blake@acmeconsulting.example".into()),
            phone: None,
            mobile: None,
            is_primary: true,
            status: "Active".into(),
            tags: None,
            notes: None,
        },
        actor,
    )?;

    contact_service::create(
        conn,
        &ContactInput {
            company_id: northwind.id.clone(),
            first_name: "Sam".into(),
            last_name: "Rivera".into(),
            job_title: Some("Procurement Lead".into()),
            email: Some("sam.rivera@northwindtraders.example".into()),
            phone: None,
            mobile: None,
            is_primary: true,
            status: "Active".into(),
            tags: None,
            notes: None,
        },
        actor,
    )?;

    let consulting_day = product_service::create(
        conn,
        &acme.workspace_id,
        &ProductInput {
            sku: Some("SVC-CONS-DAY".into()),
            r#type: "Service".into(),
            name: "Consulting Day Rate".into(),
            category: Some("Professional Services".into()),
            description: Some("One day of on-site consulting".into()),
            unit_price_cents: 90000,
            cost_cents: 0,
            tax_rate_bp: 2000,
            default_quantity_milli: 1000,
            is_active: true,
        },
        actor,
    )?;

    product_service::create(
        conn,
        &acme.workspace_id,
        &ProductInput {
            sku: Some("SVC-SETUP".into()),
            r#type: "Service".into(),
            name: "Onboarding Setup".into(),
            category: Some("Professional Services".into()),
            description: Some("One-time onboarding and configuration".into()),
            unit_price_cents: 45000,
            cost_cents: 0,
            tax_rate_bp: 2000,
            default_quantity_milli: 1000,
            is_active: true,
        },
        actor,
    )?;

    // Managed opportunity path: Company -> Opportunity -> Quote -> Order -> Invoice.
    let opportunity = opportunity_service::create(
        conn,
        &OpportunityInput {
            company_id: acme.id.clone(),
            primary_contact_id: Some(acme_contact.id.clone()),
            name: "Acme Q3 Advisory Engagement".into(),
            stage: "Negotiation".into(),
            status: "Open".into(),
            value_cents: 450000,
            currency_code: currency_code.to_string(),
            probability_bp: 7000,
            expected_close_date: None,
            owner_user_id: Some(actor_user_id.to_string()),
            lost_reason: None,
            next_step: Some("Send revised quote".into()),
        },
        actor,
    )?;

    let quote = quote_service::create(
        conn,
        &QuoteInput {
            company_id: acme.id.clone(),
            contact_id: Some(acme_contact.id.clone()),
            opportunity_id: Some(opportunity.id.clone()),
            currency_code: currency_code.to_string(),
            issue_date: None,
            expiry_date: None,
            notes: Some("Sample quote seeded by first-run setup".into()),
            terms: Some("Net 30".into()),
            lines: vec![
                QuoteLineInput {
                    product_id: Some(consulting_day.id.clone()),
                    description: consulting_day.name.clone(),
                    quantity_milli: 5000,
                    unit_price_cents: consulting_day.unit_price_cents,
                    discount_bp: 0,
                    tax_rate_bp: consulting_day.tax_rate_bp,
                },
            ],
        },
        actor,
    )?;

    quote_service::set_status(conn, &quote.quote.id, "Sent", actor)?;
    let accepted_quote = quote_service::set_status(conn, &quote.quote.id, "Accepted", actor)?;
    let order = quote_service::convert_to_order(conn, &accepted_quote.quote.id, actor)?;
    let confirmed_order = order_service::set_status(conn, &order.order.id, "Confirmed", actor)?;
    let invoice = order_service::convert_to_invoice(conn, &confirmed_order.order.id, actor)?;
    invoice_service::issue(conn, &invoice.invoice.id, actor)?;
    invoice_service::record_payment(
        conn,
        &invoice.invoice.id,
        &PaymentInput {
            amount_cents: 200000,
            paid_at: crate::domain::ids::now_iso(),
            method: Some("Bank transfer".into()),
            reference: Some("Partial payment on account".into()),
        },
        actor,
    )?;

    // Direct-quote path for the second company, left in Draft to show the
    // pipeline mid-flow (6.1 flexible sales lifecycle).
    quote_service::create(
        conn,
        &QuoteInput {
            company_id: northwind.id.clone(),
            contact_id: None,
            opportunity_id: None,
            currency_code: currency_code.to_string(),
            issue_date: None,
            expiry_date: None,
            notes: Some("Direct quote with no opportunity".into()),
            terms: Some("Net 30".into()),
            lines: vec![QuoteLineInput {
                product_id: None,
                description: "Onboarding Setup".into(),
                quantity_milli: 1000,
                unit_price_cents: 45000,
                discount_bp: 500,
                tax_rate_bp: 2000,
            }],
        },
        actor,
    )?;

    // Direct order with no quote/opportunity at all (FR-ORD-03).
    order_service::create(
        conn,
        &OrderInput {
            company_id: northwind.id.clone(),
            contact_id: None,
            currency_code: currency_code.to_string(),
            order_date: None,
            notes: Some("Direct order, billed on account".into()),
            lines: vec![crate::models::order::OrderLineInput {
                product_id: None,
                description: "Ad-hoc consulting".into(),
                quantity_milli: 2000,
                unit_price_cents: 90000,
                discount_bp: 0,
                tax_rate_bp: 2000,
            }],
        },
        actor,
    )?;

    Ok(())
}
