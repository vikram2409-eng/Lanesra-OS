use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::money::{self, DocumentTotals};
use crate::domain::numbering::{self, QUOTE};
use crate::domain::{AppError, AppResult};
use crate::models::quote::{Quote, QuoteInput, QuoteWithLines, QUOTE_STATUSES};
use crate::repositories::{audit_repo, company_repo, contact_repo, opportunity_repo, quote_repo};
use crate::services::{status_transition_service, workflow_service};

/// Quotes have no owner of their own, so a workflow assigning to "the
/// record's owner" resolves via the Quote's Company owner - the same
/// attribution invoice_service and report_service::sales_by_owner use.
fn fire_workflow(conn: &Connection, quote: &Quote, old_status: Option<&str>, new_status: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let owner_user_id = company_repo::get(conn, &quote.company_id)?.and_then(|c| c.owner_user_id);
    workflow_service::fire_event(
        conn, &quote.workspace_id, "Quote", &quote.id, old_status, new_status, owner_user_id.as_deref(), actor_user_id,
    )?;
    Ok(())
}

fn validate_relationships(conn: &Connection, input: &QuoteInput) -> AppResult<String> {
    let company = company_repo::get(conn, &input.company_id)?
        .ok_or_else(|| AppError::Validation("Selected company does not exist".into()))?;

    if let Some(contact_id) = &input.contact_id {
        let contact = contact_repo::get(conn, contact_id)?
            .ok_or_else(|| AppError::Validation("Selected contact does not exist".into()))?;
        if contact.company_id != company.id {
            return Err(AppError::Validation(
                "Contact must belong to the selected company".into(),
            ));
        }
    }

    if let Some(opportunity_id) = &input.opportunity_id {
        let opportunity = opportunity_repo::get(conn, opportunity_id)?
            .ok_or_else(|| AppError::Validation("Selected opportunity does not exist".into()))?;
        if opportunity.company_id != company.id {
            return Err(AppError::Validation(
                "Opportunity must belong to the selected company".into(),
            ));
        }
    }

    if input.lines.is_empty() {
        return Err(AppError::Validation(
            "A quote requires at least one line item".into(),
        ));
    }

    Ok(company.workspace_id)
}

fn load(conn: &Connection, id: &str) -> AppResult<QuoteWithLines> {
    let quote = quote_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Quote".into()))?;
    let lines = quote_repo::list_lines(conn, id)?;
    Ok(QuoteWithLines { quote, lines })
}

pub fn create(
    conn: &Connection,
    input: &QuoteInput,
    actor_user_id: Option<&str>,
) -> AppResult<QuoteWithLines> {
    let workspace_id = validate_relationships(conn, input)?;

    let calculations: Vec<_> = input
        .lines
        .iter()
        .map(|l| money::compute_line(l.quantity_milli, l.unit_price_cents, l.discount_bp, l.tax_rate_bp))
        .collect();
    let totals: DocumentTotals = money::aggregate_lines(&calculations);

    let id = new_uuid();
    let quote_number = numbering::allocate_number(conn, &workspace_id, &QUOTE)?;

    quote_repo::create_header(
        conn,
        &id,
        &workspace_id,
        &quote_number,
        &input.company_id,
        input.contact_id.as_deref(),
        input.opportunity_id.as_deref(),
        &input.currency_code,
        input.issue_date.as_deref(),
        input.expiry_date.as_deref(),
        input.notes.as_deref(),
        input.terms.as_deref(),
        totals,
        actor_user_id,
    )?;

    for (idx, (line_input, calc)) in input.lines.iter().zip(calculations.iter()).enumerate() {
        quote_repo::insert_line(conn, &id, line_input, calc.line_total_cents, idx as i64)?;
    }

    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("quote"),
        Some(&id),
        &format!("Created quote {}", quote_number),
        None,
    )?;

    let created = load(conn, &id)?;
    fire_workflow(conn, &created.quote, None, &created.quote.status, actor_user_id)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<QuoteWithLines> {
    load(conn, id)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Quote>> {
    Ok(quote_repo::list(conn, workspace_id)?)
}

pub fn set_status(
    conn: &Connection,
    id: &str,
    status: &str,
    actor_user_id: Option<&str>,
) -> AppResult<QuoteWithLines> {
    if !QUOTE_STATUSES.contains(&status) {
        return Err(AppError::Validation(format!("Invalid quote status '{status}'")));
    }
    let existing = load(conn, id)?;
    if existing.quote.status != status {
        status_transition_service::validate_transition(conn, &existing.quote.workspace_id, "Quote", &existing.quote.status, status)?;
    }
    quote_repo::update_status(conn, id, status, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.quote.workspace_id,
        actor_user_id,
        "status_change",
        Some("quote"),
        Some(id),
        &format!("Quote {} status changed to {}", existing.quote.quote_number, status),
        None,
    )?;
    fire_workflow(conn, &existing.quote, Some(&existing.quote.status), status, actor_user_id)?;
    load(conn, id)
}

/// Converts an accepted quote into a new order, copying company, contact
/// and line items exactly as issued. The source quote is never mutated or
/// deleted (FR-QUO-06) - only an audit "conversion" event links the two.
pub fn convert_to_order(
    conn: &Connection,
    quote_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<crate::models::order::OrderWithLines> {
    let source = load(conn, quote_id)?;
    if source.quote.status != "Accepted" {
        return Err(AppError::Validation(
            "Only an Accepted quote can be converted to an order".into(),
        ));
    }

    let order_id = new_uuid();
    let order_number = numbering::allocate_number(conn, &source.quote.workspace_id, &numbering::ORDER)?;

    let totals = DocumentTotals {
        subtotal_cents: source.quote.subtotal_cents,
        discount_cents: source.quote.discount_cents,
        tax_cents: source.quote.tax_cents,
        total_cents: source.quote.total_cents,
    };

    crate::repositories::order_repo::create_header(
        conn,
        &order_id,
        &source.quote.workspace_id,
        &order_number,
        &source.quote.company_id,
        source.quote.contact_id.as_deref(),
        Some(quote_id),
        &source.quote.currency_code,
        None,
        source.quote.notes.as_deref(),
        totals,
        actor_user_id,
    )?;

    for (idx, line) in source.lines.iter().enumerate() {
        let line_input = crate::models::order::OrderLineInput {
            product_id: line.product_id.clone(),
            description: line.description.clone(),
            quantity_milli: line.quantity_milli,
            unit_price_cents: line.unit_price_cents,
            discount_bp: line.discount_bp,
            tax_rate_bp: line.tax_rate_bp,
        };
        crate::repositories::order_repo::insert_line(
            conn,
            &order_id,
            &line_input,
            line.line_total_cents,
            idx as i64,
        )?;
    }

    audit_repo::record(
        conn,
        &source.quote.workspace_id,
        actor_user_id,
        "conversion",
        Some("quote"),
        Some(quote_id),
        &format!(
            "Converted quote {} to order {}",
            source.quote.quote_number, order_number
        ),
        None,
    )?;

    let order = crate::repositories::order_repo::get(conn, &order_id)?
        .ok_or_else(|| AppError::NotFound("Order".into()))?;
    let lines = crate::repositories::order_repo::list_lines(conn, &order_id)?;
    Ok(crate::models::order::OrderWithLines { order, lines })
}
