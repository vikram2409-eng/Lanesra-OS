-- AI Agent Platform v2 (GitHub issue #170, backend half): the MCP
-- *client* role - Lanesra calling OUT to an external MCP server, the
-- mirror image of the MCP *server* role `desktop/server/src/mcp.rs`
-- already exposes (that endpoint's own doc comment is the exact
-- initialize/tools-list/tools-call/isError JSON-RPC 2.0 contract this
-- client speaks against).
--
-- An external MCP server is modeled as an ordinary `integration_connections`
-- row (`connection_type = 'mcp'`, added to connection_service::
-- CONNECTION_TYPES) so it gets the exact same encrypted-secret handling
-- (AES-256-GCM via secret_service, `auth_mode` applied the same way a
-- REST Connection's own bearer/api_key/basic auth already is) as every
-- other Integration Hub connection - no second secret-storage mechanism.
-- ai_mcp_servers layers the MCP-specific bits (which tools this server's
-- last `tools/list` discovered, whether they're exposed to agents at
-- all) on top of that connection rather than inside it, the same
-- "Connection is generic, a feature-specific table points at it" shape
-- `integration_external_objects`/`integration_webhooks` already use.
CREATE TABLE ai_mcp_servers (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    connection_id TEXT NOT NULL REFERENCES integration_connections(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    -- Mirrors integration_connectors' own agent_tools_enabled/
    -- agent_write_tools_enabled pair (migration 0050) exactly: a master
    -- switch for exposing this server's enabled tools to agents at all,
    -- plus a second, separate switch a mutating tool also needs before
    -- it's ever offered - see ai_mcp_tools.is_write below for why that
    -- read/write split has to live per-tool here rather than being
    -- inferred from an HTTP method the way a Connector Action's is.
    agent_tools_enabled INTEGER NOT NULL DEFAULT 0,
    agent_write_tools_enabled INTEGER NOT NULL DEFAULT 0,
    last_discovered_at TEXT,
    last_discovery_status TEXT,
    last_discovery_message TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_ai_mcp_servers_workspace ON ai_mcp_servers(workspace_id);

-- One row per tool this server's own `tools/list` reported, refreshed
-- wholesale (delete-then-reinsert) on every discovery run so a tool the
-- external server has since removed doesn't linger. `is_write` has no
-- source of truth to infer from the way a Connector Action's HTTP verb
-- gives one - an MCP tool's own JSON-RPC shape carries no read/write
-- signal - so an administrator classifies it explicitly (defaults to
-- read-only, the conservative default) before it can ever be enabled
-- for write-tool exposure. `enabled` is the actual per-tool allowlist an
-- admin opts into individually; a freshly-discovered tool is never
-- auto-enabled.
CREATE TABLE ai_mcp_tools (
    id TEXT PRIMARY KEY,
    mcp_server_id TEXT NOT NULL REFERENCES ai_mcp_servers(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    input_schema_json TEXT NOT NULL DEFAULT '{}',
    is_write INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 0,
    discovered_at TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_ai_mcp_tools_server_name ON ai_mcp_tools(mcp_server_id, tool_name);
