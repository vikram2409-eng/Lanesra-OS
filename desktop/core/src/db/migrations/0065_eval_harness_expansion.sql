-- AI Agent Platform v2, Phase 6a (GitHub issue #171, evaluator-expansion
-- slice): a real evaluator type per Eval Suite instead of one fixed
-- LLM-as-judge grading path for every suite.
--
-- `evaluator_type` on ai_eval_suites: 'task_completion' (today's existing
-- behavior, unchanged - an LLM-as-judge call against a plain-English
-- success_criteria), 'structured_output' (validates the target agent's
-- current version's declared output_schema - AI Agent Platform v2 Phase 1 -
-- against the actual response, no judge call needed), or
-- 'policy_compliance' (deterministically checks whether the target run
-- had any Tool-Call Firewall denial/required-approval - AI Agent Platform
-- v2 Phase 2 - again no judge call). Defaulted to 'task_completion' so
-- every existing suite keeps its exact current behavior unchanged.
ALTER TABLE ai_eval_suites ADD COLUMN evaluator_type TEXT NOT NULL DEFAULT 'task_completion';

-- `policy_violations_count` on ai_agent_run_steps mirrors the existing
-- `tool_calls_count` column exactly (same "count something real that
-- happened during this step" shape) - how many of this step's tool calls
-- were denied or queued for approval by `policy_engine_service::evaluate`.
-- This is what the new 'policy_compliance' evaluator type above checks;
-- it's also visible on every run step regardless of evaluator type, the
-- same way tool_calls_count already is.
ALTER TABLE ai_agent_run_steps ADD COLUMN policy_violations_count INTEGER NOT NULL DEFAULT 0;
