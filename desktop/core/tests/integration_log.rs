//! Integration Hub (spec §21/§22/§23): proves `integration_log_service` -
//! settings defaults/updates, the Overview KPI aggregates are real
//! queries (not placeholders) that move when the underlying data does,
//! and log retention purging.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::integration::IntegrationSettingsUpdate;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::integration_log_service::{self, ExecutionQuery, FinishOutcome};
use lanesra_core::services::workspace_service;

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Log Co".into(), legal_name: None, currency_code: "USD".into(), locale: "en-US".into(),
        timezone: "UTC".into(), default_tax_rate_bp: 0, admin_username: "admin".into(), admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(), load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

#[test]
fn settings_default_lazily_and_persist_updates() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    let defaults = integration_log_service::get_settings(&conn, &workspace_id).unwrap();
    assert_eq!(defaults.api_rate_limit_per_minute, 300);
    assert_eq!(defaults.log_retention_days, 90);

    let updated = integration_log_service::update_settings(
        &conn, &workspace_id,
        &IntegrationSettingsUpdate { api_rate_limit_per_minute: 60, global_rate_limit_per_minute: 1000, log_retention_days: 14, file_retention_days: 3, allow_insecure_connections: true },
        Some(&admin_id),
    ).unwrap();
    assert_eq!(updated.api_rate_limit_per_minute, 60);
    assert_eq!(updated.log_retention_days, 14);
    assert!(updated.allow_insecure_connections);

    // Persisted, not just returned - a fresh read sees the same values.
    let reread = integration_log_service::get_settings(&conn, &workspace_id).unwrap();
    assert_eq!(reread.api_rate_limit_per_minute, 60);
}

#[test]
fn non_admins_cannot_change_settings() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let err = integration_log_service::update_settings(
        &conn, &workspace_id,
        &IntegrationSettingsUpdate { api_rate_limit_per_minute: 60, global_rate_limit_per_minute: 1000, log_retention_days: 14, file_retention_days: 3, allow_insecure_connections: false },
        None,
    );
    assert!(err.is_err());
}

#[test]
fn start_and_finish_write_a_real_execution_row_the_overview_and_list_both_see() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let before = integration_log_service::overview(&conn, &workspace_id).unwrap();
    assert_eq!(before.api_calls_today, 0);

    let execution_id = integration_log_service::start(&conn, &workspace_id, "api_call", None, None, "inbound", None);
    integration_log_service::finish(&conn, &execution_id, &FinishOutcome { status: "success".into(), records_written: 1, ..Default::default() });

    let after = integration_log_service::overview(&conn, &workspace_id).unwrap();
    assert_eq!(after.api_calls_today, 1, "the KPI must reflect the row just written, not a cached/placeholder value");

    let listed = integration_log_service::list_executions(&conn, &workspace_id, &ExecutionQuery { execution_type: Some("api_call".into()), ..Default::default() }).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].status, "success");
    assert_eq!(listed[0].records_written, 1);
    assert!(listed[0].duration_ms.is_some(), "finish() must have set a real elapsed duration");
}

#[test]
fn a_failed_execution_is_distinguishable_from_a_successful_one() {
    let (conn, workspace_id, _admin_id) = setup_workspace();
    let ok_id = integration_log_service::start(&conn, &workspace_id, "connector_action", None, None, "outbound", None);
    integration_log_service::finish(&conn, &ok_id, &FinishOutcome { status: "success".into(), records_written: 1, ..Default::default() });
    let fail_id = integration_log_service::start(&conn, &workspace_id, "connector_action", None, None, "outbound", None);
    integration_log_service::finish(&conn, &fail_id, &FinishOutcome { status: "failed".into(), records_failed: 1, error_message: Some("boom".into()), ..Default::default() });

    let failures = integration_log_service::list_executions(&conn, &workspace_id, &ExecutionQuery { status: Some("failed".into()), ..Default::default() }).unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].error_message.as_deref(), Some("boom"));
}

#[test]
fn purge_expired_removes_only_rows_older_than_retention() {
    let (conn, workspace_id, admin_id) = setup_workspace();
    // A very short retention (1 day) plus a manually backdated row proves
    // the purge boundary is real, not a no-op.
    integration_log_service::update_settings(
        &conn, &workspace_id,
        &IntegrationSettingsUpdate { api_rate_limit_per_minute: 300, global_rate_limit_per_minute: 3000, log_retention_days: 1, file_retention_days: 7, allow_insecure_connections: false },
        Some(&admin_id),
    ).unwrap();

    let old_id = integration_log_service::start(&conn, &workspace_id, "api_call", None, None, "inbound", None);
    integration_log_service::finish(&conn, &old_id, &FinishOutcome { status: "success".into(), ..Default::default() });
    conn.execute("UPDATE integration_executions SET started_at = datetime('now', '-10 days') WHERE id = ?1", [&old_id]).unwrap();

    let recent_id = integration_log_service::start(&conn, &workspace_id, "api_call", None, None, "inbound", None);
    integration_log_service::finish(&conn, &recent_id, &FinishOutcome { status: "success".into(), ..Default::default() });

    let purged = integration_log_service::purge_expired(&conn, &workspace_id).unwrap();
    assert_eq!(purged, 1);
    let remaining = integration_log_service::list_executions(&conn, &workspace_id, &Default::default()).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, recent_id);
}
