-- AI & Agentic Layer, Phase 1: the bring-your-own-LLM-key foundation every
-- later agent feature (MCP server, Activity Timeline summarization,
-- meeting-prep/follow-up/hygiene/reporting agents) depends on. Lanesra has
-- no SaaS billing surface to meter inference through the way a hosted
-- competitor can, so the answer here is the same "own your data, no
-- lock-in" shape every other Lanesra feature already takes: a workspace
-- supplies its own provider key, Lanesra never resells or proxies
-- inference, and the key is stored exactly the way any other secret in
-- this codebase is - see integration_secrets/secret_service.rs, reused
-- here rather than inventing a second secret store.
--
-- One row per workspace, like integration_settings (migration 0032) -
-- lazily created on first read via ai_settings_repo::ensure_default, not
-- at workspace-creation time, so an existing workspace picks this up with
-- no data migration.
--
-- provider is 'anthropic' | 'openai_compatible' (see ai_service.rs).
-- base_url is meaningful for both: for 'openai_compatible' it's required
-- (any OpenAI-compatible server - OpenAI itself, or a locally-hosted
-- Ollama/vLLM/LM Studio endpoint for a fully offline setup); for
-- 'anthropic' it defaults to the real Anthropic API when left null, but
-- can be overridden to point at a self-hosted proxy in front of it -
-- also what lets this be tested against a local stub server rather than
-- a live third-party endpoint.
CREATE TABLE ai_settings (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    provider TEXT NOT NULL DEFAULT 'anthropic',
    base_url TEXT,
    model TEXT NOT NULL DEFAULT '',
    -- Points into the shared integration_secrets store - never a plaintext
    -- column. Nullable: a workspace that hasn't configured a key yet still
    -- gets a default row (status = 'unconfigured').
    secret_id TEXT REFERENCES integration_secrets(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'unconfigured',
    last_test_message TEXT,
    last_tested_at TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id)
);
