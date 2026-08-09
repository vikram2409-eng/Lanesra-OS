//! FR-CFG: admin-defined custom fields on Companies and Contacts, via an
//! attribute side-table rather than a schema change per field. Bounded to
//! these two entities in Phase 1 - see the product backlog for why "let
//! admins define whole new entity types" is a separate, much larger ask
//! this deliberately doesn't attempt.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::custom_field::{
    CustomFieldDefinition, CustomFieldDefinitionInput, CustomFieldDefinitionUpdate, CustomFieldValues,
    CUSTOM_FIELD_ENTITY_TYPES, CUSTOM_FIELD_TYPES,
};
use crate::repositories::{company_repo, contact_repo, custom_field_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation(
            "Only an Administrator can manage custom fields".into(),
        ));
    }
    Ok(())
}

/// Turns a label into a stable field key: lowercase, non-alphanumeric
/// runs collapsed to a single underscore, trimmed. Auto-uniquified
/// against existing keys for the same entity type by appending "_2",
/// "_3", etc., rather than rejecting the save outright - a duplicate
/// label ("Industry" twice) is a plausible admin mistake, not something
/// that should block them from finishing the form.
fn slugify(conn: &Connection, workspace_id: &str, entity_type: &str, label: &str) -> AppResult<String> {
    let mut key = String::new();
    let mut last_was_sep = true; // avoids a leading underscore
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
        return Err(AppError::Validation("Field label must contain at least one letter or number".into()));
    }

    let existing = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
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

fn validate_definition_shape(input_entity_type: &str, field_type: &str, options: &[String], label: &str) -> AppResult<()> {
    if label.trim().is_empty() {
        return Err(AppError::Validation("Field label is required".into()));
    }
    if !CUSTOM_FIELD_ENTITY_TYPES.contains(&input_entity_type) {
        return Err(AppError::Validation(format!("Invalid entity type '{input_entity_type}'")));
    }
    if !CUSTOM_FIELD_TYPES.contains(&field_type) {
        return Err(AppError::Validation(format!("Invalid field type '{field_type}'")));
    }
    if field_type == "select" && options.is_empty() {
        return Err(AppError::Validation("A select field needs at least one option".into()));
    }
    Ok(())
}

pub fn create_definition(
    conn: &Connection,
    workspace_id: &str,
    input: &CustomFieldDefinitionInput,
    actor_user_id: Option<&str>,
) -> AppResult<CustomFieldDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_definition_shape(&input.entity_type, &input.field_type, &input.options, &input.label)?;
    let key = slugify(conn, workspace_id, &input.entity_type, &input.label)?;
    let id = crate::domain::ids::new_uuid();
    Ok(custom_field_repo::create_definition(conn, &id, workspace_id, &key, input, actor_user_id)?)
}

/// Any authenticated user can list active definitions (needed to render
/// the Company/Contact form); only an Administrator sees inactive ones
/// too, via the admin screen.
pub fn list_definitions(conn: &Connection, workspace_id: &str, entity_type: &str, active_only: bool) -> AppResult<Vec<CustomFieldDefinition>> {
    let all = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
    Ok(if active_only { all.into_iter().filter(|d| d.is_active).collect() } else { all })
}

pub fn update_definition(
    conn: &Connection,
    id: &str,
    input: &CustomFieldDefinitionUpdate,
    actor_user_id: Option<&str>,
) -> AppResult<CustomFieldDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = custom_field_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Custom field".into()))?;
    validate_definition_shape(&existing.entity_type, &existing.field_type, &input.options, &input.label)?;
    if existing.field_type == "select" && input.options.is_empty() {
        return Err(AppError::Validation("A select field needs at least one option".into()));
    }
    Ok(custom_field_repo::update_definition(conn, id, input, actor_user_id)?)
}

pub fn deactivate_definition(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<CustomFieldDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = custom_field_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Custom field".into()))?;
    let update = CustomFieldDefinitionUpdate {
        label: existing.label,
        options: existing.options,
        required: existing.required,
        show_in_list: existing.show_in_list,
        sort_order: existing.sort_order,
        is_active: false,
    };
    Ok(custom_field_repo::update_definition(conn, id, &update, actor_user_id)?)
}

/// Returns (workspace_id, status) - status is the entity's built-in
/// status field, which field_rule_service needs as the "status" trigger
/// context even though custom_field_service otherwise has no reason to
/// know it.
fn resolve_entity_workspace(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<(String, String)> {
    match entity_type {
        "Company" => {
            let company = company_repo::get(conn, entity_id)?.ok_or_else(|| AppError::Validation("Company does not exist".into()))?;
            Ok((company.workspace_id, company.status))
        }
        "Contact" => {
            let contact = contact_repo::get(conn, entity_id)?.ok_or_else(|| AppError::Validation("Contact does not exist".into()))?;
            let company = company_repo::get(conn, &contact.company_id)?
                .ok_or_else(|| AppError::Validation("Contact's company does not exist".into()))?;
            Ok((company.workspace_id, contact.status))
        }
        other => Err(AppError::Validation(format!("Unsupported custom field entity type '{other}'"))),
    }
}

/// Validates and persists custom field values for one Company/Contact
/// record - required-field enforcement happens here, server-side, not
/// only in the form, since this is called by the same command any client
/// (including a direct API call) would use (FR-RUL-05 makes the same
/// argument for business rules; the reasoning is identical here).
pub fn set_entity_values(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    values: &CustomFieldValues,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    let (workspace_id, status) = resolve_entity_workspace(conn, entity_type, entity_id)?;
    let definitions = list_definitions(conn, &workspace_id, entity_type, true)?;

    // FR-RUL-05: a field required by an active business rule is enforced
    // here too, not only in the form - the same reasoning as the static
    // `required` flag just above. `hide` is not enforced here: it's a
    // purely cosmetic effect with nothing to validate, so a hidden field
    // is simply left untouched (skipped) rather than cleared or blocked.
    let mut trigger_context: CustomFieldValues = values.clone();
    trigger_context.insert("status".to_string(), status);
    let rule_effects = crate::services::field_rule_service::effects_for(conn, &workspace_id, entity_type, &trigger_context)?;

    for def in &definitions {
        if rule_effects.get(&def.key).map(|e| e.as_str()) == Some("hide") {
            continue;
        }
        let value = values.get(&def.key).map(|s| s.trim()).unwrap_or("");
        let required_by_rule = rule_effects.get(&def.key).map(|e| e.as_str()) == Some("require");
        if (def.required || required_by_rule) && value.is_empty() {
            return Err(AppError::Validation(format!("{} is required", def.label)));
        }
        if !value.is_empty() {
            match def.field_type.as_str() {
                "number" if value.parse::<f64>().is_err() => {
                    return Err(AppError::Validation(format!("{} must be a number", def.label)));
                }
                "select" if !def.options.iter().any(|o| o == value) => {
                    return Err(AppError::Validation(format!("'{value}' is not a valid option for {}", def.label)));
                }
                _ => {}
            }
        }
        custom_field_repo::set_value(conn, &def.id, entity_id, value)?;
    }

    let _ = actor_user_id; // no audit entry per value write - the parent record's own create/update audit entry covers this edit
    Ok(())
}

pub fn get_entity_values(conn: &Connection, entity_id: &str) -> AppResult<CustomFieldValues> {
    Ok(custom_field_repo::get_values(conn, entity_id)?)
}
