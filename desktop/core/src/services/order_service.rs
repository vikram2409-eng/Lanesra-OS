use chrono::{Duration, Utc};
use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::money::{self, DocumentTotals};
use crate::domain::numbering::{self, INVOICE, ORDER};
use crate::domain::{AppError, AppResult};
use crate::models::invoice::InvoiceLineInput;
use crate::models::order::{Order, OrderInput, OrderWithLines, ORDER_STATUSES};
use crate::repositories::{audit_repo, company_repo, contact_repo, invoice_repo, order_repo};
use crate::services::{app_service, status_transition_service, workflow_service};

/// Orders have no owner of their own, so a workflow rule assigning to "the
/// record's owner" resolves via the Order's Company owner - the same
/// attribution invoice_service and quote_service use.
fn fire_workflow(conn: &Connection, order: &Order, old_status: Option<&str>, new_status: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let owner_user_id = company_repo::get(conn, &order.company_id)?.and_then(|c| c.owner_user_id);
    workflow_service::fire_event(
        conn, &order.workspace_id, "Order", &order.id, old_status, new_status, owner_user_id.as_deref(), actor_user_id,
    )?;
    Ok(())
}

fn validate_relationships(conn: &Connection, input: &OrderInput) -> AppResult<String> {
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

    if input.lines.is_empty() {
        return Err(AppError::Validation(
            "An order requires at least one line item".into(),
        ));
    }

    Ok(company.workspace_id)
}

fn load(conn: &Connection, id: &str) -> AppResult<OrderWithLines> {
    let order = order_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Order".into()))?;
    let lines = order_repo::list_lines(conn, id)?;
    Ok(OrderWithLines { order, lines })
}

/// Direct order creation without a quote or opportunity (FR-ORD-03).
pub fn create(
    conn: &Connection,
    input: &OrderInput,
    actor_user_id: Option<&str>,
) -> AppResult<OrderWithLines> {
    let workspace_id = validate_relationships(conn, input)?;
    app_service::require_object_write_access(conn, &workspace_id, "Order", actor_user_id)?;

    let calculations: Vec<_> = input
        .lines
        .iter()
        .map(|l| money::compute_line(l.quantity_milli, l.unit_price_cents, l.discount_bp, l.tax_rate_bp))
        .collect();
    let totals: DocumentTotals = money::aggregate_lines(&calculations);

    let id = new_uuid();
    let order_number = numbering::allocate_number(conn, &workspace_id, &ORDER)?;

    order_repo::create_header(
        conn,
        &id,
        &workspace_id,
        &order_number,
        &input.company_id,
        input.contact_id.as_deref(),
        None,
        &input.currency_code,
        input.order_date.as_deref(),
        input.notes.as_deref(),
        totals,
        actor_user_id,
    )?;

    for (idx, (line_input, calc)) in input.lines.iter().zip(calculations.iter()).enumerate() {
        order_repo::insert_line(conn, &id, line_input, calc.line_total_cents, idx as i64)?;
    }

    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("order"),
        Some(&id),
        &format!("Created order {order_number}"),
        None,
    )?;

    let created = load(conn, &id)?;
    fire_workflow(conn, &created.order, None, &created.order.status, actor_user_id)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<OrderWithLines> {
    load(conn, id)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Order>> {
    Ok(order_repo::list(conn, workspace_id)?)
}

pub fn set_status(
    conn: &Connection,
    id: &str,
    status: &str,
    actor_user_id: Option<&str>,
) -> AppResult<OrderWithLines> {
    if !ORDER_STATUSES.contains(&status) {
        return Err(AppError::Validation(format!("Invalid order status '{status}'")));
    }
    let existing = load(conn, id)?;
    app_service::require_object_write_access(conn, &existing.order.workspace_id, "Order", actor_user_id)?;
    status_transition_service::validate_transition(conn, &existing.order.workspace_id, "Order", &existing.order.status, status)?;
    order_repo::update_status(conn, id, status, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.order.workspace_id,
        actor_user_id,
        "status_change",
        Some("order"),
        Some(id),
        &format!("Order {} status changed to {}", existing.order.order_number, status),
        None,
    )?;
    fire_workflow(conn, &existing.order, Some(&existing.order.status), status, actor_user_id)?;
    load(conn, id)
}

/// Converts a confirmed order into a new invoice, copying company, contact
/// and line items exactly as confirmed. The source order is never mutated
/// or deleted - only an audit "conversion" event links the two.
pub fn convert_to_invoice(
    conn: &Connection,
    order_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<crate::models::invoice::InvoiceWithLines> {
    let source = load(conn, order_id)?;
    // The result is a new Invoice, not a write to the source Order (which
    // is never mutated), so the gate is on Invoice write access, the same
    // as calling invoice_service::create directly would require.
    app_service::require_object_write_access(conn, &source.order.workspace_id, "Invoice", actor_user_id)?;

    let invoice_id = new_uuid();
    let invoice_number = numbering::allocate_number(conn, &source.order.workspace_id, &INVOICE)?;

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let due_date = (Utc::now() + Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let totals = DocumentTotals {
        subtotal_cents: source.order.subtotal_cents,
        discount_cents: source.order.discount_cents,
        tax_cents: source.order.tax_cents,
        total_cents: source.order.total_cents,
    };

    invoice_repo::create_header(
        conn,
        &invoice_id,
        &source.order.workspace_id,
        &invoice_number,
        &source.order.company_id,
        source.order.contact_id.as_deref(),
        Some(order_id),
        &source.order.currency_code,
        Some(&today),
        Some(&due_date),
        Some("Net 30"),
        source.order.notes.as_deref(),
        totals,
        actor_user_id,
    )?;

    for (idx, line) in source.lines.iter().enumerate() {
        let line_input = InvoiceLineInput {
            product_id: line.product_id.clone(),
            description: line.description.clone(),
            quantity_milli: line.quantity_milli,
            unit_price_cents: line.unit_price_cents,
            discount_bp: line.discount_bp,
            tax_rate_bp: line.tax_rate_bp,
        };
        invoice_repo::insert_line(conn, &invoice_id, &line_input, line.line_total_cents, idx as i64)?;
    }

    audit_repo::record(
        conn,
        &source.order.workspace_id,
        actor_user_id,
        "conversion",
        Some("order"),
        Some(order_id),
        &format!(
            "Converted order {} to invoice {}",
            source.order.order_number, invoice_number
        ),
        None,
    )?;

    let invoice = invoice_repo::get(conn, &invoice_id)?.ok_or_else(|| AppError::NotFound("Invoice".into()))?;
    let lines = invoice_repo::list_lines(conn, &invoice_id)?;
    let payments = invoice_repo::list_payments(conn, &invoice_id)?;
    Ok(crate::models::invoice::InvoiceWithLines { invoice, lines, payments })
}
