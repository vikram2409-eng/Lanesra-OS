//! Admin Control Center Modernization (issue #197): aggregates the Admin
//! landing page's own "Setup Progress" / "Needs Attention" / "Platform
//! Health" sections need, each a handful of real `conn.query_row`
//! aggregates over existing tables - the same "plain aggregate query, no
//! ORM" convention `integration_log_service::overview` already uses for
//! its own KPI row, not a new tracked/cached state of its own. Every
//! count is computed fresh on every call; there is nothing to keep in
//! sync.
//!
//! "Unpublished changes" is counted the same way every 2.0-era builder's
//! own per-row badge already computes it client-side (`ScreenLayoutsAdmin.
//! tsx`/`DashboardLayoutsAdmin.tsx`'s `draft !== published` comparison),
//! just as a workspace-wide SQL aggregate: `screen_layouts`/`page_layouts`/
//! `dashboard_layouts` store a draft/published JSON pair per row (a row
//! counts if it's never been published, or its draft has diverged since),
//! while `workspace_themes`/`ai_execution_graphs` are a version-per-row
//! model (`status = 'draft'` is itself the "unpublished" signal, no diff
//! needed).

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::admin_home::{AdminHomeSummary, NeedsAttentionItem, PlatformHealth, SetupProgress};
use crate::repositories::integration_connection_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn today_start_iso() -> String {
    // Same coarse "midnight UTC" cutoff `integration_log_service::overview`
    // already uses for its own "today" KPI columns.
    let now = crate::domain::ids::now_iso();
    format!("{}T00:00:00Z", &now[..10])
}

fn soon_cutoff_iso() -> String {
    // "Expiring soon" = within the next 14 days - `credential_expires_at`
    // has never been queried anywhere in this codebase before this, so
    // there's no existing convention to match; 14 days is a reasonable
    // admin-attention window, not a spec-mandated number.
    (chrono::Utc::now() + chrono::Duration::days(14)).to_rfc3339()
}

fn count_one(conn: &Connection, sql: &str, workspace_id: &str) -> AppResult<i64> {
    Ok(conn.query_row(sql, [workspace_id], |r| r.get(0))?)
}

pub fn get_summary(conn: &Connection, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<AdminHomeSummary> {
    require_admin(conn, actor_user_id)?;
    let since = today_start_iso();

    // --- Setup Progress ---------------------------------------------------
    let active_users = count_one(conn, "SELECT COUNT(*) FROM users WHERE workspace_id = ?1 AND is_active = 1", workspace_id)?;
    let custom_roles = count_one(conn, "SELECT COUNT(*) FROM access_roles WHERE workspace_id = ?1 AND is_system = 0", workspace_id)?;
    let custom_objects = count_one(conn, "SELECT COUNT(*) FROM custom_object_definitions WHERE workspace_id = ?1 AND is_active = 1", workspace_id)?;
    let published_apps = count_one(conn, "SELECT COUNT(*) FROM app_definitions WHERE workspace_id = ?1 AND is_published = 1", workspace_id)?;
    let workflows = count_one(conn, "SELECT COUNT(*) FROM workflow_definitions WHERE workspace_id = ?1 AND is_active = 1", workspace_id)?;
    let business_rules = count_one(conn, "SELECT COUNT(*) FROM business_rules WHERE workspace_id = ?1 AND is_active = 1", workspace_id)?;
    let connections_total = count_one(conn, "SELECT COUNT(*) FROM integration_connections WHERE workspace_id = ?1", workspace_id)?;
    let backup_events = count_one(conn, "SELECT COUNT(*) FROM audit_events WHERE workspace_id = ?1 AND event_type IN ('backup','restore')", workspace_id)?;

    let setup_progress = SetupProgress {
        has_additional_users: active_users > 1,
        has_custom_access_roles: custom_roles > 0,
        has_data_model: custom_objects > 0,
        has_published_app: published_apps > 0,
        has_automation: workflows > 0 || business_rules > 0,
        has_integration: connections_total > 0,
        has_backup: backup_events > 0,
    };

    // --- Platform Health ----------------------------------------------------
    let connections_failed = integration_connection_repo::count_by_status(conn, workspace_id, "failed")?;
    let jobs_running: i64 = conn.query_row(
        "SELECT COUNT(*) FROM integration_job_runs r JOIN integration_jobs j ON j.id = r.job_id WHERE j.workspace_id = ?1 AND r.status = 'running'",
        [workspace_id],
        |r| r.get(0),
    )?;
    let jobs_failed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM integration_job_runs r JOIN integration_jobs j ON j.id = r.job_id WHERE j.workspace_id = ?1 AND r.status = 'failed' AND r.started_at >= ?2",
        rusqlite::params![workspace_id, since],
        |r| r.get(0),
    )?;
    let workflow_runs_failed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM workflow_runs WHERE workspace_id = ?1 AND outcome = 'error' AND triggered_at >= ?2",
        rusqlite::params![workspace_id, since],
        |r| r.get(0),
    )?;
    let graph_runs_failed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ai_runs WHERE workspace_id = ?1 AND status = 'failed' AND started_at >= ?2",
        rusqlite::params![workspace_id, since],
        |r| r.get(0),
    )?;
    let agent_runs_failed_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ai_agent_runs WHERE workspace_id = ?1 AND status = 'failed' AND started_at >= ?2",
        rusqlite::params![workspace_id, since],
        |r| r.get(0),
    )?;
    let agent_runs_failed_today = graph_runs_failed_today + agent_runs_failed_today;

    let draft_screens: i64 = conn.query_row(
        "SELECT COUNT(*) FROM screen_layouts WHERE workspace_id = ?1 AND (published_json IS NULL OR draft_json != published_json)",
        [workspace_id],
        |r| r.get(0),
    )?;
    let draft_pages: i64 = conn.query_row(
        "SELECT COUNT(*) FROM page_layouts WHERE workspace_id = ?1 AND (published_json IS NULL OR draft_json != published_json)",
        [workspace_id],
        |r| r.get(0),
    )?;
    let draft_dashboards: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dashboard_layouts WHERE workspace_id = ?1 AND (published_json IS NULL OR draft_json != published_json)",
        [workspace_id],
        |r| r.get(0),
    )?;
    let draft_themes = count_one(conn, "SELECT COUNT(*) FROM workspace_themes WHERE workspace_id = ?1 AND status = 'draft'", workspace_id)?;
    let draft_graphs = count_one(conn, "SELECT COUNT(*) FROM ai_execution_graphs WHERE workspace_id = ?1 AND status = 'draft'", workspace_id)?;
    let unpublished_items = draft_screens + draft_pages + draft_dashboards + draft_themes + draft_graphs;

    let last_backup_at: Option<String> = conn
        .query_row(
            "SELECT occurred_at FROM audit_events WHERE workspace_id = ?1 AND event_type = 'backup' ORDER BY occurred_at DESC LIMIT 1",
            [workspace_id],
            |r| r.get(0),
        )
        .ok();

    let expiring_credentials: i64 = conn.query_row(
        "SELECT COUNT(*) FROM integration_connections WHERE workspace_id = ?1 AND credential_expires_at IS NOT NULL AND credential_expires_at <= ?2",
        rusqlite::params![workspace_id, soon_cutoff_iso()],
        |r| r.get(0),
    )?;

    // --- Needs Attention (only surfaces non-zero items) ---------------------
    let mut needs_attention = Vec::new();
    if connections_failed > 0 {
        needs_attention.push(NeedsAttentionItem { key: "failed_integrations".into(), label: "Integration connection(s) failing".into(), count: connections_failed });
    }
    if jobs_failed_today > 0 {
        needs_attention.push(NeedsAttentionItem { key: "failed_jobs".into(), label: "Integration job(s) failed today".into(), count: jobs_failed_today });
    }
    if expiring_credentials > 0 {
        needs_attention.push(NeedsAttentionItem { key: "expiring_credentials".into(), label: "Connection credential(s) expiring within 14 days".into(), count: expiring_credentials });
    }
    if unpublished_items > 0 {
        needs_attention.push(NeedsAttentionItem { key: "unpublished_changes".into(), label: "Item(s) with unpublished changes".into(), count: unpublished_items });
    }
    if workflow_runs_failed_today > 0 {
        needs_attention.push(NeedsAttentionItem { key: "failed_workflows".into(), label: "Workflow run(s) failed today".into(), count: workflow_runs_failed_today });
    }
    if agent_runs_failed_today > 0 {
        needs_attention.push(NeedsAttentionItem { key: "failed_agents".into(), label: "Agent/team run(s) failed today".into(), count: agent_runs_failed_today });
    }
    if !setup_progress.has_backup {
        needs_attention.push(NeedsAttentionItem { key: "no_backup".into(), label: "No backup has ever been taken".into(), count: 1 });
    }

    let platform_health = PlatformHealth {
        integration_connections_failed: connections_failed,
        integration_jobs_running: jobs_running,
        integration_jobs_failed_today: jobs_failed_today,
        workflow_runs_failed_today,
        agent_runs_failed_today,
        unpublished_items,
        last_backup_at,
    };

    Ok(AdminHomeSummary { setup_progress, needs_attention, platform_health })
}
