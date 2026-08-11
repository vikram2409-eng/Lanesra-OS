use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::company::{Company, CompanyInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Company> {
    Ok(Company {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        customer_number: row.get("customer_number")?,
        name: row.get("name")?,
        status: row.get("status")?,
        owner_user_id: row.get("owner_user_id")?,
        tax_number: row.get("tax_number")?,
        billing_address: row.get("billing_address")?,
        shipping_address: row.get("shipping_address")?,
        tags: row.get("tags")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        archived_at: row.get("archived_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    customer_number: &str,
    input: &CompanyInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Company> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO companies (id, workspace_id, customer_number, name, status, owner_user_id, tax_number, billing_address, shipping_address, tags, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?12, ?13)",
        (
            id,
            workspace_id,
            customer_number,
            &input.name,
            &input.status,
            &input.owner_user_id,
            &input.tax_number,
            &input.billing_address,
            &input.shipping_address,
            &input.tags,
            &input.notes,
            &now,
            actor_user_id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Company>> {
    conn.query_row("SELECT * FROM companies WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Company>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM companies WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY name",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &CompanyInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Company> {
    let now = now_iso();
    conn.execute(
        "UPDATE companies SET name = ?1, status = ?2, owner_user_id = ?3, tax_number = ?4,
            billing_address = ?5, shipping_address = ?6, tags = ?7, notes = ?8,
            updated_at = ?9, updated_by = ?10
         WHERE id = ?11",
        (
            &input.name,
            &input.status,
            &input.owner_user_id,
            &input.tax_number,
            &input.billing_address,
            &input.shipping_address,
            &input.tags,
            &input.notes,
            &now,
            actor_user_id,
            id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Archives rather than deletes when dependent records exist (FR-COM-06);
/// this MVP always archives on delete requests since companies are almost
/// always referenced elsewhere.
pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE companies SET status = 'Archived', archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn find_duplicates_by_name(
    conn: &Connection,
    workspace_id: &str,
    name: &str,
    exclude_id: Option<&str>,
) -> rusqlite::Result<Vec<Company>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM companies WHERE workspace_id = ?1 AND lower(name) = lower(?2) AND (?3 IS NULL OR id != ?3)",
    )?;
    let rows = stmt.query_map((workspace_id, name, exclude_id), map_row)?;
    rows.collect()
}

/// Direct owner write, bypassing the full `update()` validation/audit
/// path - used by workflow_service's assign_owner action, which
/// deliberately writes at the repo layer so it never re-enters
/// company_service::update (and its own workflow firing).
pub fn set_owner(conn: &Connection, id: &str, owner_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE companies SET owner_user_id = ?1, updated_at = ?2 WHERE id = ?3",
        (owner_user_id, now_iso(), id),
    )?;
    Ok(())
}
