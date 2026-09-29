-- AI Agent Platform v2, Phase 2: a unified Policy Engine and the Tool
-- Registry it evaluates against - the foundation the Tool-Call Firewall
-- (chat_service::execute_tool/execute_agent_tool, wired in this same
-- pass) checks before any tool actually dispatches.
--
-- Every tool call already funnels through one of two dispatchers
-- (dispatch_record_tool/dispatch_admin_tool, reached from either
-- execute_tool for the fixed "records"/"admin" assistants or
-- execute_agent_tool for a named AI Agent) - this migration doesn't add a
-- second dispatch path, it adds the two tables a policy check reads
-- before either dispatcher runs.
--
-- `ai_tool_registry` is workspace-scoped risk-level *overrides* only, not
-- a full catalog: `tool_registry_service::default_risk_for` classifies
-- every native/connector tool name by convention (list_*/get_* -> Read,
-- create_record/update_record -> LowWrite, an admin-catalog
-- create_*/set_* -> Write, a connector write action -> ExternalAction,
-- archive_record -> Destructive, an identity/credential/execution tool
-- like create_user/create_api_client/run_ai_agent -> Privileged) with no
-- row required for any of that - a workspace that never opens this
-- screen sees the exact same defaults. A row here only exists to
-- deliberately reclassify one tool up or down from its default.
CREATE TABLE ai_tool_registry (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    -- 'read' | 'low_write' | 'write' | 'external_action' | 'destructive' | 'privileged'.
    risk_level TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, tool_name)
);

-- One policy row per (workspace, agent) - `agent_id IS NULL` is the
-- workspace-wide default every fixed "records"/"admin" chat call and any
-- agent with no policy of its own falls back to. No row at all (neither
-- an agent-specific nor a workspace-default one) means every call is
-- Allowed unchanged - purely additive, no behavior change for a
-- workspace that never opens this screen, the same convention every
-- other opt-in governance surface in this codebase (Voice Governance,
-- DLP routing) already follows.
CREATE TABLE ai_agent_policies (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES ai_agents(id) ON DELETE CASCADE,
    -- A tool whose effective risk is at or above this level is queued for
    -- approval instead of dispatched immediately. NULL means "never
    -- require approval by risk level" (today's behavior).
    -- 'read' | 'low_write' | 'write' | 'external_action' | 'destructive' | 'privileged'.
    require_approval_at_or_above TEXT,
    -- Tool names always denied outright for this policy's scope,
    -- regardless of risk level - checked before the risk threshold above.
    blocked_tool_names_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    -- A partial unique index (SQLite supports NULLS treated as distinct in
    -- a regular UNIQUE, which is exactly what's wanted here - many
    -- NULL-agent_id rows would violate a naive UNIQUE(workspace_id,
    -- agent_id), so the one-default-per-workspace rule is enforced by a
    -- filtered index instead) - see idx_ai_agent_policies_one_default.
    UNIQUE (workspace_id, agent_id)
);
-- SQLite treats NULL as distinct in a UNIQUE constraint, so the UNIQUE
-- above already permits only one row per real agent_id but would allow
-- unlimited agent_id IS NULL rows per workspace; this partial index
-- closes that gap and is what actually enforces "at most one
-- workspace-default policy".
CREATE UNIQUE INDEX idx_ai_agent_policies_one_default ON ai_agent_policies (workspace_id) WHERE agent_id IS NULL;
