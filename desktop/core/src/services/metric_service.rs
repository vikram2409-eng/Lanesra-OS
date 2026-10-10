//! Next-Gen program, Domain A (Intelligence Foundation), FND-02: Metric
//! definitions - a declarative description (aggregation/source/grain/
//! filters/time logic), not a working formula evaluator; see
//! `models::semantic`'s own doc comment for why. Versioned the same
//! "append a JSON snapshot, prune to a cap" way `business_rule_service`
//! already versions a rule on every update.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::semantic::{MetricDefinition, MetricDefinitionInput, MetricVersion, AGGREGATIONS};
use crate::models::system_graph::SystemEdgeTarget;
use crate::repositories::{audit_repo, custom_field_repo, glossary_repo, metric_repo};
use crate::services::{access_service, custom_object_service, entity_registry};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "MetricDefinition", "Only an Administrator can manage metric definitions")
}

fn require_valid_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<()> {
    if entity_registry::CORE_ENTITY_TYPES.contains(&entity_type) {
        return Ok(());
    }
    if custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, entity_type)? {
        return Ok(());
    }
    Err(AppError::Validation(format!("'{entity_type}' is not a recognized object type")))
}

fn validate(conn: &Connection, workspace_id: &str, input: &MetricDefinitionInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Metric name is required".into()));
    }
    if !AGGREGATIONS.contains(&input.aggregation.as_str()) {
        return Err(AppError::Validation(format!("Invalid aggregation '{}'", input.aggregation)));
    }
    require_valid_entity_type(conn, workspace_id, &input.source_entity_type)?;
    if let Some(field_key) = &input.source_field_key {
        let is_builtin = crate::domain::builtin_fields::builtin_fields_for(&input.source_entity_type).iter().any(|f| f.key == field_key);
        let is_custom = custom_field_repo::list_definitions(conn, workspace_id, &input.source_entity_type)?.iter().any(|d| d.key == *field_key);
        if !is_builtin && !is_custom {
            return Err(AppError::Validation(format!("'{field_key}' is not a recognized field on '{}'", input.source_entity_type)));
        }
    }
    if let Some(term_id) = &input.glossary_term_id {
        glossary_repo::get(conn, term_id)?.ok_or_else(|| AppError::Validation("Selected glossary term does not exist".into()))?;
    }
    serde_json::from_str::<serde_json::Value>(&input.filters_json).map_err(|e| AppError::Validation(format!("filters_json must be valid JSON: {e}")))?;
    Ok(())
}

fn slugify(conn: &Connection, workspace_id: &str, name: &str) -> AppResult<String> {
    let mut key = String::new();
    let mut last_was_sep = true;
    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
            last_was_sep = false;
        } else if !last_was_sep {
            key.push('_');
            last_was_sep = true;
        }
    }
    let base = key.trim_matches('_').to_string();
    let base = if base.is_empty() { "metric".to_string() } else { base };
    if metric_repo::get_by_key(conn, workspace_id, &base)?.is_none() {
        return Ok(base);
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if metric_repo::get_by_key(conn, workspace_id, &candidate)?.is_none() {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

/// Resolves this metric's own outgoing edges: it `depends_on` its source
/// object (and the real `custom_field` node for a custom source field,
/// falling back to the object for a built-in one - same resolution
/// `glossary_service::sync_graph_edges` uses), and `derives_from` its
/// glossary term if one is set - the exact traceability chain FND-02's
/// own acceptance criterion asks for ("a metric... can be traced to the
/// exact metadata and sources used").
fn sync_graph_node(conn: &Connection, workspace_id: &str, metric: &MetricDefinition) -> AppResult<()> {
    let mut edges = vec![SystemEdgeTarget { edge_type: "depends_on".into(), to_node_type: "custom_object".into(), to_component_id: metric.source_entity_type.clone() }];
    if let Some(field_key) = &metric.source_field_key {
        let field_id = custom_field_repo::list_definitions(conn, workspace_id, &metric.source_entity_type)?.into_iter().find(|d| d.key == *field_key).map(|d| d.id);
        if let Some(id) = field_id {
            edges.push(SystemEdgeTarget { edge_type: "depends_on".into(), to_node_type: "custom_field".into(), to_component_id: id });
        }
    }
    if let Some(term_id) = &metric.glossary_term_id {
        edges.push(SystemEdgeTarget { edge_type: "derives_from".into(), to_node_type: "business_glossary_term".into(), to_component_id: term_id.clone() });
    }
    super::system_graph_service::sync_node(conn, workspace_id, "metric_definition", &metric.id, &metric.name, "{}", &edges)
}

pub fn create(conn: &Connection, workspace_id: &str, input: &MetricDefinitionInput, actor_user_id: Option<&str>) -> AppResult<MetricDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input)?;
    let key = slugify(conn, workspace_id, input.name.trim())?;
    let id = crate::domain::ids::new_uuid();
    let created = metric_repo::create(conn, &id, workspace_id, &key, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "metric_definition", &created.id, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &created)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("metric_definition"), Some(&created.id), &format!("Created metric '{}'", created.name), None)?;
    Ok(created)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<MetricDefinition>> {
    Ok(metric_repo::get(conn, id)?)
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<MetricDefinition>> {
    let all = metric_repo::list(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|m| m.is_active).collect() } else { all })
}

pub fn update(conn: &Connection, id: &str, workspace_id: &str, input: &MetricDefinitionInput, actor_user_id: Option<&str>) -> AppResult<MetricDefinition> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input)?;
    let existing = metric_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Metric".into()))?;
    let snapshot_json = serde_json::to_string(&existing).expect("MetricDefinition is always serializable");
    metric_repo::insert_version(conn, id, &snapshot_json)?;
    let updated = metric_repo::update(conn, id, input, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &updated)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "update", Some("metric_definition"), Some(id), &format!("Updated metric '{}'", updated.name), None)?;
    Ok(updated)
}

pub fn list_versions(conn: &Connection, metric_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<MetricVersion>> {
    require_admin(conn, actor_user_id)?;
    metric_repo::list_version_rows(conn, metric_id)?
        .into_iter()
        .map(|(id, snapshot_json, saved_at)| {
            let snapshot: MetricDefinition = serde_json::from_str(&snapshot_json).map_err(|e| AppError::Validation(format!("Corrupt metric version snapshot: {e}")))?;
            Ok(MetricVersion { id, metric_definition_id: metric_id.to_string(), snapshot, saved_at })
        })
        .collect()
}

pub fn deactivate(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<MetricDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = metric_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Metric".into()))?;
    metric_repo::set_active(conn, id, false, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "deactivate", Some("metric_definition"), Some(id), &format!("Deactivated metric '{}'", existing.name), None)?;
    metric_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Metric".into()))
}

/// Hard delete is allowed (unlike a glossary term) - nothing else in this
/// v1 slice references a metric by id, so there's no dangling-reference
/// risk to guard against.
pub fn delete(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let existing = metric_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Metric".into()))?;
    metric_repo::delete(conn, id)?;
    super::system_graph_service::remove_node(conn, workspace_id, "metric_definition", id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "delete", Some("metric_definition"), Some(id), &format!("Deleted metric '{}'", existing.name), None)?;
    Ok(())
}
