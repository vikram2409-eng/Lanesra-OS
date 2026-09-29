//! Raw CRUD for `ai_execution_graphs`/`ai_graph_nodes`/`ai_graph_edges`
//! (migration 0062). See `services::execution_graph_service` for
//! validation/publish-lifecycle and `services::graph_runtime_service` for
//! execution.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension};

use crate::domain::ids::now_iso;
use crate::models::execution_graph::{ExecutionGraph, ExecutionGraphInput, GraphEdge, GraphNode};

fn map_graph_row(row: &rusqlite::Row) -> rusqlite::Result<ExecutionGraph> {
    Ok(ExecutionGraph {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        status: row.get("status")?,
        version: row.get("version")?,
        source_kind: row.get("source_kind")?,
        source_id: row.get("source_id")?,
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
        nodes: Vec::new(), // filled in by `hydrate`
        edges: Vec::new(),
    })
}

fn map_node_row(row: &rusqlite::Row) -> rusqlite::Result<GraphNode> {
    Ok(GraphNode {
        id: row.get("id")?,
        graph_id: row.get("graph_id")?,
        node_key: row.get("node_key")?,
        node_type: row.get("node_type")?,
        config_json: row.get("config_json")?,
        position_x: row.get("position_x")?,
        position_y: row.get("position_y")?,
        sort_order: row.get("sort_order")?,
    })
}

fn map_edge_row(row: &rusqlite::Row) -> rusqlite::Result<GraphEdge> {
    Ok(GraphEdge {
        id: row.get("id")?,
        graph_id: row.get("graph_id")?,
        from_node_id: row.get("from_node_id")?,
        to_node_id: row.get("to_node_id")?,
        branch_label: row.get("branch_label")?,
        sort_order: row.get("sort_order")?,
    })
}

pub fn list_nodes(conn: &Connection, graph_id: &str) -> rusqlite::Result<Vec<GraphNode>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_graph_nodes WHERE graph_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([graph_id], map_node_row)?.collect();
    rows
}

pub fn list_edges(conn: &Connection, graph_id: &str) -> rusqlite::Result<Vec<GraphEdge>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_graph_edges WHERE graph_id = ?1 ORDER BY sort_order")?;
    let rows = stmt.query_map([graph_id], map_edge_row)?.collect();
    rows
}

fn hydrate(conn: &Connection, mut graph: ExecutionGraph) -> rusqlite::Result<ExecutionGraph> {
    graph.nodes = list_nodes(conn, &graph.id)?;
    graph.edges = list_edges(conn, &graph.id)?;
    Ok(graph)
}

/// Deletes and re-inserts every node/edge for `graph_id` from `input`,
/// returning the `node_key -> id` map the caller (`execution_graph_service`)
/// used to resolve `GraphEdgeInput`'s `from_node_key`/`to_node_key` before
/// this was called - same "delete then bulk re-insert in order" convention
/// `ai_agent_pipeline_repo::replace_steps` already uses for a Pipeline's
/// steps, just for two child tables sharing one key resolution pass
/// instead of one.
pub fn replace_nodes_and_edges(conn: &Connection, graph_id: &str, input: &ExecutionGraphInput) -> rusqlite::Result<HashMap<String, String>> {
    conn.execute("DELETE FROM ai_graph_edges WHERE graph_id = ?1", [graph_id])?;
    conn.execute("DELETE FROM ai_graph_nodes WHERE graph_id = ?1", [graph_id])?;

    let mut key_to_id: HashMap<String, String> = HashMap::new();
    for (i, node) in input.nodes.iter().enumerate() {
        let id = crate::domain::ids::new_uuid();
        conn.execute(
            "INSERT INTO ai_graph_nodes (id, graph_id, node_key, node_type, config_json, position_x, position_y, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&id, graph_id, &node.node_key, &node.node_type, &node.config_json, node.position_x, node.position_y, i as i64),
        )?;
        key_to_id.insert(node.node_key.clone(), id);
    }
    for (i, edge) in input.edges.iter().enumerate() {
        let from_id = key_to_id.get(&edge.from_node_key);
        let to_id = key_to_id.get(&edge.to_node_key);
        if let (Some(from_id), Some(to_id)) = (from_id, to_id) {
            conn.execute(
                "INSERT INTO ai_graph_edges (id, graph_id, from_node_id, to_node_id, branch_label, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (crate::domain::ids::new_uuid(), graph_id, from_id, to_id, &edge.branch_label, i as i64),
            )?;
        }
        // An edge naming an unknown node_key is silently dropped here -
        // execution_graph_service::validate_input catches this as a real
        // validation error before this function is ever called, so this is
        // only reached for an already-validated input.
    }
    Ok(key_to_id)
}

pub fn create(conn: &Connection, id: &str, workspace_id: &str, input: &ExecutionGraphInput, actor_user_id: Option<&str>) -> rusqlite::Result<ExecutionGraph> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_execution_graphs (id, workspace_id, name, description, status, version, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, 'draft', 1, ?5, ?6, ?5, ?6)",
        (id, workspace_id, &input.name, &input.description, &now, actor_user_id),
    )?;
    replace_nodes_and_edges(conn, id, input)?;
    get(conn, id).map(|g| g.expect("just inserted"))
}

/// Generalized creation entry point for `execution_graph_service::
/// graph_from_workflow`/`graph_from_pipeline`'s compatibility-shape
/// mapping, which needs `source_kind`/`source_id` set at insert time
/// (`create` above always leaves them `NULL`, for a graph authored
/// directly against this engine).
pub fn create_with_source(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    input: &ExecutionGraphInput,
    source_kind: &str,
    source_id: &str,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<ExecutionGraph> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO ai_execution_graphs (id, workspace_id, name, description, status, version, source_kind, source_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, 'draft', 1, ?5, ?6, ?7, ?8, ?7, ?8)",
        (id, workspace_id, &input.name, &input.description, source_kind, source_id, &now, actor_user_id),
    )?;
    replace_nodes_and_edges(conn, id, input)?;
    get(conn, id).map(|g| g.expect("just inserted"))
}

pub fn update(conn: &Connection, id: &str, input: &ExecutionGraphInput, actor_user_id: Option<&str>) -> rusqlite::Result<ExecutionGraph> {
    let now = now_iso();
    conn.execute(
        "UPDATE ai_execution_graphs SET name = ?1, description = ?2, updated_at = ?3, updated_by = ?4 WHERE id = ?5",
        (&input.name, &input.description, &now, actor_user_id, id),
    )?;
    replace_nodes_and_edges(conn, id, input)?;
    get(conn, id).map(|g| g.expect("just updated"))
}

pub fn set_status(conn: &Connection, id: &str, status: &str, actor_user_id: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE ai_execution_graphs SET status = ?1, updated_at = ?2, updated_by = ?3 WHERE id = ?4",
        (status, now_iso(), actor_user_id, id),
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<ExecutionGraph>> {
    let row = conn.query_row("SELECT * FROM ai_execution_graphs WHERE id = ?1", [id], map_graph_row).optional()?;
    row.map(|g| hydrate(conn, g)).transpose()
}

pub fn list(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<ExecutionGraph>> {
    let mut stmt = conn.prepare("SELECT * FROM ai_execution_graphs WHERE workspace_id = ?1 ORDER BY updated_at DESC")?;
    let rows: Vec<ExecutionGraph> = stmt.query_map([workspace_id], map_graph_row)?.collect::<rusqlite::Result<_>>()?;
    rows.into_iter().map(|g| hydrate(conn, g)).collect()
}
