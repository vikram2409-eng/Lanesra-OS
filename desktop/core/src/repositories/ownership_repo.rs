//! Generic get/set of the 5 ownership system fields (record_owner_type,
//! record_owner_id, owning_org_unit_id, assigned_at, ownership_version)
//! across every User/Team-owned object. Dispatches on `object_key` to the
//! right physical table the same way `entity_registry.rs` already does for
//! its own generic lookups - a built-in object_key ("Company", "Contact",
//! ...) maps to its own table; anything else is assumed to be a Custom
//! Object key and read from the shared `custom_records` table instead.
//! Only ever interpolates a table name that came out of the fixed
//! `table_name_for` match (never `object_key` itself) into SQL text - safe
//! against injection by construction, not by escaping.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::ownership::RecordOwnership;

fn table_name_for(object_key: &str) -> Option<&'static str> {
    match object_key {
        "Company" => Some("companies"),
        "Contact" => Some("contacts"),
        "Opportunity" => Some("opportunities"),
        "Product" => Some("products"),
        "Quote" => Some("quotes"),
        "Order" => Some("orders"),
        "Invoice" => Some("invoices"),
        "Contract" => Some("contracts"),
        "Task" => Some("tasks"),
        _ => None,
    }
}

fn map_ownership_row(row: &rusqlite::Row) -> rusqlite::Result<RecordOwnership> {
    let owner_type: Option<String> = row.get(0)?;
    let owner_id: Option<String> = row.get(1)?;
    Ok(RecordOwnership {
        owner: match (owner_type, owner_id) {
            (Some(t), Some(i)) => Some(crate::models::ownership::OwnerRef { owner_type: t, owner_id: i }),
            _ => None,
        },
        owning_org_unit_id: row.get(2)?,
        assigned_at: row.get(3)?,
        ownership_version: row.get(4)?,
    })
}

pub fn get_owner(conn: &Connection, object_key: &str, id: &str) -> rusqlite::Result<Option<RecordOwnership>> {
    let sql_cols = "record_owner_type, record_owner_id, owning_org_unit_id, assigned_at, ownership_version";
    if let Some(table) = table_name_for(object_key) {
        conn.query_row(&format!("SELECT {sql_cols} FROM {table} WHERE id = ?1"), [id], map_ownership_row)
            .optional()
    } else {
        conn.query_row(
            &format!("SELECT {sql_cols} FROM custom_records WHERE id = ?1 AND object_key = ?2"),
            (id, object_key),
            map_ownership_row,
        )
        .optional()
    }
}

/// Sets owner + owning org unit and bumps ownership_version by 1. Returns
/// the number of rows affected (0 means the id/object_key pair didn't
/// match any record - the caller treats that as NotFound).
pub fn set_owner(
    conn: &Connection,
    object_key: &str,
    id: &str,
    owner_type: &str,
    owner_id: &str,
    owning_org_unit_id: Option<&str>,
) -> rusqlite::Result<usize> {
    let now = now_iso();
    if let Some(table) = table_name_for(object_key) {
        conn.execute(
            &format!(
                "UPDATE {table} SET record_owner_type = ?1, record_owner_id = ?2, owning_org_unit_id = ?3, assigned_at = ?4, ownership_version = ownership_version + 1 WHERE id = ?5"
            ),
            (owner_type, owner_id, owning_org_unit_id, &now, id),
        )
    } else {
        conn.execute(
            "UPDATE custom_records SET record_owner_type = ?1, record_owner_id = ?2, owning_org_unit_id = ?3, assigned_at = ?4, ownership_version = ownership_version + 1 WHERE id = ?5 AND object_key = ?6",
            (owner_type, owner_id, owning_org_unit_id, &now, id, object_key),
        )
    }
}

/// Every non-archived record id under `owning_org_unit_id` for this object
/// type - used by org_unit_service's move-impact preview (counts, not a
/// listing) and by ownership_service's bulk-transfer eligibility scan.
pub fn list_ids_by_owning_org_unit(conn: &Connection, object_key: &str, owning_org_unit_id: &str) -> rusqlite::Result<Vec<String>> {
    let sql = if let Some(table) = table_name_for(object_key) {
        format!("SELECT id FROM {table} WHERE owning_org_unit_id = ?1")
    } else {
        "SELECT id FROM custom_records WHERE owning_org_unit_id = ?1 AND object_key = ?2".to_string()
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows: rusqlite::Result<Vec<String>> = if table_name_for(object_key).is_some() {
        stmt.query_map([owning_org_unit_id], |r| r.get(0))?.collect()
    } else {
        stmt.query_map((owning_org_unit_id, object_key), |r| r.get(0))?.collect()
    };
    rows
}

/// Row count only, for the move-impact preview's per-object summary -
/// avoids materializing every id when only a count is shown.
pub fn count_by_owning_org_unit(conn: &Connection, object_key: &str, owning_org_unit_id: &str) -> rusqlite::Result<i64> {
    if let Some(table) = table_name_for(object_key) {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table} WHERE owning_org_unit_id = ?1"), [owning_org_unit_id], |r| r.get(0))
    } else {
        conn.query_row(
            "SELECT COUNT(*) FROM custom_records WHERE owning_org_unit_id = ?1 AND object_key = ?2",
            (owning_org_unit_id, object_key),
            |r| r.get(0),
        )
    }
}
