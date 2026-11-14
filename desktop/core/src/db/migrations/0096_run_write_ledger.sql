-- Migration 0096: idempotent write ledger for graph runs (WP-0141, ADR-0007 §5)
-- Work package: WP-0141
-- Change: additive (one new table)
-- Data touched: none
-- Re-run safe: yes (CREATE TABLE IF NOT EXISTS)
--
-- A run step's side effect is recorded here under an idempotency key derived
-- from the run, the node and the attempt (ADR-0024's "idempotency key per step
-- attempt", generalised to every graph run). Before a write is applied the
-- runtime looks the key up; if it is present the write already happened - on a
-- retry, a resume after a crash, or a double submit - and the stored result is
-- returned instead of writing again (REQ-TOOL-005, REQ-GOV-023). The ledger row
-- is written in the same transaction as the side effect, so a crash before
-- commit leaves neither, and a crash after commit leaves both: the write is
-- applied exactly once.
CREATE TABLE IF NOT EXISTS ai_run_write_ledger (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL REFERENCES ai_runs(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    attempt INTEGER NOT NULL,
    -- Distinguishes several writes of one step attempt (WP-0705 posts more
    -- than one). A single-write step uses the empty string.
    write_key TEXT NOT NULL DEFAULT '',
    -- The result the side effect returned, as JSON - replayed verbatim to the
    -- caller when the key is seen again, so a retry sees what the first call saw.
    result_json TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (run_id, node_id, attempt, write_key)
);
CREATE INDEX IF NOT EXISTS idx_run_write_ledger_run ON ai_run_write_ledger (run_id);
