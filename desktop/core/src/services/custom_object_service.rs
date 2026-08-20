//! Admin extensibility (spec §20.2): lets an Administrator define a whole
//! new business object at runtime - "Vendors", "Assets", "Projects" -
//! without a code change. A custom object's `key` is a lowercase slug that
//! becomes the `entity_type` value everywhere custom fields, business
//! rules and (from Phase D) workflow automation already key off that
//! string for built-in entities. Those subsystems needed no schema change
//! to support this; they only needed their entity_type validation to also
//! accept "any active custom object key for this workspace", which
//! `is_valid_dynamic_entity_type` below provides as a single shared check.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::custom_field::CUSTOM_FIELD_ENTITY_TYPES;
use crate::models::custom_object::{CustomObjectDefinition, CustomObjectDefinitionInput, CustomObjectDefinitionUpdate};
use crate::repositories::{custom_object_repo, custom_record_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage custom objects".into()));
    }
    Ok(())
}

/// True when `entity_type` is either one of the built-in Rust-level types,
/// or an active custom object defined for this workspace. Shared by
/// custom_field_service and custom_report_service so a custom object's
/// records get custom fields, business rules and reports "for free" -
/// nothing in those subsystems is keyed off entity_type any differently
/// than a built-in one.
pub fn is_valid_dynamic_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<bool> {
    if CUSTOM_FIELD_ENTITY_TYPES.contains(&entity_type) {
        return Ok(true);
    }
    Ok(custom_object_repo::get_by_key(conn, workspace_id, entity_type)?
        .map(|d| d.is_active)
        .unwrap_or(false))
}

/// Turns a label into a stable lowercase key - the same approach
/// custom_field_service::slugify uses for field keys, just scoped to
/// object definitions and workspace-unique instead of per-entity-type
/// unique. Auto-uniquified against existing keys the same way.
fn slugify(conn: &Connection, workspace_id: &str, label: &str) -> AppResult<String> {
    let mut key = String::new();
    let mut last_was_sep = true;
    for ch in label.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
            last_was_sep = false;
        } else if !last_was_sep {
            key.push('_');
            last_was_sep = true;
        }
    }
    let key = key.trim_end_matches('_').to_string();
    if key.is_empty() {
        return Err(AppError::Validation("Object name must contain at least one letter or number".into()));
    }
    // A custom object literally named e.g. "Company" would slug-collide in
    // spirit (not in practice, since built-in types are always PascalCase
    // and this key is always lowercase) with a built-in entity type - block
    // it outright rather than leave a confusing near-miss in the admin UI.
    if CUSTOM_FIELD_ENTITY_TYPES.iter().any(|t| t.eq_ignore_ascii_case(&key)) {
        return Err(AppError::Validation(format!("'{label}' is too close to a built-in object name - choose another")));
    }

    let existing = custom_object_repo::list(conn, workspace_id)?;
    let existing_keys: Vec<&str> = existing.iter().map(|d| d.key.as_str()).collect();
    if !existing_keys.contains(&key.as_str()) {
        return Ok(key);
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{key}_{suffix}");
        if !existing_keys.contains(&candidate.as_str()) {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

fn validate_shape(input: &CustomObjectDefinitionInput) -> AppResult<()> {
    if input.singular_label.trim().is_empty() {
        return Err(AppError::Validation("Singular name is required".into()));
    }
    if input.plural_label.trim().is_empty() {
        return Err(AppError::Validation("Plural name is required".into()));
    }
    let prefix = input.prefix.trim();
    if prefix.is_empty() || prefix.chars().count() > 20 {
        return Err(AppError::Validation("Record-number prefix must be 1-20 characters".into()));
    }
    if !(1..=10).contains(&input.digits) {
        return Err(AppError::Validation("Digit width must be between 1 and 10".into()));
    }
    if input.icon.trim().is_empty() {
        return Err(AppError::Validation("Choose an icon".into()));
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &CustomObjectDefinitionInput, actor_user_id: Option<&str>) -> AppResult<CustomObjectDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_shape(input)?;
    let key = slugify(conn, workspace_id, &input.singular_label)?;
    let id = crate::domain::ids::new_uuid();
    let created = custom_object_repo::create(conn, &id, workspace_id, &key, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "custom_object", &created.id, actor_user_id)?;
    Ok(created)
}

/// Industry Data Model packages (spec: roadmap "Industry Data Model") need
/// a *deterministic* key, not the admin UI's auto-slugified-and-uniquified
/// one: a package's other entries (a field's `entity_type`, a
/// relationship's source/target, a business rule/workflow's entity_type)
/// name this object by the exact key the package author chose, so a
/// silent "_2" suffix on collision would silently break every one of
/// those references. Collision is therefore a hard install failure here,
/// never an auto-rename - see `industry_package_service::validate` for
/// where that's actually checked before any object exists.
pub fn create_with_key(
    conn: &Connection,
    workspace_id: &str,
    key: &str,
    input: &CustomObjectDefinitionInput,
    actor_user_id: Option<&str>,
) -> AppResult<CustomObjectDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_shape(input)?;
    if key.trim().is_empty() {
        return Err(AppError::Validation("Object key is required".into()));
    }
    if CUSTOM_FIELD_ENTITY_TYPES.iter().any(|t| t.eq_ignore_ascii_case(key)) {
        return Err(AppError::Validation(format!("'{key}' is too close to a built-in object name - choose another")));
    }
    if custom_object_repo::get_by_key(conn, workspace_id, key)?.is_some() {
        return Err(AppError::Validation(format!("An object with key '{key}' already exists in this workspace")));
    }
    let id = crate::domain::ids::new_uuid();
    let created = custom_object_repo::create(conn, &id, workspace_id, key, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "custom_object", &created.id, actor_user_id)?;
    Ok(created)
}

/// Any authenticated user can list active object definitions (needed to
/// build the sidebar nav and quick-create menu); only an Administrator
/// sees inactive ones too, via the admin screen.
pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<CustomObjectDefinition>> {
    let all = custom_object_repo::list(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|d| d.is_active).collect() } else { all })
}

/// Used by `industry_package_service::export_local_workspace` to read a
/// tagged `solution_components` id back into a full definition - the
/// `metadata_id` component-tagging stores is this id, not the key.
pub fn get(conn: &Connection, id: &str) -> AppResult<Option<CustomObjectDefinition>> {
    Ok(custom_object_repo::get(conn, id)?)
}

pub fn get_by_key(conn: &Connection, workspace_id: &str, key: &str) -> AppResult<Option<CustomObjectDefinition>> {
    Ok(custom_object_repo::get_by_key(conn, workspace_id, key)?)
}

pub fn update(conn: &Connection, id: &str, input: &CustomObjectDefinitionUpdate, actor_user_id: Option<&str>) -> AppResult<CustomObjectDefinition> {
    require_admin(conn, actor_user_id)?;
    custom_object_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Custom object".into()))?;
    if input.singular_label.trim().is_empty() || input.plural_label.trim().is_empty() {
        return Err(AppError::Validation("Singular and plural names are required".into()));
    }
    let prefix = input.prefix.trim();
    if prefix.is_empty() || prefix.chars().count() > 20 {
        return Err(AppError::Validation("Record-number prefix must be 1-20 characters".into()));
    }
    if !(1..=10).contains(&input.digits) {
        return Err(AppError::Validation("Digit width must be between 1 and 10".into()));
    }
    Ok(custom_object_repo::update(conn, id, input, actor_user_id)?)
}

/// Deactivating an object hides it from navigation and new-record creation
/// but keeps its records, fields, rules and data fully intact - always
/// allowed, since it's non-destructive (ADM-CO-10's "archive it" path).
pub fn deactivate(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<CustomObjectDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = custom_object_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Custom object".into()))?;
    let update = CustomObjectDefinitionUpdate {
        singular_label: existing.singular_label,
        plural_label: existing.plural_label,
        icon: existing.icon,
        prefix: existing.prefix,
        digits: existing.digits,
        is_active: false,
    };
    Ok(custom_object_repo::update(conn, id, &update, actor_user_id)?)
}

/// Hard-deletes an object definition. Blocked while any record - active or
/// archived - still exists (ADM-CO-10): the admin's own guarded path is
/// archiving or deleting every record first, not an automatic cascade.
pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let existing = custom_object_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Custom object".into()))?;
    let remaining = custom_record_repo::list_all(conn, &existing.workspace_id, &existing.key)?;
    if !remaining.is_empty() {
        return Err(AppError::Validation(format!(
            "Cannot delete '{}' - {} record(s) still exist. Delete or archive them first, or deactivate the object instead.",
            existing.plural_label,
            remaining.len()
        )));
    }
    Ok(custom_object_repo::delete(conn, id)?)
}
