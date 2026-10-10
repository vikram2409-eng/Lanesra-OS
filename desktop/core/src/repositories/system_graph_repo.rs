//! Raw CRUD + traversal for `system_nodes`/`system_edges` (migration 0073).
//! See `services::system_graph_service` for the public sync/query API this
//! backs, and `models::system_graph`'s module doc comment for the v1
//! node/edge-type scope.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::{new_uuid, now_iso};
use crate::models::system_graph::{SystemEdge, SystemGraphHit, SystemNode};

fn map_node_row(row: &rusqlite::Row) -> rusqlite::Result<SystemNode> {
    Ok(SystemNode {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        node_type: row.get("node_type")?,
        component_id: row.get("component_id")?,
        label: row.get("label")?,
        metadata_json: row.get("metadata_json")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn map_edge_row(row: &rusqlite::Row) -> rusqlite::Result<SystemEdge> {
    Ok(SystemEdge {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        edge_type: row.get("edge_type")?,
        from_node_id: row.get("from_node_id")?,
        to_node_id: row.get("to_node_id")?,
        created_at: row.get("created_at")?,
    })
}

pub fn get_node_by_component(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> rusqlite::Result<Option<SystemNode>> {
    conn.query_row(
        "SELECT * FROM system_nodes WHERE workspace_id = ?1 AND node_type = ?2 AND component_id = ?3",
        params![workspace_id, node_type, component_id],
        map_node_row,
    )
    .optional()
}

pub fn get_node(conn: &Connection, node_id: &str) -> rusqlite::Result<Option<SystemNode>> {
    conn.query_row("SELECT * FROM system_nodes WHERE id = ?1", params![node_id], map_node_row).optional()
}

/// Upserts a node keyed by `(workspace_id, node_type, component_id)` -
/// `id` is only generated on first insert and preserved on every later
/// sync, since `system_edges` rows already reference it. Returns the
/// stable id either way.
pub fn upsert_node(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str, label: &str, metadata_json: &str) -> rusqlite::Result<String> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO system_nodes (id, workspace_id, node_type, component_id, label, metadata_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT (workspace_id, node_type, component_id) DO UPDATE SET
            label = excluded.label, metadata_json = excluded.metadata_json, updated_at = excluded.updated_at",
        params![new_uuid(), workspace_id, node_type, component_id, label, metadata_json, now],
    )?;
    let node = get_node_by_component(conn, workspace_id, node_type, component_id)?.expect("just upserted");
    Ok(node.id)
}

/// Hard-removes a node - only called on a hard delete of its owning
/// component. `ON DELETE CASCADE` on `system_edges.from_node_id`/
/// `to_node_id` removes every edge touching it. A soft-deactivate must
/// NOT call this - see `system_graph_service::remove_node`'s own doc
/// comment.
pub fn remove_node(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM system_nodes WHERE workspace_id = ?1 AND node_type = ?2 AND component_id = ?3",
        params![workspace_id, node_type, component_id],
    )?;
    Ok(())
}

/// Deletes and re-inserts every outgoing edge from `from_node_id` - the
/// same "replace this node's own edges wholesale" shape
/// `execution_graph_repo::replace_nodes_and_edges` already uses for a
/// whole graph, just scoped to one node's outgoing side instead of an
/// entire graph.
pub fn replace_outgoing_edges(conn: &Connection, workspace_id: &str, from_node_id: &str, edges: &[(&str, &str)]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM system_edges WHERE from_node_id = ?1", params![from_node_id])?;
    let now = now_iso();
    for (edge_type, to_node_id) in edges {
        conn.execute(
            "INSERT INTO system_edges (id, workspace_id, edge_type, from_node_id, to_node_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (from_node_id, to_node_id, edge_type) DO NOTHING",
            params![new_uuid(), workspace_id, edge_type, from_node_id, to_node_id, now],
        )?;
    }
    Ok(())
}

/// Every outgoing edge from one node, with no traversal - used by tests
/// and by a future admin "raw edges" debug view; the Dependency
/// Explorer's own queries go through `traverse` below instead.
pub fn list_outgoing_edges(conn: &Connection, from_node_id: &str) -> rusqlite::Result<Vec<SystemEdge>> {
    let mut stmt = conn.prepare("SELECT * FROM system_edges WHERE from_node_id = ?1")?;
    let rows: rusqlite::Result<Vec<SystemEdge>> = stmt.query_map(params![from_node_id], map_edge_row)?.collect();
    rows
}

pub fn list_nodes_by_type(conn: &Connection, workspace_id: &str, node_type: &str) -> rusqlite::Result<Vec<SystemNode>> {
    let mut stmt = conn.prepare("SELECT * FROM system_nodes WHERE workspace_id = ?1 AND node_type = ?2 ORDER BY label")?;
    let rows: rusqlite::Result<Vec<SystemNode>> = stmt.query_map(params![workspace_id, node_type], map_node_row)?.collect();
    rows
}

/// Bounded `WITH RECURSIVE` traversal in one direction. `forward = true`
/// walks `from_node_id -> to_node_id` (downstream/"what does this depend
/// on" when edges are stored root-is-dependent, or "what did this
/// trigger" for an INVOKES/DELEGATES_TO edge - direction is a property of
/// the edge type, not this function); `forward = false` walks the same
/// edges backwards (upstream/"what depends on this"). `max_depth` is a
/// hard safety bound against a real cycle (e.g. a misconfigured
/// delegate-agent loop) - the same "bound runaway recursion" precedent
/// the Policy Engine's autonomy limits already established, not just a
/// performance nicety. Depth-bounding, rather than a visited-set, is
/// sufficient here: a bounded recursive CTE always terminates, and a true
/// cycle just means some nodes are revisited at a deeper depth until the
/// bound cuts it off, which `system_graph_service` dedupes by node id
/// before returning (keeping each node's *shortest* depth).
pub fn traverse(conn: &Connection, root_node_id: &str, forward: bool, max_depth: i64) -> rusqlite::Result<Vec<SystemGraphHit>> {
    let sql = if forward {
        "WITH RECURSIVE reach(node_id, edge_type, depth) AS (
            SELECT to_node_id, edge_type, 1 FROM system_edges WHERE from_node_id = ?1
            UNION ALL
            SELECT e.to_node_id, e.edge_type, reach.depth + 1
            FROM system_edges e JOIN reach ON e.from_node_id = reach.node_id
            WHERE reach.depth < ?2
        )
        SELECT n.*, reach.edge_type AS hit_edge_type, reach.depth AS hit_depth
        FROM system_nodes n JOIN reach ON n.id = reach.node_id"
    } else {
        "WITH RECURSIVE reach(node_id, edge_type, depth) AS (
            SELECT from_node_id, edge_type, 1 FROM system_edges WHERE to_node_id = ?1
            UNION ALL
            SELECT e.from_node_id, e.edge_type, reach.depth + 1
            FROM system_edges e JOIN reach ON e.to_node_id = reach.node_id
            WHERE reach.depth < ?2
        )
        SELECT n.*, reach.edge_type AS hit_edge_type, reach.depth AS hit_depth
        FROM system_nodes n JOIN reach ON n.id = reach.node_id"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<SystemGraphHit> = stmt
        .query_map(params![root_node_id, max_depth], |row| {
            Ok(SystemGraphHit { node: map_node_row(row)?, edge_type: row.get("hit_edge_type")?, depth: row.get("hit_depth")? })
        })?
        .collect::<rusqlite::Result<_>>()?;

    // Keep each node's shortest depth/first-seen edge_type only - a cycle
    // or a diamond-shaped dependency can otherwise surface the same node
    // more than once.
    let mut seen = std::collections::HashMap::new();
    for hit in rows {
        seen.entry(hit.node.id.clone()).or_insert(hit);
    }
    let mut out: Vec<SystemGraphHit> = seen.into_values().collect();
    out.sort_by(|a, b| a.depth.cmp(&b.depth).then_with(|| a.node.label.cmp(&b.node.label)));
    Ok(out)
}
