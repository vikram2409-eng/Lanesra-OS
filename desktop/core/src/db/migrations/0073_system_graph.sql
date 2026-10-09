-- Next-Gen AI Foundry & Low-Code Platform Enhancements, Domain A (Intelligence
-- Foundation), FND-01: the Lanesra System Graph.
--
-- A canonical, incrementally-updated dependency graph over configurable
-- metadata so Lanesra can answer "what depends on this?" / "what does this
-- depend on?" generically, instead of the hand-wired per-domain COUNT(*)
-- queries `custom_object_service::count_references` uses today. Built as a
-- real vertical slice over 9 component types first (custom_object,
-- custom_field, relationship, business_rule, workflow, screen_layout,
-- page_layout, ai_agent, execution_graph) rather than every type the full
-- spec eventually wants (apps, views, dashboards, reports, tools, knowledge
-- sources, connectors, roles/policies, tests, Solution Packages, releases) -
-- see system_graph.rs's own module doc comment for the full scoping note.
--
-- `system_nodes` is upserted by `system_graph_service::sync_node`, called
-- inline post-write from each of the 9 owning services (the same
-- "many services call one shared recorder after their own write" shape
-- `audit_repo::record` already established) - never rebuilt from scratch.
-- A node is only ever removed on a hard delete of its owning component; a
-- soft-deactivate leaves the node (and its edges) in place, so "what used to
-- depend on this" stays answerable.
CREATE TABLE system_nodes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- See system_graph.rs's NODE_TYPES for the full list.
    node_type TEXT NOT NULL,
    -- The owning row's own id in its native table (e.g. a custom_objects.id).
    component_id TEXT NOT NULL,
    label TEXT NOT NULL,
    -- Small denormalized extras a Dependency Explorer list can show without
    -- a second lookup (e.g. {"is_active": true}) - opaque beyond that.
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (workspace_id, node_type, component_id)
);
CREATE INDEX idx_system_nodes_workspace_type ON system_nodes (workspace_id, node_type);

-- One row per typed dependency edge. `system_graph_service::sync_node`
-- replaces (delete-then-reinsert) every outgoing edge for a node on each
-- sync call, the same "replace this node's own edges" shape
-- `execution_graph_repo::replace_nodes_and_edges` already uses for a whole
-- graph - so a removed dependency (e.g. a Business Rule's entity_type
-- changing, which can't actually happen today but a future rename-safe
-- path might add) never leaves a stale edge behind.
CREATE TABLE system_edges (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    -- See system_graph.rs's EDGE_TYPES for the full spec vocabulary (10
    -- values); this v1 slice only ever populates DEPENDS_ON/INVOKES/
    -- DELEGATES_TO.
    edge_type TEXT NOT NULL,
    from_node_id TEXT NOT NULL REFERENCES system_nodes(id) ON DELETE CASCADE,
    to_node_id TEXT NOT NULL REFERENCES system_nodes(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    UNIQUE (from_node_id, to_node_id, edge_type)
);
CREATE INDEX idx_system_edges_from ON system_edges (from_node_id);
CREATE INDEX idx_system_edges_to ON system_edges (to_node_id);
