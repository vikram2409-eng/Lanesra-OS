-- AI & Agentic Layer, Phase 7a: the Unified AI Gateway - named,
-- admin-managed AI Providers (beyond the single workspace-wide default in
-- ai_settings), per-agent model routing across those providers
-- (primary/fallback/local_fallback with automatic failover), and real
-- token-usage accounting the routing layer's budget checks are enforced
-- against. ai_settings itself is unchanged - an agent with no routing
-- policy configured keeps using it exactly as before (backward
-- compatible with everything Phase 6a/6b already shipped) - see
-- services::ai_gateway_service's own doc comment.

-- A named provider connection - the same "several named entries, not one
-- workspace-wide row" shape Integration Hub's own Connections already
-- use, just for an LLM provider instead of a REST/SFTP/Postgres endpoint.
-- `provider` is the same vocabulary ai_settings.provider already uses,
-- plus the new `google_gemini` adapter this phase adds - see ai_service.rs.
CREATE TABLE ai_providers (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    base_url TEXT,
    model TEXT NOT NULL,
    secret_id TEXT REFERENCES integration_secrets(id) ON DELETE SET NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_providers_workspace ON ai_providers (workspace_id, is_active);

-- An agent's optional routing policy - which ai_providers row (or NULL,
-- meaning "the workspace's single ai_settings default") to try for each
-- tier, in order, plus a per-agent daily token ceiling and the sensitive-
-- entity classes that force this agent straight to local_fallback
-- regardless of the normal order (see dlp_service::scan). Its own table
-- rather than folded into ai_agents, since it's an optional,
-- independently-editable policy - the same "own table for an optional
-- extension" shape ai_agent_delegates already uses, just 1:0-or-1 here.
CREATE TABLE ai_agent_routing (
    agent_id TEXT PRIMARY KEY REFERENCES ai_agents(id) ON DELETE CASCADE,
    primary_provider_id TEXT REFERENCES ai_providers(id) ON DELETE SET NULL,
    fallback_provider_id TEXT REFERENCES ai_providers(id) ON DELETE SET NULL,
    local_fallback_provider_id TEXT REFERENCES ai_providers(id) ON DELETE SET NULL,
    temperature REAL,
    max_tokens INTEGER,
    daily_token_budget INTEGER,
    force_air_gapped_for_json TEXT NOT NULL DEFAULT '[]'
);

-- Real per-(workspace, agent, user, day) token accounting the gateway's
-- budget checks (System via ai_settings.daily_token_budget, Agent via
-- ai_agent_routing.daily_token_budget) are enforced against.
-- agent_id/user_id use '' rather than NULL for the "not an agent run" /
-- "no actor" cases - the same sentinel-empty-string convention
-- chat_conversations.agent_id already established, so the composite
-- primary key's uniqueness (NULL != NULL in SQLite) can't be broken by
-- two "system-level" rows silently coexisting.
CREATE TABLE ai_token_usage (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL DEFAULT '',
    user_id TEXT NOT NULL DEFAULT '',
    usage_date TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (workspace_id, agent_id, user_id, usage_date)
);

-- One row per gateway dispatch that didn't succeed on its primary tier -
-- the Gateway health view's own data source (Admin -> LLM & MCP ->
-- Gateway).
CREATE TABLE ai_gateway_failover_events (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    served_by TEXT NOT NULL, -- 'fallback' | 'local_fallback'
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_gateway_failover_workspace ON ai_gateway_failover_events (workspace_id, created_at);

-- The "System" tier of the spec's System -> Agent -> User budget
-- hierarchy (Solution-level aggregation is deferred to Phase 7c, once an
-- agent becomes a real Solution-taggable component there - stated
-- plainly rather than faked with an empty tier now).
ALTER TABLE ai_settings ADD COLUMN daily_token_budget INTEGER;
