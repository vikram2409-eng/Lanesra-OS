//! Read-side access to the audit trail every entity's create/update/
//! archive already writes into `audit_events` (see `audit_repo::record`,
//! called from every built-in entity's service plus `custom_record_service`).
//! Until now nothing exposed `audit_repo::list_for_entity` past the Rust
//! layer - this is that one thin, generic endpoint, usable by any entity
//! type (built-in or custom) since `entity_type`/`entity_id` are always
//! plain strings here, the same way `search_service::global_search` stays
//! entity-agnostic.
//!
//! No access check beyond authentication: matches every other read command
//! in this codebase (`company_service::get`, `global_search`, etc.) -
//! nothing here is admin-only, and viewing an entity's history requires no
//! more privilege than viewing the entity itself.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::audit::AuditEvent;
use crate::repositories::audit_repo;

pub fn list_for_entity(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<Vec<AuditEvent>> {
    Ok(audit_repo::list_for_entity(conn, entity_type, entity_id)?)
}
