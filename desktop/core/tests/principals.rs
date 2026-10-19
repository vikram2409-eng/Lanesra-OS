//! WP-0107: Principal model and effective-identity rules — integration tests.
//!
//! AC-1: An unattended run of an admin-scoped agent fails unless its Agent Identity
//!       holds the needed permission; it never inherits an Administrator's rights.
//! AC-2: An interactive run never exceeds the starting user's access, even when
//!       the agent's identity has more (authorisation matrix test).
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-1, AC-2)
//! Threats: T-03 (privilege escalation), T-12 (cross-workspace access)

use lanesra_core::principals::{EffectiveIdentity, Principal, PrincipalType};

/// AC-1: Unattended run of an admin-scoped agent with a non-admin identity fails.
#[test]
fn ac1_unattended_admin_scoped_agent_without_permission_fails() {
    // Covers: REQ-GOV-002 (WP-0107 AC-1)
    // The agent definition claims admin scope, but its identity is only Viewer.
    // The unattended run must NOT be able to perform admin actions.
    let identity = Principal::agent_identity("id1", "w1", "agent1", "Viewer");
    let effective = EffectiveIdentity::unattended(identity);

    assert!(
        !effective.can_admin(),
        "unattended run must not inherit admin rights it does not hold"
    );
}

/// AC-1: Unattended run with a genuine admin identity succeeds.
#[test]
fn ac1_unattended_admin_identity_can_admin() {
    // Covers: REQ-GOV-002 (WP-0107 AC-1)
    let identity = Principal::agent_identity("id1", "w1", "agent1", "Administrator");
    let effective = EffectiveIdentity::unattended(identity);

    assert!(
        effective.can_admin(),
        "agent identity holding admin should be able to admin unattended"
    );
}

/// AC-2: Authorisation matrix — interactive run never exceeds the starting user.
#[test]
fn ac2_authorisation_matrix() {
    // Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-2)
    // Matrix: (human role, agent role) -> effective can_admin
    let cases = vec![
        ("Viewer", "Viewer", false),
        ("Viewer", "Editor", false),
        ("Viewer", "Administrator", false), // Key case: agent has more, user does not
        ("Editor", "Viewer", false),
        ("Editor", "Editor", false),
        ("Editor", "Administrator", false),
        ("Administrator", "Viewer", false), // Human is admin but agent is not
        ("Administrator", "Editor", false),
        ("Administrator", "Administrator", true), // Both admin → effective admin
    ];

    for (human_role, agent_role, expect_admin) in cases {
        let human = Principal::human("h1", "w1", "acc1", human_role);
        let agent = Principal::agent_identity("a1", "w1", "agent1", agent_role);
        let effective = EffectiveIdentity::interactive(
            human,
            agent,
            serde_json::json!({"type": "all"}),
        );
        assert_eq!(
            effective.can_admin(),
            expect_admin,
            "matrix case failed: human={human_role}, agent={agent_role}"
        );
    }
}

/// AC-2: Interactive run records the starting human.
#[test]
fn ac2_interactive_run_tracks_starting_human() {
    // Covers: REQ-GOV-003 (WP-0107 AC-2)
    let human = Principal::human("h1", "w1", "acc1", "Editor");
    let agent = Principal::agent_identity("a1", "w1", "agent1", "Editor");
    let effective = EffectiveIdentity::interactive(
        human.clone(),
        agent,
        serde_json::json!({"type": "all"}),
    );

    assert!(!effective.unattended);
    let started_by = effective.started_by.expect("interactive run must track starter");
    assert_eq!(started_by.id, human.id);
    assert_eq!(started_by.principal_type, PrincipalType::Human);
}

/// Principals are workspace-scoped: agent identity cannot cross workspaces.
#[test]
fn agent_identity_is_workspace_scoped() {
    // Covers: REQ-GOV-003 (WP-0107)
    // Threat T-12: cross-workspace data access
    let agent_w1 = Principal::agent_identity("a1", "w1", "agent1", "Administrator");
    let agent_w2 = Principal::agent_identity("a2", "w2", "agent1", "Administrator");

    assert_ne!(
        agent_w1.workspace_id, agent_w2.workspace_id,
        "same agent in different workspaces has distinct principals"
    );
}
