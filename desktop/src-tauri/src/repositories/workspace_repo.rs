use rusqlite::Connection;

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::workspace::{Workspace, WorkspaceSetup};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get("id")?,
        business_name: row.get("business_name")?,
        legal_name: row.get("legal_name")?,
        currency_code: row.get("currency_code")?,
        locale: row.get("locale")?,
        timezone: row.get("timezone")?,
        default_tax_rate_bp: row.get("default_tax_rate_bp")?,
        operating_mode: row.get("operating_mode")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// A workspace database holds exactly one workspace row in MVP scope.
pub fn get_current(conn: &Connection) -> rusqlite::Result<Option<Workspace>> {
    conn.query_row("SELECT * FROM workspaces LIMIT 1", [], map_row)
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e)
            }
        })
}

pub fn create(conn: &Connection, setup: &WorkspaceSetup) -> rusqlite::Result<Workspace> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO workspaces (id, business_name, legal_name, currency_code, locale, timezone, default_tax_rate_bp, operating_mode, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'Personal', ?8, ?8)",
        (
            &id,
            &setup.business_name,
            &setup.legal_name,
            &setup.currency_code,
            &setup.locale,
            &setup.timezone,
            setup.default_tax_rate_bp,
            &now,
        ),
    )?;
    conn.query_row("SELECT * FROM workspaces WHERE id = ?1", [&id], map_row)
}
