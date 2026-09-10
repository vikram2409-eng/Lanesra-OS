-- AI & Agentic Layer, Phase 6b: orchestration on top of Phase 6a's
-- Agents - a deterministic, ordered Agent Pipeline; Triggers (schedule
-- or webhook - the two kinds with no existing home elsewhere: manual is
-- always available and isn't a stored row, and a Workflow Automation
-- action is a third trigger kind that needs no table of its own here
-- either, since Workflow Automation already has one); and the unified
-- run log both a Pipeline and a lone triggered Agent share. See
-- core::services::ai_orchestration_service.

CREATE TABLE ai_agent_pipelines (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);
CREATE INDEX idx_ai_agent_pipelines_workspace ON ai_agent_pipelines (workspace_id, is_active);

CREATE TABLE ai_agent_pipeline_steps (
    id TEXT PRIMARY KEY,
    pipeline_id TEXT NOT NULL REFERENCES ai_agent_pipelines(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES ai_agents(id) ON DELETE RESTRICT,
    step_order INTEGER NOT NULL,
    -- Literal text; "{{previous_output}}" is substituted with the prior
    -- step's final answer (empty for step 1 unless it has its own static
    -- prompt), "{{trigger_input}}" with whatever a schedule/webhook/
    -- workflow-action-fired run resolved from its own source.
    input_template TEXT NOT NULL
);
CREATE INDEX idx_ai_agent_pipeline_steps_pipeline ON ai_agent_pipeline_steps (pipeline_id, step_order);

CREATE TABLE ai_agent_triggers (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL, -- 'agent' | 'pipeline'
    target_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL, -- 'schedule' | 'webhook'
    interval_minutes INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT
);
CREATE INDEX idx_ai_agent_triggers_target ON ai_agent_triggers (target_type, target_id);

-- What a due schedule Trigger, an inbound webhook call, and the new
-- Workflow Automation action all enqueue into - never run inline from a
-- record save or a scheduler tick (the same enqueue/drain shape
-- migration 0033 already established for call_connector_action - see
-- connector_execution_service's own doc comment).
-- ai_orchestration_service::drain_pending_runs is the one drain for all
-- three trigger kinds.
CREATE TABLE ai_agent_pending_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    resolved_input_text TEXT NOT NULL,
    triggered_by TEXT,
    source_entity_type TEXT,
    source_entity_id TEXT,
    created_at TEXT NOT NULL
);

-- One row per run - manual, drained-from-pending, or an inline webhook
-- call alike. A lone triggered Agent is logged as a 1-step run, so a
-- single-agent run and a Pipeline run share this one shape.
CREATE TABLE ai_agent_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    status TEXT NOT NULL, -- 'succeeded' | 'failed'
    error TEXT,
    triggered_by TEXT, -- 'manual' | 'schedule' | 'webhook' | a workflow definition id
    source_entity_type TEXT,
    source_entity_id TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT
);
CREATE INDEX idx_ai_agent_runs_target ON ai_agent_runs (target_type, target_id, started_at);

CREATE TABLE ai_agent_run_steps (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES ai_agent_runs(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    input_text TEXT NOT NULL,
    output_text TEXT,
    error TEXT,
    tool_calls_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_ai_agent_run_steps_run ON ai_agent_run_steps (run_id, step_order);
