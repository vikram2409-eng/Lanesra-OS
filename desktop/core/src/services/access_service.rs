//! Access Control v1: the capability + Record Scope evaluator. Two public
//! entry points share one evaluation - `require_capability` enforces it
//! (short-circuits into an `Err`), `explain_access` narrates the exact same
//! decision for the Access Inspector (spec: "a real, traceable answer... not
//! guesswork, reads off the same evaluation the enforcement path uses").
//! `require_capability` is implemented in terms of `explain_access`, not a
//! second parallel check, so that's true by construction, not by promise.
//!
//! A user's effective grant for a (object_key, capability) is the broadest
//! Record Scope across every Access Role they hold that grants it at all -
//! the same "fold multiple grants to the strongest one" rule
//! `app_service.rs`'s own App Builder permission grants already use.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::access_role::{AccessDecision, AccessInspectorResult, AccessRoleCheck, AccessRoleGrant, Capability, RecordScope};
use crate::models::ownership::OWNER_TYPE_USER;
use crate::repositories::{access_role_repo, org_unit_repo, team_membership_repo, user_access_role_repo, user_repo};
use crate::services::{access_role_service, entity_registry, ownership_service};

/// Bootstraps this actor's Access Roles on first use - see
/// `access_role_service::ensure_user_has_access_role`'s own doc comment.
/// Silently no-ops for an actor id that doesn't resolve to a real user
/// (the caller's own "Not authenticated" check handles that case).
fn ensure_actor_bootstrapped(conn: &Connection, actor_user_id: &str) -> AppResult<()> {
    if let Some(user) = user_repo::find_by_id(conn, actor_user_id)? {
        access_role_service::ensure_user_has_access_role(conn, &user.workspace_id, actor_user_id)?;
    }
    Ok(())
}

/// The grant that applies to `object_key` for one role - its own exact-key
/// row if it has one, else its `'*'` default, else nothing.
fn resolve_grant(conn: &Connection, access_role_id: &str, object_key: &str) -> AppResult<Option<AccessRoleGrant>> {
    if let Some(g) = access_role_repo::get_grant_for_object_key(conn, access_role_id, object_key)? {
        return Ok(Some(g));
    }
    Ok(access_role_repo::get_grant_for_object_key(conn, access_role_id, "*")?)
}

/// Whether any of the actor's Access Roles carries an *explicit* grant for
/// `object_key` (its own row, never the `'*'` default) allowing `capability`.
/// Used only by `require_admin_or_explicit_update` below - ordinary record
/// CRUD is fine falling back to `'*'`, but a workspace's schema/automation/
/// org-structure config screens are not: every Standard User's own `'*'`
/// default already grants ordinary Update, and letting that same default
/// silently unlock these screens would be a real, unintended widening, not
/// the "same as today" behavior those screens must keep. Returns `false`
/// (never an error) for `actor_user_id: None` - this is an additional,
/// optional path, not itself the authentication check.
pub fn has_explicit_capability(conn: &Connection, actor_user_id: Option<&str>, object_key: &str, capability: Capability) -> AppResult<bool> {
    let Some(actor_id) = actor_user_id else { return Ok(false) };
    let roles = user_access_role_repo::list_for_user(conn, actor_id)?;
    for role in &roles {
        if let Some(grant) = access_role_repo::get_grant_for_object_key(conn, &role.id, object_key)? {
            if grant.allows(capability) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Additive companion to one of this codebase's existing per-subsystem
/// `require_admin` guards (Organization Units, Custom Objects, Workflow
/// Automation, Business Rules, Custom Fields, Custom Reports, Status
/// Transitions, Numbering, the Workspace profile, Backup/Restore, Work
/// Teams, Relationships, the App Builder). The legacy Administrator role
/// still always passes - unchanged behavior, zero regression risk. A
/// non-Administrator additionally passes if one of their Access Roles
/// explicitly grants Update on `object_key` (see `has_explicit_capability`'s
/// own doc comment on why this never falls back to `'*'`). Everyone else
/// gets back exactly the subsystem's own existing deny message.
pub fn require_admin_or_explicit_update(conn: &Connection, actor_user_id: Option<&str>, object_key: &str, deny_message: &str) -> AppResult<()> {
    if let Some(actor_id) = actor_user_id {
        if user_repo::roles_for_user(conn, actor_id)?.iter().any(|r| r == "Administrator") {
            return Ok(());
        }
    }
    if has_explicit_capability(conn, actor_user_id, object_key, Capability::Update)? {
        return Ok(());
    }
    Err(AppError::Validation(deny_message.to_string()))
}

/// The broadest Record Scope any of the actor's Access Roles grant for this
/// capability on this object_key - `None` if nothing grants it at all.
/// Doesn't check a specific record; pairs with `scope_allows_record` for that.
pub fn capability_grant_for(conn: &Connection, actor_user_id: &str, object_key: &str, capability: Capability) -> AppResult<Option<RecordScope>> {
    ensure_actor_bootstrapped(conn, actor_user_id)?;
    let roles = user_access_role_repo::list_for_user(conn, actor_user_id)?;
    let mut best: Option<RecordScope> = None;
    for role in &roles {
        if let Some(grant) = resolve_grant(conn, &role.id, object_key)? {
            if grant.allows(capability) {
                best = Some(match best {
                    Some(b) => b.max(grant.record_scope),
                    None => grant.record_scope,
                });
            }
        }
    }
    Ok(best)
}

fn is_owner(actor_user_id: &str, owner: &Option<crate::models::ownership::OwnerRef>) -> bool {
    matches!(owner, Some(o) if o.owner_type == OWNER_TYPE_USER && o.owner_id == actor_user_id)
}

/// Owner ⊂ Team: true if the actor owns the record outright, or the
/// record's Team owner counts the actor as an active member, or the
/// record's User owner shares an active Team membership with the actor.
fn is_within_team_scope(conn: &Connection, actor_user_id: &str, owner: &Option<crate::models::ownership::OwnerRef>) -> AppResult<bool> {
    if is_owner(actor_user_id, owner) {
        return Ok(true);
    }
    let Some(owner) = owner else { return Ok(false) };
    match owner.owner_type.as_str() {
        crate::models::ownership::OWNER_TYPE_TEAM => {
            let actor_teams = team_membership_repo::list_for_user(conn, actor_user_id)?;
            Ok(actor_teams.iter().any(|m| m.team_id == owner.owner_id))
        }
        OWNER_TYPE_USER => {
            let actor_team_ids: std::collections::HashSet<String> =
                team_membership_repo::list_for_user(conn, actor_user_id)?.into_iter().map(|m| m.team_id).collect();
            let owner_team_ids: std::collections::HashSet<String> =
                team_membership_repo::list_for_user(conn, &owner.owner_id)?.into_iter().map(|m| m.team_id).collect();
            Ok(actor_team_ids.intersection(&owner_team_ids).next().is_some())
        }
        _ => Ok(false),
    }
}

/// Team ⊂ OrgUnitAndBelow's own extra reach: true if the record's owning
/// Organization Unit is at or below the actor's own primary Organization
/// Unit, using the materialized `path` prefix the same way
/// `org_unit_repo::list_descendants_by_path` already does.
fn is_within_org_unit_scope(conn: &Connection, actor_user_id: &str, owning_org_unit_id: Option<&str>) -> AppResult<bool> {
    let Some(owning_org_unit_id) = owning_org_unit_id else { return Ok(false) };
    let Some(actor_org_unit_id) = user_repo::primary_org_unit_id(conn, actor_user_id)? else { return Ok(false) };
    let Some(actor_unit) = org_unit_repo::get(conn, &actor_org_unit_id)? else { return Ok(false) };
    let Some(record_unit) = org_unit_repo::get(conn, owning_org_unit_id)? else { return Ok(false) };
    Ok(record_unit.path.starts_with(&actor_unit.path))
}

/// Does `scope` cover this specific record for this actor? Each level is a
/// strict superset of the one before it (spec: Owner ⊂ Team ⊂
/// OrgUnitAndBelow ⊂ Organization), so this only ever checks the named
/// level directly - a broader scope already implies the narrower ones.
pub fn scope_allows_record(conn: &Connection, scope: RecordScope, actor_user_id: &str, object_key: &str, record_id: &str) -> AppResult<bool> {
    let ownership = match ownership_service::get_owner(conn, object_key, record_id)? {
        Some(o) => o,
        None => return Ok(false),
    };
    Ok(match scope {
        RecordScope::Owner => is_owner(actor_user_id, &ownership.owner),
        RecordScope::Team => is_within_team_scope(conn, actor_user_id, &ownership.owner)?,
        RecordScope::OrgUnitAndBelow => {
            is_within_team_scope(conn, actor_user_id, &ownership.owner)?
                || is_within_org_unit_scope(conn, actor_user_id, ownership.owning_org_unit_id.as_deref())?
        }
        RecordScope::Organization => true,
    })
}

/// The Access Inspector: walks every Access Role the actor holds, reports
/// which grant matched each one (even the losing ones) and the final
/// decision. `require_capability` calls this and turns the decision into a
/// `Result` - the two are the same evaluation, not two implementations of it.
pub fn explain_access(
    conn: &Connection,
    actor_user_id: &str,
    object_key: &str,
    capability: Capability,
    record_id: Option<&str>,
) -> AppResult<AccessInspectorResult> {
    ensure_actor_bootstrapped(conn, actor_user_id)?;
    let roles = user_access_role_repo::list_for_user(conn, actor_user_id)?;
    let mut checks = Vec::new();
    let mut best: Option<RecordScope> = None;
    let mut best_role_name: Option<String> = None;
    for role in &roles {
        let grant = resolve_grant(conn, &role.id, object_key)?;
        let granted = grant.as_ref().map(|g| g.allows(capability)).unwrap_or(false);
        if granted {
            let scope = grant.as_ref().expect("granted implies a grant exists").record_scope;
            if best.map_or(true, |b| scope > b) {
                best = Some(scope);
                best_role_name = Some(role.name.clone());
            }
        }
        checks.push(AccessRoleCheck {
            role_name: role.name.clone(),
            matched_object_key: grant.as_ref().map(|g| g.object_key.clone()),
            capability_granted: granted,
            scope: grant.map(|g| g.record_scope),
        });
    }

    let record_summary = match record_id {
        Some(id) => entity_registry::resolve(conn, object_key, id)?.map(|r| r.display_name),
        None => None,
    };

    let decision = match best {
        None => AccessDecision {
            allowed: false,
            scope: None,
            matched_role: None,
            reason: format!("No Access Role held by this user grants '{}' on '{object_key}'", capability.as_str()),
        },
        Some(scope) => {
            let role_name = best_role_name.unwrap_or_default();
            match record_id {
                None => AccessDecision {
                    allowed: true,
                    scope: Some(scope),
                    matched_role: Some(role_name.clone()),
                    reason: format!("'{role_name}' grants '{}' on '{object_key}' ({} scope)", capability.as_str(), scope.as_str()),
                },
                Some(id) => {
                    let allowed = scope_allows_record(conn, scope, actor_user_id, object_key, id)?;
                    AccessDecision {
                        allowed,
                        scope: Some(scope),
                        matched_role: Some(role_name.clone()),
                        reason: if allowed {
                            format!("'{role_name}' grants '{}' on '{object_key}' within {} scope, which covers this record", capability.as_str(), scope.as_str())
                        } else {
                            format!("The broadest grant found ('{role_name}', {} scope) doesn't cover this specific record", scope.as_str())
                        },
                    }
                }
            }
        }
    };

    Ok(AccessInspectorResult {
        actor_user_id: actor_user_id.to_string(),
        object_key: object_key.to_string(),
        capability: capability.as_str().to_string(),
        record_id: record_id.map(|s| s.to_string()),
        record_summary,
        roles_checked: checks,
        decision,
    })
}

/// The enforcement entry point. `record_id: None` (e.g. a create) only
/// requires that some grant exists at all; `Some(id)` also requires that
/// grant's scope covers this specific record.
///
/// `actor_user_id: None` is treated as an unattributed, system-driven write
/// (a chat/agent tool call, a workflow-triggered create, `invoice_service::
/// refresh_overdue`'s own scheduled sweep) and is not checked at all - the
/// same convention `app_service::require_object_write_access` already
/// follows for every one of today's ordinary CRUD call sites. A capability
/// that genuinely must never run unattributed (Assign, via
/// `ownership_service::require_assign_capability`) enforces that itself
/// before ever calling this function, rather than this function guessing
/// which callers need a stricter rule than the rest.
pub fn require_capability(conn: &Connection, actor_user_id: Option<&str>, object_key: &str, capability: Capability, record_id: Option<&str>) -> AppResult<()> {
    let Some(actor_id) = actor_user_id else { return Ok(()) };
    let result = explain_access(conn, actor_id, object_key, capability, record_id)?;
    if result.decision.allowed {
        Ok(())
    } else {
        Err(AppError::Validation(result.decision.reason))
    }
}
