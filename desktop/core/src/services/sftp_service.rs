//! Integration Hub (spec §6.1/table 4): the SFTP connection type - pure
//! Rust on both ends (`russh` + `russh-sftp`), so `test_connection` here
//! is proven in this crate's own tests against an in-process SFTP server
//! (also `russh`), not a live third-party host this environment has no
//! access to. `connect_and_test` is the one function a real
//! `integration_job_service` file-import/export run would also call.
//!
//! The server's host key must match the fingerprint configured on the
//! connection (`host_key_sha256`); an unknown or changed key is refused.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct SftpConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    username: String,
    /// The server's host-key fingerprint in OpenSSH form (`SHA256:...`).
    /// Required: without it the connection is refused and the error shows
    /// the fingerprint the server presented, for an Administrator to check
    /// out of band and paste in (threat model T-16).
    #[serde(default)]
    host_key_sha256: Option<String>,
}
fn default_port() -> u16 {
    22
}

/// Accepts the server only if its host key matches the configured
/// fingerprint, and records the fingerprint it saw either way.
struct PinnedHostKey {
    expected: Option<String>,
    seen: Arc<Mutex<Option<String>>>,
}

impl russh::client::Handler for PinnedHostKey {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        let seen = server_public_key
            .fingerprint(russh::keys::HashAlg::Sha256)
            .to_string();
        let ok = self.expected.as_deref().map(str::trim) == Some(seen.as_str());
        *self.seen.lock().unwrap_or_else(|e| e.into_inner()) = Some(seen);
        Ok(ok)
    }
}

pub async fn test_connection(
    connection: &ConnectionModel,
    secret: Option<&str>,
) -> AppResult<(Option<u16>, String)> {
    let config: SftpConfig = serde_json::from_str(&connection.config_json)
        .map_err(|e| AppError::Validation(format!("Invalid SFTP connection config: {e}")))?;
    let password = secret.ok_or_else(|| {
        AppError::Validation("This SFTP connection has no password/credential configured".into())
    })?;

    let ssh_config = Arc::new(russh::client::Config {
        inactivity_timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    });
    let seen = Arc::new(Mutex::new(None));
    let handler = PinnedHostKey {
        expected: config.host_key_sha256.clone(),
        seen: seen.clone(),
    };
    let connected =
        russh::client::connect(ssh_config, (config.host.as_str(), config.port), handler).await;
    let mut session = match connected {
        Ok(session) => session,
        Err(e) => {
            let seen = seen.lock().unwrap_or_else(|e| e.into_inner()).clone();
            return Err(AppError::Validation(
                match (seen, &config.host_key_sha256) {
                    (Some(fp), None) => format!(
                        "Confirm the SFTP server's host key before connecting: check that {}:{} \
                     presents {fp}, then add \"host_key_sha256\": \"{fp}\" to this \
                     connection's config",
                        config.host, config.port
                    ),
                    (Some(fp), Some(expected)) if fp != expected.trim() => format!(
                        "The SFTP server's host key has changed: expected {}, got {fp}. \
                     Do not connect until you have confirmed the change with the server's owner",
                        expected.trim()
                    ),
                    _ => format!("Could not reach {}:{}: {e}", config.host, config.port),
                },
            ));
        }
    };

    let authenticated = session
        .authenticate_password(&config.username, password)
        .await
        .map_err(|e| AppError::Validation(format!("SSH authentication failed: {e}")))?;
    if !authenticated.success() {
        return Err(AppError::Validation(
            "SSH authentication rejected - check the username/password".into(),
        ));
    }

    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| AppError::Validation(format!("Could not open SSH channel: {e}")))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| AppError::Validation(format!("SFTP subsystem not available: {e}")))?;
    let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| AppError::Validation(format!("Could not start SFTP session: {e}")))?;
    sftp.read_dir(".")
        .await
        .map_err(|e| AppError::Validation(format!("SFTP directory listing failed: {e}")))?;

    Ok((
        None,
        format!("Reachable - SFTP session established to {}", config.host),
    ))
}
