-- Migration 0098: Lane run notes (WP-0711, REQ-LANE-018)
-- Work package: WP-0711
-- Change: additive (one new table)
-- Data touched: none
-- Re-run safe: no (CREATE TABLE)
--
-- A note added to a lane run at any point, from the runner. One row per
-- note: which run, which step was current when it was added (null when the
-- run waited on nothing a person submits - an approval, a child lane), who
-- wrote it, the body, when. Notes are listed on the run view and the add
-- is audited (audit_events, entity lane_run/run_id) so the action appears
-- in the runner's audit trail (REQ-LANE-018 AC-1). Attachments reuse the
-- 0001 `attachments` table (entity_type `lane_run`); no migration needed
-- for them.
CREATE TABLE IF NOT EXISTS process_lane_run_notes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL,
    step_id TEXT,
    author_user_id TEXT REFERENCES users(id),
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lane_run_notes_run ON process_lane_run_notes (run_id, created_at);
