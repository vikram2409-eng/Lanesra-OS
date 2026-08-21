//! Integration Hub (spec table 4/table 20): the PostgreSQL connection
//! type. `tokio-postgres` is pure wire-protocol Rust (no native libpq
//! dependency, so it builds the same everywhere the rest of this crate
//! does). `test_connection` is proven in this crate's own tests against a
//! *real* local Postgres server (this sandbox has one installed and
//! startable) - gated `#[ignore]` since a contributor's machine, or CI,
//! won't necessarily have Postgres running; see
//! `core/tests/integration_connections.rs` for how it's run and verified.
//! SQL Server/MySQL share the exact same `Connection`/auth/secret shape
//! (`connection_type` would just be a different value) but no driver for
//! either is wired up this pass - a real, stated scope limit rather than
//! a silently-missing feature.

use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct PostgresConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    database: String,
    username: String,
}
fn default_port() -> u16 {
    5432
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: PostgresConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid Postgres connection config: {e}")))?;
    let mut conninfo = format!("host={} port={} dbname={} user={}", config.host, config.port, config.database, config.username);
    if let Some(password) = secret {
        conninfo.push_str(&format!(" password={password}"));
    }
    let (client, connection_driver) = tokio_postgres::connect(&conninfo, tokio_postgres::NoTls)
        .await
        .map_err(|e| AppError::Validation(format!("Could not connect to {}:{}: {e}", config.host, config.port)))?;
    // The connection object must be polled somewhere for the client to do
    // any work at all - tokio_postgres's own documented pattern.
    tokio::spawn(async move {
        let _ = connection_driver.await;
    });
    client
        .simple_query("SELECT 1")
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but the test query failed: {e}")))?;
    Ok((None, format!("Reachable - connected to Postgres database '{}' on {}", config.database, config.host)))
}
