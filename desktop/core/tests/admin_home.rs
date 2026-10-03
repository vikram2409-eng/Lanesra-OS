//! Admin Control Center Modernization (issue #197): `admin_home_service::
//! get_summary` is a dozen raw SQL aggregates over tables this module
//! doesn't own - `cargo check` can't catch a wrong table/column name in a
//! string literal, so this smoke test exists purely to prove every query
//! actually runs against the real schema and returns a sane shape on a
//! freshly set-up, empty workspace.

use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{admin_home_service, workspace_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: business_name.into(),
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

#[test]
fn get_summary_runs_every_aggregate_against_a_fresh_workspace() {
    let (conn, ws, admin) = setup_workspace("Admin Home Co");
    let summary = admin_home_service::get_summary(&conn, &ws, Some(&admin)).unwrap();

    // A brand-new workspace has only its one admin user, no custom roles,
    // no data model, no app, no automation, no integration, no backup -
    // every Setup Progress flag should read false.
    assert!(!summary.setup_progress.has_additional_users);
    assert!(!summary.setup_progress.has_custom_access_roles);
    assert!(!summary.setup_progress.has_data_model);
    assert!(!summary.setup_progress.has_published_app);
    assert!(!summary.setup_progress.has_automation);
    assert!(!summary.setup_progress.has_integration);
    assert!(!summary.setup_progress.has_backup);

    // Platform Health aggregates should all be zero/None on a fresh
    // workspace with no runs/connections/layouts of any kind yet.
    assert_eq!(summary.platform_health.integration_connections_failed, 0);
    assert_eq!(summary.platform_health.integration_jobs_running, 0);
    assert_eq!(summary.platform_health.integration_jobs_failed_today, 0);
    assert_eq!(summary.platform_health.workflow_runs_failed_today, 0);
    assert_eq!(summary.platform_health.agent_runs_failed_today, 0);
    assert_eq!(summary.platform_health.unpublished_items, 0);
    assert_eq!(summary.platform_health.last_backup_at, None);

    // The one thing that's always true on a workspace that has never
    // backed up: "no backup has ever been taken" shows up in Needs
    // Attention.
    assert!(summary.needs_attention.iter().any(|i| i.key == "no_backup"));
}

#[test]
fn non_admin_cannot_read_the_admin_home_summary() {
    let (conn, ws, admin) = setup_workspace("Admin Home Guard Co");
    let non_admin_input = lanesra_core::models::user::NewUser {
        username: "sales_rep".into(),
        display_name: "Sales Rep".into(),
        password: "supersecretpassword".into(),
        roles: vec!["Sales".into()],
    };
    let non_admin = lanesra_core::services::user_service::create(&conn, &ws, &non_admin_input, Some(&admin)).unwrap();
    let err = admin_home_service::get_summary(&conn, &ws, Some(&non_admin.id)).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("admin"), "{err}");
}
