//! Shared "any record of any entity_type" lookup, used by Custom
//! Relationships (Phase B), Business Rules (Phase C) and Workflow
//! Automation (Phase D) so all three can work generically across every
//! core entity and every admin-defined custom object without a schema
//! change per subsystem.
//!
//! The spec's §24.1 metadata table list proposes a physical
//! `record_registry` table that every entity would write through. This
//! module deliberately takes a cheaper, equivalent path: a lookup function
//! that dispatches on `entity_type` to the entity's own table, the same
//! pattern `custom_field_service::resolve_entity_workspace` already
//! established for custom field values. A physical registry would need a
//! backfill migration and a write-path change in every existing
//! create/update service for no behavioural difference - every caller here
//! only ever needs "does this record exist, what workspace is it in, is it
//! archived, what do I show for it", all of which the entity's own table
//! already answers directly.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_record_repo, invoice_repo, opportunity_repo, order_repo,
    product_repo, quote_repo, task_repo,
};

/// The generic view of a record any entity_type resolves to - enough for
/// relationship linking, related-list rendering, and rule/workflow
/// evaluation without each subsystem re-implementing the per-entity match.
#[derive(Debug, Clone)]
pub struct ResolvedRecord {
    pub workspace_id: String,
    /// Best-effort human-readable label - "Acme Inc", "Jane Doe", a quote
    /// number - used to render related-list rows and workflow/audit
    /// summaries without a second lookup.
    pub display_name: String,
    /// The entity's built-in status/stage field, where it has one - used as
    /// the trigger_context "status" value business rules and workflow
    /// triggers evaluate against. Empty string for entities with none.
    pub status: String,
    pub archived: bool,
}

/// Every entity_type this registry can resolve without a custom object
/// lookup - the built-in vocabulary shared by relationships, business
/// rules and workflow automation. Kept alongside (not merged with)
/// `CUSTOM_FIELD_ENTITY_TYPES` in `models::custom_field`, which is the
/// same list under a different name for a different subsystem's own
/// documentation trail.
pub const CORE_ENTITY_TYPES: &[&str] = &[
    "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task", "Product",
];

pub fn resolve(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<Option<ResolvedRecord>> {
    let resolved = match entity_type {
        "Company" => company_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.name,
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Contact" => contact_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: format!("{} {}", r.first_name, r.last_name).trim().to_string(),
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Opportunity" => opportunity_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.name,
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Quote" => quote_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.quote_number.clone(),
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Order" => order_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.order_number.clone(),
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Invoice" => invoice_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.invoice_number.clone(),
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Contract" => contract_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.title,
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Task" => task_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.title,
            status: r.status,
            archived: r.archived_at.is_some(),
        }),
        "Product" => product_repo::get(conn, entity_id)?.map(|r| ResolvedRecord {
            workspace_id: r.workspace_id,
            display_name: r.name,
            status: if r.is_active { "Active".into() } else { "Inactive".into() },
            archived: r.archived_at.is_some(),
        }),
        _ => custom_record_repo::get(conn, entity_id)?.and_then(|r| {
            if r.object_key == entity_type {
                Some(ResolvedRecord {
                    workspace_id: r.workspace_id,
                    display_name: r.primary_name,
                    status: r.status,
                    archived: r.archived_at.is_some(),
                })
            } else {
                None
            }
        }),
    };
    Ok(resolved)
}

pub fn exists(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<bool> {
    Ok(resolve(conn, entity_type, entity_id)?.is_some())
}
