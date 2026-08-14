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
        phone: row.get("phone")?,
        email: row.get("email")?,
        website: row.get("website")?,
        annual_revenue_cents: row.get("annual_revenue_cents")?,
        employee_count: row.get("employee_count")?,
        preferred_contact_method: row.get("preferred_contact_method")?,
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
        "INSERT INTO companies (id, workspace_id, customer_number, name, status, owner_user_id, tax_number, billing_address, shipping_address, tags, notes, phone, email, website, annual_revenue_cents, employee_count, preferred_contact_method, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?18, ?19)",
        // A 19-arity tuple is past rusqlite's built-in tuple Params impl, so
        // this uses the params! macro instead (no arity limit) - the same
        // pattern business_rule_repo.rs already uses for its own wide inserts.
        rusqlite::params![
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
            &input.phone,
            &input.email,
            &input.website,
            &input.annual_revenue_cents,
            &input.employee_count,
            &input.preferred_contact_method,
            &now,
            actor_user_id,
        ],
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
            phone = ?9, email = ?10, website = ?11, annual_revenue_cents = ?12,
            employee_count = ?13, preferred_contact_method = ?14,
            updated_at = ?15, updated_by = ?16
         WHERE id = ?17",
        // 17 params - past the raw-tuple Params impl's 16-item limit, so
        // params! again (see create() above).
        rusqlite::params![
            &input.name,
            &input.status,
            &input.owner_user_id,
            &input.tax_number,
            &input.billing_address,
            &input.shipping_address,
            &input.tags,
            &input.notes,
            &input.phone,
            &input.email,
            &input.website,
            &input.annual_revenue_cents,
            &input.employee_count,
            &input.preferred_contact_method,
            &now,
            actor_user_id,
            id,
        ],
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
