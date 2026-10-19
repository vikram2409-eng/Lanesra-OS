//! WP-0107: Effective-identity matrix — extended authorisation tests.
//!
//! AC-2: An interactive run never exceeds the starting user's access, even when
//!       the agent's identity has more.
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-2)
//! Threats: T-03, T-12

use lanesra_core::principals::{EffectiveIdentity, Principal};

/// The effective role is always the least privileged of human and agent.
#[test]
fn effective_role_is_least_privileged() {
    // Covers: REQ-GOV-002 (WP-0107 AC-2)
    let cases = vec![
        ("Viewer", "Administrator", "Viewer"),
        ("Administrator", "Viewer", "Viewer"),
        ("Editor", "Manager", "Editor"),
        ("Manager", "Editor", "Editor"),
        ("Viewer", "Viewer", "Viewer"),
    ];

    for (human_role, agent_role, expect_role) in cases {
        let human = Principal::human("h1", "w1", "acc1", human_role);
        let agent = Principal::agent_identity("a1", "w1", "agent1", agent_role);
        let effective = EffectiveIdentity::interactive(
            human,
            agent,
            serde_json::json!({"type": "all"}),
        );
        assert_eq!(
            effective.acting_as.access_role, expect_role,
            "human={human_role}, agent={agent_role}"
        );
    }
}

/// Unattended runs have no starting human.
#[test]
fn unattended_run_has_no_starter() {
    // Covers: REQ-GOV-002 (WP-0107 AC-1)
    let agent = Principal::agent_identity("a1", "w1", "agent1", "Editor");
    let effective = EffectiveIdentity::unattended(agent);
    assert!(effective.unattended);
    assert!(effective.started_by.is_none());
}
