//! Integration Hub (spec table 4/table 20): the SQL Server connection
//! type. `tiberius` is a pure-Rust TDS protocol implementation (no native
//! ODBC/FreeTDS dependency - same "builds the same everywhere" rationale
//! as `tokio-postgres`/`mysql_async` above), configured with the
//! `rustls` feature rather than `native-tls`, so it never pulls in an
//! OpenSSL dependency this crate otherwise avoids.
//!
//! `trust_cert()` is used unconditionally below (accept whatever
//! certificate the server presents during TDS's pre-login TLS
//! handshake, without validating it against a CA) - the same posture
//! `postgres_service.rs`'s `tokio_postgres::NoTls` already takes for
//! Postgres (no certificate validation), stated plainly rather than
//! silently assumed, and appropriate for the same reason: this
//! Connection model has nowhere to configure a custom CA bundle today.
//!
//! **Known gap, stated plainly rather than silently skipped**: unlike
//! `postgres_service.rs`/`mysql_service.rs`, whose `test_connection` is
//! proven (gated `#[ignore]`) against a real local server, this sandbox
//! has no SQL Server build available to run one against (no Microsoft
//! apt repository access, and standing up a container is out of scope
//! for this pass) - matching this crate's own precedent for SFTP's
//! server-side (`core/tests/integration_connection.rs`'s module doc:
//! "not covered by a dedicated test in this pass... stated plainly as a
//! known gap rather than silently skipped"). The failure path (a closed
//! port produces a real, structured connection error) *is* proven; the
//! success path against a genuine SQL Server instance is not.

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct SqlServerConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    database: String,
    username: String,
}
fn default_port() -> u16 {
    1433
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let parsed: SqlServerConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid SQL Server connection config: {e}")))?;
    let mut config = tiberius::Config::new();
    config.host(&parsed.host);
    config.port(parsed.port);
    config.database(&parsed.database);
    config.authentication(tiberius::AuthMethod::sql_server(&parsed.username, secret.unwrap_or_default()));
    config.trust_cert();

    let tcp = TcpStream::connect((parsed.host.as_str(), parsed.port))
        .await
        .map_err(|e| AppError::Validation(format!("Could not connect to {}:{}: {e}", parsed.host, parsed.port)))?;
    tcp.set_nodelay(true).ok();
    let mut client = tiberius::Client::connect(config, tcp.compat_write())
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but SQL Server login failed: {e}")))?;
    client
        .simple_query("SELECT 1")
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but the test query failed: {e}")))?;
    Ok((None, format!("Reachable - connected to SQL Server database '{}' on {}", parsed.database, parsed.host)))
}
