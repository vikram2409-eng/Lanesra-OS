-- Migration 0087: stored run events for live streams and replay (ADR-0008 §2, §3)
-- Work package: WP-0112
-- Change: additive (one table, two indexes)
-- Data touched: none; runs started before this migration have no events
-- Re-run safe: no (CREATE TABLE)
--
-- One row per run event, in the ADR-0008 envelope. seq is gapless and
-- increasing within one run and orders its events; id is a cursor across
-- runs, so a stream that follows several runs can resume after a restart
-- (SSE Last-Event-ID). Events are written as the run's spans open and close,
-- then published on the in-process bus; a reconnecting client replays from
-- here, never from memory.
CREATE TABLE ai_run_events (
    id INTEGER PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    run_kind TEXT NOT NULL CHECK (run_kind IN ('pipeline', 'graph')),
    run_id TEXT NOT NULL,
    seq INTEGER NOT NULL,
    type TEXT NOT NULL,
    at TEXT NOT NULL,
    step_id TEXT,
    span_id TEXT,
    parent_span_id TEXT,
    data_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE (run_kind, run_id, seq)
);
CREATE INDEX idx_ai_run_events_run ON ai_run_events(run_id, seq);
CREATE INDEX idx_ai_run_events_workspace ON ai_run_events(workspace_id, id);
