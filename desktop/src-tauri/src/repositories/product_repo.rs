use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::product::{Product, ProductInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        product_number: row.get("product_number")?,
        sku: row.get("sku")?,
        r#type: row.get("type")?,
        name: row.get("name")?,
        category: row.get("category")?,
        description: row.get("description")?,
        unit_price_cents: row.get("unit_price_cents")?,
        cost_cents: row.get("cost_cents")?,
        tax_rate_bp: row.get("tax_rate_bp")?,
        default_quantity_milli: row.get("default_quantity_milli")?,
        is_active: row.get::<_, i64>("is_active")? != 0,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    product_number: &str,
    input: &ProductInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Product> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO products (id, workspace_id, product_number, sku, type, name, category, description, unit_price_cents, cost_cents, tax_rate_bp, default_quantity_milli, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?14, ?15)",
        (
            id,
            workspace_id,
            product_number,
            &input.sku,
            &input.r#type,
            &input.name,
            &input.category,
            &input.description,
            input.unit_price_cents,
            input.cost_cents,
            input.tax_rate_bp,
            input.default_quantity_milli,
            input.is_active as i64,
            &now,
            actor_user_id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Product>> {
    conn.query_row("SELECT * FROM products WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Product>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM products WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY name",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ProductInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Product> {
    let now = now_iso();
    conn.execute(
        "UPDATE products SET sku = ?1, type = ?2, name = ?3, category = ?4, description = ?5,
            unit_price_cents = ?6, cost_cents = ?7, tax_rate_bp = ?8, default_quantity_milli = ?9,
            is_active = ?10, updated_at = ?11, updated_by = ?12
         WHERE id = ?13",
        (
            &input.sku,
            &input.r#type,
            &input.name,
            &input.category,
            &input.description,
            input.unit_price_cents,
            input.cost_cents,
            input.tax_rate_bp,
            input.default_quantity_milli,
            input.is_active as i64,
            &now,
            actor_user_id,
            id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE products SET is_active = 0, archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}
