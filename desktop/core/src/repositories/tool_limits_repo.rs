//! Raw CRUD for migration 0086's tool caps, rate limits, call window and
//! stored results. See `services::tool_limits_service`.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::tool_limits::{AgentToolCap, StoredToolResult, ToolRateLimit};

fn map_cap(row: &rusqlite::Row) -> rusqlite::Result<AgentToolCap> {
    Ok(AgentToolCap {
        agent_id: row.get("agent_id")?,
        tool_name: row.get("tool_name")?,
        max_calls_per_step: row.get("max_calls_per_step")?,
        failures_before_disable: row.get("failures_before_disable")?,
        max_result_chars: row.get("max_result_chars")?,
        consecutive_failures: row.get("consecutive_failures")?,
        disabled_at: row.get("disabled_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_caps(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<AgentToolCap>> {
    let mut stmt =
        conn.prepare("SELECT * FROM ai_agent_tool_caps WHERE agent_id = ?1 ORDER BY tool_name")?;
    let rows = stmt.query_map([agent_id], map_cap)?;
    rows.collect()
}

pub fn get_cap(
    conn: &Connection,
    agent_id: &str,
    tool_name: &str,
) -> rusqlite::Result<Option<AgentToolCap>> {
    conn.query_row(
        "SELECT * FROM ai_agent_tool_caps WHERE agent_id = ?1 AND tool_name = ?2",
        [agent_id, tool_name],
        map_cap,
    )
    .optional()
}

/// Writes a binding's caps, keeping its live failure state.
pub fn upsert_cap(
    conn: &Connection,
    agent_id: &str,
    tool_name: &str,
    max_calls_per_step: Option<i64>,
    failures_before_disable: Option<i64>,
    max_result_chars: Option<i64>,
    actor: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_agent_tool_caps (agent_id, tool_name, max_calls_per_step, failures_before_disable, max_result_chars, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(agent_id, tool_name) DO UPDATE SET
           max_calls_per_step = excluded.max_calls_per_step,
           failures_before_disable = excluded.failures_before_disable,
           max_result_chars = excluded.max_result_chars,
           updated_at = excluded.updated_at,
           updated_by = excluded.updated_by",
        rusqlite::params![
            agent_id,
            tool_name,
            max_calls_per_step,
            failures_before_disable,
            max_result_chars,
            now_iso(),
            actor
        ],
    )?;
    Ok(())
}

pub fn delete_caps_except(
    conn: &Connection,
    agent_id: &str,
    keep: &[String],
) -> rusqlite::Result<()> {
    for cap in list_caps(conn, agent_id)? {
        if !keep.contains(&cap.tool_name) {
            conn.execute(
                "DELETE FROM ai_agent_tool_caps WHERE agent_id = ?1 AND tool_name = ?2",
                [agent_id, &cap.tool_name],
            )?;
        }
    }
    Ok(())
}

/// Records one outcome; returns the binding's failures in a row after it.
pub fn record_outcome(
    conn: &Connection,
    agent_id: &str,
    tool_name: &str,
    ok: bool,
) -> rusqlite::Result<i64> {
    if ok {
        conn.execute(
            "UPDATE ai_agent_tool_caps SET consecutive_failures = 0 WHERE agent_id = ?1 AND tool_name = ?2",
            [agent_id, tool_name],
        )?;
    } else {
        conn.execute(
            "UPDATE ai_agent_tool_caps SET consecutive_failures = consecutive_failures + 1 WHERE agent_id = ?1 AND tool_name = ?2",
            [agent_id, tool_name],
        )?;
    }
    Ok(get_cap(conn, agent_id, tool_name)?
        .map(|c| c.consecutive_failures)
        .unwrap_or(0))
}

pub fn set_disabled(
    conn: &Connection,
    agent_id: &str,
    tool_name: &str,
    disabled: bool,
) -> rusqlite::Result<()> {
    if disabled {
        conn.execute(
            "UPDATE ai_agent_tool_caps SET disabled_at = ?3 WHERE agent_id = ?1 AND tool_name = ?2",
            rusqlite::params![agent_id, tool_name, now_iso()],
        )?;
    } else {
        conn.execute(
            "UPDATE ai_agent_tool_caps SET disabled_at = NULL, consecutive_failures = 0 WHERE agent_id = ?1 AND tool_name = ?2",
            [agent_id, tool_name],
        )?;
    }
    Ok(())
}

fn map_limit(row: &rusqlite::Row) -> rusqlite::Result<ToolRateLimit> {
    Ok(ToolRateLimit {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        scope: row.get("scope")?,
        scope_key: row.get("scope_key")?,
        max_calls: row.get("max_calls")?,
        per_seconds: row.get("per_seconds")?,
        created_at: row.get("created_at")?,
    })
}

pub fn list_limits(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ToolRateLimit>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_tool_rate_limits WHERE workspace_id = ?1 ORDER BY scope, scope_key",
    )?;
    let rows = stmt.query_map([workspace_id], map_limit)?;
    rows.collect()
}

#[allow(clippy::too_many_arguments)]
pub fn upsert_limit(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    scope: &str,
    scope_key: &str,
    max_calls: i64,
    per_seconds: i64,
    actor: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_tool_rate_limits (id, workspace_id, scope, scope_key, max_calls, per_seconds, created_at, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(workspace_id, scope, scope_key) DO UPDATE SET
           max_calls = excluded.max_calls, per_seconds = excluded.per_seconds",
        rusqlite::params![id, workspace_id, scope, scope_key, max_calls, per_seconds, now_iso(), actor],
    )?;
    Ok(())
}

pub fn delete_limit(conn: &Connection, workspace_id: &str, id: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM ai_tool_rate_limits WHERE workspace_id = ?1 AND id = ?2",
        [workspace_id, id],
    )
}

/// Calls in the window since `since_ms` matching the scope; returns the
/// count and the oldest matching call time.
pub fn count_calls_since(
    conn: &Connection,
    workspace_id: &str,
    scope: &str,
    scope_key: &str,
    since_ms: i64,
) -> rusqlite::Result<(i64, Option<i64>)> {
    let (filter, key): (&str, Option<&str>) = match scope {
        "tool" => ("AND tool_name = ?3", Some(scope_key)),
        "connector" => ("AND connector_id = ?3", Some(scope_key)),
        _ => ("", None),
    };
    let sql = format!(
        "SELECT COUNT(*), MIN(called_at_ms) FROM ai_tool_call_window WHERE workspace_id = ?1 AND called_at_ms > ?2 {filter}"
    );
    match key {
        Some(k) => conn.query_row(&sql, rusqlite::params![workspace_id, since_ms, k], |r| {
            Ok((r.get(0)?, r.get(1)?))
        }),
        None => conn.query_row(&sql, rusqlite::params![workspace_id, since_ms], |r| {
            Ok((r.get(0)?, r.get(1)?))
        }),
    }
}

pub fn record_call(
    conn: &Connection,
    workspace_id: &str,
    tool_name: &str,
    connector_id: Option<&str>,
    at_ms: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_tool_call_window (workspace_id, tool_name, connector_id, called_at_ms) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![workspace_id, tool_name, connector_id, at_ms],
    )?;
    Ok(())
}

pub fn prune_calls_before(
    conn: &Connection,
    workspace_id: &str,
    before_ms: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM ai_tool_call_window WHERE workspace_id = ?1 AND called_at_ms <= ?2",
        rusqlite::params![workspace_id, before_ms],
    )?;
    Ok(())
}

pub fn store_result(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    agent_id: Option<&str>,
    tool_name: &str,
    body: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO ai_tool_results (id, workspace_id, agent_id, tool_name, body, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, workspace_id, agent_id, tool_name, body, now_iso()],
    )?;
    Ok(())
}

pub fn get_result(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
) -> rusqlite::Result<Option<StoredToolResult>> {
    conn.query_row(
        "SELECT id, agent_id, tool_name, body, created_at FROM ai_tool_results WHERE workspace_id = ?1 AND id = ?2",
        [workspace_id, id],
        |r| {
            Ok(StoredToolResult {
                id: r.get(0)?,
                agent_id: r.get(1)?,
                tool_name: r.get(2)?,
                body: r.get(3)?,
                created_at: r.get(4)?,
            })
        },
    )
    .optional()
}
