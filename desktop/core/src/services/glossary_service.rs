//! Next-Gen program, Domain A (Intelligence Foundation), FND-02: Business
//! Glossary terms. A term's own System Graph node has no outgoing edges
//! of its own - its edges are *derived from* whatever
//! `semantic_mappings` point at it (see `sync_graph_edges` below, called
//! both from here and from `semantic_mapping_service` whenever a mapping
//! referencing this term changes).

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::semantic::{BusinessGlossaryTerm, BusinessGlossaryTermInput, DATA_CLASSIFICATIONS};
use crate::models::system_graph::SystemEdgeTarget;
use crate::repositories::{audit_repo, custom_field_repo, glossary_repo, semantic_mapping_repo};
use crate::services::access_service;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "BusinessGlossaryTerm", "Only an Administrator can manage the business glossary")
}

fn validate(input: &BusinessGlossaryTermInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Term name is required".into()));
    }
    if input.definition.trim().is_empty() {
        return Err(AppError::Validation("Term definition is required".into()));
    }
    if !DATA_CLASSIFICATIONS.contains(&input.data_classification.as_str()) {
        return Err(AppError::Validation(format!("Invalid data classification '{}'", input.data_classification)));
    }
    Ok(())
}

/// Rebuilds `term_id`'s own outgoing `derives_from` edges from its
/// current `semantic_mappings` - called after any mapping pointing at
/// this term is created or removed, so the term's System Graph node
/// always reflects exactly what it's mapped to right now, never a stale
/// edge from a deleted mapping.
pub fn sync_graph_edges(conn: &Connection, workspace_id: &str, term: &BusinessGlossaryTerm) -> AppResult<()> {
    let mappings = semantic_mapping_repo::list_for_term(conn, &term.id)?;
    let mut edges = Vec::with_capacity(mappings.len());
    for m in &mappings {
        match &m.field_key {
            // A custom field has a real `custom_field` node keyed by its
            // own id (see custom_field_service::sync_graph_node) - look
            // it up for real field-level precision. A built-in field has
            // no such row to point at, so the edge falls back to the
            // object itself - still correct, just coarser.
            Some(field_key) => {
                let field_id = custom_field_repo::list_definitions(conn, workspace_id, &m.entity_type)?.into_iter().find(|d| d.key == *field_key).map(|d| d.id);
                match field_id {
                    Some(id) => edges.push(SystemEdgeTarget { edge_type: "derives_from".into(), to_node_type: "custom_field".into(), to_component_id: id }),
                    None => edges.push(SystemEdgeTarget { edge_type: "derives_from".into(), to_node_type: "custom_object".into(), to_component_id: m.entity_type.clone() }),
                }
            }
            None => edges.push(SystemEdgeTarget { edge_type: "derives_from".into(), to_node_type: "custom_object".into(), to_component_id: m.entity_type.clone() }),
        }
    }
    super::system_graph_service::sync_node(conn, workspace_id, "business_glossary_term", &term.id, &term.name, "{}", &edges)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &BusinessGlossaryTermInput, actor_user_id: Option<&str>) -> AppResult<BusinessGlossaryTerm> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    if glossary_repo::get_by_name(conn, workspace_id, input.name.trim())?.is_some() {
        return Err(AppError::Conflict(format!("A glossary term named '{}' already exists", input.name.trim())));
    }
    let id = crate::domain::ids::new_uuid();
    let created = glossary_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "business_glossary_term", &created.id, actor_user_id)?;
    sync_graph_edges(conn, workspace_id, &created)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("business_glossary_term"), Some(&created.id), &format!("Created glossary term '{}'", created.name), None)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<BusinessGlossaryTerm>> {
    Ok(glossary_repo::get(conn, id)?)
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<BusinessGlossaryTerm>> {
    let all = glossary_repo::list(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|t| t.is_active).collect() } else { all })
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &BusinessGlossaryTermInput, actor_user_id: Option<&str>) -> AppResult<BusinessGlossaryTerm> {
    require_admin(conn, actor_user_id)?;
    validate(input)?;
    glossary_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Glossary term".into()))?;
    if let Some(existing) = glossary_repo::get_by_name(conn, workspace_id, input.name.trim())? {
        if existing.id != id {
            return Err(AppError::Conflict(format!("A glossary term named '{}' already exists", input.name.trim())));
        }
    }
    let updated = glossary_repo::update(conn, id, input, actor_user_id)?;
    sync_graph_edges(conn, workspace_id, &updated)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("business_glossary_term"), Some(id), &format!("Updated glossary term '{}'", updated.name), None)?;
    Ok(updated)
}

/// No hard delete - a term can be referenced by mappings and metrics;
/// deactivating keeps its System Graph node (and every existing
/// mapping/metric reference to it) resolvable, same "soft-deactivate
/// keeps the node" rule FND-01 established for every other type.
pub fn deactivate(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<BusinessGlossaryTerm> {
    require_admin(conn, actor_user_id)?;
    let existing = glossary_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Glossary term".into()))?;
    glossary_repo::set_active(conn, id, false, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "deactivate", Some("business_glossary_term"), Some(id), &format!("Deactivated glossary term '{}'", existing.name), None)?;
    glossary_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Glossary term".into()))
}
