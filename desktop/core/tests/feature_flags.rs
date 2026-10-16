//! Workspace feature flags (WP-0010, ADR-0012).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::feature_flag::{Edition, FeatureFlagOverrideInput};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::audit_repo;
use lanesra_core::services::feature_flag_service::{self, PLAYGROUND_SHELL};
use lanesra_core::services::{user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Flag Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn input(enabled: Option<bool>) -> FeatureFlagOverrideInput {
    FeatureFlagOverrideInput {
        key: PLAYGROUND_SHELL.key.to_string(),
        enabled,
    }
}

#[test]
fn a_building_flag_is_off_and_its_guard_returns_feature_disabled() {
    // Covers: REQ-PRE-006 (WP-0010 AC-1, AC-3: off state)
    let (conn, ws, _admin) = setup_workspace();
    for edition in [Edition::Desktop, Edition::Workspace] {
        assert!(
            !feature_flag_service::is_enabled(&conn, &ws, edition, &PLAYGROUND_SHELL, false)
                .unwrap()
        );
        match feature_flag_service::require_enabled(&conn, &ws, edition, &PLAYGROUND_SHELL, false) {
            Err(AppError::NotFound(msg)) => {
                assert_eq!(msg, "feature_disabled: playground.shell")
            }
            other => panic!("expected feature_disabled, got {other:?}"),
        }
    }
}

#[test]
fn building_flags_are_hidden_and_cannot_be_changed_outside_developer_mode() {
    // Covers: REQ-PRE-006 (WP-0010 AC-2)
    let (conn, ws, admin) = setup_workspace();
    let listed = feature_flag_service::list(&conn, &ws, Edition::Desktop, false).unwrap();
    assert!(!listed.developer_mode);
    assert!(listed.flags.iter().all(|f| f.key != PLAYGROUND_SHELL.key));

    let err = feature_flag_service::set_override(
        &conn,
        &ws,
        Edition::Desktop,
        &input(Some(true)),
        Some(&admin),
        false,
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)), "{err:?}");
    assert!(!feature_flag_service::is_enabled(
        &conn,
        &ws,
        Edition::Desktop,
        &PLAYGROUND_SHELL,
        false
    )
    .unwrap());
}

#[test]
fn in_developer_mode_an_administrator_can_turn_a_flag_on_and_reset_it() {
    // Covers: REQ-PRE-006 (WP-0010 AC-3: on state)
    let (conn, ws, admin) = setup_workspace();
    let after_on = feature_flag_service::set_override(
        &conn,
        &ws,
        Edition::Workspace,
        &input(Some(true)),
        Some(&admin),
        true,
    )
    .unwrap();
    let shell = after_on
        .flags
        .iter()
        .find(|f| f.key == PLAYGROUND_SHELL.key)
        .expect("listed in developer mode");
    assert!(shell.enabled && shell.overridden && shell.can_toggle);
    feature_flag_service::require_enabled(&conn, &ws, Edition::Workspace, &PLAYGROUND_SHELL, true)
        .unwrap();
    feature_flag_service::require_enabled(&conn, &ws, Edition::Desktop, &PLAYGROUND_SHELL, true)
        .unwrap();

    // The same override never leaks into a release build of the same data.
    assert!(!feature_flag_service::is_enabled(
        &conn,
        &ws,
        Edition::Workspace,
        &PLAYGROUND_SHELL,
        false
    )
    .unwrap());

    let after_reset = feature_flag_service::set_override(
        &conn,
        &ws,
        Edition::Workspace,
        &input(None),
        Some(&admin),
        true,
    )
    .unwrap();
    let shell = after_reset
        .flags
        .iter()
        .find(|f| f.key == PLAYGROUND_SHELL.key)
        .unwrap();
    assert!(!shell.enabled && !shell.overridden);
}

#[test]
fn every_change_is_audited() {
    // Covers: REQ-PRE-006 (WP-0010)
    let (conn, ws, admin) = setup_workspace();
    for enabled in [Some(true), Some(false), None] {
        feature_flag_service::set_override(
            &conn,
            &ws,
            Edition::Desktop,
            &input(enabled),
            Some(&admin),
            true,
        )
        .unwrap();
    }
    let events = audit_repo::list_for_entity(&conn, "FeatureFlag", PLAYGROUND_SHELL.key).unwrap();
    assert_eq!(events.len(), 3);
    assert!(events
        .iter()
        .all(|e| e.event_type == "feature_flag_changed"));
    assert!(events
        .iter()
        .all(|e| e.user_id.as_deref() == Some(admin.as_str())));
}

#[test]
fn only_an_administrator_can_change_a_flag_and_unknown_keys_are_rejected() {
    // Covers: REQ-PRE-006 (WP-0010)
    let (conn, ws, admin) = setup_workspace();
    let rep = user_service::create(
        &conn,
        &ws,
        &NewUser {
            username: "rep".into(),
            display_name: "Rep".into(),
            password: "anothersecretpw".into(),
            roles: vec!["Sales".to_string()],
        },
        Some(&admin),
    )
    .unwrap();
    let denied = feature_flag_service::set_override(
        &conn,
        &ws,
        Edition::Desktop,
        &input(Some(true)),
        Some(&rep.id),
        true,
    );
    assert!(denied.is_err());

    let unknown = feature_flag_service::set_override(
        &conn,
        &ws,
        Edition::Desktop,
        &FeatureFlagOverrideInput {
            key: "no_such.flag".into(),
            enabled: Some(true),
        },
        Some(&admin),
        true,
    )
    .unwrap_err();
    assert!(matches!(unknown, AppError::Validation(m) if m.contains("Unknown feature flag")));
}

#[test]
fn every_signed_in_user_can_read_the_flag_list() {
    // Covers: REQ-PRE-006 (WP-0010)
    let (conn, ws, _admin) = setup_workspace();
    let listed = feature_flag_service::list(&conn, &ws, Edition::Workspace, true).unwrap();
    assert!(listed.developer_mode);
    assert!(listed
        .flags
        .iter()
        .any(|f| f.key == PLAYGROUND_SHELL.key && f.stage == "building" && !f.enabled));
}
