-- Next-Gen AI Foundry & Low-Code Platform Enhancements, Domain A
-- (Intelligence Foundation), FND-03: the Unified Test & Evaluation
-- Framework.
--
-- One execution framework for deterministic tests and AI evaluations, so
-- deployment quality is consistent across platform components. Per the
-- issue's own "Note on existing platform state": this does not
-- re-invent either kind of test. It wraps what's already real -
-- `business_rule_service::test_rules`, `workflow_service::test_workflows`,
-- `access_service::explain_access`, `screen_layout_service::
-- resolve_effective_layout`, `mapping_service`'s field-map transform, the
-- Evaluation Harness's agent run + judge call, and `graph_runtime_service`
-- for Agent Teams - behind one Test Case Definition / Test Run shape, so
-- a Solution Release can run a mix of them in one validation job and get
-- one readiness result back.
CREATE TABLE test_case_definitions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    -- 'business_rule' | 'workflow' | 'access_security' | 'screen_visibility'
    -- | 'integration_mapping' | 'agent_eval' | 'agent_team_eval'.
    test_type TEXT NOT NULL,
    -- What this case targets - an entity_type (business_rule/workflow/
    -- access_security/screen_visibility), a Mapping id (integration_mapping),
    -- an AI Agent id (agent_eval) or an Execution Graph id (agent_team_eval).
    -- Which it is follows directly from test_type; see
    -- test_eval_service::require_valid_target.
    target_id TEXT NOT NULL,
    -- The Dataset (spec wording): synthetic/sample record values, expected
    -- state changes/errors, evaluator criteria, cost/latency thresholds -
    -- one opaque JSON blob whose shape depends on test_type, documented on
    -- `models::test_eval::TestCaseDefinitionInput`. Kept opaque (like
    -- `metric_definitions.filters_json`) rather than seven separate
    -- column sets for seven test types.
    dataset_json TEXT NOT NULL DEFAULT '{}',
    cost_threshold_usd REAL,
    latency_threshold_ms INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    UNIQUE (workspace_id, name)
);

-- One unified validation job - a batch of mixed-type case results with a
-- rolled-up readiness verdict. `solution_id` is set when this run was
-- triggered as a Solution Release's deployment validation (see
-- test_eval_service::run_for_solution); NULL for an ad hoc run from the
-- admin screen. ON DELETE SET NULL (not CASCADE) because a run's own
-- historical result still has standalone value after its Solution is
-- deleted - the same "don't let an unrelated delete silently erase
-- results" stance `metric_versions` takes for its own parent.
CREATE TABLE test_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    solution_id TEXT REFERENCES solutions(id) ON DELETE SET NULL,
    -- 'running' | 'completed'.
    status TEXT NOT NULL,
    passed_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    -- 'manual' | 'deployment_validation'.
    triggered_by TEXT NOT NULL DEFAULT 'manual',
    started_at TEXT NOT NULL,
    finished_at TEXT,
    created_by TEXT
);
CREATE INDEX idx_test_runs_workspace ON test_runs (workspace_id, started_at DESC);
CREATE INDEX idx_test_runs_solution ON test_runs (solution_id);

-- One case's result within a run. `test_case_id` is nullable and the
-- name/type are denormalized onto the row - a case definition can be
-- edited or deleted later without corrupting a past run's own record of
-- what it actually tested, the same "denormalize what must survive the
-- source row changing" reasoning `metric_versions.snapshot_json` already
-- applies, just as plain columns here instead of a JSON snapshot since
-- the result row needs them for direct querying/display.
CREATE TABLE test_run_case_results (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    test_case_id TEXT REFERENCES test_case_definitions(id) ON DELETE SET NULL,
    test_case_name TEXT NOT NULL,
    test_type TEXT NOT NULL,
    passed INTEGER NOT NULL,
    -- The actual outcome this case's executor produced (e.g. resolved
    -- field_effects, the access decision, the resolved visible field set,
    -- the mapped target row, the judge's verdict) - what "Results produce
    -- ... evidence" asks for.
    evidence_json TEXT NOT NULL DEFAULT '{}',
    trace_text TEXT,
    runtime_ms INTEGER NOT NULL,
    -- Always 0 in this slice - no retry policy exists yet for any of the
    -- seven executors this framework drives. The column is real (not
    -- left out) so a later retry policy fills it in without a migration;
    -- see test_eval_service's own doc comment.
    retries INTEGER NOT NULL DEFAULT 0,
    -- NULL in this slice - no token/dollar cost metering exists anywhere
    -- in this codebase yet for an agent/pipeline/graph run to report.
    -- Named as a follow-up, not silently dropped: the column exists now
    -- so real metering (whenever it lands) only has to start writing to
    -- it, not add it.
    cost_usd REAL,
    -- 'n/a' for the five deterministic types; 'clean' | 'violation' for
    -- agent_eval/agent_team_eval, mirroring ai_eval_repo's own
    -- policy_violations-count-to-verdict reduction.
    policy_outcome TEXT NOT NULL DEFAULT 'n/a',
    -- A free-text "exact component version" reference (spec: "records
    -- exact component versions") - whatever version-shaped identifier
    -- the target type already has: an agent's current_version_id, a
    -- metric/rule/workflow/mapping's own updated_at timestamp (no
    -- incrementing version counter exists on those types outside their
    -- own version-history tables), or "live" where neither applies.
    component_version_ref TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_test_run_case_results_run ON test_run_case_results (run_id);
