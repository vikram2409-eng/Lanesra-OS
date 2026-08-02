use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::domain::money::DocumentTotals;
use crate::models::order::{Order, OrderLine, OrderLineInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Order> {
    Ok(Order {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        order_number: row.get("order_number")?,
        company_id: row.get("company_id")?,
        contact_id: row.get("contact_id")?,
        source_quote_id: row.get("source_quote_id")?,
        status: row.get("status")?,
        currency_code: row.get("currency_code")?,
        subtotal_cents: row.get("subtotal_cents")?,
        discount_cents: row.get("discount_cents")?,
        tax_cents: row.get("tax_cents")?,
        total_cents: row.get("total_cents")?,
        order_date: row.get("order_date")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

fn map_line_row(row: &rusqlite::Row) -> rusqlite::Result<OrderLine> {
    Ok(OrderLine {
        id: row.get("id")?,
        order_id: row.get("order_id")?,
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

#[allow(clippy::too_many_arguments)]
pub fn create_header(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    order_number: &str,
    company_id: &str,
    contact_id: Option<&str>,
    source_quote_id: Option<&str>,
    currency_code: &str,
    order_date: Option<&str>,
    notes: Option<&str>,
    totals: DocumentTotals,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO orders (id, workspace_id, order_number, company_id, contact_id, source_quote_id, status, currency_code, subtotal_cents, discount_cents, tax_cents, total_cents, order_date, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'Draft', ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?14, ?15)",
        (
            id, workspace_id, order_number, company_id, contact_id, source_quote_id, currency_code,
            totals.subtotal_cents, totals.discount_cents, totals.tax_cents, totals.total_cents,
            order_date, notes, &now, actor_user_id,
        ),
    )?;
    Ok(())
}

pub fn insert_line(
    conn: &Connection,
    order_id: &str,
    input: &OrderLineInput,
    line_total_cents: i64,
    sort_order: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO order_lines (id, order_id, product_id, description, quantity_milli, unit_price_cents, discount_bp, tax_rate_bp, line_total_cents, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            new_uuid(),
            order_id,
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

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Order>> {
    conn.query_row("SELECT * FROM orders WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Order>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM orders WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn list_lines(conn: &Connection, order_id: &str) -> rusqlite::Result<Vec<OrderLine>> {
    let mut stmt =
        conn.prepare("SELECT * FROM order_lines WHERE order_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([order_id], map_line_row)?;
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
        "UPDATE orders SET status = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        (status, &now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE orders SET archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}
