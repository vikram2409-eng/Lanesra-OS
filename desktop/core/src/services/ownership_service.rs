//! Enterprise Access Foundation, Phase 1: Record Ownership (spec §2). Every
//! User/Team-owned object now carries record_owner_type/record_owner_id/
//! owning_org_unit_id/assigned_at/ownership_version (0054_ownership_fields.sql)
//! - this service is the one place that reads/writes them, via
//! `ownership_repo`'s generic object_key dispatch.
//!
//! `require_assign_capability` was a stated placeholder for Phase 1: the
//! real per-object, per-scope "Assign" capability is Access Control v1's
//! Access Role engine (`access_service`). It now delegates there, scoped to
//! the specific record being reassigned - the one seam that phase named,
//! replaced without touching either of this function's two call sites'
//! own shape.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::access_role::Capability;
use crate::models::ownership::{OwnerRef, OwnershipIneligible, OwnershipMode, OwnershipTransferDryRun, RecordOwnership, OWNER_TYPE_TEAM, OWNER_TYPE_USER};
use crate::repositories::{ownership_repo, user_repo, work_team_repo};
use crate::services::access_service;
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

/// Real Access Role "Assign" capability check, scoped to the specific
/// record being reassigned - delegates to `access_service::require_capability`,
/// which resolves the actor's Access Roles' broadest granted scope for
/// `object_key` and checks it covers `id`. Unlike ordinary create/read/
/// update/delete (where `access_service::require_capability` treats a
/// `None` actor as an unattributed system write and skips the check),
/// reassigning an owner must always be attributed to a real person - so
/// this function keeps its own explicit "Not authenticated" guard rather
/// than inheriting that function's more permissive default.
pub fn require_assign_capability(conn: &Connection, object_key: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    access_service::require_capability(conn, Some(actor_id), object_key, Capability::Assign, Some(id))
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
    require_assign_capability(conn, object_key, id, actor_user_id)?;
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
    // No blanket capability check here - `set_owner`'s own per-id call to
    // `require_assign_capability` below is already record-aware, so a mixed
    // batch correctly fails only the ids outside the actor's granted scope
    // rather than the whole batch failing (or succeeding) on one check.
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
