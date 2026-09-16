//! Enterprise Access Foundation, Phase 1: Record Ownership (spec §2). Every
//! User/Team-owned object now carries record_owner_type/record_owner_id/
//! owning_org_unit_id/assigned_at/ownership_version (0054_ownership_fields.sql)
//! - this service is the one place that reads/writes them, via
//! `ownership_repo`'s generic object_key dispatch.
//!
//! `require_assign_capability` is a stated placeholder: the real
//! per-object, per-scope "Assign" capability is Phase 2's Access Role
//! engine. Until that exists, changing an owner or moving an Organization
//! Unit requires the actor to hold the existing coarse "Administrator"
//! role - one function, one seam, so Phase 2 only has to replace this
//! function's body, not every call site.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ownership::{OwnerRef, OwnershipIneligible, OwnershipMode, OwnershipTransferDryRun, RecordOwnership, OWNER_TYPE_TEAM, OWNER_TYPE_USER};
use crate::repositories::{ownership_repo, user_repo, work_team_repo};
use crate::services::bulk_action_service::BulkActionResult;
use crate::services::custom_object_service;

/// Every built-in object_key this service knows how to own - the same
/// PascalCase vocabulary `entity_registry::CORE_ENTITY_TYPES` already
/// established, reused rather than re-invented.
pub const BUILTIN_OWNED_OBJECT_KEYS: &[&str] = &[
    "Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task",
];

/// Fixed Ownership Mode per built-in object (spec §2.1). A static map, not
/// a physical per-object-type metadata table - see entity_registry.rs's
/// own doc comment for the precedent this follows: a built-in object's
/// ownership shape is a platform fact, not per-workspace configuration, so
/// a lookup table would only add a backfill migration and a write path for
/// no behavioural difference. Every built-in object here is User/Team
/// Owned; none of this product's built-ins are reference/config data (that
/// pattern - Organization Owned - only shows up in Custom Objects so far).
const BUILTIN_OWNERSHIP_MODES: &[(&str, OwnershipMode)] = &[
    ("Company", OwnershipMode::UserTeamOwned),
    ("Contact", OwnershipMode::UserTeamOwned),
    ("Opportunity", OwnershipMode::UserTeamOwned),
    ("Product", OwnershipMode::UserTeamOwned),
    ("Quote", OwnershipMode::UserTeamOwned),
    ("Order", OwnershipMode::UserTeamOwned),
    ("Invoice", OwnershipMode::UserTeamOwned),
    ("Contract", OwnershipMode::UserTeamOwned),
    ("Task", OwnershipMode::UserTeamOwned),
];

pub fn ownership_mode_for(conn: &Connection, workspace_id: &str, object_key: &str) -> AppResult<OwnershipMode> {
    if let Some((_, mode)) = BUILTIN_OWNERSHIP_MODES.iter().find(|(k, _)| *k == object_key) {
        return Ok(*mode);
    }
    let def = custom_object_service::get_by_key(conn, workspace_id, object_key)?
        .ok_or_else(|| AppError::NotFound(format!("Object '{object_key}'")))?;
    Ok(OwnershipMode::from_str(&def.ownership_mode).unwrap_or(OwnershipMode::UserTeamOwned))
}

/// PLACEHOLDER for Phase 2's real Access Role "Assign" capability check -
/// replace this function's body, not its callers, when that engine ships.
pub fn require_assign_capability(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can change record ownership (placeholder for the real Assign capability, coming in Phase 2)".into()));
    }
    Ok(())
}

pub fn get_owner(conn: &Connection, object_key: &str, id: &str) -> AppResult<Option<RecordOwnership>> {
    Ok(ownership_repo::get_owner(conn, object_key, id)?)
}

/// Used by org_unit_service's move-impact preview to sum how many records
/// of `object_key` sit under one specific Organization Unit.
pub fn count_owned_in_org_unit(conn: &Connection, object_key: &str, owning_org_unit_id: &str) -> AppResult<i64> {
    Ok(ownership_repo::count_by_owning_org_unit(conn, object_key, owning_org_unit_id)?)
}

fn validate_owner_ref(conn: &Connection, owner: &OwnerRef) -> AppResult<()> {
    match owner.owner_type.as_str() {
        OWNER_TYPE_USER => {
            if user_repo::find_by_id(conn, &owner.owner_id)?.is_none() {
                return Err(AppError::Validation("Owner user not found".into()));
            }
        }
        OWNER_TYPE_TEAM => {
            let team = work_team_repo::get(conn, &owner.owner_id)?.ok_or_else(|| AppError::Validation("Owner team not found".into()))?;
            if !team.can_own_records {
                return Err(AppError::Validation(format!("'{}' is not configured to own records", team.name)));
            }
        }
        other => return Err(AppError::Validation(format!("Unknown owner type '{other}' - must be USER or TEAM"))),
    }
    Ok(())
}

pub fn set_owner(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
    owner: &OwnerRef,
    owning_org_unit_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    require_assign_capability(conn, actor_user_id)?;
    let mode = ownership_mode_for(conn, workspace_id, object_key)?;
    if mode != OwnershipMode::UserTeamOwned {
        return Err(AppError::Validation(format!("'{object_key}' is not User/Team Owned - it has no individual record owner to assign")));
    }
    validate_owner_ref(conn, owner)?;
    let affected = ownership_repo::set_owner(conn, object_key, id, &owner.owner_type, &owner.owner_id, owning_org_unit_id)?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("{object_key} record")));
    }
    let details = serde_json::json!({"owner_type": owner.owner_type, "owner_id": owner.owner_id, "owning_org_unit_id": owning_org_unit_id});
    crate::repositories::audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "ownership_transferred",
        Some(object_key),
        Some(id),
        &format!("Owner changed to {} {}", owner.owner_type, owner.owner_id),
        Some(&details.to_string()),
    )?;
    Ok(())
}

/// Default owner on record creation: the creating user, at their own
/// primary Organization Unit - called once, right after insert, from each
/// of the 10 object services' own `create` function. `preferred_owner_user_id`
/// is the object's own pre-existing owner_user_id input field where one
/// exists (Company/Opportunity/Contract/Task/Custom Object already let a
/// create form name an owner explicitly - spec §2.4: "default owner =
/// creating user unless workflow, team routing or app policy sets another
/// eligible owner") - None falls back to the creator. Deliberately does
/// NOT call require_assign_capability (a user creating their own record is
/// always allowed to become its initial owner; Assign only gates *later*
/// reassignment) and never fails the create itself - an ownership-mode
/// mismatch (e.g. an Organization Owned object with no owner concept)
/// silently no-ops rather than blocking record creation.
pub fn set_default_owner_on_create(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    id: &str,
    creator_user_id: Option<&str>,
    preferred_owner_user_id: Option<&str>,
) -> AppResult<()> {
    let Some(owner) = preferred_owner_user_id.or(creator_user_id) else { return Ok(()) };
    if ownership_mode_for(conn, workspace_id, object_key)? != OwnershipMode::UserTeamOwned {
        return Ok(());
    }
    let org_unit_id = creator_user_id.and_then(|c| user_repo::primary_org_unit_id(conn, c).ok().flatten());
    ownership_repo::set_owner(conn, object_key, id, OWNER_TYPE_USER, owner, org_unit_id.as_deref())?;
    Ok(())
}

pub fn bulk_transfer_dry_run(conn: &Connection, workspace_id: &str, object_key: &str, ids: &[String], new_owner: &OwnerRef) -> AppResult<OwnershipTransferDryRun> {
    let mode = ownership_mode_for(conn, workspace_id, object_key)?;
    let mut eligible_ids = Vec::new();
    let mut ineligible = Vec::new();
    if mode != OwnershipMode::UserTeamOwned {
        for id in ids {
            ineligible.push(OwnershipIneligible { id: id.clone(), reason: format!("'{object_key}' is not User/Team Owned") });
        }
        return Ok(OwnershipTransferDryRun { eligible_ids, ineligible });
    }
    if let Err(e) = validate_owner_ref(conn, new_owner) {
        for id in ids {
            ineligible.push(OwnershipIneligible { id: id.clone(), reason: e.to_string() });
        }
        return Ok(OwnershipTransferDryRun { eligible_ids, ineligible });
    }
    for id in ids {
        match ownership_repo::get_owner(conn, object_key, id)? {
            Some(_) => eligible_ids.push(id.clone()),
            None => ineligible.push(OwnershipIneligible { id: id.clone(), reason: "Record not found".into() }),
        }
    }
    Ok(OwnershipTransferDryRun { eligible_ids, ineligible })
}

pub fn bulk_transfer_commit(
    conn: &Connection,
    workspace_id: &str,
    object_key: &str,
    ids: &[String],
    new_owner: &OwnerRef,
    owning_org_unit_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<BulkActionResult>> {
    require_assign_capability(conn, actor_user_id)?;
    let dry_run = bulk_transfer_dry_run(conn, workspace_id, object_key, ids, new_owner)?;
    let mut results = Vec::new();
    for id in &dry_run.eligible_ids {
        results.push(match set_owner(conn, workspace_id, object_key, id, new_owner, owning_org_unit_id, actor_user_id) {
            Ok(()) => BulkActionResult { id: id.clone(), ok: true, error: None },
            Err(e) => BulkActionResult { id: id.clone(), ok: false, error: Some(e.to_string()) },
        });
    }
    for ineligible in dry_run.ineligible {
        results.push(BulkActionResult { id: ineligible.id, ok: false, error: Some(ineligible.reason) });
    }
    Ok(results)
}
