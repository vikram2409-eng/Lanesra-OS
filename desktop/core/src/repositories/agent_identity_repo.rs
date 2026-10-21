use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::agent_identity::{AgentCredential, AgentIdentity, AgentIdentityInput};

fn not_found_as_none<T>(r: rusqlite::Result<T>) -> rusqlite::Result<Option<T>> {
    match r {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn map_identity(row: &rusqlite::Row) -> rusqlite::Result<AgentIdentity> {
    Ok(AgentIdentity {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        owner_user_id: row.get("owner_user_id")?,
        is_administrator: row.get::<_, i64>("is_administrator")? != 0,
        access_role_ids: Vec::new(),
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn with_roles(conn: &Connection, mut identity: AgentIdentity) -> rusqlite::Result<AgentIdentity> {
    let mut stmt = conn.prepare(
        "SELECT access_role_id FROM agent_identity_access_roles
         WHERE agent_identity_id = ?1 ORDER BY access_role_id",
    )?;
    identity.access_role_ids = stmt
        .query_map([&identity.id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(identity)
}

fn set_roles(conn: &Connection, id: &str, access_role_ids: &[String]) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM agent_identity_access_roles WHERE agent_identity_id = ?1",
        [id],
    )?;
    for role_id in access_role_ids {
        conn.execute(
            "INSERT OR IGNORE INTO agent_identity_access_roles (agent_identity_id, access_role_id)
             VALUES (?1, ?2)",
            (id, role_id),
        )?;
    }
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AgentIdentity>> {
    let identity = not_found_as_none(conn.query_row(
        "SELECT * FROM agent_identities WHERE id = ?1",
        [id],
        map_identity,
    ))?;
    identity.map(|i| with_roles(conn, i)).transpose()
}

pub fn list_for_workspace(
    conn: &Connection,
    workspace_id: &str,
) -> rusqlite::Result<Vec<AgentIdentity>> {
    let mut stmt =
        conn.prepare("SELECT * FROM agent_identities WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt
        .query_map([workspace_id], map_identity)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter().map(|i| with_roles(conn, i)).collect()
}

pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &AgentIdentityInput,
    actor: Option<&str>,
) -> rusqlite::Result<AgentIdentity> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO agent_identities
            (id, workspace_id, name, description, owner_user_id, is_administrator, status,
             created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?8, ?7, ?8)",
        (
            id,
            workspace_id,
            input.name.trim(),
            input.description.as_deref(),
            &input.owner_user_id,
            i64::from(input.is_administrator),
            &now,
            actor,
        ),
    )?;
    set_roles(conn, id, &input.access_role_ids)?;
    get(conn, id).map(|i| i.expect("just inserted"))
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &AgentIdentityInput,
    actor: Option<&str>,
) -> rusqlite::Result<AgentIdentity> {
    conn.execute(
        "UPDATE agent_identities
         SET name = ?2, description = ?3, owner_user_id = ?4, is_administrator = ?5,
             updated_at = ?6, updated_by = ?7
         WHERE id = ?1",
        (
            id,
            input.name.trim(),
            input.description.as_deref(),
            &input.owner_user_id,
            i64::from(input.is_administrator),
            now_iso(),
            actor,
        ),
    )?;
    set_roles(conn, id, &input.access_role_ids)?;
    get(conn, id).map(|i| i.expect("just updated"))
}

pub fn set_status(
    conn: &Connection,
    id: &str,
    status: &str,
    actor: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE agent_identities SET status = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?1",
        (id, status, now_iso(), actor),
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM agent_identities WHERE id = ?1", [id])?;
    Ok(())
}

fn map_credential(row: &rusqlite::Row) -> rusqlite::Result<AgentCredential> {
    Ok(AgentCredential {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        agent_identity_id: row.get("agent_identity_id")?,
        label: row.get("label")?,
        expires_at: row.get("expires_at")?,
        revoked_at: row.get("revoked_at")?,
        revoked_by: row.get("revoked_by")?,
        last_used_at: row.get("last_used_at")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert_credential(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    agent_identity_id: &str,
    label: &str,
    token_hash: &str,
    expires_at: &str,
    actor: Option<&str>,
) -> rusqlite::Result<AgentCredential> {
    conn.execute(
        "INSERT INTO agent_identity_credentials
            (id, workspace_id, agent_identity_id, label, token_hash, expires_at, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            id,
            workspace_id,
            agent_identity_id,
            label,
            token_hash,
            expires_at,
            now_iso(),
            actor,
        ),
    )?;
    get_credential(conn, id).map(|c| c.expect("just inserted"))
}

pub fn get_credential(conn: &Connection, id: &str) -> rusqlite::Result<Option<AgentCredential>> {
    not_found_as_none(conn.query_row(
        "SELECT * FROM agent_identity_credentials WHERE id = ?1",
        [id],
        map_credential,
    ))
}

pub fn find_credential_by_hash(
    conn: &Connection,
    token_hash: &str,
) -> rusqlite::Result<Option<AgentCredential>> {
    not_found_as_none(conn.query_row(
        "SELECT * FROM agent_identity_credentials WHERE token_hash = ?1",
        [token_hash],
        map_credential,
    ))
}

pub fn list_credentials(
    conn: &Connection,
    agent_identity_id: &str,
) -> rusqlite::Result<Vec<AgentCredential>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM agent_identity_credentials WHERE agent_identity_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([agent_identity_id], map_credential)?
        .collect();
    rows
}

pub fn revoke_credential(conn: &Connection, id: &str, actor: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE agent_identity_credentials SET revoked_at = ?2, revoked_by = ?3
         WHERE id = ?1 AND revoked_at IS NULL",
        (id, now_iso(), actor),
    )?;
    Ok(())
}

pub fn touch_credential(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE agent_identity_credentials SET last_used_at = ?2 WHERE id = ?1",
        (id, now_iso()),
    )?;
    Ok(())
}
