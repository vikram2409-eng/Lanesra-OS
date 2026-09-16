//! Integration Hub (spec table 4/table 20): the Redis connection type.
//! The official `redis` crate is used async/tokio-based (`tokio-comp`
//! feature) with no TLS feature enabled - a plain, unencrypted
//! connection, the common case for a private-network Redis and the
//! same posture `tokio_postgres::NoTls` already takes for Postgres.
//! `test_connection` is proven in this crate's own tests against a
//! *real* local Redis server (this sandbox has one installed and
//! startable) - gated `#[ignore]` since a contributor's machine, or CI,
//! won't necessarily have one running; see
//! `core/tests/integration_connection.rs` for how it's run and verified.

use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct RedisConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default)]
    db: i64,
}
fn default_port() -> u16 {
    6379
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: RedisConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid Redis connection config: {e}")))?;
    let url = match secret {
        Some(password) => format!("redis://:{password}@{}:{}/{}", config.host, config.port, config.db),
        None => format!("redis://{}:{}/{}", config.host, config.port, config.db),
    };
    let client = redis::Client::open(url).map_err(|e| AppError::Validation(format!("Invalid Redis connection: {e}")))?;
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::Validation(format!("Could not connect to {}:{}: {e}", config.host, config.port)))?;
    redis::cmd("PING")
        .query_async::<String>(&mut con)
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but the test command failed: {e}")))?;
    Ok((None, format!("Reachable - connected to Redis on {} (db {})", config.host, config.db)))
}
