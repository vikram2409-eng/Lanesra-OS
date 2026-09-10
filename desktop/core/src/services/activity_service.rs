//! AI & Agentic Layer, Phase 3: the Unified Activity Timeline's
//! foundation - a generic log of external interactions (email, call,
//! message) tied to a Company, Contact or Opportunity, complementary to
//! `audit_service` (which answers "what did this workspace's own users
//! change on this record", not "what happened around it").
//!
//! Scoped to exactly the three entity types the product backlog names -
//! Company, Contact, Opportunity - not every built-in/custom object;
//! widening that is its own future decision, not assumed here.
//!
//! Every entry logged through this service today is `source: "manual"`
//! - a person recording an interaction they just had. That's a
//! complete, honest feature on its own, not a placeholder: nothing here
//! pretends to auto-ingest anything. Deliberately **not** built this
//! pass (named on the product backlog as its own proposed item, not
//! silently dropped): a live email Connection (IMAP/Gmail/Microsoft
//! Graph) auto-matching by address, call-transcript/recording upload,
//! and a Slack connector - each needs its own external-service decision
//! (accepted formats, OAuth app registration) this pass doesn't make.
//! When one of those lands, it calls `log_activity` exactly like a
//! human does today, just with `source` set to that connection's id
//! instead of `"manual"` - the schema already has room for it.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::domain::AppError;
use crate::models::activity::{Activity, ActivityInput, ACTIVITY_CHANNELS, ACTIVITY_DIRECTIONS, ACTIVITY_ENTITY_TYPES};
use crate::repositories::activity_repo;
use crate::services::{company_service, contact_service, opportunity_service};

/// Confirms the referenced record actually exists by calling that
/// entity's own existing `get()` - already `NotFound` on a bad id, so
/// no new existence-check code, same FK-validation style
/// `contact_service::create`'s own company lookup already uses. Returns
/// the record's `workspace_id`, the same way `contact_service::create`
/// derives its workspace from the parent Company rather than trusting a
/// caller-supplied value.
fn require_entity_exists(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<String> {
    match entity_type {
        "Company" => company_service::get(conn, entity_id).map(|c| c.workspace_id),
        "Contact" => contact_service::get(conn, entity_id).map(|c| c.workspace_id),
        "Opportunity" => opportunity_service::get(conn, entity_id).map(|o| o.workspace_id),
        other => Err(AppError::Validation(format!("Unsupported activity entity type '{other}'"))),
    }
}

fn validate(conn: &Connection, input: &ActivityInput) -> AppResult<String> {
    if !ACTIVITY_ENTITY_TYPES.contains(&input.entity_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Activities can only be logged against {}, not '{}'",
            ACTIVITY_ENTITY_TYPES.join("/"),
            input.entity_type
        )));
    }
    if !ACTIVITY_CHANNELS.contains(&input.channel.as_str()) {
        return Err(AppError::Validation(format!("Unknown activity channel '{}'", input.channel)));
    }
    if let Some(direction) = &input.direction {
        if !ACTIVITY_DIRECTIONS.contains(&direction.as_str()) {
            return Err(AppError::Validation(format!("Unknown activity direction '{direction}'")));
        }
    }
    if input.body.trim().is_empty() {
        return Err(AppError::Validation("Activity body is required".into()));
    }
    if input.occurred_at.trim().is_empty() {
        return Err(AppError::Validation("Activity occurred_at is required".into()));
    }
    require_entity_exists(conn, &input.entity_type, &input.entity_id)
}

/// Logs one interaction. No permission check beyond authentication -
/// same convention `audit_service::list_for_entity`'s own doc comment
/// states for reading a record's history: logging or viewing what
/// happened around a record needs no more privilege than viewing the
/// record itself already does.
pub fn log_activity(conn: &Connection, input: &ActivityInput, actor_user_id: Option<&str>) -> AppResult<Activity> {
    let workspace_id = validate(conn, input)?;
    Ok(activity_repo::create(conn, &workspace_id, input, "manual", actor_user_id)?)
}

pub fn list_for_entity(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<Vec<Activity>> {
    Ok(activity_repo::list_for_entity(conn, entity_type, entity_id)?)
}
