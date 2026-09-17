//! Voice-First Mode, PR 1: resolves Voice's own capability+risk policy for
//! one user (spec §5/§5.1) - a second, narrower gate that composes with,
//! and can only narrow, `access_service::require_capability` (spec: "Voice
//! permissions can only narrow or govern existing access; they cannot grant
//! access the user does not already possess"). Every Voice capability check
//! anywhere in this feature calls `voice_capability_allows` *and* the real
//! `access_service::require_capability` - both must pass.

use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::voice::{MaxActionLevel, VoicePolicyBinding, VoicePolicyBindingInput};
use crate::repositories::{user_access_role_repo, user_repo, voice_repo};

/// Same Administrator-only check every other admin-gated service defines
/// for itself (`access_role_service`/`work_team_service`'s own private
/// copies) - editing Voice Governance policy is exactly that kind of
/// action, not a Voice capability itself.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage Voice Governance policy".into()));
    }
    Ok(())
}

/// One user's fully-resolved Voice policy - not a stored row itself, the
/// union of the workspace default plus every Access Role binding that
/// applies to them, exactly the "fold to the strongest/broadest" precedent
/// `access_service::capability_grant_for` already uses for object
/// capabilities. `processing_boundary` is the one field resolved the
/// opposite way - see `strictest_boundary`'s own doc comment for why a
/// privacy ceiling must take the *most* restrictive applicable value, not
/// the most permissive.
#[derive(Debug, Clone)]
pub struct EffectivePolicy {
    pub can_use_voice: bool,
    pub can_search: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_act: bool,
    pub can_bulk_act: bool,
    pub can_external_act: bool,
    pub can_use_agents: bool,
    pub max_action_level: MaxActionLevel,
    pub max_unlock_minutes: i64,
    pub processing_boundary: String,
}

fn boundary_rank(b: &str) -> u8 {
    match b {
        "local_only" => 0,
        "approved_private" => 1,
        _ => 2, // "cloud" - least restrictive
    }
}

/// The strictest (lowest-rank) `processing_boundary` among the applicable
/// bindings wins - a privacy/compliance ceiling should never be silently
/// loosened just because one of a user's several Access Roles happens to
/// carry a laxer default. Unlike the capability booleans (where "can this
/// role additionally do X" should broaden), a processing boundary answers
/// "is this user ever allowed to send speech somewhere less private", which
/// must be the most cautious applicable answer.
fn strictest_boundary(a: &str, b: &str) -> String {
    if boundary_rank(a) <= boundary_rank(b) { a.to_string() } else { b.to_string() }
}

fn fold(acc: EffectivePolicy, b: &VoicePolicyBinding) -> EffectivePolicy {
    EffectivePolicy {
        can_use_voice: acc.can_use_voice || b.can_use_voice,
        can_search: acc.can_search || b.can_search,
        can_create: acc.can_create || b.can_create,
        can_update: acc.can_update || b.can_update,
        can_act: acc.can_act || b.can_act,
        can_bulk_act: acc.can_bulk_act || b.can_bulk_act,
        can_external_act: acc.can_external_act || b.can_external_act,
        can_use_agents: acc.can_use_agents || b.can_use_agents,
        max_action_level: acc.max_action_level.max(b.max_action_level),
        max_unlock_minutes: acc.max_unlock_minutes.max(b.max_unlock_minutes),
        processing_boundary: strictest_boundary(&acc.processing_boundary, &b.processing_boundary),
    }
}

/// Resolves the effective policy for one user: starts from the workspace
/// default binding (`access_role_id IS NULL`, always present after the
/// 0057 migration's bootstrap insert), then folds in every Access Role
/// binding for a role the user actually holds.
pub fn effective_policy(conn: &Connection, user_id: &str) -> AppResult<EffectivePolicy> {
    let Some(user) = user_repo::find_by_id(conn, user_id)? else { return Ok(zero_policy()) };

    let default_binding = voice_repo::get_default_policy_binding(conn, &user.workspace_id)?;
    let mut policy = default_binding.map(binding_to_policy).unwrap_or_else(zero_policy);

    let roles = user_access_role_repo::list_for_user(conn, user_id)?;
    for role in &roles {
        if let Some(binding) = voice_repo::get_policy_binding_for_role(conn, &user.workspace_id, &role.id)? {
            policy = fold(policy, &binding);
        }
    }
    Ok(policy)
}

fn binding_to_policy(b: VoicePolicyBinding) -> EffectivePolicy {
    EffectivePolicy {
        can_use_voice: b.can_use_voice,
        can_search: b.can_search,
        can_create: b.can_create,
        can_update: b.can_update,
        can_act: b.can_act,
        can_bulk_act: b.can_bulk_act,
        can_external_act: b.can_external_act,
        can_use_agents: b.can_use_agents,
        max_action_level: b.max_action_level,
        max_unlock_minutes: b.max_unlock_minutes,
        processing_boundary: b.processing_boundary,
    }
}

fn zero_policy() -> EffectivePolicy {
    EffectivePolicy {
        can_use_voice: false,
        can_search: false,
        can_create: false,
        can_update: false,
        can_act: false,
        can_bulk_act: false,
        can_external_act: false,
        can_use_agents: false,
        max_action_level: MaxActionLevel::AskOnly,
        max_unlock_minutes: 15,
        processing_boundary: "cloud".to_string(),
    }
}

pub fn voice_capability_allows(conn: &Connection, user_id: &str, capability: &str) -> AppResult<bool> {
    let policy = effective_policy(conn, user_id)?;
    Ok(match capability {
        "use_voice" => policy.can_use_voice,
        "search" => policy.can_search,
        "create" => policy.can_create,
        "update" => policy.can_update,
        "act" => policy.can_act,
        "bulk_act" => policy.can_bulk_act,
        "external_act" => policy.can_external_act,
        "use_agents" => policy.can_use_agents,
        _ => false,
    })
}

pub fn list_policy_bindings(conn: &Connection, workspace_id: &str) -> AppResult<Vec<VoicePolicyBinding>> {
    Ok(voice_repo::list_policy_bindings(conn, workspace_id)?)
}

/// Called once from `workspace_service::first_run_setup` for every new
/// workspace - migration 0057's own bootstrap insert only reaches
/// workspaces that already existed when that migration ran, so a workspace
/// created afterwards needs this instead to get a default policy binding at
/// all (`effective_policy` would otherwise fall back to `zero_policy()`,
/// which is safe but leaves Voice's own policy table with no row to edit).
pub fn ensure_workspace_default(conn: &Connection, workspace_id: &str) -> AppResult<()> {
    Ok(voice_repo::ensure_default_policy_binding(conn, workspace_id)?)
}

/// Companion to `access_role_service::ensure_system_roles_seeded`, called
/// right after it lazily creates the "Full Access" system role for a
/// workspace - gives that role full Voice capabilities immediately, the
/// same values migration 0057's own bootstrap insert already gives "Full
/// Access" for a workspace old enough to have had that role at migration
/// time (see that migration's own comment). Idempotent, same "no-op if
/// already present" shape as every other lazy-bootstrap helper here.
pub fn ensure_full_access_binding(conn: &Connection, workspace_id: &str, access_role_id: &str) -> AppResult<()> {
    if voice_repo::get_policy_binding_for_role(conn, workspace_id, access_role_id)?.is_some() {
        return Ok(());
    }
    let input = VoicePolicyBindingInput {
        access_role_id: Some(access_role_id.to_string()),
        can_use_voice: true,
        can_search: true,
        can_create: true,
        can_update: true,
        can_act: true,
        can_bulk_act: true,
        can_external_act: true,
        can_use_agents: true,
        max_action_level: "act_with_confirmation".to_string(),
        processing_boundary: "cloud".to_string(),
        max_unlock_minutes: 30,
    };
    voice_repo::upsert_policy_binding(conn, &new_uuid(), workspace_id, &input)?;
    Ok(())
}

/// Editing Voice Governance policy is Administrator-only, same as editing
/// Access Roles themselves (`access_role_service`'s own gate) - a policy
/// that could grant itself broader Voice access is exactly the kind of
/// circular-bootstrap risk that seam already avoids for capability grants.
pub fn upsert_policy_binding(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>, input: &VoicePolicyBindingInput) -> AppResult<VoicePolicyBinding> {
    require_admin(conn, actor_user_id)?;
    let id = crate::domain::ids::new_uuid();
    Ok(voice_repo::upsert_policy_binding(conn, &id, workspace_id, input)?)
}
