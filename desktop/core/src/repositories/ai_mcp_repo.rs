//! Raw CRUD for `ai_mcp_servers`/`ai_mcp_tools` (migration
//! `0064_mcp_client.sql`). `McpServer.base_url`/`auth_mode` are read off
//! the underlying `integration_connections` row via a join - this table
//! never duplicates those columns, see this module's own migration doc
//! comment. See `services::mcp_client_service` for discovery, the real
//! outbound tool call, and the Tool-Call Firewall bridge.

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::ai_mcp::{DiscoveredTool, McpServer, McpTool};

fn map_server_row(row: &rusqlite::Row) -> rusqlite::Result<McpServer> {
    Ok(McpServer {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        connection_id: row.get("connection_id")?,
        name: row.get("name")?,
        base_url: row.get("base_url")?,
        auth_mode: row.get("auth_mode")?,
        agent_tools_enabled: row.get::<_, i64>("agent_tools_enabled")? != 0,
        agent_write_tools_enabled: row.get::<_, i64>("agent_write_tools_enabled")? != 0,
        last_discovered_at: row.get("last_discovered_at")?,
        last_discovery_status: row.get("last_discovery_status")?,
        last_discovery_message: row.get("last_discovery_message")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

const SERVER_SELECT: &str = "SELECT s.id, s.workspace_id, s.connection_id, s.name, c.base_url, c.auth_mode,
        s.agent_tools_enabled, s.agent_write_tools_enabled, s.last_discovered_at, s.last_discovery_status,
        s.last_discovery_message, s.created_at, s.created_by, s.updated_at, s.updated_by
     FROM ai_mcp_servers s JOIN integration_connections c ON c.id = s.connection_id";

pub fn create_server(conn: &Connection, workspace_id: &str, connection_id: &str, name: &str, actor_user_id: Option<&str>) -> rusqlite::Result<McpServer> {
    let id = new_uuid();
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_mcp_servers (id, workspace_id, connection_id, name, agent_tools_enabled, agent_write_tools_enabled, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, 0, 0, ?5, ?6, ?5, ?6)",
        rusqlite::params![id, workspace_id, connection_id, name, now, actor_user_id],
    )?;
    get_server(conn, &id).map(|r| r.expect("just inserted"))
}

pub fn get_server(conn: &Connection, id: &str) -> rusqlite::Result<Option<McpServer>> {
    conn.query_row(&format!("{SERVER_SELECT} WHERE s.id = ?1"), [id], map_server_row).optional()
}

pub fn list_servers_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<McpServer>> {
    let mut stmt = conn.prepare(&format!("{SERVER_SELECT} WHERE s.workspace_id = ?1 ORDER BY s.name"))?;
    let rows = stmt.query_map([workspace_id], map_server_row)?.collect();
    rows
}

pub fn update_server(conn: &Connection, id: &str, name: &str, agent_tools_enabled: bool, agent_write_tools_enabled: bool, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_mcp_servers SET name = ?1, agent_tools_enabled = ?2, agent_write_tools_enabled = ?3, updated_at = ?4, updated_by = ?5 WHERE id = ?6",
        rusqlite::params![name, agent_tools_enabled as i64, agent_write_tools_enabled as i64, now_iso(), actor_user_id, id],
    )?;
    Ok(())
}

pub fn set_discovery_result(conn: &Connection, id: &str, status: &str, message: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_mcp_servers SET last_discovered_at = ?1, last_discovery_status = ?2, last_discovery_message = ?3 WHERE id = ?4",
        rusqlite::params![now_iso(), status, message, id],
    )?;
    Ok(())
}

pub fn delete_server(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM ai_mcp_servers WHERE id = ?1", [id])?;
    Ok(())
}

fn map_tool_row(row: &rusqlite::Row) -> rusqlite::Result<McpTool> {
    Ok(McpTool {
        id: row.get("id")?,
        mcp_server_id: row.get("mcp_server_id")?,
        tool_name: row.get("tool_name")?,
        description: row.get("description")?,
        input_schema_json: row.get("input_schema_json")?,
        is_write: row.get::<_, i64>("is_write")? != 0,
        enabled: row.get::<_, i64>("enabled")? != 0,
        discovered_at: row.get("discovered_at")?,
    })
}

pub fn list_tools(conn: &Connection, mcp_server_id: &str) -> rusqlite::Result<Vec<McpTool>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_mcp_tools WHERE mcp_server_id = ?1 ORDER BY tool_name")?;
    let rows = stmt.query_map([mcp_server_id], map_tool_row)?.collect();
    rows
}

pub fn get_tool(conn: &Connection, mcp_server_id: &str, tool_name: &str) -> rusqlite::Result<Option<McpTool>> {
    conn.query_row("SELECT * FROM ai_mcp_tools WHERE mcp_server_id = ?1 AND tool_name = ?2", rusqlite::params![mcp_server_id, tool_name], map_tool_row).optional()
}

/// Replaces every tool row for `mcp_server_id` with a fresh discovery
/// result - a tool the external server no longer reports is dropped, a
/// genuinely new one is inserted as disabled/read-only (the conservative
/// default, see migration's own comment), and a tool present in both the
/// old and new set keeps whatever `is_write`/`enabled` an admin already
/// set on it rather than resetting to the default on every re-discovery.
pub fn replace_tools(conn: &Connection, mcp_server_id: &str, discovered: &[DiscoveredTool]) -> rusqlite::Result<Vec<McpTool>> {
    let existing = list_tools(conn, mcp_server_id)?;
    let now = now_iso();
    conn.execute("DELETE FROM ai_mcp_tools WHERE mcp_server_id = ?1", [mcp_server_id])?;
    for tool in discovered {
        let carried = existing.iter().find(|t| t.tool_name == tool.tool_name);
        let is_write = carried.map(|t| t.is_write).unwrap_or(false);
        let enabled = carried.map(|t| t.enabled).unwrap_or(false);
        conn.execute(
            "INSERT INTO ai_mcp_tools (id, mcp_server_id, tool_name, description, input_schema_json, is_write, enabled, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![new_uuid(), mcp_server_id, tool.tool_name, tool.description, tool.input_schema_json, is_write as i64, enabled as i64, now],
        )?;
    }
    list_tools(conn, mcp_server_id)
}

pub fn set_tool_flags(conn: &Connection, mcp_server_id: &str, tool_name: &str, is_write: bool, enabled: bool) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE ai_mcp_tools SET is_write = ?1, enabled = ?2 WHERE mcp_server_id = ?3 AND tool_name = ?4",
        rusqlite::params![is_write as i64, enabled as i64, mcp_server_id, tool_name],
    )
}

/// Every `(server, tool)` pair currently eligible to be offered as an
/// agent tool at all - `agent_tools_enabled` on the server and `enabled`
/// on the tool - across the whole workspace. `mcp_client_service::
/// build_entries` applies the second, per-tool `agent_write_tools_enabled`
/// gate on top of this (a plain SQL join can't express "unless is_write
/// and the server's write gate is off" as cleanly as one more filter in
/// Rust can).
pub fn list_enabled_tools_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<(McpServer, McpTool)>> {
    let servers = list_servers_for_workspace(conn, workspace_id)?;
    let mut out = Vec::new();
    for server in servers {
        if !server.agent_tools_enabled {
            continue;
        }
        for tool in list_tools(conn, &server.id)? {
            if tool.enabled {
                out.push((server.clone(), tool));
            }
        }
    }
    Ok(out)
}
