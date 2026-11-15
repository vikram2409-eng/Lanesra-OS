-- Migration 0097: Lane cases (WP-0714, REQ-LANE-021)
-- Work package: WP-0714
-- Change: additive (one new table)
-- Data touched: none
-- Re-run safe: no (CREATE TABLE)
--
-- A lane case is a named test against a seeded workspace, stored with the
-- lane version it tests (WP-0714 §4) so it travels in the Solution export
-- (§5) and the Unified Test Run can discover it. The case itself is JSON
-- (case, lane, version, anchor, fixtures, assert) - the store keeps it
-- verbatim, and `process_lane_publish::run_lane_case` parses it. One row
-- per (lane, version, case name); re-storing a case of the same name
-- replaces it, so an export/import round-trip is idempotent.
CREATE TABLE IF NOT EXISTS process_lane_cases (
    id TEXT PRIMARY KEY,
    lane_id TEXT NOT NULL REFERENCES process_lanes(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    name TEXT NOT NULL,
    case_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (lane_id, version, name)
);
CREATE INDEX IF NOT EXISTS idx_process_lane_cases_lane ON process_lane_cases (lane_id, version);
