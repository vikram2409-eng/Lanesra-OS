use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::contact::{Contact, ContactInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        contact_number: row.get("contact_number")?,
        company_id: row.get("company_id")?,
        first_name: row.get("first_name")?,
        last_name: row.get("last_name")?,
        job_title: row.get("job_title")?,
        email: row.get("email")?,
        phone: row.get("phone")?,
        mobile: row.get("mobile")?,
        is_primary: row.get::<_, i64>("is_primary")? != 0,
        status: row.get("status")?,
        tags: row.get("tags")?,
        notes: row.get("notes")?,
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
    contact_number: &str,
    input: &ContactInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Contact> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO contacts (id, workspace_id, contact_number, company_id, first_name, last_name, job_title, email, phone, mobile, is_primary, status, tags, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?15, ?16)",
        (
            id,
            workspace_id,
            contact_number,
            &input.company_id,
            &input.first_name,
            &input.last_name,
            &input.job_title,
            &input.email,
            &input.phone,
            &input.mobile,
            input.is_primary as i64,
            &input.status,
            &input.tags,
            &input.notes,
            &now,
            actor_user_id,
        ),
    )?;
    get(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Contact>> {
    conn.query_row("SELECT * FROM contacts WHERE id = ?1", [id], map_row)
        .optional()
}

pub fn list_by_company(conn: &Connection, company_id: &str) -> rusqlite::Result<Vec<Contact>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE company_id = ?1 AND archived_at IS NULL ORDER BY last_name, first_name",
    )?;
    let rows = stmt.query_map([company_id], map_row)?;
    rows.collect()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Contact>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE workspace_id = ?1 AND archived_at IS NULL ORDER BY last_name, first_name",
    )?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ContactInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Contact> {
    let now = now_iso();
    conn.execute(
        "UPDATE contacts SET company_id = ?1, first_name = ?2, last_name = ?3, job_title = ?4,
            email = ?5, phone = ?6, mobile = ?7, is_primary = ?8, status = ?9, tags = ?10, notes = ?11,
            updated_at = ?12, updated_by = ?13
         WHERE id = ?14",
        (
            &input.company_id,
            &input.first_name,
            &input.last_name,
            &input.job_title,
            &input.email,
            &input.phone,
            &input.mobile,
            input.is_primary as i64,
            &input.status,
            &input.tags,
            &input.notes,
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
        "UPDATE contacts SET status = 'Archived', archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        (&now, actor_user_id, id),
    )?;
    Ok(())
}

pub fn find_duplicates_by_email(
    conn: &Connection,
    company_id: &str,
    email: &str,
    exclude_id: Option<&str>,
) -> rusqlite::Result<Vec<Contact>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE company_id = ?1 AND lower(email) = lower(?2) AND (?3 IS NULL OR id != ?3)",
    )?;
    let rows = stmt.query_map((company_id, email, exclude_id), map_row)?;
    rows.collect()
}
