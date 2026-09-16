use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::organization::{Organization, OrganizationUpdate};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Organization> {
    Ok(Organization {
        workspace_id: row.get("id")?,
        name: row.get("business_name")?,
        legal_name: row.get("legal_name")?,
        code: row.get::<_, Option<String>>("org_code")?.unwrap_or_default(),
        status: row.get("org_status")?,
        default_currency: row.get("currency_code")?,
        locale: row.get("locale")?,
        timezone: row.get("timezone")?,
        // Nullable at the schema level (a workspace inserted directly, e.g.
        // in a test harness, may not have bootstrapped one yet) -
        // organization_service::get_or_bootstrap is what actually
        // guarantees this is always set for any caller that goes through
        // the service layer.
        root_org_unit_id: row.get::<_, Option<String>>("root_org_unit_id")?.unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn get(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Option<Organization>> {
    conn.query_row("SELECT * FROM workspaces WHERE id = ?1", [workspace_id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn update(conn: &Connection, workspace_id: &str, input: &OrganizationUpdate) -> rusqlite::Result<Organization> {
    conn.execute(
        "UPDATE workspaces SET org_code = ?1, org_status = ?2, updated_at = ?3 WHERE id = ?4",
        (&input.code, &input.status, now_iso(), workspace_id),
    )?;
    conn.query_row("SELECT * FROM workspaces WHERE id = ?1", [workspace_id], map_row)
}

pub fn set_root_org_unit(conn: &Connection, workspace_id: &str, org_unit_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE workspaces SET root_org_unit_id = ?1, updated_at = ?2 WHERE id = ?3",
        (org_unit_id, now_iso(), workspace_id),
    )?;
    Ok(())
}
