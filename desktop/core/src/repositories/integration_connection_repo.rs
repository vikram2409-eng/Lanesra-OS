//! Raw CRUD for `integration_connections` (migration 0032) - see
//! `services::connection_service` for validation, secret handling and
//! Test Connection.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::Connection as ConnectionModel;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ConnectionModel> {
    let secret_id: Option<String> = row.get("secret_id")?;
    Ok(ConnectionModel {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        connection_type: row.get("connection_type")?,
        base_url: row.get("base_url")?,
        auth_mode: row.get("auth_mode")?,
        has_secret: secret_id.is_some(),
        config_json: row.get("config_json")?,
        owner_user_id: row.get("owner_user_id")?,
        status: row.get("status")?,
        last_test_at: row.get("last_test_at")?,
        last_test_status: row.get("last_test_status")?,
        last_test_message: row.get("last_test_message")?,
        last_failure_at: row.get("last_failure_at")?,
        credential_expires_at: row.get("credential_expires_at")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    connection_type: &str,
    base_url: Option<&str>,
    auth_mode: &str,
    secret_id: Option<&str>,
    config_json: &str,
    owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ConnectionModel> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_connections
            (id, workspace_id, name, connection_type, base_url, auth_mode, secret_id, config_json, owner_user_id, status, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'disabled', ?10, ?11, ?10, ?11)",
        rusqlite::params![id, workspace_id, name, connection_type, base_url, auth_mode, secret_id, config_json, owner_user_id, now, actor_user_id],
    )?;
    get(conn, id).map(|c| c.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ConnectionModel>> {
    conn.query_row("SELECT * FROM integration_connections WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn secret_id_for(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    // Two layers of "absent" here, not one: the row might not exist at
    // all (`QueryReturnedNoRows`), or it might exist with a NULL
    // `secret_id` (any `auth_mode == "none"` connection, or one whose
    // secret hasn't been set yet) - the inner `Option<String>` already
    // covers that second case, so this must not also wrap it in `Some`.
    conn.query_row("SELECT secret_id FROM integration_connections WHERE id = ?1", [id], |r| r.get::<_, Option<String>>(0))
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ConnectionModel>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_connections WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    base_url: Option<&str>,
    auth_mode: &str,
    secret_id: Option<&str>,
    config_json: &str,
    owner_user_id: Option<&str>,
    status: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ConnectionModel> {
    conn.execute(
        "UPDATE integration_connections SET name = ?1, base_url = ?2, auth_mode = ?3, secret_id = ?4, config_json = ?5, owner_user_id = ?6, status = ?7, updated_at = ?8, updated_by = ?9 WHERE id = ?10",
        rusqlite::params![name, base_url, auth_mode, secret_id, config_json, owner_user_id, status, now_iso(), actor_user_id, id],
    )?;
    get(conn, id).map(|c| c.expect("just updated"))
}

pub fn set_test_result(conn: &Connection, id: &str, status: &str, message: &str, failed: bool) -> rusqlite::Result<()> {
    let now = now_iso();
    if failed {
        conn.execute(
            "UPDATE integration_connections SET status = ?1, last_test_at = ?2, last_test_status = ?1, last_test_message = ?3, last_failure_at = ?2 WHERE id = ?4",
            rusqlite::params![status, now, message, id],
        )?;
    } else {
        conn.execute(
            "UPDATE integration_connections SET status = ?1, last_test_at = ?2, last_test_status = ?1, last_test_message = ?3 WHERE id = ?4",
            rusqlite::params![status, now, message, id],
        )?;
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_connections WHERE id = ?1", [id])?;
    Ok(())
}

/// How many connections in this workspace currently report `status` -
/// the Overview screen's "active"/"failed connections" KPIs (spec §3.1).
pub fn count_by_status(conn: &Connection, workspace_id: &str, status: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM integration_connections WHERE workspace_id = ?1 AND status = ?2",
        rusqlite::params![workspace_id, status],
        |r| r.get(0),
    )
}

/// How many other resources currently point at this Connection -
/// `Delete` is blocked while any exist (spec table 21's "Delete if no
/// dependencies"). Integration Jobs don't reference a Connection
/// directly - they point at an External Object, which does - so the
/// External Objects count below already covers a Job transitively
/// (deleting the External Object a Job depends on is itself blocked by
/// `external_object_service::delete`'s own dependency check).
pub fn dependency_count(conn: &Connection, id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT
            (SELECT COUNT(*) FROM integration_connection_refs WHERE connection_id = ?1) +
            (SELECT COUNT(*) FROM integration_webhooks WHERE connection_id = ?1) +
            (SELECT COUNT(*) FROM integration_external_objects WHERE connection_id = ?1)",
        [id],
        |r| r.get(0),
    )
}
