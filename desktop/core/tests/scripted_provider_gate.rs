//! The scripted model provider is for tests only (WP-0005 AC-3,
//! REQ-PRE-003): no settings API can select it, and a build without the
//! `test-support` feature has no `scripted` dispatch path even if a row
//! names it. CI runs this file twice: in the normal test run (feature on)
//! and on its own without the feature, which is how a release build sees it.

use lanesra_core::domain::AppError;
use lanesra_core::models::ai::{AiProviderInput, AiSettingsInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{ai_provider_service, ai_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Gate Co".into(),
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

const MASTER_KEY: [u8; 32] = [7u8; 32];

#[test]
fn no_settings_api_accepts_the_scripted_provider() {
    // Covers: REQ-PRE-003 (WP-0005 AC-3)
    let (conn, ws, admin) = setup_workspace();
    let settings = ai_service::save_settings(
        &conn,
        &ws,
        &MASTER_KEY,
        &AiSettingsInput {
            provider: "scripted".into(),
            base_url: None,
            model: "m".into(),
            api_key: Some("k".into()),
        },
        Some(&admin),
    );
    assert!(
        matches!(&settings, Err(AppError::Validation(m)) if m.contains("Unknown AI provider")),
        "{settings:?}"
    );

    let tier = ai_provider_service::create(
        &conn,
        &ws,
        &MASTER_KEY,
        &AiProviderInput {
            name: "Scripted".into(),
            provider: "scripted".into(),
            base_url: None,
            model: "m".into(),
            api_key: Some("k".into()),
        },
        Some(&admin),
    );
    assert!(
        matches!(&tier, Err(AppError::Validation(m)) if m.contains("Unknown AI provider")),
        "{tier:?}"
    );
}

/// Without the feature, even a row forced past the settings API cannot
/// reach a scripted dispatch path: the provider is simply unknown.
#[cfg(not(feature = "test-support"))]
#[tokio::test]
async fn without_the_feature_a_forced_scripted_row_has_no_dispatch_path() {
    // Covers: REQ-PRE-003 (WP-0005 AC-3)
    let (conn, ws, admin) = setup_workspace();
    ai_service::save_settings(
        &conn,
        &ws,
        &MASTER_KEY,
        &AiSettingsInput {
            provider: "anthropic".into(),
            base_url: None,
            model: "m".into(),
            api_key: Some("k".into()),
        },
        Some(&admin),
    )
    .unwrap();
    conn.execute(
        "UPDATE ai_settings SET provider = 'scripted' WHERE workspace_id = ?1",
        [&ws],
    )
    .unwrap();

    let result = ai_service::complete_with_tools(&conn, &ws, &MASTER_KEY, "sys", &[], &[]).await;
    assert!(
        matches!(&result, Err(AppError::Validation(m)) if m.contains("No tool-calling completion implemented for provider 'scripted'")),
        "{:?}",
        result.err()
    );
}
