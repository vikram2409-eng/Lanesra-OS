use chrono::Utc;
use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::money::{self, DocumentTotals};
use crate::domain::numbering::{self, INVOICE};
use crate::domain::{AppError, AppResult};
use crate::models::invoice::{Invoice, InvoiceInput, InvoiceWithLines, PaymentInput, INVOICE_STATUSES};
use crate::repositories::{audit_repo, company_repo, contact_repo, invoice_repo};

fn validate_relationships(conn: &Connection, input: &InvoiceInput) -> AppResult<String> {
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
            "An invoice requires at least one line item".into(),
        ));
    }

    Ok(company.workspace_id)
}

fn load(conn: &Connection, id: &str) -> AppResult<InvoiceWithLines> {
    let invoice = invoice_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Invoice".into()))?;
    let lines = invoice_repo::list_lines(conn, id)?;
    let payments = invoice_repo::list_payments(conn, id)?;
    Ok(InvoiceWithLines { invoice, lines, payments })
}

/// Direct invoice creation without an order (FR-INV-03).
pub fn create(
    conn: &Connection,
    input: &InvoiceInput,
    actor_user_id: Option<&str>,
) -> AppResult<InvoiceWithLines> {
    let workspace_id = validate_relationships(conn, input)?;

    let calculations: Vec<_> = input
        .lines
        .iter()
        .map(|l| money::compute_line(l.quantity_milli, l.unit_price_cents, l.discount_bp, l.tax_rate_bp))
        .collect();
    let totals: DocumentTotals = money::aggregate_lines(&calculations);

    let id = new_uuid();
    let invoice_number = numbering::allocate_number(conn, &workspace_id, &INVOICE)?;

    invoice_repo::create_header(
        conn,
        &id,
        &workspace_id,
        &invoice_number,
        &input.company_id,
        input.contact_id.as_deref(),
        None,
        &input.currency_code,
        input.issue_date.as_deref(),
        input.due_date.as_deref(),
        input.payment_terms.as_deref(),
        input.notes.as_deref(),
        totals,
        actor_user_id,
    )?;

    for (idx, (line_input, calc)) in input.lines.iter().zip(calculations.iter()).enumerate() {
        invoice_repo::insert_line(conn, &id, line_input, calc.line_total_cents, idx as i64)?;
    }

    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "create",
        Some("invoice"),
        Some(&id),
        &format!("Created invoice {invoice_number}"),
        None,
    )?;

    load(conn, &id)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<InvoiceWithLines> {
    load(conn, id)
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Invoice>> {
    Ok(invoice_repo::list(conn, workspace_id)?)
}

pub fn issue(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<InvoiceWithLines> {
    set_status(conn, id, "Issued", actor_user_id)
}

pub fn void(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<InvoiceWithLines> {
    // FR-INV-07: issued invoices cannot be hard-deleted, only voided/cancelled.
    set_status(conn, id, "Void", actor_user_id)
}

fn set_status(
    conn: &Connection,
    id: &str,
    status: &str,
    actor_user_id: Option<&str>,
) -> AppResult<InvoiceWithLines> {
    if !INVOICE_STATUSES.contains(&status) {
        return Err(AppError::Validation(format!("Invalid invoice status '{status}'")));
    }
    let existing = load(conn, id)?;
    invoice_repo::update_status(conn, id, status, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.invoice.workspace_id,
        actor_user_id,
        "status_change",
        Some("invoice"),
        Some(id),
        &format!("Invoice {} status changed to {}", existing.invoice.invoice_number, status),
        None,
    )?;
    load(conn, id)
}

/// Records a payment and recomputes the invoice's paid/outstanding balance
/// and status (6.5 order-to-invoice process).
pub fn record_payment(
    conn: &Connection,
    invoice_id: &str,
    payment: &PaymentInput,
    actor_user_id: Option<&str>,
) -> AppResult<InvoiceWithLines> {
    if payment.amount_cents <= 0 {
        return Err(AppError::Validation("Payment amount must be positive".into()));
    }
    let existing = load(conn, invoice_id)?;
    if matches!(existing.invoice.status.as_str(), "Void" | "Cancelled" | "Draft") {
        return Err(AppError::Validation(format!(
            "Cannot record a payment against a {} invoice",
            existing.invoice.status
        )));
    }

    invoice_repo::record_payment(
        conn,
        invoice_id,
        payment.amount_cents,
        &payment.paid_at,
        payment.method.as_deref(),
        payment.reference.as_deref(),
        actor_user_id,
    )?;

    let updated = load(conn, invoice_id)?;
    let new_status = if updated.invoice.balance_cents <= 0 {
        "Paid"
    } else if updated.invoice.amount_paid_cents > 0 {
        "Partially Paid"
    } else {
        updated.invoice.status.as_str()
    };
    if new_status != updated.invoice.status {
        invoice_repo::update_status(conn, invoice_id, new_status, actor_user_id)?;
    }

    audit_repo::record(
        conn,
        &existing.invoice.workspace_id,
        actor_user_id,
        "payment",
        Some("invoice"),
        Some(invoice_id),
        &format!(
            "Recorded payment of {} on invoice {}",
            money::format_major(payment.amount_cents),
            existing.invoice.invoice_number
        ),
        None,
    )?;

    load(conn, invoice_id)
}

/// Flips Issued/Partially Paid invoices past their due date to Overdue
/// (6.5: "Automatically classify overdue invoices based on due date and
/// unpaid balance"). Intended to be called on dashboard/app load.
pub fn refresh_overdue(conn: &Connection, workspace_id: &str) -> AppResult<usize> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let invoices = invoice_repo::list(conn, workspace_id)?;
    let mut updated = 0;
    for invoice in invoices {
        let is_candidate = matches!(invoice.status.as_str(), "Issued" | "Partially Paid");
        let is_past_due = invoice.due_date.as_deref().is_some_and(|d| d < today.as_str());
        if is_candidate && is_past_due && invoice.balance_cents > 0 {
            invoice_repo::update_status(conn, &invoice.id, "Overdue", None)?;
            updated += 1;
        }
    }
    Ok(updated)
}
