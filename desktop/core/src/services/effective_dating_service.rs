//! Engine hardening item #5 (documented gap: "no effective-dating query
//! support"): answers "is this record's stated validity window in effect
//! as of date X" - not "what did this record look like as of date X".
//! `custom_field_values` has one row per field per entity with no history/
//! versioning table, so true point-in-time *reconstruction* of a past
//! field value is out of scope - there is nothing to reconstruct from.
//! This is a predicate over each record's *currently stored*
//! `valid_from`/`valid_to` pair instead.
//!
//! Convention, not new admin config: any active custom object with an
//! active `date`-typed `valid_from` custom field is automatically
//! eligible - the same field-key convention Lanesra Industry Foundation's
//! `party_role`, `party_relationship`, `organization_unit`, `location`,
//! `contact_point`, `external_identifier` and `consent_preference`
//! objects already use. Built-in entities are out of scope here since
//! none of them use this convention today.
//!
//! Scope, stated plainly: `active_record_ids_as_of` is wired into
//! `custom_report_service::run_with_as_of` (the custom report
//! builder's "as of" picker), which is where an admin's ad-hoc "how many
//! Locations were active on this date" question naturally lives. Wiring
//! the same predicate into every built-in list view's filter bar as well
//! is real, larger follow-up work (each list view has its own filter UI),
//! not attempted in this pass.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::repositories::{custom_field_repo, custom_record_repo};
use crate::services::custom_object_service;

/// Whether `entity_type` is eligible for `active_record_ids_as_of` below -
/// an active custom object with an active, `date`-typed `valid_from`
/// field. Exposed separately so a list/report UI can decide whether to
/// even offer an "as of" filter for a given object.
pub fn is_effective_dated(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<bool> {
    let is_active_custom_object = custom_object_service::get_by_key(conn, workspace_id, entity_type)?.is_some_and(|d| d.is_active);
    if !is_active_custom_object {
        return Ok(false);
    }
    Ok(custom_field_repo::list_definitions(conn, workspace_id, entity_type)?
        .iter()
        .any(|d| d.key == "valid_from" && d.is_active && d.field_type == "date"))
}

/// Every non-archived record id of `entity_type` whose currently stored
/// `valid_from`/`valid_to` window includes `as_of` (an ISO `YYYY-MM-DD`
/// date, compared lexicographically - the same convention
/// `workflow_service`'s date-trigger matching already relies on):
/// `valid_from <= as_of AND (valid_to is blank OR valid_to >= as_of)`.
/// A record with no `valid_from` value recorded is excluded rather than
/// treated as always-valid - "effective-dated" claims a start date.
pub fn active_record_ids_as_of(conn: &Connection, workspace_id: &str, entity_type: &str, as_of: &str) -> AppResult<Vec<String>> {
    if !is_effective_dated(conn, workspace_id, entity_type)? {
        return Err(AppError::Validation(format!(
            "'{entity_type}' is not effective-dated - it needs an active 'valid_from' date custom field"
        )));
    }
    let mut ids = Vec::new();
    for record in custom_record_repo::list(conn, workspace_id, entity_type)? {
        if record.archived_at.is_some() {
            continue;
        }
        let values = custom_field_repo::get_values(conn, &record.id)?;
        let valid_from = values.get("valid_from").map(String::as_str).unwrap_or("");
        if valid_from.is_empty() || valid_from > as_of {
            continue;
        }
        let valid_to = values.get("valid_to").map(String::as_str).unwrap_or("");
        if !valid_to.is_empty() && valid_to < as_of {
            continue;
        }
        ids.push(record.id);
    }
    Ok(ids)
}
