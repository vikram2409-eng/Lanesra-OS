//! AI Agent Platform v2, Phase 2: raw CRUD for `ai_agent_policies`
//! (migration `0061_agent_policy_engine.sql`) - one row per (workspace,
//! agent), or a workspace-wide default when `agent_id` is `None`. See
//! `services::policy_engine_service` for how a policy is resolved and
//! evaluated.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_agent_policy::{AiAgentPolicy, AiAgentPolicyInput};
use crate::models::ai_tool_registry::RiskLevel;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentPolicy> {
    let require_approval: Option<String> = row.get("require_approval_at_or_above")?;
    let blocked_json: String = row.get("blocked_tool_names_json")?;
    Ok(AiAgentPolicy {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        agent_id: row.get("agent_id")?,
        require_approval_at_or_above: require_approval.and_then(|s| RiskLevel::from_str(&s)),
        blocked_tool_names: serde_json::from_str(&blocked_json).unwrap_or_default(),
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

/// `agent_id = None` looks up the workspace-wide default policy.
pub fn get(conn: &Connection, workspace_id: &str, agent_id: Option<&str>) -> rusqlite::Result<Option<AiAgentPolicy>> {
    match agent_id {
        Some(agent_id) => conn
            .query_row("SELECT * FROM ai_agent_policies WHERE workspace_id = ?1 AND agent_id = ?2", rusqlite::params![workspace_id, agent_id], map_row)
            .optional(),
        None => conn.query_row("SELECT * FROM ai_agent_policies WHERE workspace_id = ?1 AND agent_id IS NULL", [workspace_id], map_row).optional(),
    }
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<AiAgentPolicy>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_policies WHERE workspace_id = ?1 ORDER BY agent_id IS NULL DESC, created_at ASC")?;
    let rows = stmt.query_map([workspace_id], map_row)?.collect();
    rows
}

/// Upsert-by-`(workspace_id, agent_id)` - `agent_id = None` upserts the
/// one workspace-default row (`idx_ai_agent_policies_one_default` is what
/// actually enforces there's at most one).
pub fn upsert(conn: &Connection, workspace_id: &str, agent_id: Option<&str>, input: &AiAgentPolicyInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentPolicy> {
    let now = now_iso();
    let require_approval = input.require_approval_at_or_above.map(|r| r.as_str());
    let blocked_json = serde_json::to_string(&input.blocked_tool_names).unwrap_or_else(|_| "[]".into());
    let existing = get(conn, workspace_id, agent_id)?;
    if let Some(existing) = existing {
        conn.execute(
            "UPDATE ai_agent_policies SET require_approval_at_or_above = ?1, blocked_tool_names_json = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?5",
            rusqlite::params![require_approval, blocked_json, now, actor_user_id, existing.id],
        )?;
    } else {
        let id = new_uuid();
        conn.execute(
            "INSERT INTO ai_agent_policies (id, workspace_id, agent_id, require_approval_at_or_above, blocked_tool_names_json, created_at, created_by, updated_at, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?6, ?7)",
            rusqlite::params![id, workspace_id, agent_id, require_approval, blocked_json, now, actor_user_id],
        )?;
    }
    Ok(get(conn, workspace_id, agent_id)?.expect("just upserted"))
}
