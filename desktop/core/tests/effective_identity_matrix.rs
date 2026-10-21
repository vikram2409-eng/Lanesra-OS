//! WP-0107 AC-1 and AC-2: the full authorisation matrix for admin-scoped
//! work, over real users and Access Control v1's `Administrator` role.
//! Every principal in a run must hold Administrator; none can lend it.
//!
//! Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-1, AC-2). Threats: T-03.

use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::principals::{AgentIdentityGrant, EffectiveIdentity, PrincipalType};
use lanesra_core::services::{user_service, workspace_service};

#[derive(Clone, Copy, Debug)]
enum Person {
    Administrator,
    NonAdministrator,
    Nobody,
}

#[derive(Clone, Copy, Debug)]
enum Identity {
    Administrator,
    NonAdministrator,
    Missing,
}

#[test]
fn every_principal_must_hold_administrator() {
    // Covers: REQ-GOV-002, REQ-GOV-003 (WP-0107 AC-1, AC-2)
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let (workspace, admin) = workspace_service::first_run_setup(
        &conn,
        &WorkspaceSetup {
            business_name: "Matrix Co".into(),
            legal_name: None,
            currency_code: "USD".into(),
            locale: "en-US".into(),
            timezone: "UTC".into(),
            default_tax_rate_bp: 0,
            admin_username: "admin".into(),
            admin_display_name: "Admin".into(),
            admin_password: "supersecretpassword".into(),
            load_sample_data: false,
        },
    )
    .unwrap();
    let ws = workspace.id;
    let rep = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "rep".into(),
            display_name: "Sales Rep".into(),
            password: "supersecretpassword".into(),
            roles: vec!["Sales".into()],
        },
        Some(&admin.id),
    )
    .unwrap()
    .id;

    use Identity as I;
    use Person as P;
    // (person who started the run, the run's agent identity, allowed?)
    let cases = [
        (P::Administrator, I::Missing, true),
        (P::Administrator, I::Administrator, true),
        (P::Administrator, I::NonAdministrator, false),
        (P::NonAdministrator, I::Missing, false),
        (P::NonAdministrator, I::Administrator, false),
        (P::NonAdministrator, I::NonAdministrator, false),
        (P::Nobody, I::Missing, false),
        (P::Nobody, I::Administrator, true),
        (P::Nobody, I::NonAdministrator, false),
    ];
    for (person, identity, allowed) in cases {
        let who = match person {
            P::Administrator => EffectiveIdentity::interactive(&admin.id),
            P::NonAdministrator => EffectiveIdentity::interactive(&rep),
            // Unattended, even with an Administrator's records behind it.
            P::Nobody => EffectiveIdentity::unattended(Some(&admin.id)),
        };
        let who = match identity {
            I::Missing => who,
            I::Administrator | I::NonAdministrator => who.with_agent_identity(AgentIdentityGrant {
                id: "identity-1".into(),
                workspace_id: ws.clone(),
                is_administrator: matches!(identity, I::Administrator),
            }),
        };
        assert_eq!(
            who.require_admin(&conn, &ws).is_ok(),
            allowed,
            "person={person:?} identity={identity:?}"
        );
    }
}

#[test]
fn a_run_acts_as_the_person_or_as_its_agent_identity() {
    // Covers: REQ-GOV-003 (WP-0107 AC-1)
    let interactive = EffectiveIdentity::interactive("u1");
    assert_eq!(interactive.principal_type(), PrincipalType::Human);
    assert_eq!(interactive.record_actor(), Some("u1"));
    assert!(!interactive.is_unattended());

    let unattended = EffectiveIdentity::unattended(None);
    assert_eq!(unattended.principal_type(), PrincipalType::AgentIdentity);
    assert_eq!(unattended.record_actor(), None);
    assert!(unattended.is_unattended());

    // `from_actor` keeps today's callers' meaning: a user id is the
    // person who started the run, `None` is no person at all.
    assert_eq!(
        EffectiveIdentity::from_actor(Some("u1")),
        EffectiveIdentity::interactive("u1")
    );
    assert_eq!(
        EffectiveIdentity::from_actor(None),
        EffectiveIdentity::unattended(None)
    );
}
