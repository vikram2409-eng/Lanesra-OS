-- AI & Agentic Layer, Phase 7e: Human-in-the-loop approval gates and
-- basic observability on top of Phase 6b/7d's orchestration engine.
--
-- A sequential Pipeline step can now be flagged `requires_approval`: the
-- run pauses right after that step (status 'awaiting_approval') instead
-- of continuing automatically, and an Administrator later approves
-- (optionally editing the step's output before it feeds the next step)
-- or rejects it. Resuming a paused run needs the run's own original
-- trigger input (a later step's template may reference {{trigger_input}}
-- directly, not just {{previous_output}}) and where to resume from -
-- both persisted here rather than recomputed, since a real approval may
-- come from an entirely separate request much later.
--
-- `started_at`/`finished_at` per step is this phase's basic tracing
-- primitive - real wall-clock timing per agent call, the foundation the
-- new OTLP JSON exporter (ai_orchestration_service::run_to_otlp_json)
-- builds its spans from. See core::services::ai_orchestration_service.
ALTER TABLE ai_agent_pipeline_steps ADD COLUMN requires_approval INTEGER NOT NULL DEFAULT 0;
ALTER TABLE ai_agent_runs ADD COLUMN trigger_input TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_agent_runs ADD COLUMN paused_at_step_order INTEGER;
ALTER TABLE ai_agent_runs ADD COLUMN resume_previous_output TEXT;
ALTER TABLE ai_agent_run_steps ADD COLUMN started_at TEXT;
ALTER TABLE ai_agent_run_steps ADD COLUMN finished_at TEXT;
