//! WP-0108: Agent Identity credentials — issuance, validation, revocation.
//!
//! Lanesra-issued short-lived tokens bound to one Agent Identity and a scope.
//! - Tokens are shown only once at issuance; only the hash is stored.
//! - A revoked or expired credential is refused immediately.
//! - Credential material never appears in prompts, traces, logs, or API responses.
//!
//! Covers: REQ-GOV-002, REQ-GOV-029 (WP-0108 AC-1, AC-2)
//! Threats: T-03 (credential theft), T-09 (credential leakage to models)

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult, ErrorClass};

/// Token lifetime: 24 hours by default (short-lived).
const DEFAULT_TOKEN_LIFETIME_SECS: i64 = 24 * 60 * 60;

/// A validated credential: what the caller learns, never the token itself.
#[derive(Debug, Clone)]
pub struct ValidatedCredential {
    pub credential_id: String,
    pub agent_identity_id: String,
    pub workspace_id: String,
    pub scope: serde_json::Value,
}

/// Hash a token with SHA-256 for storage/lookup.
/// The plaintext token is never stored.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a new random token.
/// Uses system time + counter for uniqueness (no rand crate in core).
pub fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // LCG for additional mixing (same approach as retry.rs)
    let mut seed = nanos ^ 0x9E3779B97F4A7C15;
    seed ^= seed >> 30;
    seed = seed.wrapping_mul(0xBF58476D1CE4E5B9);
    seed ^= seed >> 27;
    seed = seed.wrapping_mul(0x94D049BB133111EB);
    seed ^= seed >> 31;
    format!(
        "lanesra_{:016x}{:016x}",
        nanos & 0xFFFF_FFFF_FFFF_FFFF,
        seed & 0xFFFF_FFFF_FFFF_FFFF
    )
}

/// Issue a new credential for an agent identity.
/// Returns the plaintext token ONCE — it is never stored.
///
/// AC-1: The token must never be logged, traced, or included in prompts.
pub fn issue_credential(
    conn: &Connection,
    agent_identity_id: &str,
    workspace_id: &str,
    label: &str,
    scope: &serde_json::Value,
    created_by: &str,
) -> AppResult<String> {
    // Verify the identity exists and is active
    let status: String = conn
        .query_row(
            "SELECT status FROM ai_agent_identities WHERE id = ?1 AND workspace_id = ?2",
            rusqlite::params![agent_identity_id, workspace_id],
            |row| row.get(0),
        )
        .map_err(|_| {
            AppError::Validation(format!("Agent identity {agent_identity_id} not found"))
        })?;

    if status != "active" {
        return Err(AppError::Validation(
            "Cannot issue credential for disabled identity".to_string(),
        ));
    }

    let token = generate_token();
    let token_hash = hash_token(&token);
    let credential_id = new_uuid();

    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(DEFAULT_TOKEN_LIFETIME_SECS);

    conn.execute(
        "INSERT INTO ai_agent_credentials
         (id, agent_identity_id, workspace_id, label, token_hash, scope, expires_at, created_at, created_by_principal_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            credential_id,
            agent_identity_id,
            workspace_id,
            label,
            token_hash,
            scope.to_string(),
            expires_at.to_rfc3339(),
            now.to_rfc3339(),
            created_by,
        ],
    )
    .map_err(|e| AppError::classified(ErrorClass::Internal, "internal.database", format!("Failed to issue credential: {e}")))?;

    // Return the plaintext token ONCE. The caller must display it to the admin
    // and never store it. Only the hash is in the database.
    Ok(token)
}

/// Validate a credential token.
/// Returns the validated credential (without token material) or an error.
///
/// AC-2: Revoked or expired credentials are refused immediately.
pub fn validate_credential(conn: &Connection, token: &str) -> AppResult<ValidatedCredential> {
    let token_hash = hash_token(token);

    let row: Option<(String, String, String, String, Option<String>)> = conn
        .query_row(
            "SELECT id, agent_identity_id, workspace_id, scope, revoked_at
             FROM ai_agent_credentials
             WHERE token_hash = ?1",
            rusqlite::params![token_hash],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .ok();

    let (credential_id, agent_identity_id, workspace_id, scope_str, revoked_at) =
        row.ok_or_else(|| AppError::Validation("Invalid credential".to_string()))?;

    // AC-2: Revoked credentials are refused immediately.
    if revoked_at.is_some() {
        return Err(AppError::Validation(
            "Credential has been revoked".to_string(),
        ));
    }

    // AC-2: Expired credentials are refused immediately.
    let expires_at: String = conn
        .query_row(
            "SELECT expires_at FROM ai_agent_credentials WHERE id = ?1",
            rusqlite::params![credential_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::classified(ErrorClass::Internal, "internal.database", format!("Failed to check expiry: {e}")))?;

    let expiry = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|e| AppError::classified(ErrorClass::Internal, "internal.database", format!("Invalid expiry format: {e}")))?;
    if chrono::Utc::now() > expiry.with_timezone(&chrono::Utc) {
        return Err(AppError::Validation("Credential has expired".to_string()));
    }

    // Check the identity is still active
    let identity_status: String = conn
        .query_row(
            "SELECT status FROM ai_agent_identities WHERE id = ?1",
            rusqlite::params![agent_identity_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::Validation("Agent identity not found".to_string()))?;
    if identity_status != "active" {
        return Err(AppError::Validation(
            "Agent identity is disabled".to_string(),
        ));
    }

    // Update last_used_at (fire and forget — not critical)
    let _ = conn.execute(
        "UPDATE ai_agent_credentials SET last_used_at = ?1 WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().to_rfc3339(), credential_id],
    );

    let scope: serde_json::Value =
        serde_json::from_str(&scope_str).unwrap_or(serde_json::json!({"type": "all"}));

    Ok(ValidatedCredential {
        credential_id,
        agent_identity_id,
        workspace_id,
        scope,
    })
}

/// Revoke a credential immediately.
/// After revocation, validate_credential refuses it.
pub fn revoke_credential(
    conn: &Connection,
    credential_id: &str,
    revoked_by: &str,
) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let updated = conn
        .execute(
            "UPDATE ai_agent_credentials
             SET revoked_at = ?1, revoked_by_principal_id = ?2
             WHERE id = ?3 AND revoked_at IS NULL",
            rusqlite::params![now, revoked_by, credential_id],
        )
        .map_err(|e| AppError::classified(ErrorClass::Internal, "internal.database", format!("Failed to revoke credential: {e}")))?;

    if updated == 0 {
        return Err(AppError::Validation(
            "Credential not found or already revoked".to_string(),
        ));
    }
    Ok(())
}

/// Redact credential material from a string (for logs/traces).
/// Replaces anything looking like a Lanesra token with [REDACTED].
///
/// AC-1: Credentials must never appear in prompts, traces, logs, or API responses.
pub fn redact_credential_material(text: &str) -> String {
    // Lanesra tokens start with "lanesra_" followed by 32 hex chars
    let mut result = text.to_string();
    let mut start = 0;
    while let Some(pos) = result[start..].find("lanesra_") {
        let abs_pos = start + pos;
        let end = (abs_pos + 8 + 32).min(result.len());
        // Check if the next 32 chars are hex
        let candidate = &result[abs_pos..end];
        if candidate.len() == 40 && candidate[8..].chars().all(|c| c.is_ascii_hexdigit()) {
            result.replace_range(abs_pos..end, "[REDACTED]");
            start = abs_pos + 10; // Length of "[REDACTED]"
        } else {
            start = abs_pos + 8;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_hash_is_deterministic() {
        let h1 = hash_token("test_token_123");
        let h2 = hash_token("test_token_123");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex
    }

    #[test]
    fn different_tokens_have_different_hashes() {
        let h1 = hash_token("token_a");
        let h2 = hash_token("token_b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn generated_tokens_are_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
        assert!(t1.starts_with("lanesra_"));
    }

    #[test]
    fn redact_removes_token_material() {
        let text = "Error with token lanesra_1234567890abcdef1234567890abcdef in request";
        let redacted = redact_credential_material(text);
        assert!(!redacted.contains("lanesra_1234567890abcdef1234567890abcdef"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redact_leaves_normal_text_alone() {
        let text = "Normal log message without tokens";
        assert_eq!(redacted_credential_material(text), text);
    }
}
