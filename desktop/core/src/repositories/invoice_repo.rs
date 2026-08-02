use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::domain::money::DocumentTotals;
use crate::models::invoice::{Invoice, InvoiceLine, InvoiceLineInput, Payment};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Invoice> {
    Ok(Invoice {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        invoice_number: row.get("invoice_number")?,
        company_id: row.get("company_id")?,
        contact_id: row.get("contact_id")?,
        source_order_id: row.get("source_order_id")?,
        status: row.get("status")?,
        currency_code: row.get("currency_code")?,
        subtotal_cents: row.get("subtotal_cents")?,
        discount_cents: row.get("discount_cents")?,
        tax_cents: row.get("tax_cents")?,
        total_cents: row.get("total_cents")?,
        amount_paid_cents: row.get("amount_paid_cents")?,
        balance_cents: row.get("balance_cents")?,
        issue_date: row.get("issue_date")?,
        due_date: row.get("due_date")?,
        payment_terms: row.get("payment_terms")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

fn map_line_row(row: &rusqlite::Row) -> rusqlite::Result<InvoiceLine> {
    Ok(InvoiceLine {
        id: row.get("id")?,
        invoice_id: row.get("invoice_id")?,
        product_id: row.get("product_id")?,
        description: row.get("description")?,
        quantity_milli: row.get("quantity_milli")?,
        unit_price_cents: row.get("unit_price_cents")?,
        discount_bp: row.get("discount_bp")?,
        tax_rate_bp: row.get("tax_rate_bp")?,
        line_total_cents: row.get("line_total_cents")?,
        sort_order: row.get("sort_order")?,
    })
}

fn map_payment_row(row: &rusqlite::Row) -> rusqlite::Result<Payment> {
    Ok(Payment {
        id: row.get("id")?,
        invoice_id: row.get("invoice_id")?,
        amount_cents: row.get("amount_cents")?,
        paid_at: row.get("paid_at")?,
        method: row.get("method")?,
        reference: row.get("reference")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_header(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    invoice_number: &str,
    company_id: &str,
    contact_id: Option<&str>,
    source_order_id: Option<&str>,
    currency_code: &str,
    issue_date: Option<&str>,
    due_date: Option<&str>,
    payment_terms: Option<&str>,
    notes: Option<&str>,
    totals: DocumentTotals,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO invoices (id, workspace_id, invoice_number, company_id, contact_id, source_order_id, status, currency_code, subtotal_cents, discount_cents, tax_cents, total_cents, amount_paid_cents, balance_cents, issue_date, due_date, payment_terms, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'Draft', ?7, ?8, ?9, ?10, ?11, 0, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?16, ?17)",
        rusqlite::params![
            id, workspace_id, invoice_number, company_id, contact_id, source_order_id, currency_code,
            totals.subtotal_cents, totals.discount_cents, totals.tax_cents, totals.total_cents,
            issue_date, due_date, payment_terms, notes, &now, actor_user_id,
        ],
    )?;
    Ok(())
}

pub fn insert_line(
    conn: &Connection,
    invoice_id: &str,
    input: &InvoiceLineInput,
    line_total_cents: i64,
    sort_order: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO invoice_lines (id, invoice_id, product_id, description, quantity_milli, unit_price_cents, discount_bp, tax_rate_bp, line_total_cents, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            new_uuid(),
            invoice_id,
            &input.product_id,
            &input.description,
            input.quantity_milli,
            input.unit_price_cents,
            input.discount_bp,
            input.tax_rate_bp,
            line_total_cents,
            sort_order,
        ),
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Invoice>> {
    conn.query_row("SELECT * FROM invoices WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Invoice>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM invoices WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn list_lines(conn: &Connection, invoice_id: &str) -> rusqlite::Result<Vec<InvoiceLine>> {
    let mut stmt =
        conn.prepare("SELECT * FROM invoice_lines WHERE invoice_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([invoice_id], map_line_row)?;
    rows.collect()
}

pub fn list_payments(conn: &Connection, invoice_id: &str) -> rusqlite::Result<Vec<Payment>> {
    let mut stmt =
        conn.prepare("SELECT * FROM payments WHERE invoice_id = ?1 ORDER BY paid_at")?;
    let rows = stmt.query_map([invoice_id], map_payment_row)?;
    rows.collect()
}

pub fn update_status(
    conn: &Connection,
    id: &str,
    status: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE invoices SET status = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        (status, &now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn record_payment(
    conn: &Connection,
    invoice_id: &str,
    amount_cents: i64,
    paid_at: &str,
    method: Option<&str>,
    reference: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO payments (id, invoice_id, amount_cents, paid_at, method, reference, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (new_uuid(), invoice_id, amount_cents, paid_at, method, reference, &now, actor_user_id),
    )?;

    conn.execute(
        "UPDATE invoices SET amount_paid_cents = amount_paid_cents + ?1, balance_cents = total_cents - (amount_paid_cents + ?1), updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        (amount_cents, &now, actor_user_id, invoice_id),
    )?;
    Ok(())
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE invoices SET archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}
