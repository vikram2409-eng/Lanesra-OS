-- AI & Agentic Layer, Phase 7d: a real Evaluation Harness. An Eval Suite
-- is a named set of golden test Cases (an input plus a plain-English
-- success criteria) run against one Agent or Pipeline; each run grades
-- every case with an LLM-as-judge call (ai_service::complete - no new
-- infrastructure, the same completion primitive Phase 4's natural-
-- language reporting already uses) rather than a brittle exact-match
-- comparison an open-ended agent response could never satisfy. See
-- core::services::ai_eval_service.
CREATE TABLE ai_eval_suites (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    target_type TEXT NOT NULL, -- 'agent' | 'pipeline' - same TRIGGER_TARGET_TYPES shape
    target_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_eval_suites_workspace ON ai_eval_suites (workspace_id);

CREATE TABLE ai_eval_cases (
    id TEXT PRIMARY KEY,
    suite_id TEXT NOT NULL REFERENCES ai_eval_suites(id) ON DELETE CASCADE,
    case_order INTEGER NOT NULL,
    input_text TEXT NOT NULL,
    -- Plain-English description of what a correct response looks like -
    -- graded by the judge call, not string-matched, since an agent's
    -- response is never byte-for-byte reproducible.
    success_criteria TEXT NOT NULL
);
CREATE INDEX idx_ai_eval_cases_suite ON ai_eval_cases (suite_id, case_order);

-- One row per "Run suite" click - mirrors ai_agent_runs' own
-- one-row-per-run shape.
CREATE TABLE ai_eval_runs (
    id TEXT PRIMARY KEY,
    suite_id TEXT NOT NULL REFERENCES ai_eval_suites(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    status TEXT NOT NULL, -- 'completed' - a case failing its own grade is not a run failure
    passed_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL,
    finished_at TEXT
);
CREATE INDEX idx_ai_eval_runs_suite ON ai_eval_runs (suite_id, started_at);

CREATE TABLE ai_eval_case_results (
    id TEXT PRIMARY KEY,
    eval_run_id TEXT NOT NULL REFERENCES ai_eval_runs(id) ON DELETE CASCADE,
    case_id TEXT NOT NULL,
    -- Denormalized off the case at run time, so a result still reads
    -- sensibly even if the case is later edited or deleted.
    input_text TEXT NOT NULL,
    success_criteria TEXT NOT NULL,
    actual_output TEXT,
    passed INTEGER NOT NULL DEFAULT 0,
    judge_reasoning TEXT,
    -- Set instead of the above when the target agent/pipeline run itself
    -- failed (a provider error, a deleted agent) - graded as failed
    -- without a wasted judge call.
    error TEXT
);
CREATE INDEX idx_ai_eval_case_results_run ON ai_eval_case_results (eval_run_id);
