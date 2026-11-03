//! WP-0106: Team Workspace first-run and sign-in hardening.
//!
//! - AC-1: over HTTP, first-run setup without the one-time token is refused,
//!   with it succeeds once, and the token cannot be reused.
//! - AC-2: repeated failed sign-ins are delayed, then locked out, and the
//!   lockout expires; each failure, lockout and unlock is audited.
//! - AC-3: the public base URL is validated and exposed through one core
//!   function, with the environment variable overriding the stored value.
//! - AC-4: an idle session expires after the configured timeout, the
//!   absolute lifetime still applies, and "sign out all" ends every session.
//!
//! Covers: REQ-PRE-019 (WP-0106 AC-1..AC-5).

use std::net::SocketAddr;

use lanesra_core::db::open_workspace_db;
use lanesra_core::domain::AppError;
use lanesra_core::models::security::SecuritySettingsUpdate;
use lanesra_core::models::user::Credentials;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::session_repo;
use lanesra_core::services::{
    auth_service, security_settings_service, web_session_service, workspace_service,
};
use lanesra_server::{build_router, setup_token, SecurityConfig, ServerState};
use rusqlite::Connection;
use serde_json::{json, Value};

fn setup() -> WorkspaceSetup {
    WorkspaceSetup {
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
    }
}

fn error_code(e: &AppError) -> Option<&str> {
    match e {
        AppError::Classified { code, .. } => Some(code),
        _ => None,
    }
}

fn audit_count(conn: &Connection, event_type: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events WHERE event_type = ?1",
        [event_type],
        |r| r.get(0),
    )
    .unwrap()
}

// ---- AC-1: the one-time setup token, over HTTP ----------------------------

async fn post(client: &reqwest::Client, url: String, body: Value) -> Value {
    client
        .post(url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn first_run_requires_the_one_time_setup_token() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("lanesra.sqlite3");
    let state = ServerState::new(
        open_workspace_db(&db_path).unwrap(),
        db_path.clone(),
        SecurityConfig::default(),
    );
    let frontend_dir = dir.path().join("frontend");
    std::fs::create_dir_all(&frontend_dir).unwrap();
    let app = build_router(state, frontend_dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    // Simulate server startup on a fresh install: a token is generated.
    let bootstrap = open_workspace_db(&db_path).unwrap();
    let token = setup_token::ensure(&db_path, &bootstrap)
        .unwrap()
        .expect("a fresh install has a setup token");
    drop(bootstrap);
    assert!(setup_token::path(&db_path).exists());

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/invoke/first_run_setup");
    let s = json!({
        "business_name": "Gate Co", "legal_name": null, "currency_code": "USD",
        "locale": "en-US", "timezone": "UTC", "default_tax_rate_bp": 0,
        "admin_username": "admin", "admin_display_name": "Admin",
        "admin_password": "supersecretpassword", "load_sample_data": false
    });

    // No token → refused, and no workspace was created.
    let no_token = post(&client, url.clone(), json!({ "setup": s })).await;
    assert_eq!(no_token["ok"], false, "{no_token}");

    // Wrong token → refused.
    let wrong = post(
        &client,
        url.clone(),
        json!({ "setup": s, "setup_token": "not-the-token" }),
    )
    .await;
    assert_eq!(wrong["ok"], false, "{wrong}");
    assert!(
        setup_token::path(&db_path).exists(),
        "token not consumed yet"
    );

    // Correct token → succeeds once, and the token is consumed.
    let ok = post(
        &client,
        url.clone(),
        json!({ "setup": s, "setup_token": token }),
    )
    .await;
    assert_eq!(ok["ok"], true, "{ok}");
    assert!(
        !setup_token::path(&db_path).exists(),
        "the token file is deleted after a successful setup"
    );

    // Reusing the (now deleted) token cannot create a second workspace.
    let reuse = post(
        &client,
        url.clone(),
        json!({ "setup": s, "setup_token": token }),
    )
    .await;
    assert_eq!(reuse["ok"], false, "{reuse}");
}

// ---- AC-2: throttling and lockout (injected clock) ------------------------

#[test]
fn repeated_failed_sign_ins_lock_out_then_expire_and_are_audited() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open_workspace_db(&dir.path().join("lanesra.sqlite3")).unwrap();
    let (ws, admin) = workspace_service::first_run_setup(&conn, &setup()).unwrap();

    // Three failures locks out, for 15 minutes.
    security_settings_service::update_settings(
        &conn,
        &ws.id,
        &SecuritySettingsUpdate {
            lockout_max_failures: 3,
            lockout_minutes: 15,
            session_lifetime_hours: 336,
            session_idle_minutes: 0,
            public_base_url: None,
        },
        Some(&admin.id),
    )
    .unwrap();

    let wrong = Credentials {
        username: "admin".into(),
        password: "wrong".into(),
    };
    let right = Credentials {
        username: "admin".into(),
        password: "supersecretpassword".into(),
    };
    let t0 = 1_000_000_000_000i64;

    // Failures 1 and 2: wrong password, not yet locked.
    for i in 0..2 {
        let e = auth_service::login_at(&conn, &ws.id, &wrong, None, t0 + i).unwrap_err();
        assert!(matches!(e, AppError::Validation(_)), "{e:?}");
    }
    // Failure 3 trips the lockout (still reported as an invalid password).
    let third = auth_service::login_at(&conn, &ws.id, &wrong, None, t0 + 2).unwrap_err();
    assert!(matches!(third, AppError::Validation(_)), "{third:?}");

    // Now even the correct password is refused while locked.
    let locked = auth_service::login_at(&conn, &ws.id, &right, None, t0 + 3).unwrap_err();
    assert_eq!(
        error_code(&locked),
        Some("policy.account_locked"),
        "{locked:?}"
    );

    // Once the window passes, the correct password works again (unlock).
    let after = t0 + 15 * 60_000 + 100;
    let user = auth_service::login_at(&conn, &ws.id, &right, None, after).unwrap();
    assert_eq!(user.username, "admin");

    assert_eq!(audit_count(&conn, "failed_login"), 3);
    assert_eq!(audit_count(&conn, "sign_in_locked"), 1);
    assert_eq!(audit_count(&conn, "sign_in_unlocked"), 1);
    assert_eq!(audit_count(&conn, "login"), 1);

    // A later clean sign-in is not throttled and logs no new lock/unlock.
    auth_service::login_at(&conn, &ws.id, &right, None, after + 1_000).unwrap();
    assert_eq!(audit_count(&conn, "sign_in_unlocked"), 1);
}

// ---- AC-3: public base URL validated and exposed --------------------------

#[test]
fn public_base_url_is_validated_and_exposed_through_one_function() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open_workspace_db(&dir.path().join("lanesra.sqlite3")).unwrap();
    let (ws, admin) = workspace_service::first_run_setup(&conn, &setup()).unwrap();

    assert!(security_settings_service::validate_public_base_url("https://ws.example.com/").is_ok());
    assert!(security_settings_service::validate_public_base_url("ftp://x").is_err());
    assert!(security_settings_service::validate_public_base_url("not a url").is_err());

    // Stored value is exposed (trailing slash trimmed).
    security_settings_service::update_settings(
        &conn,
        &ws.id,
        &SecuritySettingsUpdate {
            lockout_max_failures: 10,
            lockout_minutes: 15,
            session_lifetime_hours: 336,
            session_idle_minutes: 0,
            public_base_url: Some("https://ws.example.com/".into()),
        },
        Some(&admin.id),
    )
    .unwrap();
    std::env::remove_var("LANESRA_PUBLIC_URL");
    assert_eq!(
        security_settings_service::effective_public_base_url(&conn, &ws.id).unwrap(),
        Some("https://ws.example.com".into())
    );

    // The environment variable overrides the stored value.
    std::env::set_var("LANESRA_PUBLIC_URL", "https://edge.example.net");
    assert_eq!(
        security_settings_service::effective_public_base_url(&conn, &ws.id).unwrap(),
        Some("https://edge.example.net".into())
    );
    std::env::remove_var("LANESRA_PUBLIC_URL");
}

// ---- AC-4: session idle / absolute expiry and sign-out-all ----------------

fn set_session_times(conn: &Connection, id: &str, last_seen: &str, expires: &str) {
    conn.execute(
        "UPDATE web_sessions SET last_seen_at = ?1, expires_at = ?2 WHERE id = ?3",
        (last_seen, expires, id),
    )
    .unwrap();
}

#[test]
fn sessions_respect_idle_and_absolute_expiry_and_can_all_be_ended() {
    use chrono::{Duration, TimeZone, Utc};
    let dir = tempfile::tempdir().unwrap();
    let conn = open_workspace_db(&dir.path().join("lanesra.sqlite3")).unwrap();
    let (ws, admin) = workspace_service::first_run_setup(&conn, &setup()).unwrap();

    // A 30-minute idle timeout.
    security_settings_service::update_settings(
        &conn,
        &ws.id,
        &SecuritySettingsUpdate {
            lockout_max_failures: 10,
            lockout_minutes: 15,
            session_lifetime_hours: 336,
            session_idle_minutes: 30,
            public_base_url: None,
        },
        Some(&admin.id),
    )
    .unwrap();

    let base = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
    let token = session_repo::create(&conn, &ws.id, &admin.id).unwrap();
    set_session_times(
        &conn,
        &token,
        &base.to_rfc3339(),
        &(base + Duration::hours(336)).to_rfc3339(),
    );

    // Within the idle window → resolves and bumps last_seen.
    assert_eq!(
        session_repo::resolve_and_touch_at(&conn, &token, base + Duration::minutes(10)).unwrap(),
        Some(admin.id.clone())
    );
    // Idle beyond the timeout → expired (measured from the last touch).
    assert_eq!(
        session_repo::resolve_and_touch_at(
            &conn,
            &token,
            base + Duration::minutes(10) + Duration::minutes(31),
        )
        .unwrap(),
        None,
        "an idle session expires"
    );

    // A fresh session past its absolute expiry is rejected regardless of idle.
    let token2 = session_repo::create(&conn, &ws.id, &admin.id).unwrap();
    set_session_times(
        &conn,
        &token2,
        &base.to_rfc3339(),
        &base.to_rfc3339(), // already expired
    );
    assert_eq!(
        session_repo::resolve_and_touch_at(&conn, &token2, base + Duration::seconds(1)).unwrap(),
        None,
        "absolute expiry still applies"
    );

    // Sign out all: two live sessions, then none.
    let a = session_repo::create(&conn, &ws.id, &admin.id).unwrap();
    let b = session_repo::create(&conn, &ws.id, &admin.id).unwrap();
    assert_eq!(
        web_session_service::list_active_sessions(&conn, &ws.id, Some(&admin.id), Some(&a))
            .unwrap()
            .len(),
        2
    );
    let ended = web_session_service::sign_out_all(&conn, &ws.id, Some(&admin.id)).unwrap();
    assert!(ended >= 2, "ended {ended}");
    assert!(
        web_session_service::list_active_sessions(&conn, &ws.id, Some(&admin.id), Some(&b))
            .unwrap()
            .is_empty()
    );
}
