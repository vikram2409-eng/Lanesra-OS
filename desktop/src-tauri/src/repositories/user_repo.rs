use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::user::{User, UserRecord};

pub const ROLES: &[&str] = &["Administrator", "Manager", "Sales", "Finance", "ReadOnly"];

pub fn ensure_roles_seeded(conn: &Connection) -> rusqlite::Result<()> {
    for role in ROLES {
        conn.execute(
            "INSERT OR IGNORE INTO roles (id, name) VALUES (?1, ?2)",
            (new_uuid(), role),
        )?;
    }
    Ok(())
}

fn map_record(row: &rusqlite::Row) -> rusqlite::Result<UserRecord> {
    Ok(UserRecord {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        username: row.get("username")?,
        display_name: row.get("display_name")?,
        password_hash: row.get("password_hash")?,
        is_active: row.get::<_, i64>("is_active")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn roles_for_user(conn: &Connection, user_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT r.name FROM roles r
         JOIN user_roles ur ON ur.role_id = r.id
         WHERE ur.user_id = ?1
         ORDER BY r.name",
    )?;
    let rows = stmt.query_map([user_id], |row| row.get::<_, String>(0))?;
    rows.collect()
}

pub fn to_public(record: UserRecord, roles: Vec<String>) -> User {
    User {
        id: record.id,
        workspace_id: record.workspace_id,
        username: record.username,
        display_name: record.display_name,
        is_active: record.is_active,
        roles,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    username: &str,
    display_name: &str,
    password_hash: &str,
    roles: &[String],
) -> rusqlite::Result<UserRecord> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO users (id, workspace_id, username, display_name, password_hash, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
        (&id, workspace_id, username, display_name, password_hash, &now),
    )?;
    for role_name in roles {
        let role_id: String = conn.query_row(
            "SELECT id FROM roles WHERE name = ?1",
            [role_name],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
            (&id, role_id),
        )?;
    }
    find_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

pub fn find_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<UserRecord>> {
    conn.query_row("SELECT * FROM users WHERE id = ?1", [id], map_record)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e)
            }
        })
}

pub fn find_by_username(
    conn: &Connection,
    workspace_id: &str,
    username: &str,
) -> rusqlite::Result<Option<UserRecord>> {
    conn.query_row(
        "SELECT * FROM users WHERE workspace_id = ?1 AND username = ?2",
        (workspace_id, username),
        map_record,
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

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<UserRecord>> {
    let mut stmt = conn.prepare("SELECT * FROM users WHERE workspace_id = ?1 ORDER BY username")?;
    let rows = stmt.query_map([workspace_id], map_record)?;
    rows.collect()
}
