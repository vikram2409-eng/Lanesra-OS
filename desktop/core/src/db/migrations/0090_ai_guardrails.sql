-- Migration 0090: output guardrails as versioned policy objects (WP-0133)
-- Work package: WP-0133
-- Change: additive (three tables)
-- Data touched: none; built-in guardrails are seeded lazily per workspace
-- Re-run safe: no (CREATE TABLE)
--
-- A guardrail is a reusable, named output check of one kind (REQ-GOV-007):
--   rule   - the text must/ must not contain something, or its URLs' domains
--            must be on an allow-list;
--   pii    - the text leaks no PII (reuses dlp_service);
--   schema - the text parses as JSON conforming to a small schema;
--   judge  - an LLM decides, from a configured instruction (ADR-0011).
-- Its checker configuration is versioned: editing a guardrail writes a new
-- ai_guardrail_versions row and bumps ai_guardrails.current_version, so a run
-- can record exactly which version it evaluated (WP-0133 AC-2).

CREATE TABLE ai_guardrails (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- Stable key for a built-in ('pii_leak', 'tone', ...); NULL for a custom one.
    builtin_key TEXT,
    name TEXT NOT NULL,
    description TEXT,
    checker_type TEXT NOT NULL,                 -- 'rule' | 'pii' | 'schema' | 'judge'
    is_builtin INTEGER NOT NULL DEFAULT 0,
    current_version INTEGER NOT NULL DEFAULT 1,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    UNIQUE (workspace_id, builtin_key)
);

CREATE TABLE ai_guardrail_versions (
    id TEXT PRIMARY KEY,
    guardrail_id TEXT NOT NULL REFERENCES ai_guardrails(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    config_json TEXT NOT NULL,                  -- checker-specific configuration
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    UNIQUE (guardrail_id, version)
);

-- One row per guardrail evaluation that happened in the context of a run, so
-- the run records which guardrail version it checked against and the outcome.
CREATE TABLE ai_guardrail_evaluations (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    run_kind TEXT NOT NULL,
    run_id TEXT NOT NULL,
    guardrail_id TEXT NOT NULL REFERENCES ai_guardrails(id) ON DELETE CASCADE,
    guardrail_version INTEGER NOT NULL,
    passed INTEGER NOT NULL,
    detail TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_ai_guardrail_versions_guardrail ON ai_guardrail_versions(guardrail_id);
CREATE INDEX idx_ai_guardrail_evaluations_run ON ai_guardrail_evaluations(workspace_id, run_kind, run_id);
