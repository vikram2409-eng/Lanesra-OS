-- Voice-First Mode: an optional LLM-backed conversational layer. One
-- workspace-wide row - `enabled` gates whether `voice_execution_service`
-- ever falls back to an LLM rewrite when the deterministic
-- `voice_planner_service::plan()` returns Unsupported (default off: no
-- behavior change for a workspace that never opts in). `provider_id`
-- names which of this workspace's already-configured `ai_providers` rows
-- to use for that rewrite call - the same per-workspace "pick a concrete
-- Claude/OpenAI-compatible/Gemini row" indirection Agent Model Routing
-- already established, reused here rather than inventing a second one;
-- `NULL` falls back to the plain workspace `ai_settings` default, exactly
-- like an agent with no routing policy configured.
CREATE TABLE voice_llm_settings (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL DEFAULT 0,
    provider_id TEXT REFERENCES ai_providers(id) ON DELETE SET NULL,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
