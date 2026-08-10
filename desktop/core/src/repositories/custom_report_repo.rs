use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::custom_report::{CustomReport, CustomReportInput, CustomReportUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CustomReport> {
    Ok(CustomReport {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        entity_type: row.get("entity_type")?,
        group_by_source: row.get("group_by_source")?,
        group_by_field: row.get("group_by_field")?,
        aggregate: row.get("aggregate")?,
        sum_field_key: row.get("sum_field_key")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn create(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &CustomReportInput,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<CustomReport> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO custom_reports
            (id, workspace_id, name, entity_type, group_by_source, group_by_field, aggregate, sum_field_key, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)",
        rusqlite::params![
            id, workspace_id, input.name, input.entity_type, input.group_by_source, input.group_by_field,
            input.aggregate, input.sum_field_key, now, actor_user_id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<CustomReport>> {
    conn.query_row("SELECT * FROM custom_reports WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<CustomReport>> {
    let mut stmt = conn.prepare("SELECT * FROM custom_reports WHERE workspace_id = ?1 ORDER BY created_at")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

pub fn update(conn: &Connection, id: &str, input: &CustomReportUpdate, actor_user_id: Option<&str>) -> rusqlite::Result<CustomReport> {
    conn.execute(
        "UPDATE custom_reports
         SET name = ?1, group_by_source = ?2, group_by_field = ?3, aggregate = ?4, sum_field_key = ?5, updated_at = ?6, updated_by = ?7
         WHERE id = ?8",
        rusqlite::params![
            input.name, input.group_by_source, input.group_by_field, input.aggregate, input.sum_field_key,
            now_iso(), actor_user_id, id,
        ],
    )?;
    get(conn, id).map(|r| r.expect("just updated"))
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM custom_reports WHERE id = ?1", [id])?;
    Ok(())
}
