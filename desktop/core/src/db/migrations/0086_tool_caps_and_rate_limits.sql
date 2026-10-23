-- Migration 0086: per-tool caps on an agent's tools, and tool rate limits (WP-0140)
-- Work package: WP-0140
-- Change: additive (four tables, two indexes)
-- Data touched: none; with no rows nothing is capped or limited, as before
-- Re-run safe: no (CREATE TABLE)
--
-- ai_agent_tool_caps: an agent's binding to one tool can cap how many times
-- one step calls it, disable it after a number of failures in a row, and cap
-- how much of its result stays in the model's context (the rest is stored in
-- ai_tool_results and referenced). consecutive_failures and disabled_at are
-- the binding's live state; an Administrator re-enables a disabled binding.
--
-- ai_tool_rate_limits: at most max_calls in per_seconds, for one tool, one
-- connector (any of its actions) or the whole workspace (scope_key '*'),
-- checked by the Tool-Call Firewall before a call. ai_tool_call_window holds
-- the recent calls those limits count.
CREATE TABLE ai_agent_tool_caps (
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    max_calls_per_step INTEGER CHECK (max_calls_per_step IS NULL OR max_calls_per_step > 0),
    failures_before_disable INTEGER CHECK (failures_before_disable IS NULL OR failures_before_disable > 0),
    max_result_chars INTEGER CHECK (max_result_chars IS NULL OR max_result_chars >= 100),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    disabled_at TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    PRIMARY KEY (agent_id, tool_name)
);

CREATE TABLE ai_tool_rate_limits (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    scope TEXT NOT NULL CHECK (scope IN ('tool', 'connector', 'workspace')),
    scope_key TEXT NOT NULL,
    max_calls INTEGER NOT NULL CHECK (max_calls > 0),
    per_seconds INTEGER NOT NULL CHECK (per_seconds > 0),
    created_at TEXT NOT NULL,
    created_by TEXT,
    UNIQUE (workspace_id, scope, scope_key)
);

CREATE TABLE ai_tool_call_window (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    connector_id TEXT,
    called_at_ms INTEGER NOT NULL
);
CREATE INDEX idx_ai_tool_call_window ON ai_tool_call_window(workspace_id, called_at_ms);

CREATE TABLE ai_tool_results (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id TEXT,
    tool_name TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_tool_results_workspace ON ai_tool_results(workspace_id, created_at);
