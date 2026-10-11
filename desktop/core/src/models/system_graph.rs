//! Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
//! Lanesra System Graph (migration 0073) - a canonical, incrementally
//! updated dependency graph over configurable metadata, so an impact/
//! dependency query ("what depends on this?", "what does this depend
//! on?") is answered generically by `services::system_graph_service`
//! instead of a hand-wired per-domain `COUNT(*)` like
//! `custom_object_service::count_references`.
//!
//! Every enum-shaped field here is a plain `String` validated against one
//! of the `&[&str]` constants below, the same convention `workflow.rs`'s
//! `trigger_type`/`action_type` and `execution_graph.rs`'s `node_type`
//! already use, not a Rust enum.
//!
//! **Scope note**: the full spec names ~20 node types (apps, objects,
//! fields, relationships, screens, views, dashboards, reports, business
//! rules, workflows, agents, agent teams, tools, knowledge sources,
//! connectors, integration jobs, roles, policies, tests, Solution
//! Packages, releases) and 10 edge types. This first pass ships a real,
//! verifiable vertical slice over the 9 types below - the ones with an
//! already-reliable `entity_type`/FK-shaped relationship to derive edges
//! from without inventing new parsing/inference logic - and populates 3
//! of the 10 edge types. The rest are a named, documented follow-up (see
//! issue #256), not silently dropped: `EDGE_TYPES` already declares the
//! full spec vocabulary so a later domain can target it even before every
//! edge kind is populated, the same forward-compatible-vocabulary
//! approach `ai_agent_model_refs`' logical names already used.
//!
//! **FND-02 (Semantic Metadata Layer) addition**: `business_glossary_term`
//! and `metric_definition` node types - see `models::semantic`'s own doc
//! comment. Reuses the existing `depends_on` edge type and is the first
//! to populate `derives_from` (a term/metric's link to what it's mapped
//! to or sourced from) - 4 of the 10 edge types now populated.
//!
//! **FND-03 (Unified Test & Evaluation Framework) addition**:
//! `test_case_definition` - the spec's own "tests" node type, named in
//! this module's scope note from the start. See
//! `services::test_eval_service::sync_graph_node` for its `depends_on`
//! edge: a business_rule/workflow/access_security/screen_visibility case
//! targets an entity_type, so it points at that `custom_object` node
//! (there's no single rule/workflow id to point at - those test types
//! exercise every active rule/workflow for the entity, the same scope
//! `test_rules`/`test_workflows` already test at); an agent_eval/
//! agent_team_eval case points at its `ai_agent`/`execution_graph` node.
//! An integration_mapping case has no edge - `Mapping` isn't a System
//! Graph node type yet, named as a follow-up rather than fabricated.

pub const NODE_TYPES: &[&str] = &[
    "custom_object",
    "custom_field",
    "relationship",
    "business_rule",
    "workflow",
    "screen_layout",
    "page_layout",
    "ai_agent",
    "execution_graph",
    // FND-02 (Semantic Metadata Layer) additions.
    "business_glossary_term",
    "metric_definition",
    // FND-03 (Unified Test & Evaluation Framework) addition.
    "test_case_definition",
];

pub const EDGE_TYPES: &[&str] = &[
    "uses",
    "reads",
    "writes",
    "depends_on",
    "exposes",
    "triggers",
    "invokes",
    "packaged_in",
    "grants_access_to",
    "derives_from",
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemNode {
    pub id: String,
    pub workspace_id: String,
    pub node_type: String,
    pub component_id: String,
    pub label: String,
    pub metadata_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemEdge {
    pub id: String,
    pub workspace_id: String,
    pub edge_type: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub created_at: String,
}

/// One edge `system_graph_service::sync_node` should (re)create from the
/// node currently being synced, addressed by the *other* node's natural
/// key (`node_type`/`component_id`) rather than its `system_nodes.id` -
/// that id may not exist yet (e.g. a Workflow referencing an Agent that
/// hasn't synced this transaction) and the service resolves or creates a
/// stub target node first, the same "resolve key -> id before writing the
/// edge" shape `execution_graph_repo::replace_nodes_and_edges` already
/// uses for `node_key`.
#[derive(Debug, Clone)]
pub struct SystemEdgeTarget {
    pub edge_type: String,
    pub to_node_type: String,
    pub to_component_id: String,
}

/// A `SystemNode` plus the direct edge that reached it from the query's
/// root - what `system_graph_service::get_dependents`/`get_dependencies`
/// and the transitive `get_impact`/`get_lineage` all return, so a caller
/// (the Dependency Explorer, or a future Domain G breaking-change check)
/// never has to re-join `system_edges` itself to learn *why* a node is in
/// the result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemGraphHit {
    pub node: SystemNode,
    pub edge_type: String,
    pub depth: i64,
}
