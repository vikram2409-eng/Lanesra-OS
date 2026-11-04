//! WP-0106: per-workspace security settings — sign-in lockout thresholds,
//! web-session lifetime and idle timeout, and the public base URL. The
//! public base URL is the one core function the SSO work packages
//! (WP-0102/WP-0103) read to build redirect and assertion URLs; the
//! `LANESRA_PUBLIC_URL` environment variable overrides the stored value.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::security::{SecuritySettings, SecuritySettingsUpdate};
use crate::repositories::security_settings_repo as repo;

const PUBLIC_URL_ENV: &str = "LANESRA_PUBLIC_URL";

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

/// Validates a public base URL: it must be an absolute `http`/`https` URL
/// with a host, and is returned trimmed of any trailing slash. Empty input
/// is rejected by the caller before this is reached.
pub fn validate_public_base_url(raw: &str) -> AppResult<String> {
    let url = raw.trim().trim_end_matches('/');
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| {
            AppError::Validation("Public URL must start with http:// or https://".into())
        })?;
    let host = rest.split('/').next().unwrap_or("");
    let host = host.split('@').next_back().unwrap_or(""); // drop any userinfo
    let host = host.split(':').next().unwrap_or(""); // drop any port
    if host.is_empty() || host.contains(' ') {
        return Err(AppError::Validation(
            "Public URL must include a host, e.g. https://workspace.example.com".into(),
        ));
    }
    Ok(url.to_string())
}

pub fn get_settings(conn: &Connection, workspace_id: &str) -> AppResult<SecuritySettings> {
    Ok(repo::ensure_default(conn, workspace_id)?)
}

pub fn update_settings(
    conn: &Connection,
    workspace_id: &str,
    input: &SecuritySettingsUpdate,
    actor_user_id: Option<&str>,
) -> AppResult<SecuritySettings> {
    require_admin(conn, actor_user_id)?;

    let public = match input.public_base_url.as_deref().map(str::trim) {
        Some("") | None => None,
        Some(raw) => Some(validate_public_base_url(raw)?),
    };

    repo::ensure_default(conn, workspace_id)?;
    Ok(repo::update(
        conn,
        workspace_id,
        input.lockout_max_failures.clamp(1, 1000),
        input.lockout_minutes.clamp(1, 7 * 24 * 60),
        input.session_lifetime_hours.clamp(1, 365 * 24),
        input.session_idle_minutes.clamp(0, 365 * 24 * 60),
        public.as_deref(),
        actor_user_id,
    )?)
}

/// The one function the SSO work packages read for the server's external
/// URL. `LANESRA_PUBLIC_URL` wins when set (and is validated); otherwise the
/// admin-configured value is used. `None` means neither is set.
pub fn effective_public_base_url(
    conn: &Connection,
    workspace_id: &str,
) -> AppResult<Option<String>> {
    if let Ok(env) = std::env::var(PUBLIC_URL_ENV) {
        if !env.trim().is_empty() {
            return Ok(Some(validate_public_base_url(&env)?));
        }
    }
    Ok(get_settings(conn, workspace_id)?.public_base_url)
}
