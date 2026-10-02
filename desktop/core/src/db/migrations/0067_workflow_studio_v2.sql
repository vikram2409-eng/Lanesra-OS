-- UX/UI Modernization, Workflow Studio 2.0 (issue #193): rebuilds
-- WorkflowAutomationAdmin.tsx onto the Shared Visual Builder Framework
-- (#192) and exposes the Execution Graph runtime's (Phase 3) node types
-- this builder never surfaced (Switch/Loop/Parallel/Join/Run Agent, plus
-- two new node types below) - without migrating a single existing saved
-- Workflow's execution path.
--
-- `graph_id` is nullable and defaults to NULL: every Workflow that
-- exists today, and every new Workflow an admin builds with just a
-- Trigger/Conditions/flat Actions, keeps `graph_id` NULL forever and
-- keeps firing through workflow_service's own flat executor completely
-- unchanged - this is what makes the issue's mandatory parity gate true
-- by construction rather than by after-the-fact comparison testing. Only
-- a Workflow an admin deliberately upgrades (by adding a Switch, Loop,
-- Parallel, Join, Run Agent, Run Agent Team or Evaluate Result node on
-- the new canvas) gets a `graph_id`, at which point
-- `workflow_service::fire_event` dispatches it to
-- `graph_runtime_service::start_run` instead - see
-- `workflow_service::upgrade_to_graph`.
ALTER TABLE workflow_definitions ADD COLUMN graph_id TEXT REFERENCES ai_execution_graphs(id);

-- The queue `run_workflow` enqueues into for an upgraded (graph_id set)
-- workflow - a record-save-time context can't call the async
-- `graph_runtime_service::start_run` directly, the exact same
-- enqueue-not-inline shape `ai_agent_pending_runs` (migration 0040)
-- already established for `run_ai_agent`. `graph_runtime_service::
-- drain_pending_graph_runs` is the async drain.
CREATE TABLE graph_pending_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    graph_id TEXT NOT NULL REFERENCES ai_execution_graphs(id),
    trigger_input TEXT NOT NULL,
    triggered_by TEXT,
    source_entity_type TEXT,
    source_entity_id TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_graph_pending_runs_workspace ON graph_pending_runs(workspace_id, created_at);
