//! Admin Automation & Customization addendum, Phase 2 (spec §2.5): a
//! dedicated Status Transition editor. An Administrator can list the
//! allowed From -> To pairs for an entity's status/stage field; the moment
//! at least one active rule exists for an entity type, every transition on
//! that entity type must match one of those rules, enforced at each
//! entity's existing status-changing call site (`company_service::update`,
//! `quote_service::set_status`, etc. - see `TRANSITION_ENTITY_TYPES`'s doc
//! comment for which entities and why). With zero rules defined, an entity
//! type's transitions stay fully unrestricted - today's behavior,
//! preserved so this feature is opt-in per entity type.

use rusqlite::Connection;

use crate::domain::builtin_fields;
use crate::domain::{AppError, AppResult};
use crate::models::status_transition::{StatusTransition, StatusTransitionInput, TRANSITION_ENTITY_TYPES};
use crate::repositories::{status_transition_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage status transitions".into()));
    }
    Ok(())
}

/// The field this entity type transitions on - `stage` for Opportunity,
/// `status` for everything else - matching `workflow::transition_field_for`.
fn transition_field_key(entity_type: &str) -> &'static str {
    if entity_type == "Opportunity" { "stage" } else { "status" }
}

/// The real allowed values for `entity_type`'s transition field (its
/// select-type builtin field's options), used to validate a rule's
/// `from_status`/`to_status` at definition time.
fn valid_values_for(entity_type: &str) -> &'static [&'static str] {
    builtin_fields::find_builtin_field(entity_type, transition_field_key(entity_type))
        .map(|f| f.options)
        .unwrap_or(&[])
}

fn validate_input(entity_type: &str, input: &StatusTransitionInput) -> AppResult<()> {
    if !TRANSITION_ENTITY_TYPES.contains(&entity_type) {
        return Err(AppError::Validation(format!(
            "'{entity_type}' does not support status transition rules"
        )));
    }
    let valid = valid_values_for(entity_type);
    if let Some(from) = &input.from_status {
        if !valid.contains(&from.as_str()) {
            return Err(AppError::Validation(format!("'{from}' is not a valid {} for {entity_type}", transition_field_key(entity_type))));
        }
    }
    if !valid.contains(&input.to_status.as_str()) {
        return Err(AppError::Validation(format!("'{}' is not a valid {} for {entity_type}", input.to_status, transition_field_key(entity_type))));
    }
    Ok(())
}

pub fn list(conn: &Connection, workspace_id: &str, entity_type: &str, actor_user_id: Option<&str>) -> AppResult<Vec<StatusTransition>> {
    require_admin(conn, actor_user_id)?;
    Ok(status_transition_repo::list(conn, workspace_id, entity_type)?)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &StatusTransitionInput, actor_user_id: Option<&str>) -> AppResult<StatusTransition> {
    require_admin(conn, actor_user_id)?;
    validate_input(&input.entity_type, input)?;
    let id = crate::domain::ids::new_uuid();
    Ok(status_transition_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    status_transition_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Status transition rule".into()))?;
    Ok(status_transition_repo::set_active(conn, id, is_active, actor_user_id)?)
}

pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    status_transition_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Status transition rule".into()))?;
    Ok(status_transition_repo::delete(conn, id)?)
}

/// Called from each of `TRANSITION_ENTITY_TYPES`'s status-changing entry
/// points right before the value is actually written. A no-op (always
/// `Ok`) when `old_status == new_status` (re-saving without transitioning
/// never needs a rule) or when no active rule exists for this entity type
/// (fully unrestricted, the default). Once at least one active rule
/// exists, `new_status` must be reachable from `old_status` per some rule
/// - either an exact `from_status` match, or a wildcard rule
/// (`from_status: None`, "from any status").
pub fn validate_transition(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    old_status: &str,
    new_status: &str,
) -> AppResult<()> {
    if old_status == new_status {
        return Ok(());
    }
    let rules = status_transition_repo::list_active(conn, workspace_id, entity_type)?;
    if rules.is_empty() {
        return Ok(());
    }
    let allowed = rules.iter().any(|r| {
        r.to_status == new_status && r.from_status.as_deref().is_none_or(|from| from == old_status)
    });
    if allowed {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{entity_type} cannot move from '{old_status}' to '{new_status}' - that transition isn't allowed"
        )))
    }
}
