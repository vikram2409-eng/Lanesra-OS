-- Migration 0083: run and step fields every later run feature needs (ADR-0007)
-- Work package: WP-0110
-- Change: data (additive columns and an index, then stray retired statuses rewritten)
-- Data touched: ai_runs rows in the retired waiting_tool or waiting_agent status become paused with stop_reason interrupted
-- Re-run safe: no (ALTER TABLE; the data step only touches rows still in a retired status)
--
-- Runs gain why they stopped (stop_reason), what they were replayed or forked
-- from (parent_run_id, fork_of_step_id), their budget and cost, and the worker
-- lease ADR-0007 section 5 uses to find interrupted runs. Steps gain token and
-- cost totals and what went into their context and guardrails. A step's
-- timings are its existing started_at and finished_at; a step that kept a
-- partial answer is the completed_partial status, not a column.
--
-- waiting_tool and waiting_agent are retired. Nothing ever set them, so any
-- row in one is a stray: it becomes paused with stop_reason interrupted, which
-- a person can resume.
ALTER TABLE ai_runs ADD COLUMN stop_reason TEXT;
ALTER TABLE ai_runs ADD COLUMN parent_run_id TEXT REFERENCES ai_runs(id) ON DELETE SET NULL;
ALTER TABLE ai_runs ADD COLUMN fork_of_step_id TEXT REFERENCES ai_run_nodes(id) ON DELETE SET NULL;
ALTER TABLE ai_runs ADD COLUMN budget_json TEXT;
ALTER TABLE ai_runs ADD COLUMN cost_usd REAL;
ALTER TABLE ai_runs ADD COLUMN lease_owner TEXT;
ALTER TABLE ai_runs ADD COLUMN lease_expires_at TEXT;

ALTER TABLE ai_run_nodes ADD COLUMN tokens_in INTEGER;
ALTER TABLE ai_run_nodes ADD COLUMN tokens_out INTEGER;
ALTER TABLE ai_run_nodes ADD COLUMN cost_usd REAL;
ALTER TABLE ai_run_nodes ADD COLUMN context_receipt_json TEXT;
ALTER TABLE ai_run_nodes ADD COLUMN guardrail_verdicts_json TEXT;

-- The interrupted-run sweep (WP-0141) looks up running runs by lease expiry.
CREATE INDEX idx_ai_runs_status_lease ON ai_runs(status, lease_expires_at);

UPDATE ai_runs
   SET status = 'paused', stop_reason = 'interrupted'
 WHERE status IN ('waiting_tool', 'waiting_agent');
