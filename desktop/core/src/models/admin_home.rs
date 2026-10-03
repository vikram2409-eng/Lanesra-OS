//! Admin Control Center Modernization (issue #197): the Admin landing
//! page's own aggregate data - Setup Progress / Needs Attention /
//! Platform Health - computed fresh on every open from this codebase's
//! own existing status/health fields (Integration Hub connection/job
//! status, Workflow/Execution Graph run outcomes, the draft/published
//! state every 2.0-era builder already tracks, `audit_events`), not a
//! new tracked state of its own. See `services::admin_home_service`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SetupProgress {
    pub has_additional_users: bool,
    pub has_custom_access_roles: bool,
    pub has_data_model: bool,
    pub has_published_app: bool,
    pub has_automation: bool,
    pub has_integration: bool,
    pub has_backup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NeedsAttentionItem {
    pub key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformHealth {
    pub integration_connections_failed: i64,
    pub integration_jobs_running: i64,
    pub integration_jobs_failed_today: i64,
    pub workflow_runs_failed_today: i64,
    pub agent_runs_failed_today: i64,
    pub unpublished_items: i64,
    pub last_backup_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminHomeSummary {
    pub setup_progress: SetupProgress,
    pub needs_attention: Vec<NeedsAttentionItem>,
    pub platform_health: PlatformHealth,
}
