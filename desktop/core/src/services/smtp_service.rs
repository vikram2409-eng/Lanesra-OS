//! Integration Hub (spec table 4: "Email / SMTP | P2 | Optional outbound
//! notification integration") - the SMTP connection type, and the send
//! action a Workflow/webhook-style notification could use later.
//! `test_connection` is proven against a minimal in-process raw-TCP SMTP
//! double in this crate's own tests (see
//! `core/tests/integration_connections.rs`) - not a live mail relay,
//! which this environment can't reach, but the exact SMTP dialogue
//! (`EHLO`/`AUTH`/`QUIT`) a real one would run.

use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct SmtpConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    username: Option<String>,
}
fn default_port() -> u16 {
    587
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: SmtpConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid SMTP connection config: {e}")))?;
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host).port(config.port).tls(Tls::None);
    if let (Some(username), Some(password)) = (&config.username, secret) {
        builder = builder.credentials(Credentials::new(username.clone(), password.to_string()));
    }
    let mailer: AsyncSmtpTransport<Tokio1Executor> = builder.build();
    let ok = mailer.test_connection().await.map_err(|e| AppError::Validation(format!("Could not reach {}:{}: {e}", config.host, config.port)))?;
    if ok {
        Ok((None, format!("Reachable - SMTP handshake succeeded with {}", config.host)))
    } else {
        Err(AppError::Validation("SMTP server did not accept the connection".into()))
    }
}
