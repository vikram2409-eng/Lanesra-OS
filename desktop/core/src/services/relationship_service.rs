//! Admin extensibility Phase B (spec §20.3/§21): lets an Administrator
//! define a relationship between any two entity types - built-in or
//! custom object - and lets any user link/unlink actual records through
//! it. Related-list rendering for any record is one function
//! (`related_records_for`) away regardless of how many relationship
//! definitions exist, matching the "custom relationships automatically
//! become eligible for related lists without requiring application code
//! changes" requirement (spec Platform Extensibility §7).

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::relationship::{
    RelatedRecord, RelationshipDefinition, RelationshipDefinitionInput, RelationshipDefinitionUpdate, RelationshipInstance,
    DELETE_BEHAVIORS, RELATIONSHIP_TYPES,
};
use crate::repositories::{relationship_repo, user_repo};
use crate::services::{custom_object_service, entity_registry};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage relationships".into()));
    }
    Ok(())
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

fn slugify(conn: &Connection, workspace_id: &str, source: &str, target: &str) -> AppResult<String> {
    let base = format!("{source}_{target}").to_lowercase();
    let existing = relationship_repo::list_definitions(conn, workspace_id)?;
    let existing_keys: Vec<&str> = existing.iter().map(|d| d.key.as_str()).collect();
    if !existing_keys.contains(&base.as_str()) {
        return Ok(base);
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !existing_keys.contains(&candidate.as_str()) {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

fn validate_shape(conn: &Connection, workspace_id: &str, input: &RelationshipDefinitionInput) -> AppResult<()> {
    require_valid_entity_type(conn, workspace_id, &input.source_entity_type)?;
    require_valid_entity_type(conn, workspace_id, &input.target_entity_type)?;
    if input.source_entity_type == input.target_entity_type {
        return Err(AppError::Validation("A relationship must connect two different object types".into()));
    }
    if !RELATIONSHIP_TYPES.contains(&input.relationship_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid relationship type '{}'", input.relationship_type)));
    }
    if !DELETE_BEHAVIORS.contains(&input.delete_behavior.as_str()) {
        return Err(AppError::Validation(format!("Invalid delete behavior '{}'", input.delete_behavior)));
    }
    if input.forward_label.trim().is_empty() || input.reverse_label.trim().is_empty() {
        return Err(AppError::Validation("Both direction labels are required".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &RelationshipDefinitionInput, actor_user_id: Option<&str>) -> AppResult<RelationshipDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_shape(conn, workspace_id, input)?;
    let key = slugify(conn, workspace_id, &input.source_entity_type, &input.target_entity_type)?;
    let id = crate::domain::ids::new_uuid();
    Ok(relationship_repo::create_definition(conn, &id, workspace_id, &key, input, actor_user_id)?)
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<RelationshipDefinition>> {
    let all = relationship_repo::list_definitions(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|d| d.is_active).collect() } else { all })
}

pub fn update(conn: &Connection, id: &str, input: &RelationshipDefinitionUpdate, actor_user_id: Option<&str>) -> AppResult<RelationshipDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = relationship_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This relationship is protected by the system and cannot be modified".into()));
    }
    if !DELETE_BEHAVIORS.contains(&input.delete_behavior.as_str()) {
        return Err(AppError::Validation(format!("Invalid delete behavior '{}'", input.delete_behavior)));
    }
    if input.forward_label.trim().is_empty() || input.reverse_label.trim().is_empty() {
        return Err(AppError::Validation("Both direction labels are required".into()));
    }
    Ok(relationship_repo::update_definition(conn, id, input, actor_user_id)?)
}

/// Hard-deletes a definition. Blocked while any link through it still
/// exists, mirroring custom_object_service::delete's "archive/deactivate
/// instead, or clear it out first" rule.
pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let existing = relationship_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This relationship is protected by the system and cannot be deleted".into()));
    }
    let count = relationship_repo::count_instances_for_definition(conn, id)?;
    if count > 0 {
        return Err(AppError::Validation(format!(
            "Cannot delete this relationship - {count} record(s) are still linked through it. Unlink them first, or deactivate the relationship instead."
        )));
    }
    Ok(relationship_repo::delete_definition(conn, id)?)
}

fn resolve_or_error(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<entity_registry::ResolvedRecord> {
    entity_registry::resolve(conn, entity_type, entity_id)?
        .ok_or_else(|| AppError::Validation(format!("{entity_type} record does not exist")))
}

/// Links two records through `definition_id`. `source_entity_type`/
/// `target_entity_type` must match the definition's own orientation
/// exactly - the frontend always knows which side it's calling from, so
/// this stays a direct, unambiguous write rather than guessing direction.
pub fn link(
    conn: &Connection,
    workspace_id: &str,
    definition_id: &str,
    source_entity_type: &str,
    source_id: &str,
    target_entity_type: &str,
    target_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<RelationshipInstance> {
    let def = relationship_repo::get_definition(conn, definition_id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if !def.is_active {
        return Err(AppError::Validation("This relationship is not active".into()));
    }
    if def.source_entity_type != source_entity_type || def.target_entity_type != target_entity_type {
        return Err(AppError::Validation("Record types do not match this relationship's definition".into()));
    }

    let source = resolve_or_error(conn, source_entity_type, source_id)?;
    let target = resolve_or_error(conn, target_entity_type, target_id)?;
    if source.workspace_id != workspace_id || target.workspace_id != workspace_id {
        return Err(AppError::Validation("Both records must belong to this workspace".into()));
    }

    match def.relationship_type.as_str() {
        "many_to_one" => {
            if relationship_repo::count_by_source(conn, definition_id, source_id)? > 0 {
                return Err(AppError::Validation(format!(
                    "This {source_entity_type} record is already linked as {} - unlink it first to relink",
                    def.forward_label
                )));
            }
        }
        "one_to_one" => {
            if relationship_repo::count_by_source(conn, definition_id, source_id)? > 0
                || relationship_repo::count_by_target(conn, definition_id, target_id)? > 0
            {
                return Err(AppError::Validation("One or both records are already linked through this one-to-one relationship".into()));
            }
        }
        _ => {} // many_to_many: only the UNIQUE(definition, source, target) constraint applies
    }

    let id = crate::domain::ids::new_uuid();
    relationship_repo::create_instance(conn, &id, workspace_id, definition_id, source_entity_type, source_id, target_entity_type, target_id, actor_user_id)
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                AppError::Validation("These two records are already linked".into())
            }
            other => AppError::from(other),
        })
}

pub fn unlink(conn: &Connection, instance_id: &str, _actor_user_id: Option<&str>) -> AppResult<()> {
    relationship_repo::get_instance(conn, instance_id)?.ok_or_else(|| AppError::NotFound("Link".into()))?;
    Ok(relationship_repo::delete_instance(conn, instance_id)?)
}

/// Every related record for `entity_type`/`entity_id`, across every active
/// relationship it participates in from either direction - what a detail
/// page's "related lists" section renders. `show_related_list = 0`
/// definitions are still linkable but omitted here, since that flag exists
/// specifically to keep an internal-only relationship out of this view.
pub fn related_records_for(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str) -> AppResult<Vec<RelatedRecord>> {
    let defs = relationship_repo::list_definitions_for_entity(conn, workspace_id, entity_type)?;
    let mut out = Vec::new();

    for def in defs.iter().filter(|d| d.show_related_list) {
        if def.source_entity_type == entity_type {
            for inst in relationship_repo::list_instances_where_source(conn, &def.id, entity_id)? {
                if let Some(other) = entity_registry::resolve(conn, &inst.target_entity_type, &inst.target_id)? {
                    out.push(RelatedRecord {
                        instance_id: inst.id,
                        relationship_definition_id: def.id.clone(),
                        relationship_key: def.key.clone(),
                        label: def.forward_label.clone(),
                        entity_type: inst.target_entity_type,
                        entity_id: inst.target_id,
                        display_name: other.display_name,
                        status: other.status,
                        archived: other.archived,
                    });
                }
            }
        }
        // Not `else if` - a self-referencing-shaped pair of custom objects
        // could theoretically match both; in practice source != target is
        // enforced at creation, so at most one branch ever contributes rows
        // for a given definition.
        if def.target_entity_type == entity_type {
            for inst in relationship_repo::list_instances_where_target(conn, &def.id, entity_id)? {
                if let Some(other) = entity_registry::resolve(conn, &inst.source_entity_type, &inst.source_id)? {
                    out.push(RelatedRecord {
                        instance_id: inst.id,
                        relationship_definition_id: def.id.clone(),
                        relationship_key: def.key.clone(),
                        label: def.reverse_label.clone(),
                        entity_type: inst.source_entity_type,
                        entity_id: inst.source_id,
                        display_name: other.display_name,
                        status: other.status,
                        archived: other.archived,
                    });
                }
            }
        }
    }

    Ok(out)
}

/// Checked before a record is archived/deleted: any `restrict` relationship
/// still linking to it blocks the operation; any `archive` relationship
/// has its link rows silently cleared instead (ADM-CR-06 - "destructive
/// cascade is not the default", so the linked *other* record is never
/// touched, only the link itself).
pub fn enforce_delete_behavior(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<()> {
    let instances = relationship_repo::list_instances_for_record(conn, entity_type, entity_id)?;
    if instances.is_empty() {
        return Ok(());
    }

    let mut to_clear = Vec::new();
    for inst in &instances {
        let def = relationship_repo::get_definition(conn, &inst.relationship_definition_id)?;
        let restrict = def.map(|d| d.delete_behavior == "restrict").unwrap_or(true);
        if restrict {
            return Err(AppError::Validation(
                "This record is still linked to other records through a custom relationship - unlink it first, or change the relationship's delete behavior to Archive".into(),
            ));
        }
        to_clear.push(inst.id.clone());
    }
    for id in to_clear {
        relationship_repo::delete_instance(conn, &id)?;
    }
    Ok(())
}
