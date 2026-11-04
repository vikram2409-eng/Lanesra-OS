//! WP-0106: Team Workspace sign-in hardening — per-workspace security
//! settings and the active-session view used by the admin screen.

use serde::{Deserialize, Serialize};

/// Per-workspace security configuration (migration 0089). One row per
/// workspace, lazily created by `security_settings_repo::ensure_default`.
#[derive(Debug, Clone, Serialize)]
pub struct SecuritySettings {
    pub workspace_id: String,
    /// Failed sign-ins (per account, per source address) before a lockout.
    pub lockout_max_failures: i64,
    /// How long a lockout lasts, in minutes.
    pub lockout_minutes: i64,
    /// Absolute web-session lifetime, in hours.
    pub session_lifetime_hours: i64,
    /// Idle timeout in minutes; 0 means no idle timeout.
    pub session_idle_minutes: i64,
    /// The public base URL the server is reached at, if an admin set one.
    /// `LANESRA_PUBLIC_URL` overrides this at runtime when present.
    pub public_base_url: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

/// The admin-editable fields of [`SecuritySettings`].
#[derive(Debug, Clone, Deserialize)]
pub struct SecuritySettingsUpdate {
    pub lockout_max_failures: i64,
    pub lockout_minutes: i64,
    pub session_lifetime_hours: i64,
    pub session_idle_minutes: i64,
    /// Empty/absent clears the stored URL.
    #[serde(default)]
    pub public_base_url: Option<String>,
}

/// A live web session, for the admin "active sessions" list.
#[derive(Debug, Clone, Serialize)]
pub struct ActiveSession {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
    /// True for the session making the request, so the UI can mark "this device".
    pub current: bool,
}
