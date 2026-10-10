//! Next-Gen program, Domain A (Intelligence Foundation), FND-02: an
//! object/field's mapping to a Business Glossary term and/or a fixed
//! semantic role. A mapping has no System Graph node of its own - it's
//! purely what drives the *glossary term's* own node edges (see
//! `glossary_service::sync_graph_edges`, called here on create/delete).

use rusqlite::Connection;

use crate::domain::{builtin_fields, AppError, AppResult};
use crate::models::semantic::{SemanticMappingInput, SEMANTIC_ROLES};
use crate::repositories::{audit_repo, custom_field_repo, glossary_repo, semantic_mapping_repo};
use crate::services::{access_service, custom_object_service, entity_registry};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "SemanticMapping", "Only an Administrator can manage semantic mappings")
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

fn require_valid_field(conn: &Connection, workspace_id: &str, entity_type: &str, field_key: &str) -> AppResult<()> {
    if builtin_fields::builtin_fields_for(entity_type).iter().any(|f| f.key == field_key) {
        return Ok(());
    }
    if custom_field_repo::list_definitions(conn, workspace_id, entity_type)?.iter().any(|d| d.key == field_key) {
        return Ok(());
    }
    Err(AppError::Validation(format!("'{field_key}' is not a recognized field on '{entity_type}'")))
}

fn validate(conn: &Connection, workspace_id: &str, input: &SemanticMappingInput) -> AppResult<()> {
    require_valid_entity_type(conn, workspace_id, &input.entity_type)?;
    if let Some(field_key) = &input.field_key {
        require_valid_field(conn, workspace_id, &input.entity_type, field_key)?;
    }
    if let Some(term_id) = &input.glossary_term_id {
        glossary_repo::get(conn, term_id)?.ok_or_else(|| AppError::Validation("Selected glossary term does not exist".into()))?;
    }
    if let Some(role) = &input.semantic_role {
        if !SEMANTIC_ROLES.contains(&role.as_str()) {
            return Err(AppError::Validation(format!("'{role}' is not a recognized semantic role")));
        }
    }
    if input.glossary_term_id.is_none() && input.semantic_role.is_none() {
        return Err(AppError::Validation("A mapping needs a glossary term, a semantic role, or both".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &SemanticMappingInput, actor_user_id: Option<&str>) -> AppResult<crate::models::semantic::SemanticMapping> {
    require_admin(conn, actor_user_id)?;
    validate(conn, workspace_id, input)?;
    if semantic_mapping_repo::exact_duplicate_exists(conn, workspace_id, input)? {
        return Err(AppError::Conflict("This exact mapping already exists".into()));
    }
    let id = crate::domain::ids::new_uuid();
    let created = semantic_mapping_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    if let Some(term_id) = &created.glossary_term_id {
        if let Some(term) = glossary_repo::get(conn, term_id)? {
            super::glossary_service::sync_graph_edges(conn, workspace_id, &term)?;
        }
    }
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("semantic_mapping"), Some(&created.id), &format!("Mapped {}{} ", created.entity_type, created.field_key.as_deref().map(|f| format!(".{f}")).unwrap_or_default()), None)?;
    Ok(created)
}

pub fn list_for_entity(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<crate::models::semantic::SemanticMapping>> {
    Ok(semantic_mapping_repo::list_for_entity(conn, workspace_id, entity_type)?)
}

pub fn list_for_term(conn: &Connection, term_id: &str) -> AppResult<Vec<crate::models::semantic::SemanticMapping>> {
    Ok(semantic_mapping_repo::list_for_term(conn, term_id)?)
}

pub fn delete(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let existing = semantic_mapping_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Mapping".into()))?;
    semantic_mapping_repo::delete(conn, id)?;
    if let Some(term_id) = &existing.glossary_term_id {
        if let Some(term) = glossary_repo::get(conn, term_id)? {
            super::glossary_service::sync_graph_edges(conn, workspace_id, &term)?;
        }
    }
    audit_repo::record(conn, workspace_id, actor_user_id, "delete", Some("semantic_mapping"), Some(id), &format!("Removed mapping on {}", existing.entity_type), None)?;
    Ok(())
}
