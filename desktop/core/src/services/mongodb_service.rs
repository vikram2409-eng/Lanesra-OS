//! Integration Hub (spec table 4/table 20): the MongoDB connection type.
//! The official `mongodb` driver is async/tokio-based and defaults to
//! `rustls-tls` (no OpenSSL dependency, matching every other DB driver
//! this crate uses). `test_connection` is proven in this crate's own
//! tests against a *real* local MongoDB server (downloaded from
//! MongoDB's own official tarball distribution for this sandbox, which
//! has no MongoDB package in its own apt repos) - gated `#[ignore]`
//! since a contributor's machine, or CI, won't necessarily have one
//! running; see `core/tests/integration_connection.rs` for how it's run
//! and verified.

use serde::Deserialize;

use crate::domain::{AppError, AppResult};
use crate::models::integration::Connection as ConnectionModel;

#[derive(Debug, Clone, Deserialize)]
struct MongoConfig {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    database: String,
    username: String,
}
fn default_port() -> u16 {
    27017
}

pub async fn test_connection(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: MongoConfig = serde_json::from_str(&connection.config_json).map_err(|e| AppError::Validation(format!("Invalid MongoDB connection config: {e}")))?;
    let uri = format!("mongodb://{}:{}@{}:{}/{}", config.username, secret.unwrap_or_default(), config.host, config.port, config.database);
    let client = mongodb::Client::with_uri_str(&uri)
        .await
        .map_err(|e| AppError::Validation(format!("Could not connect to {}:{}: {e}", config.host, config.port)))?;
    client
        .database(&config.database)
        .run_command(mongodb::bson::doc! { "ping": 1 })
        .await
        .map_err(|e| AppError::Validation(format!("Connected, but the test command failed: {e}")))?;
    Ok((None, format!("Reachable - connected to MongoDB database '{}' on {}", config.database, config.host)))
}
