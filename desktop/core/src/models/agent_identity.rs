//! WP-0108: Agent Identities and their credentials (ADR-0006, ADR-0016 §1).
//! See `migrations/0081_agent_identities.sql`.

use serde::{Deserialize, Serialize};

pub const AGENT_IDENTITY_STATUSES: &[&str] = &["active", "disabled"];

/// An Agent Identity as the API and UI see it. It carries no credential
/// material: tokens and their hashes never leave the repository layer.
#[derive(Debug, Clone, Serialize)]
pub struct AgentIdentity {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub is_administrator: bool,
    pub access_role_ids: Vec<String>,
    pub status: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentIdentityInput {
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    #[serde(default)]
    pub is_administrator: bool,
    #[serde(default)]
    pub access_role_ids: Vec<String>,
}

/// A credential's metadata. The token hash is deliberately not a field.
#[derive(Debug, Clone, Serialize)]
pub struct AgentCredential {
    pub id: String,
    pub workspace_id: String,
    pub agent_identity_id: String,
    pub label: String,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    pub revoked_by: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

/// Returned once, when a credential is issued or rotated. `token` is the
/// only copy that will ever exist.
#[derive(Debug, Clone, Serialize)]
pub struct IssuedAgentCredential {
    pub credential: AgentCredential,
    pub token: String,
}
