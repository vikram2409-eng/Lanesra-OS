-- Migration 0095: Process Lane definitions (ADR-0023, ADR-0024)
-- Work package: WP-0703
-- Change: additive (two new tables; three new columns on ai_runs; one new column on ai_run_nodes)
-- Data touched: none; the new columns are NULL for existing rows
-- Re-run safe: no (ALTER TABLE ADD COLUMN)
--
-- A Process Lane is a named business process on one Custom Object, made of
-- ordered stages and steps, defined as data in one JSON Schema
-- (program/spec/schemas/process-lane.schema.json) shared by the online demo,
-- desktop and Team Workspace. A lane is versioned, Draft -> Published, and a
-- run keeps the version it started with (ADR-0024 §1). A lane compiles to an
-- execution graph with source kind 'lane' (ADR-0024 §3); the compiled graph
-- is what a run executes, so the run's graph_id pins its lane version.
CREATE TABLE IF NOT EXISTS process_lanes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- The lane id from the definition, e.g. 'lane-wh-receiving' (ADR-0024 §2:
    -- 'lane-' + kebab-case). Unique per workspace; the same definition may
    -- be imported into another workspace under the same key.
    lane_key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    object_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, lane_key)
);

CREATE TABLE IF NOT EXISTS process_lane_versions (
    id TEXT PRIMARY KEY,
    lane_id TEXT NOT NULL REFERENCES process_lanes(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    -- 'draft' | 'published'. A published version is never edited in place;
    -- a change publishes a new version (ADR-0024 §1). At most one draft
    -- version exists per lane; the service enforces that.
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published')),
    -- The lane definition JSON (format_version + lane), exactly as imported,
    -- per the schema in program/spec/schemas/process-lane.schema.json.
    definition_json TEXT NOT NULL,
    -- The compiled execution graph for a published version (NULL while draft).
    graph_id TEXT REFERENCES ai_execution_graphs(id),
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (lane_id, version)
);
CREATE INDEX IF NOT EXISTS idx_process_lane_versions_lane ON process_lane_versions (lane_id, version);

-- ADR-0024 §1 / SOLUTION-DESIGN §18: a lane run is a Run, not a new kind of
-- object. The anchor record it is about and the run data it collects live
-- on the run row. NULL for every run that is not a lane run.
ALTER TABLE ai_runs ADD COLUMN anchor_object_key TEXT;
ALTER TABLE ai_runs ADD COLUMN anchor_record_id TEXT;
ALTER TABLE ai_runs ADD COLUMN run_data_json TEXT;
CREATE INDEX IF NOT EXISTS idx_ai_runs_anchor ON ai_runs (anchor_object_key, anchor_record_id);

-- SOLUTION-DESIGN §18: a run node carries its lane stage, so the runner's
-- stage bar and the stage timing reports read the grouping from the run
-- rows. Step times are the row's started_at/finished_at, which already
-- exist. NULL for nodes that are not lane steps.
ALTER TABLE ai_run_nodes ADD COLUMN stage_id TEXT;
