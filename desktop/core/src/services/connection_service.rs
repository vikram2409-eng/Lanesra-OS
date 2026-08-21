//! Integration Hub (spec §4): Connections - a workspace-specific
//! authenticated endpoint instance, reusable across Connectors, Webhooks
//! and Integration Jobs. CRUD here is plain sync (like every other
//! service in this crate); `test_connection` is the one genuinely
//! `async fn`, since it makes a real outbound call - see this crate's
//! `Cargo.toml` comment on why only this narrow slice of the codebase
//! needs to be async at all.
//!
//! `connection_type` is one of `"rest"` | `"webhook"` | `"sftp"` |
//! `"postgres"` | `"odata"` | `"smtp"` (spec table 4). `auth_mode` is one
//! of `"none"` | `"api_key"` | `"basic"` | `"bearer"` | `"custom_header"`
//! | `"oauth2_client_credentials"` | `"oauth2_authorization_code"` (spec
//! table 5) - mTLS is the one auth mode never implemented, even
//! best-effort: the spec itself marks it "Future".
//!
//! Every `test_connection` variant below is proven against a real local
//! listener spun up inside this crate's own tests - not a live third-party
//! endpoint, which this environment has no access to - but the exact same
//! code path a live REST API, SFTP server or Postgres database would hit.
//! See each connection-type test in `core/tests/integration_connections.rs`
//! for what's actually exercised.

use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde::Deserialize;

use crate::domain::ids::new_uuid;
use crate::domain::{AppError, AppResult};
use crate::models::integration::{Connection as ConnectionModel, ConnectionInput, ConnectionTestResult, ConnectionUpdate};
use crate::repositories::{integration_connection_repo, integration_secret_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

pub const CONNECTION_TYPES: &[&str] = &["rest", "webhook", "sftp", "postgres", "odata", "smtp"];
const AUTH_MODES: &[&str] = &["none", "api_key", "basic", "bearer", "custom_header", "oauth2_client_credentials", "oauth2_authorization_code"];

fn validate_types(connection_type: &str, auth_mode: &str) -> AppResult<()> {
    if !CONNECTION_TYPES.contains(&connection_type) {
        return Err(AppError::Validation(format!("Unknown connection type '{connection_type}'")));
    }
    if !AUTH_MODES.contains(&auth_mode) {
        return Err(AppError::Validation(format!("Unknown auth mode '{auth_mode}'")));
    }
    Ok(())
}

/// Encrypts `secret_value` (if any) into a fresh `integration_secrets` row
/// and returns its id - `None` in, `None` out, matching `auth_mode ==
/// "none"`.
fn store_secret(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], label: &str, secret_value: Option<&str>, actor_user_id: Option<&str>) -> AppResult<Option<String>> {
    match secret_value {
        Some(value) if !value.is_empty() => {
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, value)?;
            let id = new_uuid();
            integration_secret_repo::insert(conn, &id, workspace_id, label, &ciphertext, &nonce, actor_user_id)?;
            Ok(Some(id))
        }
        _ => Ok(None),
    }
}

pub fn create(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], input: &ConnectionInput, actor_user_id: Option<&str>) -> AppResult<ConnectionModel> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Connection name is required".into()));
    }
    validate_types(&input.connection_type, &input.auth_mode)?;
    let id = new_uuid();
    let secret_id = store_secret(conn, workspace_id, master_key, &format!("{} auth secret", input.name), input.secret_value.as_deref(), actor_user_id)?;
    Ok(integration_connection_repo::insert(
        conn,
        &id,
        workspace_id,
        input.name.trim(),
        &input.connection_type,
        input.base_url.as_deref(),
        &input.auth_mode,
        secret_id.as_deref(),
        &input.config_json,
        input.owner_user_id.as_deref(),
        actor_user_id,
    )?)
}

fn get_owned(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<ConnectionModel> {
    let connection = integration_connection_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Connection".into()))?;
    if connection.workspace_id != workspace_id {
        return Err(AppError::NotFound("Connection".into()));
    }
    Ok(connection)
}

pub fn get(conn: &Connection, workspace_id: &str, id: &str) -> AppResult<ConnectionModel> {
    get_owned(conn, workspace_id, id)
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ConnectionModel>> {
    Ok(integration_connection_repo::list_for_workspace(conn, workspace_id)?)
}

pub fn update(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], id: &str, input: &ConnectionUpdate, actor_user_id: Option<&str>) -> AppResult<ConnectionModel> {
    require_admin(conn, actor_user_id)?;
    let existing = get_owned(conn, workspace_id, id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Connection name is required".into()));
    }
    validate_types(&existing.connection_type, &input.auth_mode)?;
    // `secret_value: Some(v)` rotates the secret in place (same
    // `secret_id`, new ciphertext) so nothing pointing at this connection
    // needs updating - spec §20's "secret rotation without editing
    // dependent workflows/jobs". Absent/empty leaves the existing secret
    // (if any) untouched.
    let secret_id = match (&input.secret_value, &existing.has_secret) {
        (Some(value), true) if !value.is_empty() => {
            let existing_secret_id = integration_connection_repo::secret_id_for(conn, id)?.expect("has_secret implies a secret_id");
            let (ciphertext, nonce) = super::secret_service::encrypt(master_key, value)?;
            integration_secret_repo::rotate(conn, &existing_secret_id, &ciphertext, &nonce)?;
            Some(existing_secret_id)
        }
        (Some(value), false) if !value.is_empty() => store_secret(conn, workspace_id, master_key, &format!("{} auth secret", input.name), Some(value), actor_user_id)?,
        _ => integration_connection_repo::secret_id_for(conn, id)?,
    };
    Ok(integration_connection_repo::update(
        conn,
        id,
        input.name.trim(),
        input.base_url.as_deref(),
        &input.auth_mode,
        secret_id.as_deref(),
        &input.config_json,
        input.owner_user_id.as_deref(),
        &input.status,
        actor_user_id,
    )?)
}

pub fn delete(conn: &Connection, workspace_id: &str, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    get_owned(conn, workspace_id, id)?;
    let deps = integration_connection_repo::dependency_count(conn, id)?;
    if deps > 0 {
        return Err(AppError::Conflict(format!(
            "This connection is still used by {deps} other reference/webhook/job/external object - remove those first"
        )));
    }
    Ok(integration_connection_repo::delete(conn, id)?)
}

/// The real, decrypted secret value for a connection's own auth, if any -
/// only ever called at the moment an outbound call needs it (test,
/// connector execution, webhook signing uses a *different* secret - see
/// `webhook_service`). Never returned to any frontend-facing command.
pub(crate) fn resolve_secret(conn: &Connection, master_key: &[u8; 32], connection_id: &str) -> AppResult<Option<String>> {
    match integration_connection_repo::secret_id_for(conn, connection_id)? {
        None => Ok(None),
        Some(secret_id) => {
            let stored = integration_secret_repo::get(conn, &secret_id)?.ok_or_else(|| AppError::NotFound("Connection secret".into()))?;
            Ok(Some(super::secret_service::decrypt(master_key, &stored.ciphertext, &stored.nonce)?))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RestConfig {
    #[serde(default = "default_timeout_ms")]
    timeout_ms: u64,
    #[serde(default)]
    default_headers: Vec<(String, String)>,
    #[serde(default = "default_true")]
    tls_verify: bool,
    #[serde(default)]
    test_path: Option<String>,
}
fn default_timeout_ms() -> u64 {
    10_000
}
fn default_true() -> bool {
    true
}

pub(crate) fn apply_auth(builder: reqwest::RequestBuilder, auth_mode: &str, secret: Option<&str>) -> reqwest::RequestBuilder {
    match auth_mode {
        "api_key" => match secret {
            Some(s) => builder.header("X-Api-Key", s),
            None => builder,
        },
        "bearer" | "oauth2_client_credentials" | "oauth2_authorization_code" => match secret {
            Some(s) => builder.bearer_auth(s),
            None => builder,
        },
        "basic" => match secret.and_then(|s| s.split_once(':')) {
            Some((user, pass)) => builder.basic_auth(user, Some(pass)),
            None => builder,
        },
        "custom_header" => match secret.and_then(|s| s.split_once(':')) {
            Some((name, value)) => builder.header(name, value),
            None => builder,
        },
        _ => builder,
    }
}

/// Runs Test Connection (spec §4.4) for whichever `connection_type` this
/// connection is - reachability + auth + one safe operation, reporting
/// latency and a remediation message without ever exposing the secret
/// itself.
pub async fn test_connection(conn: &Connection, workspace_id: &str, master_key: &[u8; 32], id: &str, actor_user_id: Option<&str>) -> AppResult<ConnectionTestResult> {
    require_admin(conn, actor_user_id)?;
    let connection = get_owned(conn, workspace_id, id)?;
    let secret = resolve_secret(conn, master_key, id)?;
    let started = Instant::now();

    let result = match connection.connection_type.as_str() {
        "rest" | "webhook" | "odata" => test_http(&connection, secret.as_deref()).await,
        "sftp" => super::sftp_service::test_connection(&connection, secret.as_deref()).await,
        "postgres" => super::postgres_service::test_connection(&connection, secret.as_deref()).await,
        "smtp" => super::smtp_service::test_connection(&connection, secret.as_deref()).await,
        other => Err(AppError::Validation(format!("No test implemented for connection type '{other}'"))),
    };

    let latency_ms = started.elapsed().as_millis() as u64;
    let test_result = match result {
        // A `status_code` only ever comes back from the HTTP-like branch,
        // and only once a response was actually received - so it's the
        // right signal for "ok" there (a 4xx/5xx is a reachable endpoint
        // that still failed the test, spec §4.4's "HTTP/error status").
        // The other branches (postgres/sftp/smtp) never set one and only
        // ever return `Ok` on genuine success, so `None` defaults to ok.
        Ok((status_code, message)) => {
            let ok = status_code.map(|code| (200..400).contains(&code)).unwrap_or(true);
            ConnectionTestResult { ok, latency_ms, status_code, message }
        }
        Err(e) => ConnectionTestResult { ok: false, latency_ms, status_code: None, message: e.to_string() },
    };
    integration_connection_repo::set_test_result(
        conn,
        id,
        if test_result.ok { "connected" } else { "failed" },
        &test_result.message,
        !test_result.ok,
    )?;
    Ok(test_result)
}

async fn test_http(connection: &ConnectionModel, secret: Option<&str>) -> AppResult<(Option<u16>, String)> {
    let config: RestConfig = serde_json::from_str(&connection.config_json).unwrap_or_default();
    let base_url = connection.base_url.as_deref().ok_or_else(|| AppError::Validation("This connection has no base URL configured".into()))?;
    let url = match &config.test_path {
        Some(path) => format!("{}{}", base_url.trim_end_matches('/'), path),
        None => base_url.to_string(),
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .danger_accept_invalid_certs(!config.tls_verify)
        .build()
        .map_err(|e| AppError::Validation(format!("could not build HTTP client: {e}")))?;
    let mut builder = client.get(&url);
    for (name, value) in &config.default_headers {
        builder = builder.header(name, value);
    }
    builder = apply_auth(builder, &connection.auth_mode, secret);
    let response = builder.send().await.map_err(|e| AppError::Validation(format!("Could not reach {url}: {e}")))?;
    let status = response.status();
    // A response - any response - means the endpoint is reachable; a
    // 4xx/5xx is a real, structured "failed" result (real status code
    // preserved for the UI), not the same kind of failure as a genuine
    // network-level error (DNS/refused/timeout), which stays an `Err`
    // above and carries no status code at all.
    let message = if status.is_success() || status.is_redirection() { format!("Reachable - HTTP {status}") } else { format!("Endpoint responded with HTTP {status}") };
    Ok((Some(status.as_u16()), message))
}
