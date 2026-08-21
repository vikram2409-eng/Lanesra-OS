-- Integration Hub / Workflow Automation integration (spec §17): the
-- "Call Connector Action" workflow action step. `apply_action` in
-- `workflow_service` is (and stays) a plain sync function like every
-- other workflow action - see this crate's Cargo.toml comment on why
-- async is scoped narrowly to the functions that genuinely make an
-- outbound call. So this action doesn't call out synchronously: it
-- enqueues a row here (same "store, then drain asynchronously" shape
-- `integration_pending_events`/webhook fan-out already uses for the exact
-- same sync/async boundary), and
-- `connector_execution_service::drain_pending_actions` - a real `async
-- fn` - performs the actual HTTP call and logs the outcome to
-- `integration_executions` on whatever cadence a caller already polls on
-- (the server's scheduler loop, or desktop's existing client-poll pattern
-- for scheduled workflows).
CREATE TABLE integration_pending_actions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    connector_id TEXT NOT NULL REFERENCES integration_connectors(id) ON DELETE CASCADE,
    action_key TEXT NOT NULL,
    reference_key TEXT NOT NULL,
    params_json TEXT NOT NULL,
    source_entity_type TEXT,
    source_entity_id TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_integration_pending_actions_workspace ON integration_pending_actions(workspace_id, created_at);
