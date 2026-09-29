//! AI Agent Platform v2, Phase 3: the Shared Execution Graph Runtime's
//! durable graph *definition* - `ai_execution_graphs`/`ai_graph_nodes`/
//! `ai_graph_edges` (migration 0062). See `graph_run.rs` for the durable
//! *execution* state (`ai_runs`/`ai_run_nodes`) a run of one of these
//! graphs produces, and `services::graph_runtime_service`'s module doc
//! comment for how each `node_type` below actually executes.
//!
//! Every enum-shaped field here is a plain `String` validated against one
//! of the `&[&str]` constants below - the same convention `workflow.rs`'s
//! `trigger_type`/`action_type` and `ai_agent_pipeline.rs`'s `topology`
//! already use, not a Rust enum.

pub const GRAPH_STATUSES: &[&str] = &["draft", "published", "disabled"];

pub const NODE_TYPES: &[&str] = &[
    "trigger", "condition", "action", "agent", "router", "parallel_split", "join", "approval", "delay", "transform", "loop", "end",
];

/// A node type whose config must declare exactly one outgoing edge with no
/// `branch_label` - everything except the explicitly multi-branch types
/// (`condition`, `router`, `approval`, `loop`) and the fan-out type
/// (`parallel_split`, whose N outgoing edges are all unconditional but
/// there can be more than one) and the terminal type (`end`, which has
/// none). Used by `execution_graph_service::validate_for_publish`.
pub fn is_single_unconditional_outgoing(node_type: &str) -> bool {
    matches!(node_type, "trigger" | "action" | "agent" | "delay" | "transform")
}

pub const JOIN_MODES: &[&str] = &["all", "first_successful", "n_of_m", "timeout_partial"];

/// `ai_execution_graphs.source_kind` - set only when this graph was
/// generated as a compatibility-shape mapping of an existing Workflow or
/// Pipeline (see `execution_graph_service::graph_from_workflow`/
/// `graph_from_pipeline`); `None` for a graph authored directly against
/// this engine.
pub const GRAPH_SOURCE_KINDS: &[&str] = &["workflow", "pipeline"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub graph_id: String,
    pub node_key: String,
    pub node_type: String,
    /// Node-type-specific config. See `services::graph_runtime_service`'s
    /// module doc comment for the JSON shape each `node_type` expects
    /// (mirrors, e.g., `WorkflowAction.params_json`'s "opaque JSON string,
    /// shape depends on action_type" convention).
    pub config_json: String,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNodeInput {
    pub node_key: String,
    pub node_type: String,
    #[serde(default = "default_config_json")]
    pub config_json: String,
    #[serde(default)]
    pub position_x: Option<f64>,
    #[serde(default)]
    pub position_y: Option<f64>,
    #[serde(default)]
    pub sort_order: i64,
}
fn default_config_json() -> String {
    "{}".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub graph_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub branch_label: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphEdgeInput {
    /// References the *other* nodes in this same `create`/`update` call by
    /// `node_key`, not a real node id - real ids don't exist yet for a
    /// graph being authored in one shot. `execution_graph_service` resolves
    /// `node_key` -> the freshly-inserted `id` before writing the edge row.
    pub from_node_key: String,
    pub to_node_key: String,
    #[serde(default)]
    pub branch_label: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGraph {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub version: i64,
    pub source_kind: Option<String>,
    pub source_id: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGraphInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub nodes: Vec<GraphNodeInput>,
    pub edges: Vec<GraphEdgeInput>,
}
