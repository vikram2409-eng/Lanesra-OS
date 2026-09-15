//! Integration Hub (spec table 4/table 20): the MySQL/MariaDB connection
//! type. `mysql_async` is pure wire-protocol Rust (no native
//! `libmysqlclient` dependency, so it builds the same everywhere the rest
//! of this crate does - same rationale as `tokio-postgres` above it) and
//! is configured with the `rustls-tls` feature rather than native-tls, so
//! it never pulls in an OpenSSL dependency this crate otherwise avoids
//! (see `reqwest`'s own `rustls-tls` choice in `Cargo.toml`). Connections
//! that don't request TLS (the common case for a local/private-network
//! database, and what `test_connection` below does) never touch the TLS
//! stack at all. `test_connection` is proven in this crate's own tests
//! against a *real* local MariaDB server (this sandbox has one installed
//! and startable) - gated `#[ignore]` since a contributor's machine, or
//! CI, won't necessarily have one running; see
//! `core/tests/integration_connection.rs` for how it's run and verified.

use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct MysqlConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    database: String,
    username: String,
}
fn default_port() -> u16 {
    3306
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: MysqlConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid MySQL connection config: {e}")))?;
    let opts = mysql_async::OptsBuilder::default()
        .ip_or_hostname(config.host.clone())
        .tcp_port(config.port)
        .db_name(Some(config.database.clone()))
        .user(Some(config.username.clone()))
        .pass(secret);
    let mut conn = mysql_async::Conn::new(opts)
        .await
        .map_err(|e| AppError::Validation(format!("Could not connect to {}:{}: {e}", config.host, config.port)))?;
    mysql_async::prelude::Queryable::query_drop(&mut conn, "SELECT 1")
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but the test query failed: {e}")))?;
    Ok((None, format!("Reachable - connected to MySQL database '{}' on {}", config.database, config.host)))
}
