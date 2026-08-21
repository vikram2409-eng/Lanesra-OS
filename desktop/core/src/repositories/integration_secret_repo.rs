//! Raw CRUD for `integration_secrets` (migration 0032) - see
//! `services::secret_service` for the AES-256-GCM encrypt/decrypt this
//! backs. This repo only ever stores/returns ciphertext+nonce; nothing
//! here ever sees a plaintext secret value.

use rusqlite::Connection;

use crate::domain::ids::now_iso;

pub struct StoredSecret {
    pub id: String,
    pub ciphertext: String,
    pub nonce: String,
}

pub fn insert(conn: &Connection, id: &str, workspace_id: &str, label: &str, ciphertext: &str, nonce: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_secrets (id, workspace_id, label, ciphertext, nonce, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, workspace_id, label, ciphertext, nonce, now_iso(), actor_user_id],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<StoredSecret>> {
    conn.query_row("SELECT id, ciphertext, nonce FROM integration_secrets WHERE id = ?1", [id], |row| {
        Ok(StoredSecret { id: row.get(0)?, ciphertext: row.get(1)?, nonce: row.get(2)? })
    })
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

/// Rotation (spec §20: "Provide secret rotation without editing dependent
/// workflows/jobs") - overwrites this same row's ciphertext/nonce in
/// place, so whatever already points at this `secret_id` (a Connection, a
/// Webhook) keeps working unchanged.
pub fn rotate(conn: &Connection, id: &str, ciphertext: &str, nonce: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE integration_secrets SET ciphertext = ?1, nonce = ?2, rotated_at = ?3 WHERE id = ?4",
        rusqlite::params![ciphertext, nonce, now_iso(), id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_secrets WHERE id = ?1", [id])?;
    Ok(())
}
