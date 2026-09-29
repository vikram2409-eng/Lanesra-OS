-- AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime.
--
-- Today Workflow Automation (workflow_service.rs: trigger -> conditions ->
-- a flat action list, no branching/parallel/join/loop) and AI Orchestration
-- (ai_orchestration_service.rs: a fixed sequential/consensus/peer_review
-- Pipeline) are two separate, fixed-shape engines. This migration adds one
-- durable, node-based Execution Graph runtime both can eventually express
-- themselves as (see execution_graph_service::graph_from_workflow/
-- graph_from_pipeline) without touching workflow_service.rs's or
-- ai_orchestration_service.rs's own tables or execution paths - those keep
-- running completely unchanged. The new engine is additive: new graphs
-- (and, in a later phase, a visual Agent Team Builder) run on it; existing
-- saved Workflows and Pipelines are unaffected until something explicitly
-- migrates them.
--
-- `ai_execution_graphs` / `ai_graph_nodes` / `ai_graph_edges` are the
-- durable graph *definition* (edited while `status = 'draft'`, immutable
-- once `status = 'published'` - execution_graph_service::publish enforces
-- this the same way agent_version_service enforces a Published agent
-- version's immutability). `ai_runs` / `ai_run_nodes` are the durable
-- *execution* state - one row per run, one row per node-attempt - so a run
-- survives a process restart: graph_runtime_service::resume_run reads
-- ai_runs.current_node_id + context_json back from these two tables and
-- continues exactly where it left off, the same durability property
-- ai_agent_runs already gives Pipeline runs, generalized to an arbitrary
-- graph shape instead of a fixed step order.
CREATE TABLE ai_execution_graphs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    -- 'draft' | 'published' | 'disabled'. Only a 'draft' graph's nodes/edges
    -- may be edited; publish runs execution_graph_service::validate_for_publish
    -- (unreachable nodes, missing End, invalid Join/Loop shape) first.
    status TEXT NOT NULL DEFAULT 'draft',
    version INTEGER NOT NULL DEFAULT 1,
    -- Set only when this graph was generated as a compatibility-shape
    -- mapping of an existing Workflow or Pipeline (see
    -- execution_graph_service::graph_from_workflow/graph_from_pipeline) -
    -- NULL for a graph authored directly against this engine.
    -- 'workflow' | 'pipeline'.
    source_kind TEXT,
    source_id TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);

-- One row per node. `node_key` is the stable, human-referenceable id a
-- template ({{node_key.output}}), an edge, or a Join's incoming-branch list
-- addresses - `id` is only the storage primary key, so a graph can be
-- re-exported/re-imported (Solution Packages, a later phase) without every
-- reference silently breaking on a new random id.
CREATE TABLE ai_graph_nodes (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES ai_execution_graphs(id) ON DELETE CASCADE,
    node_key TEXT NOT NULL,
    -- 'trigger' | 'condition' | 'action' | 'agent' | 'router' | 'parallel_split'
    -- | 'join' | 'approval' | 'delay' | 'transform' | 'loop' | 'end'.
    node_type TEXT NOT NULL,
    -- Node-type-specific configuration, e.g. {action_type, params_json} for
    -- an Action node (the exact shape workflow_service::apply_action already
    -- takes), {agent_id, input_template} for an Agent node, {mode,
    -- required_count} for a Join, {max_iterations, break_conditions} for a
    -- Loop. See models::execution_graph for the typed config each node_type
    -- expects.
    config_json TEXT NOT NULL DEFAULT '{}',
    -- Reserved for the visual Agent Team Builder canvas (a later phase) -
    -- NULL is correct for every graph authored via API/fixtures in this one.
    position_x REAL,
    position_y REAL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (graph_id, node_key)
);

CREATE TABLE ai_graph_edges (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES ai_execution_graphs(id) ON DELETE CASCADE,
    from_node_id TEXT NOT NULL REFERENCES ai_graph_nodes(id) ON DELETE CASCADE,
    to_node_id TEXT NOT NULL REFERENCES ai_graph_nodes(id) ON DELETE CASCADE,
    -- Which outgoing branch of a multi-branch node this edge is: 'true'/
    -- 'false' out of a Condition, a Router's own branch label, 'approved'/
    -- 'rejected' out of an Approval, 'body'/'exit' out of a Loop. NULL for
    -- an unconditional single-outgoing-edge node (Trigger, Action, Agent,
    -- Parallel Split's own branches, Delay, Transform).
    branch_label TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- One row per run of a published graph - the direct analogue of
-- ai_agent_runs, generalized from a fixed step order to an arbitrary graph
-- position (`current_node_id`) plus a named-output context bag
-- (`context_json`, keyed by node_key) any downstream node's template can
-- address, not just "the previous step's output".
CREATE TABLE ai_runs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    graph_id TEXT NOT NULL REFERENCES ai_execution_graphs(id) ON DELETE CASCADE,
    -- 'queued' | 'planning' | 'running' | 'waiting_tool' | 'waiting_agent' |
    -- 'waiting_approval' | 'waiting_scheduled' | 'paused' | 'completed' |
    -- 'failed' | 'cancelled'. Fail-closed like ai_agent_runs.start_run: a
    -- row interrupted before it reaches a terminal or waiting status (a
    -- panic, a process kill) is left as whatever waiting/running status it
    -- last checkpointed at, which is exactly the state resume_run needs to
    -- pick it back up - never silently "succeeded".
    status TEXT NOT NULL DEFAULT 'queued',
    trigger_input TEXT NOT NULL DEFAULT '{}',
    -- Every completed node's output, keyed by node_key, as a JSON object -
    -- what a later node's {{node_key.field}} template resolves against.
    context_json TEXT NOT NULL DEFAULT '{}',
    current_node_id TEXT REFERENCES ai_graph_nodes(id),
    -- Set only while `status = 'waiting_approval'`: the ai_approvals row
    -- (Phase 1's durable Approval Service - the one real approval primitive
    -- this engine consolidates onto, see this migration's own module-level
    -- doc comment in graph_runtime_service.rs) this run is blocked on.
    pending_approval_id TEXT REFERENCES ai_approvals(id),
    -- Set only while `status = 'waiting_scheduled'` (a Delay node): when
    -- graph_runtime_service::resume_due_delays should next pick this run
    -- back up, mirroring workflow_service::run_scheduled's own sweep
    -- pattern for the Workflow engine's `scheduled` trigger type.
    resume_at TEXT,
    error_message TEXT,
    triggered_by TEXT,
    source_entity_type TEXT,
    source_entity_id TEXT,
    -- Compared against the Policy Engine's max-steps default (see
    -- graph_runtime_service::MAX_STEPS_DEFAULT) after every node
    -- transition, so a malformed or runaway graph can't loop forever even
    -- if its own Loop-node bounds are misconfigured.
    steps_executed INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL,
    finished_at TEXT
);
CREATE INDEX idx_ai_runs_workspace_status ON ai_runs (workspace_id, status);
CREATE INDEX idx_ai_runs_resume_at ON ai_runs (status, resume_at) WHERE status = 'waiting_scheduled';

-- One row per node execution attempt within a run - the direct analogue of
-- ai_agent_run_steps, except keyed by node_id (not a fixed step_order) and
-- carrying an `attempt` counter so a bounded Loop node's Nth iteration and
-- (N+1)th iteration are two distinct, individually-inspectable rows rather
-- than one row silently overwritten.
CREATE TABLE ai_run_nodes (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES ai_runs(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL REFERENCES ai_graph_nodes(id),
    node_key TEXT NOT NULL,
    node_type TEXT NOT NULL,
    attempt INTEGER NOT NULL DEFAULT 1,
    -- 'running' | 'waiting_approval' | 'completed' | 'failed' | 'skipped'.
    status TEXT NOT NULL DEFAULT 'running',
    input_json TEXT,
    output_json TEXT,
    error_message TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT
);
CREATE INDEX idx_ai_run_nodes_run ON ai_run_nodes (run_id);
