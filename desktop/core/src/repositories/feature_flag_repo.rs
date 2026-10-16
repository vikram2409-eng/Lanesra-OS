use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::feature_flag::FeatureFlagOverride;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<FeatureFlagOverride> {
    Ok(FeatureFlagOverride {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        flag_key: row.get("flag_key")?,
        enabled: row.get::<_, i64>("enabled")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        updated_by_principal_id: row.get("updated_by_principal_id")?,
    })
}

pub fn get(
    conn: &Connection,
    workspace_id: &str,
    flag_key: &str,
) -> rusqlite::Result<Option<FeatureFlagOverride>> {
    conn.query_row(
        "SELECT * FROM feature_flag_overrides WHERE workspace_id = ?1 AND flag_key = ?2",
        (workspace_id, flag_key),
        map_row,
    )
    .map(Some)
    .or_else(|e| {
        if e == rusqlite::Error::QueryReturnedNoRows {
            Ok(None)
        } else {
            Err(e)
        }
    })
}

pub fn upsert(
    conn: &Connection,
    workspace_id: &str,
    flag_key: &str,
    enabled: bool,
    actor: Option<&str>,
) -> rusqlite::Result<FeatureFlagOverride> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO feature_flag_overrides
            (id, workspace_id, flag_key, enabled, created_at, updated_at, updated_by_principal_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6)
         ON CONFLICT (workspace_id, flag_key)
         DO UPDATE SET enabled = ?4, updated_at = ?5, updated_by_principal_id = ?6",
        (
            new_uuid(),
            workspace_id,
            flag_key,
            i64::from(enabled),
            now,
            actor,
        ),
    )?;
    get(conn, workspace_id, flag_key).map(|r| r.expect("just upserted"))
}

pub fn delete(conn: &Connection, workspace_id: &str, flag_key: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM feature_flag_overrides WHERE workspace_id = ?1 AND flag_key = ?2",
        (workspace_id, flag_key),
    )?;
    Ok(())
}
