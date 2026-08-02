use chrono::{Duration, Utc};
use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::new_uuid;

pub const SESSION_LIFETIME_HOURS: i64 = 24 * 14;

/// Creates a new web session for `user_id` and returns its opaque token
/// (the value to set as the session cookie).
pub fn create(conn: &Connection, workspace_id: &str, user_id: &str) -> rusqlite::Result<String> {
    let id = new_uuid();
    let now = Utc::now();
    let expires_at = now + Duration::hours(SESSION_LIFETIME_HOURS);
    conn.execute(
        "INSERT INTO web_sessions (id, workspace_id, user_id, created_at, last_seen_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?4, ?5)",
        (
            &id,
            workspace_id,
            user_id,
            now.to_rfc3339(),
            expires_at.to_rfc3339(),
        ),
    )?;
    Ok(id)
}

/// Returns the session's user_id if the token exists and has not expired,
/// bumping last_seen_at. Returns None for a missing or expired session -
/// callers should treat that as "not logged in", not as an error.
pub fn resolve_and_touch(conn: &Connection, session_id: &str) -> rusqlite::Result<Option<String>> {
    let user_id: Option<String> = conn
        .query_row(
            "SELECT user_id FROM web_sessions WHERE id = ?1 AND expires_at > ?2",
            (session_id, Utc::now().to_rfc3339()),
            |row| row.get(0),
        )
        .optional()?;

    if user_id.is_some() {
        conn.execute(
            "UPDATE web_sessions SET last_seen_at = ?1 WHERE id = ?2",
            (Utc::now().to_rfc3339(), session_id),
        )?;
    }

    Ok(user_id)
}

pub fn delete(conn: &Connection, session_id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM web_sessions WHERE id = ?1", [session_id])?;
    Ok(())
}

pub fn delete_expired(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM web_sessions WHERE expires_at <= ?1",
        [Utc::now().to_rfc3339()],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory_db;

    fn seed_workspace_and_user(conn: &Connection) -> (String, String) {
        conn.execute(
            "INSERT INTO workspaces (id, business_name, created_at, updated_at) VALUES ('ws1', 'Test', '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO users (id, workspace_id, username, display_name, password_hash, is_active, created_at, updated_at)
             VALUES ('u1', 'ws1', 'admin', 'Admin', 'hash', 1, '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();
        ("ws1".into(), "u1".into())
    }

    #[test]
    fn creates_and_resolves_a_session() {
        let conn = open_in_memory_db().unwrap();
        let (ws, user) = seed_workspace_and_user(&conn);

        let token = create(&conn, &ws, &user).unwrap();
        let resolved = resolve_and_touch(&conn, &token).unwrap();

        assert_eq!(resolved, Some(user));
    }

    #[test]
    fn unknown_token_resolves_to_none() {
        let conn = open_in_memory_db().unwrap();
        seed_workspace_and_user(&conn);

        assert_eq!(resolve_and_touch(&conn, "nonexistent-token").unwrap(), None);
    }

    #[test]
    fn deleted_session_no_longer_resolves() {
        let conn = open_in_memory_db().unwrap();
        let (ws, user) = seed_workspace_and_user(&conn);

        let token = create(&conn, &ws, &user).unwrap();
        delete(&conn, &token).unwrap();

        assert_eq!(resolve_and_touch(&conn, &token).unwrap(), None);
    }

    #[test]
    fn expired_session_does_not_resolve() {
        let conn = open_in_memory_db().unwrap();
        let (ws, user) = seed_workspace_and_user(&conn);

        let id = new_uuid();
        let past = Utc::now() - Duration::hours(1);
        conn.execute(
            "INSERT INTO web_sessions (id, workspace_id, user_id, created_at, last_seen_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?4, ?4)",
            (&id, &ws, &user, past.to_rfc3339()),
        )
        .unwrap();

        assert_eq!(resolve_and_touch(&conn, &id).unwrap(), None);
    }
}
