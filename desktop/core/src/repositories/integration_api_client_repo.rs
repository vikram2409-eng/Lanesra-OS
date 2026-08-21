//! Raw CRUD for `integration_api_clients`/`integration_api_credentials`
//! (migration 0032) - see `services::api_client_service` for the issued
//! `{client_id}.{secret}` shape and why the secret is hashed, not
//! encrypted.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::ApiClient;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ApiClient> {
    let scopes_json: String = row.get("scopes_json")?;
    Ok(ApiClient {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        client_id: row.get("client_id")?,
        status: row.get("status")?,
        scopes: serde_json::from_str(&scopes_json).unwrap_or_default(),
        allowed_cidr: row.get("allowed_cidr")?,
        owner_user_id: row.get("owner_user_id")?,
        last_used_at: row.get("last_used_at")?,
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
    client_id: &str,
    scopes_json: &str,
    allowed_cidr: Option<&str>,
    owner_user_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ApiClient> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_api_clients (id, workspace_id, name, client_id, status, scopes_json, allowed_cidr, owner_user_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, ?8, ?9, ?8, ?9)",
        rusqlite::params![id, workspace_id, name, client_id, scopes_json, allowed_cidr, owner_user_id, now, actor_user_id],
    )?;
    get(conn, id).map(|c| c.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ApiClient>> {
    conn.query_row("SELECT * FROM integration_api_clients WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn get_by_client_id(conn: &Connection, client_id: &str) -> rusqlite::Result<Option<ApiClient>> {
    conn.query_row("SELECT * FROM integration_api_clients WHERE client_id = ?1", [client_id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ApiClient>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_api_clients WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn set_status(conn: &Connection, id: &str, status: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_api_clients SET status = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        rusqlite::params![status, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn touch_last_used(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE integration_api_clients SET last_used_at = ?1 WHERE id = ?2", rusqlite::params![now_iso(), id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_api_clients WHERE id = ?1", [id])?;
    Ok(())
}

// --- credentials -------------------------------------------------------

pub fn insert_credential(conn: &Connection, id: &str, workspace_id: &str, api_client_id: &str, secret_hash: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_api_credentials (id, workspace_id, api_client_id, secret_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, workspace_id, api_client_id, secret_hash, now_iso()],
    )?;
    Ok(())
}

/// The current hash for a client - rotation just calls `insert_credential`
/// again and this returns the newest row, so nothing else needs to change
/// (spec §8.1: rotation without disrupting the client record itself).
pub fn current_hash_for(conn: &Connection, api_client_id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT secret_hash FROM integration_api_credentials WHERE api_client_id = ?1 ORDER BY created_at DESC LIMIT 1",
        [api_client_id],
        |r| r.get(0),
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}
