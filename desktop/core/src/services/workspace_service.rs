use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{Duration, Utc};
use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::company::CompanyInput;
use crate::models::contact::ContactInput;
use crate::models::contract::ContractInput;
use crate::models::invoice::PaymentInput;
use crate::models::opportunity::OpportunityInput;
use crate::models::order::OrderInput;
use crate::models::product::ProductInput;
use crate::models::quote::{QuoteInput, QuoteLineInput};
use crate::models::task::TaskInput;
use crate::models::user::{NewUser, User};
use crate::models::workspace::{DashboardKpiPrefs, Workspace, WorkspaceLogo, WorkspaceSetup, WorkspaceUpdate};
use crate::repositories::{audit_repo, user_repo, workspace_repo};
use crate::services::{
    auth_service, company_service, contact_service, contract_service, invoice_service,
    opportunity_service, order_service, product_service, quote_service, task_service, user_service,
};

/// FR-BRD-02: keeps the stored blob small - a business logo has no reason
/// to be larger than this once client-side compressed, and SQLite rows
/// stay cheap to read on every print preview / dashboard load.
const MAX_LOGO_BYTES: usize = 256 * 1024;
const ALLOWED_LOGO_MIME: &[&str] = &["image/png", "image/jpeg"];

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation(
            "Only an Administrator can edit the workspace profile".into(),
        ));
    }
    Ok(())
}

fn current(conn: &Connection) -> AppResult<Workspace> {
    workspace_repo::get_current(conn)?.ok_or_else(|| AppError::Validation("No workspace has been set up yet".into()))
}

/// FR-BRD-01: an Administrator can edit the workspace profile at any time
/// after first-run - previously there was no way to do this at all.
pub fn update(conn: &Connection, input: &WorkspaceUpdate, actor_user_id: Option<&str>) -> AppResult<Workspace> {
    require_admin(conn, actor_user_id)?;
    if input.business_name.trim().is_empty() {
        return Err(AppError::Validation("Business name is required".into()));
    }
    if input.currency_code.trim().len() != 3 {
        return Err(AppError::Validation("Currency code must be 3 letters".into()));
    }

    let workspace = current(conn)?;
    let updated = workspace_repo::update(conn, &workspace.id, input)?;

    audit_repo::record(
        conn,
        &workspace.id,
        actor_user_id,
        "update",
        Some("workspace"),
        Some(&workspace.id),
        "Updated workspace profile",
        None,
    )?;

    Ok(updated)
}

/// FR-BRD-02: rejects anything that isn't PNG/JPEG or that's too large
/// server-side, not just in the client's upload widget - the same "don't
/// trust the client" posture every other validated command already takes.
pub fn set_logo(conn: &Connection, input: &WorkspaceLogo, actor_user_id: Option<&str>) -> AppResult<Workspace> {
    require_admin(conn, actor_user_id)?;
    if !ALLOWED_LOGO_MIME.contains(&input.logo_mime.as_str()) {
        return Err(AppError::Validation("Logo must be a PNG or JPEG image".into()));
    }
    let decoded = BASE64
        .decode(&input.logo_base64)
        .map_err(|e| AppError::Validation(format!("Invalid logo image data: {e}")))?;
    if decoded.len() > MAX_LOGO_BYTES {
        return Err(AppError::Validation(format!(
            "Logo is too large ({} KB) - please use an image under {} KB",
            decoded.len() / 1024,
            MAX_LOGO_BYTES / 1024
        )));
    }

    let workspace = current(conn)?;
    let updated = workspace_repo::set_logo(conn, &workspace.id, &input.logo_base64, &input.logo_mime)?;

    audit_repo::record(
        conn,
        &workspace.id,
        actor_user_id,
        "update",
        Some("workspace"),
        Some(&workspace.id),
        "Updated workspace logo",
        None,
    )?;

    Ok(updated)
}

/// FR-BRD-03: reverts the print letterhead to text-only.
pub fn clear_logo(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<Workspace> {
    require_admin(conn, actor_user_id)?;
    let workspace = current(conn)?;
    let updated = workspace_repo::clear_logo(conn, &workspace.id)?;

    audit_repo::record(
        conn,
        &workspace.id,
        actor_user_id,
        "update",
        Some("workspace"),
        Some(&workspace.id),
        "Removed workspace logo",
        None,
    )?;

    Ok(updated)
}

/// FR-KPI: lets an Administrator choose which Dashboard KPI tiles show,
/// and in what order - an empty list resets to "show every KPI, default
/// order" (stored as NULL, not an empty JSON array, so a workspace that
/// never touches this setting behaves identically to before it existed).
pub fn set_dashboard_kpis(conn: &Connection, prefs: &DashboardKpiPrefs, actor_user_id: Option<&str>) -> AppResult<Workspace> {
    require_admin(conn, actor_user_id)?;
    let workspace = current(conn)?;
    let json = if prefs.keys.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&prefs.keys).map_err(|e| AppError::Validation(format!("Could not encode KPI preferences: {e}")))?)
    };
    let updated = workspace_repo::set_dashboard_kpi_prefs(conn, &workspace.id, json.as_deref())?;

    audit_repo::record(
        conn,
        &workspace.id,
        actor_user_id,
        "update",
        Some("workspace"),
        Some(&workspace.id),
        "Updated Dashboard KPI preferences",
        None,
    )?;

    Ok(updated)
}

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
    let workspace_id = workspace_repo::get_current(conn)?.unwrap().id;

    // A second sample user so the Tasks "By Owner" view has more than one
    // owner to group by.
    let sales_rep = user_service::create(
        conn,
        &workspace_id,
        &NewUser {
            username: "morgan".into(),
            display_name: "Morgan Reyes".into(),
            password: "sample-password-123".into(),
            roles: vec!["Sales".into()],
        },
        actor,
    )?;

    let acme = company_service::create(
        conn,
        &workspace_id,
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

    // Contract sourced from the accepted quote, with a renewal date inside
    // the 60-day alert window so the dashboard's renewal KPI has data
    // (FR-CTR-05). Deliberately has no opportunity link (FR-CTR-03).
    let renewal_date = (Utc::now() + Duration::days(45)).format("%Y-%m-%d").to_string();
    let contract = contract_service::create(
        conn,
        &ContractInput {
            company_id: acme.id.clone(),
            contact_id: Some(acme_contact.id.clone()),
            source_quote_id: Some(accepted_quote.quote.id.clone()),
            title: "Acme Consulting Master Services Agreement".into(),
            r#type: Some("Master Services Agreement".into()),
            value_cents: 540000,
            currency_code: currency_code.to_string(),
            owner_user_id: Some(actor_user_id.to_string()),
            start_date: Some((Utc::now() - Duration::days(200)).format("%Y-%m-%d").to_string()),
            end_date: Some(renewal_date.clone()),
            renewal_date: Some(renewal_date.clone()),
            notice_period_days: Some(30),
            status: "Active".into(),
            notes: Some("Sample contract seeded by first-run setup".into()),
        },
        actor,
    )?;

    // A general task, plus one linked to each of an opportunity, a
    // contract and a company (FR-TSK-02), spread across overdue, due
    // today/upcoming and completed so every task view has sample data.
    task_service::create(
        conn,
        &acme.workspace_id,
        &TaskInput {
            title: "Prepare quarterly business review".into(),
            description: None,
            owner_user_id: Some(sales_rep.id.clone()),
            priority: "Normal".into(),
            status: "Not Started".into(),
            due_date: Some((Utc::now() + Duration::days(7)).format("%Y-%m-%d").to_string()),
            reminder_at: None,
            related_type: None,
            related_id: None,
        },
        actor,
    )?;

    task_service::create(
        conn,
        &acme.workspace_id,
        &TaskInput {
            title: "Follow up on revised quote".into(),
            description: Some("Opportunity next step from the pipeline".into()),
            owner_user_id: Some(actor_user_id.to_string()),
            priority: "High".into(),
            status: "In Progress".into(),
            due_date: Some((Utc::now() - Duration::days(2)).format("%Y-%m-%d").to_string()),
            reminder_at: None,
            related_type: Some("Opportunity".into()),
            related_id: Some(opportunity.id.clone()),
        },
        actor,
    )?;

    task_service::create(
        conn,
        &acme.workspace_id,
        &TaskInput {
            title: "Review renewal terms with legal".into(),
            description: None,
            owner_user_id: Some(actor_user_id.to_string()),
            priority: "Urgent".into(),
            status: "Not Started".into(),
            due_date: Some((Utc::now() + Duration::days(15)).format("%Y-%m-%d").to_string()),
            reminder_at: None,
            related_type: Some("Contract".into()),
            related_id: Some(contract.id.clone()),
        },
        actor,
    )?;

    task_service::create(
        conn,
        &acme.workspace_id,
        &TaskInput {
            title: "Send welcome package".into(),
            description: None,
            owner_user_id: Some(actor_user_id.to_string()),
            priority: "Normal".into(),
            status: "Completed".into(),
            due_date: Some((Utc::now() - Duration::days(10)).format("%Y-%m-%d").to_string()),
            reminder_at: None,
            related_type: Some("Company".into()),
            related_id: Some(acme.id.clone()),
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
