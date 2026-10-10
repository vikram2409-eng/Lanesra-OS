//! Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
//! Lanesra System Graph's public API.
//!
//! `sync_node`/`remove_node` are the "leaf-dependency" half - called
//! inline, post-write, from each of the 9 owning services (the same
//! shape `audit_repo::record` already established for the whole
//! codebase), so this module never calls back into any of them and there
//! is no circular dependency to manage. The query half
//! (`get_dependents`/`get_dependencies`/`get_impact`/`get_lineage`) is
//! what the Dependency Explorer (and, later, Domain G's breaking-change
//! detection) actually calls.
//!
//! See `models::system_graph`'s module doc comment for the v1 node/edge
//! type scope - this module has no opinion on which 9 types exist today;
//! it just syncs/queries whatever `(node_type, component_id)` it's given.

use rusqlite::Connection;

use crate::domain::errors::AppResult;
use crate::models::system_graph::{SystemEdgeTarget, SystemGraphHit, SystemNode};
use crate::repositories::system_graph_repo;

/// A traversal more than this many hops out is almost certainly not a
/// useful answer to "what depends on this" - bounds `get_impact`/
/// `get_lineage` the same way the Policy Engine already bounds agent
/// delegation depth, and is the actual safety net against a real cycle
/// (e.g. a misconfigured delegate-agent loop), not just a performance
/// nicety.
pub const MAX_TRAVERSAL_DEPTH: i64 = 25;

/// Upserts the node for `(node_type, component_id)` and replaces every
/// outgoing edge it should have, in one call. Each `SystemEdgeTarget`'s
/// destination is resolved by its own natural key; if the target hasn't
/// synced yet this transaction (e.g. a Workflow invoking an Agent that
/// hasn't been synced in this same call ordering), a minimal stub node is
/// created for it so the edge is never silently dropped - the stub gets
/// its real label on that component's own next sync.
pub fn sync_node(
    conn: &Connection,
    workspace_id: &str,
    node_type: &str,
    component_id: &str,
    label: &str,
    metadata_json: &str,
    edges: &[SystemEdgeTarget],
) -> AppResult<()> {
    let from_id = system_graph_repo::upsert_node(conn, workspace_id, node_type, component_id, label, metadata_json)?;

    let mut resolved: Vec<(&str, String)> = Vec::with_capacity(edges.len());
    for edge in edges {
        let to_id = match system_graph_repo::get_node_by_component(conn, workspace_id, &edge.to_node_type, &edge.to_component_id)? {
            Some(node) => node.id,
            // Stub: same component_id, a placeholder label - the owning
            // service's own next sync (create/update of that component)
            // overwrites the label via the same upsert_node ON CONFLICT
            // path, never a second code path.
            None => system_graph_repo::upsert_node(conn, workspace_id, &edge.to_node_type, &edge.to_component_id, &edge.to_component_id, "{}")?,
        };
        resolved.push((edge.edge_type.as_str(), to_id));
    }
    let edge_refs: Vec<(&str, &str)> = resolved.iter().map(|(t, id)| (*t, id.as_str())).collect();
    system_graph_repo::replace_outgoing_edges(conn, workspace_id, &from_id, &edge_refs)?;
    Ok(())
}

/// Hard-removes a node and (via `ON DELETE CASCADE`) every edge touching
/// it. Only call this from a component's own hard-delete path - a
/// soft-deactivate must leave the node in place, so "what used to depend
/// on this" stays answerable from the graph even after the component
/// itself is inactive.
pub fn remove_node(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<()> {
    system_graph_repo::remove_node(conn, workspace_id, node_type, component_id)?;
    Ok(())
}

fn resolve_root(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Option<String>> {
    Ok(system_graph_repo::get_node_by_component(conn, workspace_id, node_type, component_id)?.map(|n| n.id))
}

/// This component's own node, if it has synced at least once.
pub fn get_node(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Option<SystemNode>> {
    Ok(system_graph_repo::get_node_by_component(conn, workspace_id, node_type, component_id)?)
}

pub fn list_nodes_by_type(conn: &Connection, workspace_id: &str, node_type: &str) -> AppResult<Vec<SystemNode>> {
    Ok(system_graph_repo::list_nodes_by_type(conn, workspace_id, node_type)?)
}

/// Direct (depth-1) downstream neighbors - "what does this depend on /
/// invoke / delegate to".
pub fn get_dependencies(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Vec<SystemGraphHit>> {
    match resolve_root(conn, workspace_id, node_type, component_id)? {
        Some(id) => Ok(system_graph_repo::traverse(conn, &id, true, 1)?),
        None => Ok(Vec::new()),
    }
}

/// Direct (depth-1) upstream neighbors - "what depends on / invokes /
/// delegates to this".
pub fn get_dependents(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Vec<SystemGraphHit>> {
    match resolve_root(conn, workspace_id, node_type, component_id)? {
        Some(id) => Ok(system_graph_repo::traverse(conn, &id, false, 1)?),
        None => Ok(Vec::new()),
    }
}

/// Transitive downstream closure, bounded by `MAX_TRAVERSAL_DEPTH` - "if
/// I change this, what's the full blast radius of things it feeds into".
pub fn get_lineage(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Vec<SystemGraphHit>> {
    match resolve_root(conn, workspace_id, node_type, component_id)? {
        Some(id) => Ok(system_graph_repo::traverse(conn, &id, true, MAX_TRAVERSAL_DEPTH)?),
        None => Ok(Vec::new()),
    }
}

/// Transitive upstream closure, bounded by `MAX_TRAVERSAL_DEPTH` - "if I
/// change this, what's the full blast radius of things that depend on
/// it" - the direct generalization of
/// `custom_object_service::count_references` from a flat per-domain count
/// to a real transitive answer. Minimum acceptance criterion for FND-01:
/// this must return every transitive dependent with no false missing
/// references, for the 9 node types this v1 slice syncs.
pub fn get_impact(conn: &Connection, workspace_id: &str, node_type: &str, component_id: &str) -> AppResult<Vec<SystemGraphHit>> {
    match resolve_root(conn, workspace_id, node_type, component_id)? {
        Some(id) => Ok(system_graph_repo::traverse(conn, &id, false, MAX_TRAVERSAL_DEPTH)?),
        None => Ok(Vec::new()),
    }
}
