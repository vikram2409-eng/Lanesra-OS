//! Integration Hub (spec §6.1/table 4): the SFTP connection type - pure
//! Rust on both ends (`russh` + `russh-sftp`), so `test_connection` here
//! is proven in this crate's own tests against an in-process SFTP server
//! (also `russh`), not a live third-party host this environment has no
//! access to. `connect_and_test` is the one function a real
//! `integration_job_service` file-import/export run would also call.

use std::sync::Arc;
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
}
fn default_port() -> u16 {
    22
}

struct QuietHandler;

#[async_trait::async_trait]
impl russh::client::Handler for QuietHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _server_public_key: &russh::keys::key::PublicKey) -> Result<bool, Self::Error> {
        // Self-hosted, admin-configured endpoints, same trust model the
        // rest of Integration Hub's "TLS verification enabled by default,
        // admin can accept the risk" story uses elsewhere - host-key
        // pinning is a real future hardening item, not attempted here.
        Ok(true)
    }
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: SftpConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid SFTP connection config: {e}")))?;
    let password = secret.ok_or_else(|| AppError::Validation("This SFTP connection has no password/credential configured".into()))?;

    let ssh_config = Arc::new(russh::client::Config { inactivity_timeout: Some(Duration::from_secs(10)), ..Default::default() });
    let mut session = russh::client::connect(ssh_config, (config.host.as_str(), config.port), QuietHandler)
        .await
        .map_err(|e| AppError::Validation(format!("Could not reach {}:{}: {e}", config.host, config.port)))?;

    let authenticated = session
        .authenticate_password(&config.username, password)
        .await
        .map_err(|e| AppError::Validation(format!("SSH authentication failed: {e}")))?;
    if !authenticated {
        return Err(AppError::Validation("SSH authentication rejected - check the username/password".into()));
    }

    let channel = session.channel_open_session().await.map_err(|e| AppError::Validation(format!("Could not open SSH channel: {e}")))?;
    channel.request_subsystem(true, "sftp").await.map_err(|e| AppError::Validation(format!("SFTP subsystem not available: {e}")))?;
    let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| AppError::Validation(format!("Could not start SFTP session: {e}")))?;
    sftp.read_dir(".").await.map_err(|e| AppError::Validation(format!("SFTP directory listing failed: {e}")))?;

    Ok((None, format!("Reachable - SFTP session established to {}", config.host)))
}
