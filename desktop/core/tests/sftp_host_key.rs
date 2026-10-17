//! SFTP host-key pinning (threat model T-16, RAID R-28): the client
//! refuses a server whose host key is unknown or has changed. The server
//! here is an in-process `russh` SSH server on a loopback port; it rejects
//! every password, so reaching "authentication rejected" proves the host
//! key was accepted.

use std::sync::Arc;

use lanesra_core::domain::AppError;
use lanesra_core::models::integration::Connection;
use lanesra_core::services::sftp_service;
use russh::keys::ssh_key::private::Ed25519Keypair;
use russh::keys::{HashAlg, PrivateKey};

struct RejectAll;

impl russh::server::Handler for RejectAll {
    type Error = russh::Error;
}

fn host_key(seed: u8) -> PrivateKey {
    PrivateKey::from(Ed25519Keypair::from_seed(&[seed; 32]))
}

fn fingerprint(key: &PrivateKey) -> String {
    key.public_key().fingerprint(HashAlg::Sha256).to_string()
}

/// Serve one SSH connection with `key` and return the port.
async fn serve_once(key: PrivateKey) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let config = Arc::new(russh::server::Config {
        keys: vec![key],
        auth_rejection_time: std::time::Duration::from_millis(10),
        ..Default::default()
    });
    tokio::spawn(async move {
        if let Ok((socket, _)) = listener.accept().await {
            if let Ok(session) = russh::server::run_stream(config, socket, RejectAll).await {
                let _ = session.await;
            }
        }
    });
    port
}

fn connection(port: u16, pinned: Option<&str>) -> Connection {
    let mut config = serde_json::json!({ "host": "127.0.0.1", "port": port, "username": "u" });
    if let Some(fp) = pinned {
        config["host_key_sha256"] = serde_json::json!(fp);
    }
    Connection {
        id: "c1".into(),
        workspace_id: "w1".into(),
        name: "sftp".into(),
        connection_type: "sftp".into(),
        base_url: None,
        auth_mode: "basic".into(),
        has_secret: true,
        config_json: config.to_string(),
        owner_user_id: None,
        status: "active".into(),
        last_test_at: None,
        last_test_status: None,
        last_test_message: None,
        last_failure_at: None,
        credential_expires_at: None,
        created_at: "2026-10-17T00:00:00Z".into(),
        created_by: None,
        updated_at: "2026-10-17T00:00:00Z".into(),
        updated_by: None,
    }
}

fn message(result: Result<(Option<u16>, String), AppError>) -> String {
    match result {
        Err(AppError::Validation(m)) => m,
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn an_unpinned_host_is_refused_and_its_fingerprint_shown() {
    // Covers: REQ-PRE-009 (WP-0011, threat T-16)
    let key = host_key(1);
    let fp = fingerprint(&key);
    let port = serve_once(key).await;
    let msg = message(sftp_service::test_connection(&connection(port, None), Some("pw")).await);
    assert!(msg.contains("Confirm the SFTP server's host key"), "{msg}");
    assert!(msg.contains(&fp), "{msg}");
}

#[tokio::test]
async fn a_changed_host_key_is_refused() {
    // Covers: REQ-PRE-009 (WP-0011, threat T-16)
    let pinned = fingerprint(&host_key(2));
    let port = serve_once(host_key(3)).await;
    let msg =
        message(sftp_service::test_connection(&connection(port, Some(&pinned)), Some("pw")).await);
    assert!(msg.contains("host key has changed"), "{msg}");
}

#[tokio::test]
async fn the_pinned_host_key_is_accepted() {
    // Covers: REQ-PRE-009 (WP-0011, threat T-16)
    let key = host_key(4);
    let fp = fingerprint(&key);
    let port = serve_once(key).await;
    let msg =
        message(sftp_service::test_connection(&connection(port, Some(&fp)), Some("pw")).await);
    // Past the host-key check: the server rejects the password.
    assert!(msg.contains("SSH authentication rejected"), "{msg}");
}
