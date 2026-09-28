//! AI Agent Platform v2, Phase 1: raw CRUD for `ai_agent_versions`
//! (migration `0059_ai_agent_versioning.sql`). Every agent already gets a
//! `v1` row from either the migration's own backfill or
//! `ai_agent_repo::create_initial_version` - this file is what
//! `services::agent_version_service` uses to move a version through
//! Draft -> Test -> Published -> Deprecated -> Disabled from then on.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai::AiAgentModelRouting;
use crate::models::ai_agent::{AiAgentVersion, AiAgentVersionInput};

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentVersion> {
    let action_names_json: String = row.get("action_names_json")?;
    let delegate_agent_ids_json: String = row.get("delegate_agent_ids_json")?;
    let skill_ids_json: String = row.get("skill_ids_json")?;
    let model_routing_json: Option<String> = row.get("model_routing_json")?;
    let output_schema_json: Option<String> = row.get("output_schema_json")?;
    Ok(AiAgentVersion {
        id: row.get("id")?,
        agent_id: row.get("agent_id")?,
        version_number: row.get("version_number")?,
        status: row.get("status")?,
        name: row.get("name")?,
        description: row.get("description")?,
        icon: row.get("icon")?,
        system_prompt: row.get("system_prompt")?,
        memory_md: row.get("memory_md")?,
        guardrails_md: row.get("guardrails_md")?,
        action_names: serde_json::from_str(&action_names_json).unwrap_or_default(),
        delegate_agent_ids: serde_json::from_str(&delegate_agent_ids_json).unwrap_or_default(),
        skill_ids: serde_json::from_str(&skill_ids_json).unwrap_or_default(),
        model_routing: model_routing_json.and_then(|j| serde_json::from_str::<AiAgentModelRouting>(&j).ok()),
        output_schema: output_schema_json.and_then(|j| serde_json::from_str(&j).ok()),
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        published_at: row.get("published_at")?,
    })
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiAgentVersion>> {
    conn.query_row("SELECT * FROM ai_agent_versions WHERE id = ?1", [id], map_row).optional()
}

/// Most recent first - the natural order for a version-history tab.
pub fn list_by_agent(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<AiAgentVersion>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_agent_versions WHERE agent_id = ?1 ORDER BY version_number DESC")?;
    let rows = stmt.query_map([agent_id], map_row)?;
    rows.collect()
}

pub fn next_version_number(conn: &Connection, agent_id: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COALESCE(MAX(version_number), 0) + 1 FROM ai_agent_versions WHERE agent_id = ?1", [agent_id], |r| r.get(0))
}

/// Always inserted as `status = 'draft'` - `agent_version_service::publish`
/// is the only path that ever sets `status = 'published'`.
#[allow(clippy::too_many_arguments)]
pub fn create_draft(conn: &Connection, agent_id: &str, version_number: i64, input: &AiAgentVersionInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentVersion> {
    let id = new_uuid();
    let now = now_iso();
    let action_names_json = serde_json::to_string(&input.action_names).unwrap_or_else(|_| "[]".into());
    let delegate_agent_ids_json = serde_json::to_string(&input.delegate_agent_ids).unwrap_or_else(|_| "[]".into());
    let skill_ids_json = serde_json::to_string(&input.skill_ids).unwrap_or_else(|_| "[]".into());
    let model_routing_json = input.model_routing.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default());
    let output_schema_json = input.output_schema.as_ref().map(|s| s.to_string());
    conn.execute(
        "INSERT INTO ai_agent_versions (id, agent_id, version_number, status, name, description, icon, system_prompt, memory_md, guardrails_md, action_names_json, delegate_agent_ids_json, skill_ids_json, model_routing_json, output_schema_json, created_at, created_by)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, '', '', ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            id, agent_id, version_number, input.name, input.description, input.icon, input.system_prompt,
            action_names_json, delegate_agent_ids_json, skill_ids_json, model_routing_json, output_schema_json,
            now, actor_user_id,
        ],
    )?;
    Ok(get(conn, &id)?.expect("just inserted"))
}

/// A full content overwrite of a Draft/Test version - `agent_version_
/// service::update_draft` is the only caller, and it's the one that
/// enforces "only while status is draft/test", not this function.
pub fn update_content(conn: &Connection, id: &str, input: &AiAgentVersionInput) -> rusqlite::Result<AiAgentVersion> {
    let action_names_json = serde_json::to_string(&input.action_names).unwrap_or_else(|_| "[]".into());
    let delegate_agent_ids_json = serde_json::to_string(&input.delegate_agent_ids).unwrap_or_else(|_| "[]".into());
    let skill_ids_json = serde_json::to_string(&input.skill_ids).unwrap_or_else(|_| "[]".into());
    let model_routing_json = input.model_routing.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default());
    let output_schema_json = input.output_schema.as_ref().map(|s| s.to_string());
    conn.execute(
        "UPDATE ai_agent_versions SET name = ?1, description = ?2, icon = ?3, system_prompt = ?4, action_names_json = ?5, delegate_agent_ids_json = ?6, skill_ids_json = ?7, model_routing_json = ?8, output_schema_json = ?9 WHERE id = ?10",
        rusqlite::params![input.name, input.description, input.icon, input.system_prompt, action_names_json, delegate_agent_ids_json, skill_ids_json, model_routing_json, output_schema_json, id],
    )?;
    Ok(get(conn, id)?.expect("just updated"))
}

pub fn set_status(conn: &Connection, id: &str, status: &str, published_at: Option<&str>) -> rusqlite::Result<AiAgentVersion> {
    conn.execute("UPDATE ai_agent_versions SET status = ?1, published_at = COALESCE(?2, published_at) WHERE id = ?3", rusqlite::params![status, published_at, id])?;
    Ok(get(conn, id)?.expect("just updated"))
}

pub fn set_current_version(conn: &Connection, agent_id: &str, version_id: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE ai_agents SET current_version_id = ?1 WHERE id = ?2", rusqlite::params![version_id, agent_id])?;
    Ok(())
}
