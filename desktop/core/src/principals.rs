//! WP-0107: Principal model and effective-identity rules.
//!
//! Defines the three principal types (human, agent_identity, api_client) and the
//! effective-identity rules that determine what permissions an agent run operates with.
//!
//! Per ADR-0016 §1:
//! - Human principals live in the workspace database, referring to the account by id
//! - Agent Identity and API client principals belong to exactly one workspace
//! - An agent never sees more than its user (REQ-GOV-002)
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107)
//! Threats: T-03 (privilege escalation), T-12 (cross-workspace access)

use serde::{Deserialize, Serialize};

/// The three principal types in LanesraOS.
///
/// Per ADR-0016 §1 and ADR-0006:
/// - `Human`: a person who can sign in (via password, OIDC, or SAML)
/// - `AgentIdentity`: an AI agent's identity, scoped to one workspace
/// - `ApiClient`: a programmatic client, scoped to one workspace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalType {
    Human,
    AgentIdentity,
    ApiClient,
}

/// A principal: an entity that can act in the system.
///
/// Every principal has:
/// - An Access Role (e.g. Viewer, Editor, Administrator)
/// - A Record Scope (which records it can see)
///
/// Per ADR-0016 §1, Agent Identity and API client principals belong to exactly
/// one workspace and never cross it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Unique identifier for this principal
    pub id: String,
    /// The type of principal
    pub principal_type: PrincipalType,
    /// The workspace this principal belongs to
    pub workspace_id: String,
    /// Access Role: Viewer, Editor, Administrator, etc.
    pub access_role: String,
    /// Record Scope: JSON describing which records are visible
    pub record_scope: serde_json::Value,
    /// For Human principals: the account id in the control database
    /// For AgentIdentity: the agent id
    /// For ApiClient: the client id
    pub subject_id: String,
}

impl Principal {
    /// Create a human principal.
    pub fn human(id: &str, workspace_id: &str, account_id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            principal_type: PrincipalType::Human,
            workspace_id: workspace_id.to_string(),
            access_role: role.to_string(),
            record_scope: serde_json::json!({"type": "all"}),
            subject_id: account_id.to_string(),
        }
    }

    /// Create an agent identity principal.
    pub fn agent_identity(id: &str, workspace_id: &str, agent_id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            principal_type: PrincipalType::AgentIdentity,
            workspace_id: workspace_id.to_string(),
            access_role: role.to_string(),
            record_scope: serde_json::json!({"type": "all"}),
            subject_id: agent_id.to_string(),
        }
    }

    /// Create an API client principal.
    pub fn api_client(id: &str, workspace_id: &str, client_id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            principal_type: PrincipalType::ApiClient,
            workspace_id: workspace_id.to_string(),
            access_role: role.to_string(),
            record_scope: serde_json::json!({"type": "all"}),
            subject_id: client_id.to_string(),
        }
    }

    /// Check if this principal has administrator privileges.
    pub fn is_admin(&self) -> bool {
        self.access_role.eq_ignore_ascii_case("administrator")
            || self.access_role.eq_ignore_ascii_case("admin")
    }
}

/// The effective identity for an agent run.
///
/// Per WP-0107 and REQ-GOV-002 (an agent never sees more than its user):
///
/// - **Interactive run** (triggered by a human): acts as the intersection of
///   (starting human's permissions ∩ agent's ontology scope ∩ agent identity's permissions).
///   The run never exceeds the starting user's access, even if the agent's identity has more.
///
/// - **Unattended run** (schedule/webhook/trigger, no human actor): acts as its
///   Agent Identity only. It never inherits an Administrator's rights.
///   An admin-scoped agent cannot run unattended unless its Agent Identity
///   genuinely holds the needed permission.
#[derive(Debug, Clone)]
pub struct EffectiveIdentity {
    /// The principal the run acts as
    pub acting_as: Principal,
    /// For interactive runs: the human who started the run
    pub started_by: Option<Principal>,
    /// Whether this is an unattended run (no human actor)
    pub unattended: bool,
}

impl EffectiveIdentity {
    /// Compute the effective identity for an interactive run.
    ///
    /// The run acts with the intersection of:
    /// 1. The starting human's permissions
    /// 2. The agent's ontology scope
    /// 3. The agent identity's permissions
    ///
    /// The result never exceeds the starting user's access.
    pub fn interactive(
        human: Principal,
        agent_identity: Principal,
        agent_ontology_scope: serde_json::Value,
    ) -> Self {
        assert_eq!(
            human.principal_type,
            PrincipalType::Human,
            "interactive run must start with a human"
        );
        assert_eq!(
            human.workspace_id, agent_identity.workspace_id,
            "principals must be in the same workspace"
        );

        // The effective role is the LEAST privileged of human and agent identity.
        // This ensures the run never exceeds the starting user's access.
        let effective_role = least_privileged_role(&human.access_role, &agent_identity.access_role);

        // The effective scope is the intersection of all three scopes.
        let effective_scope = intersect_scopes(&[
            &human.record_scope,
            &agent_ontology_scope,
            &agent_identity.record_scope,
        ]);

        let acting_as = Principal {
            id: format!("effective:{}", human.id),
            principal_type: PrincipalType::Human, // Acts on behalf of the human
            workspace_id: human.workspace_id.clone(),
            access_role: effective_role,
            record_scope: effective_scope,
            subject_id: human.subject_id.clone(),
        };

        Self {
            acting_as,
            started_by: Some(human),
            unattended: false,
        }
    }

    /// Compute the effective identity for an unattended run.
    ///
    /// The run acts as its Agent Identity only. It never inherits
    /// an Administrator's rights from anywhere.
    pub fn unattended(agent_identity: Principal) -> Self {
        assert_eq!(
            agent_identity.principal_type,
            PrincipalType::AgentIdentity,
            "unattended run must use an agent identity"
        );

        Self {
            acting_as: agent_identity.clone(),
            started_by: None,
            unattended: true,
        }
    }

    /// Check if this effective identity can perform an admin-only action.
    ///
    /// For unattended runs: only if the Agent Identity itself is an admin.
    /// For interactive runs: only if the effective (intersected) role is admin,
    /// which requires BOTH the human and the agent identity to be admins.
    pub fn can_admin(&self) -> bool {
        self.acting_as.is_admin()
    }
}

/// Role hierarchy from least to most privileged.
fn role_rank(role: &str) -> u8 {
    match role.to_ascii_lowercase().as_str() {
        "viewer" => 1,
        "editor" => 2,
        "manager" => 3,
        "administrator" | "admin" => 4,
        _ => 0, // Unknown roles are treated as least privileged
    }
}

/// Return the least privileged of two roles.
fn least_privileged_role(a: &str, b: &str) -> String {
    if role_rank(a) <= role_rank(b) {
        a.to_string()
    } else {
        b.to_string()
    }
}

/// Intersect multiple record scopes.
/// For now, the intersection is the most restrictive scope.
/// A full implementation would compute the actual set intersection.
fn intersect_scopes(scopes: &[&serde_json::Value]) -> serde_json::Value {
    // If any scope is restricted, the intersection is restricted.
    // This is a conservative approximation.
    for scope in scopes {
        if scope.get("type").and_then(|t| t.as_str()) != Some("all") {
            return (*scope).clone();
        }
    }
    serde_json::json!({"type": "all"})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principal_types_are_distinct() {
        let human = Principal::human("h1", "w1", "acc1", "Editor");
        let agent = Principal::agent_identity("a1", "w1", "agent1", "Viewer");
        let client = Principal::api_client("c1", "w1", "client1", "Viewer");

        assert_eq!(human.principal_type, PrincipalType::Human);
        assert_eq!(agent.principal_type, PrincipalType::AgentIdentity);
        assert_eq!(client.principal_type, PrincipalType::ApiClient);
    }

    #[test]
    fn unattended_run_never_inherits_admin() {
        // An agent identity with Viewer role cannot admin, even if
        // the agent definition claims admin scope.
        let agent = Principal::agent_identity("a1", "w1", "agent1", "Viewer");
        let effective = EffectiveIdentity::unattended(agent);
        assert!(effective.unattended);
        assert!(!effective.can_admin());
    }

    #[test]
    fn unattended_admin_agent_can_admin() {
        // An agent identity that genuinely holds admin CAN admin unattended.
        let agent = Principal::agent_identity("a1", "w1", "agent1", "Administrator");
        let effective = EffectiveIdentity::unattended(agent);
        assert!(effective.can_admin());
    }

    #[test]
    fn interactive_run_never_exceeds_user() {
        // Human is Viewer, agent identity is Administrator.
        // The effective identity must be Viewer (least privileged).
        let human = Principal::human("h1", "w1", "acc1", "Viewer");
        let agent = Principal::agent_identity("a1", "w1", "agent1", "Administrator");
        let effective =
            EffectiveIdentity::interactive(human, agent, serde_json::json!({"type": "all"}));
        assert!(!effective.unattended);
        assert_eq!(effective.acting_as.access_role, "Viewer");
        assert!(!effective.can_admin());
    }

    #[test]
    fn interactive_run_with_admin_human_and_admin_agent() {
        // Both human and agent are admin → effective is admin.
        let human = Principal::human("h1", "w1", "acc1", "Administrator");
        let agent = Principal::agent_identity("a1", "w1", "agent1", "Administrator");
        let effective =
            EffectiveIdentity::interactive(human, agent, serde_json::json!({"type": "all"}));
        assert!(effective.can_admin());
    }
}
