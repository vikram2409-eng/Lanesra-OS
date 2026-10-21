//! WP-0108: Agent Identities and Lanesra-issued agent credentials
//! (ADR-0006, ADR-0016 §1).
//!
//! An Agent Identity is the principal an unattended run acts as. Its rights
//! are Access Control v1's: Access Roles plus an explicit Administrator flag.
//! Managing identities and credentials is Administrator-only and audited.
//!
//! Credentials are short-lived tokens (`lanesra_agent_` + 48 hex characters
//! from the OS random source). Only `sha256(token)` is stored; the token is
//! returned once, at issue or rotation, and is never written to a log,
//! trace, audit row or prompt (AC-1). A revoked or expired credential, or
//! one whose identity is disabled, is refused on its next use (AC-2).
//!
//! Covers: REQ-GOV-002, REQ-GOV-029 (WP-0108). Threats: T-03, T-09.

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::agent_identity::{
    AgentCredential, AgentIdentity, AgentIdentityInput, IssuedAgentCredential,
};
use crate::principals::AgentIdentityGrant;
use crate::repositories::{access_role_repo, agent_identity_repo, audit_repo, user_repo};

/// Every agent token starts with this, so scans and redaction can find it.
pub const TOKEN_PREFIX: &str = "lanesra_agent_";
const TOKEN_RANDOM_BYTES: usize = 24;
pub const DEFAULT_LIFETIME_HOURS: i64 = 24;
/// Credentials are short-lived: at most one week.
pub const MAX_LIFETIME_HOURS: i64 = 24 * 7;

fn require_admin(conn: &Connection, actor: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor)
}

fn refused(message: &str) -> AppError {
    AppError::classified(ErrorClass::Policy, "policy.access_denied", message)
}

fn audit(
    conn: &Connection,
    workspace_id: &str,
    actor: Option<&str>,
    event_type: &str,
    identity_id: &str,
    summary: &str,
) -> AppResult<()> {
    audit_repo::record(
        conn,
        workspace_id,
        actor,
        event_type,
        Some("AgentIdentity"),
        Some(identity_id),
        summary,
        None,
    )?;
    Ok(())
}

fn validate_input(
    conn: &Connection,
    workspace_id: &str,
    input: &AgentIdentityInput,
) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation(
            "Agent Identity name is required".into(),
        ));
    }
    match user_repo::find_by_id(conn, &input.owner_user_id)? {
        Some(owner) if owner.workspace_id == workspace_id => {}
        _ => {
            return Err(AppError::Validation(
                "The owner must be a user in this workspace".into(),
            ))
        }
    }
    for role_id in &input.access_role_ids {
        match access_role_repo::get(conn, role_id)? {
            Some(role) if role.workspace_id == workspace_id => {}
            _ => {
                return Err(AppError::Validation(format!(
                    "Access Role '{role_id}' does not exist in this workspace"
                )))
            }
        }
    }
    Ok(())
}

fn get_in_workspace(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<AgentIdentity> {
    match agent_identity_repo::get(conn, id)? {
        Some(identity) if identity.workspace_id == workspace_id => Ok(identity),
        _ => Err(AppError::NotFound("Agent Identity".into())),
    }
}

pub fn list(
    conn: &Connection,
    workspace_id: &str,
    actor: Option<&str>,
) -> AppResult<Vec<AgentIdentity>> {
    require_admin(conn, actor)?;
    Ok(agent_identity_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &AgentIdentityInput,
    actor: Option<&str>,
) -> AppResult<AgentIdentity> {
    require_admin(conn, actor)?;
    validate_input(conn, workspace_id, input)?;
    let identity = agent_identity_repo::insert(conn, &new_uuid(), workspace_id, input, actor)?;
    audit(
        conn,
        workspace_id,
        actor,
        "agent_identity_created",
        &identity.id,
        &format!("Created Agent Identity '{}'", identity.name),
    )?;
    Ok(identity)
}

pub fn update(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    input: &AgentIdentityInput,
    actor: Option<&str>,
) -> AppResult<AgentIdentity> {
    require_admin(conn, actor)?;
    get_in_workspace(conn, workspace_id, id)?;
    validate_input(conn, workspace_id, input)?;
    let identity = agent_identity_repo::update(conn, id, input, actor)?;
    audit(
        conn,
        workspace_id,
        actor,
        "agent_identity_updated",
        id,
        &format!("Updated Agent Identity '{}'", identity.name),
    )?;
    Ok(identity)
}

pub fn set_active(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    active: bool,
    actor: Option<&str>,
) -> AppResult<AgentIdentity> {
    require_admin(conn, actor)?;
    let identity = get_in_workspace(conn, workspace_id, id)?;
    let status = if active { "active" } else { "disabled" };
    agent_identity_repo::set_status(conn, id, status, actor)?;
    audit(
        conn,
        workspace_id,
        actor,
        if active {
            "agent_identity_enabled"
        } else {
            "agent_identity_disabled"
        },
        id,
        &format!("Agent Identity '{}' is now {status}", identity.name),
    )?;
    get_in_workspace(conn, workspace_id, id)
}

pub fn delete(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    actor: Option<&str>,
) -> AppResult<()> {
    require_admin(conn, actor)?;
    let identity = get_in_workspace(conn, workspace_id, id)?;
    agent_identity_repo::delete(conn, id)?;
    audit(
        conn,
        workspace_id,
        actor,
        "agent_identity_deleted",
        id,
        &format!("Deleted Agent Identity '{}'", identity.name),
    )
}

/// What a run acting as this identity may do (`principals.rs`).
pub fn grant_for_run(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
) -> AppResult<AgentIdentityGrant> {
    let identity = get_in_workspace(conn, workspace_id, id)?;
    if identity.status != "active" {
        return Err(refused("This Agent Identity is disabled"));
    }
    Ok(AgentIdentityGrant {
        id: identity.id,
        workspace_id: identity.workspace_id,
        is_administrator: identity.is_administrator,
    })
}

pub fn list_credentials(
    conn: &Connection,
    workspace_id: &str,
    identity_id: &str,
    actor: Option<&str>,
) -> AppResult<Vec<AgentCredential>> {
    require_admin(conn, actor)?;
    get_in_workspace(conn, workspace_id, identity_id)?;
    Ok(agent_identity_repo::list_credentials(conn, identity_id)?)
}

fn new_token() -> String {
    format!(
        "{TOKEN_PREFIX}{}",
        super::secret_service::generate_random_secret(TOKEN_RANDOM_BYTES)
    )
}

/// Issues a credential and returns its token, the only copy there will be.
pub fn issue_credential(
    conn: &Connection,
    workspace_id: &str,
    identity_id: &str,
    label: &str,
    lifetime_hours: Option<i64>,
    actor: Option<&str>,
) -> AppResult<IssuedAgentCredential> {
    require_admin(conn, actor)?;
    let identity = get_in_workspace(conn, workspace_id, identity_id)?;
    if identity.status != "active" {
        return Err(AppError::Validation(
            "A disabled Agent Identity cannot be given a credential".into(),
        ));
    }
    if label.trim().is_empty() {
        return Err(AppError::Validation(
            "A credential label is required".into(),
        ));
    }
    let hours = lifetime_hours.unwrap_or(DEFAULT_LIFETIME_HOURS);
    if !(1..=MAX_LIFETIME_HOURS).contains(&hours) {
        return Err(AppError::Validation(format!(
            "A credential lasts between 1 and {MAX_LIFETIME_HOURS} hours"
        )));
    }
    let token = new_token();
    let expires_at = (Utc::now() + Duration::hours(hours)).to_rfc3339();
    let credential = agent_identity_repo::insert_credential(
        conn,
        &new_uuid(),
        workspace_id,
        identity_id,
        label.trim(),
        &super::secret_service::hash_api_secret(&token),
        &expires_at,
        actor,
    )?;
    audit(
        conn,
        workspace_id,
        actor,
        "agent_credential_issued",
        identity_id,
        &format!(
            "Issued credential '{}' for Agent Identity '{}', expires {expires_at}",
            credential.label, identity.name
        ),
    )?;
    Ok(IssuedAgentCredential { credential, token })
}

pub fn revoke_credential(
    conn: &Connection,
    workspace_id: &str,
    credential_id: &str,
    actor: Option<&str>,
) -> AppResult<()> {
    require_admin(conn, actor)?;
    let credential = match agent_identity_repo::get_credential(conn, credential_id)? {
        Some(c) if c.workspace_id == workspace_id => c,
        _ => return Err(AppError::NotFound("Agent credential".into())),
    };
    agent_identity_repo::revoke_credential(conn, credential_id, actor)?;
    audit(
        conn,
        workspace_id,
        actor,
        "agent_credential_revoked",
        &credential.agent_identity_id,
        &format!("Revoked credential '{}'", credential.label),
    )
}

/// Revokes a credential and issues its replacement with the same label.
pub fn rotate_credential(
    conn: &Connection,
    workspace_id: &str,
    credential_id: &str,
    lifetime_hours: Option<i64>,
    actor: Option<&str>,
) -> AppResult<IssuedAgentCredential> {
    require_admin(conn, actor)?;
    let old = match agent_identity_repo::get_credential(conn, credential_id)? {
        Some(c) if c.workspace_id == workspace_id => c,
        _ => return Err(AppError::NotFound("Agent credential".into())),
    };
    revoke_credential(conn, workspace_id, credential_id, actor)?;
    issue_credential(
        conn,
        workspace_id,
        &old.agent_identity_id,
        &old.label,
        lifetime_hours,
        actor,
    )
}

/// Checks a presented token and returns what the run may do as that
/// identity. Every refusal is the same `policy.access_denied` class.
pub fn authenticate(
    conn: &Connection,
    workspace_id: &str,
    token: &str,
) -> AppResult<AgentIdentityGrant> {
    let hash = super::secret_service::hash_api_secret(token);
    let credential = match agent_identity_repo::find_credential_by_hash(conn, &hash)? {
        Some(c) if c.workspace_id == workspace_id => c,
        _ => return Err(refused("This agent credential is not valid")),
    };
    if credential.revoked_at.is_some() {
        return Err(refused("This agent credential has been revoked"));
    }
    let expires_at = DateTime::parse_from_rfc3339(&credential.expires_at).map_err(|e| {
        AppError::classified(
            ErrorClass::Internal,
            "internal",
            format!("Agent credential has an unreadable expiry: {e}"),
        )
    })?;
    if Utc::now() >= expires_at {
        return Err(refused("This agent credential has expired"));
    }
    let grant = grant_for_run(conn, workspace_id, &credential.agent_identity_id)?;
    agent_identity_repo::touch_credential(conn, &credential.id)?;
    Ok(grant)
}

/// Replaces anything shaped like an agent token with `[REDACTED]`, for
/// text that is about to be logged or traced.
pub fn redact_credential_material(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(TOKEN_PREFIX) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + TOKEN_PREFIX.len()..];
        let hex_len = after
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(after.len());
        if hex_len > 0 {
            out.push_str("[REDACTED]");
        } else {
            out.push_str(TOKEN_PREFIX);
        }
        rest = &after[hex_len..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_recognisable() {
        let a = new_token();
        let b = new_token();
        assert_ne!(a, b);
        assert!(a.starts_with(TOKEN_PREFIX));
        assert_eq!(a.len(), TOKEN_PREFIX.len() + TOKEN_RANDOM_BYTES * 2);
    }

    #[test]
    fn redaction_removes_every_token_and_keeps_other_text() {
        let a = new_token();
        let b = new_token();
        let text = format!("first {a}, then {b}. A bare {TOKEN_PREFIX} stays.");
        let redacted = redact_credential_material(&text);
        assert!(
            !redacted.contains(&a) && !redacted.contains(&b),
            "{redacted}"
        );
        assert_eq!(
            redacted,
            format!("first [REDACTED], then [REDACTED]. A bare {TOKEN_PREFIX} stays.")
        );
        assert_eq!(
            redact_credential_material("no tokens here"),
            "no tokens here"
        );
    }
}
