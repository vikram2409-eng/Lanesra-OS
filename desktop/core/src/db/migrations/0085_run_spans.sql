-- Migration 0085: trace spans for every run, and where their bodies go (ADR-0008)
-- Work package: WP-0111
-- Change: additive (two tables, one index, one ai_settings column)
-- Data touched: none; ai_settings rows get trace_bodies 'off', the default ADR-0013 #2 set
-- Re-run safe: no (CREATE TABLE, ALTER TABLE)
--
-- One trace per run: a pipeline run (ai_agent_runs, run_kind 'pipeline') or
-- an execution-graph run (ai_runs, run_kind 'graph'). run_id is not a
-- foreign key because it names a row in one of two tables. Spans nest run ->
-- step -> model_call -> tool_call -> policy; a delegation is a child step.
-- seq orders a run's spans by when they started.
--
-- Prompt and output bodies are kept apart from the spans, and only when the
-- workspace's trace_bodies setting is 'redacted' or 'full'; with 'off' (the
-- default) no body is written at all.
CREATE TABLE ai_run_spans (
    span_id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    parent_span_id TEXT REFERENCES ai_run_spans(span_id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    run_kind TEXT NOT NULL CHECK (run_kind IN ('pipeline', 'graph')),
    run_id TEXT NOT NULL,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('run', 'step', 'context', 'model_call', 'tool_call', 'policy', 'guardrail', 'approval')),
    name TEXT NOT NULL,
    status TEXT CHECK (status IN ('ok', 'error', 'cancelled')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_ms INTEGER,
    attributes_json TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_ai_run_spans_run ON ai_run_spans(run_kind, run_id, seq);

CREATE TABLE ai_run_span_bodies (
    span_id TEXT NOT NULL REFERENCES ai_run_spans(span_id) ON DELETE CASCADE,
    part TEXT NOT NULL CHECK (part IN ('input', 'output')),
    body TEXT NOT NULL,
    PRIMARY KEY (span_id, part)
);

ALTER TABLE ai_settings ADD COLUMN trace_bodies TEXT NOT NULL DEFAULT 'off'
    CHECK (trace_bodies IN ('off', 'redacted', 'full'));
