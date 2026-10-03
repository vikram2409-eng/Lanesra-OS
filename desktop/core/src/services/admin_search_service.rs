//! Admin Control Center Modernization (issue #197): "Global Admin Search
//! across labels/descriptions/component/object/field names with
//! direct-open results." Deliberately a separate module from
//! `search_service::global_search` - that one is proven, by its own doc
//! comment, to be record search only (Companies/Contacts/.../Custom
//! Object records); this one searches admin *metadata* (the definitions
//! themselves - a Custom Object's own label, a Business Rule's own name,
//! not a record of either), admin-only, and each hit carries the
//! `AdminTab` key the frontend's `Settings.tsx` landing page already
//! uses to navigate there directly.
//!
//! Same "a handful of per-type `LIKE` queries, capped total results, no
//! ranking beyond source order" shape `search_service::global_search`
//! already established - not a second search engine, the same one
//! pattern applied to a different table set.

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domain::AppResult;

const MAX_RESULTS: usize = 25;

#[derive(Debug, Clone, Serialize)]
pub struct AdminSearchResult {
    pub category: String,
    pub entity_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    /// The `AdminTab` key `Settings.tsx`'s landing page already switches
    /// on - lets a result open directly instead of just naming a screen.
    pub admin_tab: String,
}

fn like_pattern(query: &str) -> String {
    format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
}

fn collect(
    conn: &Connection,
    out: &mut Vec<AdminSearchResult>,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
    row_fn: impl Fn(&rusqlite::Row) -> rusqlite::Result<AdminSearchResult>,
) -> AppResult<()> {
    if out.len() >= MAX_RESULTS {
        return Ok(());
    }
    let remaining = (MAX_RESULTS - out.len()) as i64;
    let sql = format!("{sql} LIMIT {remaining}");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params, row_fn)?;
    for row in rows {
        out.push(row?);
    }
    Ok(())
}

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

pub fn admin_search(conn: &Connection, workspace_id: &str, query: &str, actor_user_id: Option<&str>) -> AppResult<Vec<AdminSearchResult>> {
    require_admin(conn, actor_user_id)?;
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }
    let like = like_pattern(query);
    let mut out = Vec::new();

    collect(
        conn, &mut out,
        "SELECT id, singular_label, plural_label FROM custom_object_definitions WHERE workspace_id = ?1 AND (singular_label LIKE ?2 ESCAPE '\\' OR plural_label LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| Ok(AdminSearchResult { category: "Custom Object".into(), entity_id: r.get("id")?, title: r.get("plural_label")?, subtitle: None, admin_tab: "objects".into() }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, entity_type, label FROM custom_field_definitions WHERE workspace_id = ?1 AND (label LIKE ?2 ESCAPE '\\' OR key LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let entity_type: String = r.get("entity_type")?;
            Ok(AdminSearchResult { category: "Custom Field".into(), entity_id: r.get("id")?, title: r.get("label")?, subtitle: Some(entity_type), admin_tab: "fields".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, entity_type, name FROM business_rules WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| {
            let entity_type: String = r.get("entity_type")?;
            Ok(AdminSearchResult { category: "Business Rule".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(entity_type), admin_tab: "rules".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, entity_type, name FROM workflow_definitions WHERE workspace_id = ?1 AND (name LIKE ?2 ESCAPE '\\' OR description LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let entity_type: String = r.get("entity_type")?;
            Ok(AdminSearchResult { category: "Workflow".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(entity_type), admin_tab: "workflow".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, entity_type, name FROM screen_layouts WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| {
            let entity_type: String = r.get("entity_type")?;
            Ok(AdminSearchResult { category: "Screen Layout".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(entity_type), admin_tab: "layouts".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, entity_type, name FROM page_layouts WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| {
            let entity_type: String = r.get("entity_type")?;
            Ok(AdminSearchResult { category: "Page".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(entity_type), admin_tab: "pageBuilder".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name FROM dashboard_layouts WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| Ok(AdminSearchResult { category: "Dashboard".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: None, admin_tab: "dashboards".into() }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name, description FROM ai_agents WHERE workspace_id = ?1 AND is_active = 1 AND (name LIKE ?2 ESCAPE '\\' OR description LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let description: Option<String> = r.get("description")?;
            Ok(AdminSearchResult { category: "AI Agent".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: description, admin_tab: "aiAgents".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name, description FROM ai_skills WHERE workspace_id = ?1 AND is_active = 1 AND (name LIKE ?2 ESCAPE '\\' OR description LIKE ?2 ESCAPE '\\')",
        params![workspace_id, like],
        |r| {
            let description: String = r.get("description")?;
            Ok(AdminSearchResult { category: "Skill".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(description), admin_tab: "aiSkills".into() })
        },
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name FROM ai_execution_graphs WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| Ok(AdminSearchResult { category: "Agent Team".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: None, admin_tab: "agentTeams".into() }),
    )?;

    collect(
        conn, &mut out,
        "SELECT id, name, connection_type FROM integration_connections WHERE workspace_id = ?1 AND name LIKE ?2 ESCAPE '\\'",
        params![workspace_id, like],
        |r| {
            let connection_type: String = r.get("connection_type")?;
            Ok(AdminSearchResult { category: "Integration Connection".into(), entity_id: r.get("id")?, title: r.get("name")?, subtitle: Some(connection_type), admin_tab: "integrations".into() })
        },
    )?;

    Ok(out)
}
