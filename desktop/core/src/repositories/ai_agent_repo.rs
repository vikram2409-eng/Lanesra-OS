//! Raw CRUD for `ai_agents`/`ai_agent_delegates`/`ai_skills`/
//! `ai_agent_skills` (migration 0038) - the AI Agent Foundry's Agents and
//! Skills. See `services::ai_agent_service` for validation/gating and
//! `services::chat_service` for how an agent actually runs.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::ai::AiAgentModelRouting;
use crate::models::ai_agent::{AiAgentDefinition, AiAgentInput, AiAgentMemorySnapshot, AiSkill, AiSkillInput};

fn map_agent_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentDefinition> {
    let action_names_json: String = row.get("action_names_json")?;
    Ok(AiAgentDefinition {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        icon: row.get("icon")?,
        system_prompt: row.get("system_prompt")?,
        memory_md: row.get("memory_md")?,
        action_names: serde_json::from_str(&action_names_json).unwrap_or_default(),
        // Filled in by `hydrate` - not a column on this table.
        delegate_agent_ids: Vec::new(),
        skill_ids: Vec::new(),
        model_routing: None,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn list_delegate_ids(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT delegate_agent_id FROM ai_agent_delegates WHERE agent_id = ?1")?;
    let rows = stmt.query_map([agent_id], |r| r.get::<_, String>(0))?;
    rows.collect()
}

pub fn list_skill_ids(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT skill_id FROM ai_agent_skills WHERE agent_id = ?1")?;
    let rows = stmt.query_map([agent_id], |r| r.get::<_, String>(0))?;
    rows.collect()
}

fn hydrate(conn: &Connection, mut agent: AiAgentDefinition) -> rusqlite::Result<AiAgentDefinition> {
    agent.delegate_agent_ids = list_delegate_ids(conn, &agent.id)?;
    agent.skill_ids = list_skill_ids(conn, &agent.id)?;
    agent.model_routing = get_routing(conn, &agent.id)?;
    Ok(agent)
}

// --- Phase 7a: per-agent Gateway routing policy ----------------------------

fn map_routing_row(row: &rusqlite::Row) -> rusqlite::Result<AiAgentModelRouting> {
    let force_air_gapped_for_json: String = row.get("force_air_gapped_for_json")?;
    Ok(AiAgentModelRouting {
        primary_provider_id: row.get("primary_provider_id")?,
        fallback_provider_id: row.get("fallback_provider_id")?,
        local_fallback_provider_id: row.get("local_fallback_provider_id")?,
        temperature: row.get("temperature")?,
        max_tokens: row.get("max_tokens")?,
        daily_token_budget: row.get("daily_token_budget")?,
        force_air_gapped_for: serde_json::from_str(&force_air_gapped_for_json).unwrap_or_default(),
    })
}

pub fn get_routing(conn: &Connection, agent_id: &str) -> rusqlite::Result<Option<AiAgentModelRouting>> {
    conn.query_row("SELECT * FROM ai_agent_routing WHERE agent_id = ?1", [agent_id], map_routing_row).optional()
}

/// A full overwrite (delete-then-insert, or NULL-everything if `routing`
/// is `None`) - same "no merge logic, the caller sends the complete
/// document" shape `update_memory` already uses for `memory_md`.
pub fn set_routing(conn: &Connection, agent_id: &str, routing: Option<&AiAgentModelRouting>) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_routing WHERE agent_id = ?1", [agent_id])?;
    if let Some(r) = routing {
        let force_air_gapped_for_json = serde_json::to_string(&r.force_air_gapped_for).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT INTO ai_agent_routing (agent_id, primary_provider_id, fallback_provider_id, local_fallback_provider_id, temperature, max_tokens, daily_token_budget, force_air_gapped_for_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                agent_id,
                r.primary_provider_id,
                r.fallback_provider_id,
                r.local_fallback_provider_id,
                r.temperature,
                r.max_tokens,
                r.daily_token_budget,
                force_air_gapped_for_json,
            ],
        )?;
    }
    Ok(())
}

fn replace_delegates(conn: &Connection, agent_id: &str, delegate_ids: &[String]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_delegates WHERE agent_id = ?1", [agent_id])?;
    for d in delegate_ids {
        conn.execute("INSERT OR IGNORE INTO ai_agent_delegates (agent_id, delegate_agent_id) VALUES (?1, ?2)", (agent_id, d))?;
    }
    Ok(())
}

fn replace_skills(conn: &Connection, agent_id: &str, skill_ids: &[String]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_agent_skills WHERE agent_id = ?1", [agent_id])?;
    for s in skill_ids {
        conn.execute("INSERT OR IGNORE INTO ai_agent_skills (agent_id, skill_id) VALUES (?1, ?2)", (agent_id, s))?;
    }
    Ok(())
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &AiAgentInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentDefinition> {
    let now = now_iso();
    let action_names_json = serde_json::to_string(&input.action_names).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO ai_agents (id, workspace_id, name, description, icon, system_prompt, memory_md, action_names_json, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', ?7, 1, ?8, ?9, ?8, ?9)",
        rusqlite::params![id, workspace_id, input.name, input.description, input.icon, input.system_prompt, action_names_json, now, actor_user_id],
    )?;
    replace_delegates(conn, id, &input.delegate_agent_ids)?;
    replace_skills(conn, id, &input.skill_ids)?;
    get(conn, id).map(|a| a.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &AiAgentInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentDefinition> {
    let now = now_iso();
    let action_names_json = serde_json::to_string(&input.action_names).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "UPDATE ai_agents SET name = ?1, description = ?2, icon = ?3, system_prompt = ?4, action_names_json = ?5, updated_at = ?6, updated_by = ?7 WHERE id = ?8",
        rusqlite::params![input.name, input.description, input.icon, input.system_prompt, action_names_json, now, actor_user_id, id],
    )?;
    replace_delegates(conn, id, &input.delegate_agent_ids)?;
    replace_skills(conn, id, &input.skill_ids)?;
    get(conn, id).map(|a| a.expect("just updated"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiAgentDefinition>> {
    let row = conn.query_row("SELECT * FROM ai_agents WHERE id = ?1", [id], map_agent_row).optional()?;
    row.map(|a| hydrate(conn, a)).transpose()
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> rusqlite::Result<Vec<AiAgentDefinition>> {
    let sql = if active_only {
        "SELECT * FROM ai_agents WHERE workspace_id = ?1 AND is_active = 1 ORDER BY name"
    } else {
        "SELECT * FROM ai_agents WHERE workspace_id = ?1 ORDER BY name"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<AiAgentDefinition> = stmt.query_map([workspace_id], map_agent_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|a| hydrate(conn, a)).collect()
}

pub fn set_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<AiAgentDefinition> {
    let now = now_iso();
    conn.execute("UPDATE ai_agents SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", rusqlite::params![is_active, now, actor_user_id, id])?;
    get(conn, id).map(|a| a.expect("just updated"))
}

/// The agent's own `update_memory` tool call, and an admin's direct edit
/// in its form, both go through this - a full overwrite, no merge logic
/// (see `ai_agent.rs`'s own doc comment on why).
/// Phase 7b: snapshots the *current* `memory_md` into
/// `ai_agent_memory_history` before overwriting it - `changed_by` is
/// `"agent"` for the always-available `update_memory` tool
/// (`chat_service::execute_agent_tool`), or the acting admin's user id for
/// a direct edit (`ai_agent_service::set_memory`) - both paths go through
/// this one function, so the history table can't be bypassed. No snapshot
/// is written when the prior value was empty (nothing written yet - see
/// migration 0038's own doc comment on `memory_md`'s `''` sentinel) or
/// identical to the new value (a genuine no-op write).
pub fn update_memory(conn: &Connection, id: &str, memory_md: &str, changed_by: &str) -> rusqlite::Result<()> {
    let existing: Option<String> = conn.query_row("SELECT memory_md FROM ai_agents WHERE id = ?1", [id], |r| r.get(0)).optional()?;
    if let Some(prev) = existing {
        if !prev.is_empty() && prev != memory_md {
            conn.execute(
                "INSERT INTO ai_agent_memory_history (id, agent_id, memory_md, changed_by, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![crate::domain::ids::new_uuid(), id, prev, changed_by, now_iso()],
            )?;
        }
    }
    conn.execute("UPDATE ai_agents SET memory_md = ?1 WHERE id = ?2", (memory_md, id))?;
    Ok(())
}

/// Most recent first - the natural order for an admin reviewing "what has
/// this agent learned/changed over time".
pub fn list_memory_history(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<AiAgentMemorySnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, memory_md, changed_by, created_at FROM ai_agent_memory_history WHERE agent_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([agent_id], |r| {
        Ok(AiAgentMemorySnapshot { id: r.get(0)?, agent_id: r.get(1)?, memory_md: r.get(2)?, changed_by: r.get(3)?, created_at: r.get(4)? })
    })?;
    rows.collect()
}

// --- Skills ---------------------------------------------------------------

fn map_skill_row(row: &rusqlite::Row) -> rusqlite::Result<AiSkill> {
    Ok(AiSkill {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        instructions_md: row.get("instructions_md")?,
        is_active: row.get("is_active")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

pub fn create_skill(conn: &Connection, id: &str, workspace_id: &str, input: &AiSkillInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiSkill> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_skills (id, workspace_id, name, description, instructions_md, is_active, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?6, ?7)",
        rusqlite::params![id, workspace_id, input.name, input.description, input.instructions_md, now, actor_user_id],
    )?;
    get_skill(conn, id).map(|s| s.expect("just inserted"))
}

pub fn update_skill(conn: &Connection, id: &str, input: &AiSkillInput, actor_user_id: Option<&str>) -> rusqlite::Result<AiSkill> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_skills SET name = ?1, description = ?2, instructions_md = ?3, updated_at = ?4, updated_by = ?5 WHERE id = ?6",
        rusqlite::params![input.name, input.description, input.instructions_md, now, actor_user_id, id],
    )?;
    get_skill(conn, id).map(|s| s.expect("just updated"))
}

pub fn get_skill(conn: &Connection, id: &str) -> rusqlite::Result<Option<AiSkill>> {
    conn.query_row("SELECT * FROM ai_skills WHERE id = ?1", [id], map_skill_row).optional()
}

pub fn list_skills(conn: &Connection, workspace_id: &str, active_only: bool) -> rusqlite::Result<Vec<AiSkill>> {
    let sql = if active_only {
        "SELECT * FROM ai_skills WHERE workspace_id = ?1 AND is_active = 1 ORDER BY name"
    } else {
        "SELECT * FROM ai_skills WHERE workspace_id = ?1 ORDER BY name"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([workspace_id], map_skill_row)?.collect();
    rows
}

pub fn set_skill_active(conn: &Connection, id: &str, is_active: bool, actor_user_id: Option<&str>) -> rusqlite::Result<AiSkill> {
    let now = now_iso();
    conn.execute("UPDATE ai_skills SET is_active = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4", rusqlite::params![is_active, now, actor_user_id, id])?;
    get_skill(conn, id).map(|s| s.expect("just updated"))
}
