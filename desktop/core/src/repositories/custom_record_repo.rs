use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::custom_record::{CustomRecord, CustomRecordInput, CustomRecordUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CustomRecord> {
    Ok(CustomRecord {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        object_key: row.get("object_key")?,
        display_number: row.get("display_number")?,
        primary_name: row.get("primary_name")?,
        status: row.get("status")?,
        owner_user_id: row.get("owner_user_id")?,
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
    display_number: &str,
    input: &CustomRecordInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<CustomRecord> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO custom_records
            (id, workspace_id, object_key, display_number, primary_name, status, owner_user_id, notes, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)",
        rusqlite::params![
            id, workspace_id, input.object_key, display_number, input.primary_name, input.status,
            input.owner_user_id, input.notes, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|d| d.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<CustomRecord>> {
    conn.query_row("SELECT * FROM custom_records WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// All non-archived records for one custom object, newest first - the
/// same "hide archived by default" convention every list screen in the
/// product already follows.
pub fn list(conn: &Connection, workspace_id: &str, object_key: &str) -> rusqlite::Result<Vec<CustomRecord>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM custom_records
         WHERE workspace_id = ?1 AND object_key = ?2 AND archived_at IS NULL
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map((workspace_id, object_key), map_row)?.collect();
    rows
}

/// Every record for the object, including archived - used by
/// custom_object_service to decide whether a definition can be
/// hard-deleted (only when this list is empty).
pub fn list_all(conn: &Connection, workspace_id: &str, object_key: &str) -> rusqlite::Result<Vec<CustomRecord>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM custom_records WHERE workspace_id = ?1 AND object_key = ?2 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map((workspace_id, object_key), map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &CustomRecordUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<CustomRecord> {
    conn.execute(
        "UPDATE custom_records
         SET primary_name = ?1, status = ?2, owner_user_id = ?3, notes = ?4, updated_at = ?5, updated_by = ?6
         WHERE id = ?7",
        rusqlite::params![input.primary_name, input.status, input.owner_user_id, input.notes, now_iso(), actor_user_id, id],
    )?;
    get(conn, id).map(|d| d.expect("just updated"))
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> rusqlite::Result<CustomRecord> {
    let now = now_iso();
    conn.execute(
        "UPDATE custom_records SET archived_at = ?1, updated_at = ?1, updated_by = ?2 WHERE id = ?3",
        rusqlite::params![now, actor_user_id, id],
    )?;
    get(conn, id).map(|d| d.expect("just archived"))
}

/// See company_repo::set_owner's comment - same reasoning.
pub fn set_owner(conn: &Connection, id: &str, owner_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE custom_records SET owner_user_id = ?1, updated_at = ?2 WHERE id = ?3",
        (owner_user_id, now_iso(), id),
    )?;
    Ok(())
}
