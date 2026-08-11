//! FR-CFG: admin-defined custom fields on every major entity, via an
//! attribute side-table rather than a schema change per field. Also backs
//! custom fields on admin-defined custom objects (see
//! custom_object_service) - a custom object's key is just one more
//! entity_type string here, validated dynamically instead of against the
//! fixed CUSTOM_FIELD_ENTITY_TYPES list.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::custom_field::{
    CustomFieldDefinition, CustomFieldDefinitionInput, CustomFieldDefinitionUpdate, CustomFieldValues, CUSTOM_FIELD_TYPES,
};
use crate::models::business_rule::builtin_trigger_field_for;
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_field_repo, invoice_repo, opportunity_repo, order_repo,
    product_repo, quote_repo, task_repo, user_repo,
};
use crate::services::business_rule_service;

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

fn validate_definition_shape(
    conn: &Connection,
    workspace_id: &str,
    input_entity_type: &str,
    field_type: &str,
    options: &[String],
    label: &str,
) -> AppResult<()> {
    if label.trim().is_empty() {
        return Err(AppError::Validation("Field label is required".into()));
    }
    // Accepts the nine built-in entity types plus any active admin-defined
    // custom object for this workspace - see custom_object_service for why
    // a custom object's records need no special-casing here at all.
    if !super::custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, input_entity_type)? {
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
    validate_definition_shape(conn, workspace_id, &input.entity_type, &input.field_type, &input.options, &input.label)?;
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
    validate_definition_shape(conn, &existing.workspace_id, &existing.entity_type, &existing.field_type, &input.options, &input.label)?;
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

/// Returns (workspace_id, builtin_trigger_value) - the value is whatever
/// `field_rule::builtin_trigger_field_for(entity_type)` names for this
/// entity ("status" for most, "is_active" as "true"/"false" for Product),
/// which field_rule_service needs as trigger context even though
/// custom_field_service otherwise has no reason to know it.
fn resolve_entity_workspace(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<(String, String)> {
    let missing = |what: &str| AppError::Validation(format!("{what} does not exist"));
    match entity_type {
        "Company" => {
            let r = company_repo::get(conn, entity_id)?.ok_or_else(|| missing("Company"))?;
            Ok((r.workspace_id, r.status))
        }
        "Contact" => {
            let contact = contact_repo::get(conn, entity_id)?.ok_or_else(|| missing("Contact"))?;
            let company = company_repo::get(conn, &contact.company_id)?.ok_or_else(|| missing("Contact's company"))?;
            Ok((company.workspace_id, contact.status))
        }
        "Opportunity" => {
            let r = opportunity_repo::get(conn, entity_id)?.ok_or_else(|| missing("Opportunity"))?;
            Ok((r.workspace_id, r.status))
        }
        "Quote" => {
            let r = quote_repo::get(conn, entity_id)?.ok_or_else(|| missing("Quote"))?;
            Ok((r.workspace_id, r.status))
        }
        "Order" => {
            let r = order_repo::get(conn, entity_id)?.ok_or_else(|| missing("Order"))?;
            Ok((r.workspace_id, r.status))
        }
        "Invoice" => {
            let r = invoice_repo::get(conn, entity_id)?.ok_or_else(|| missing("Invoice"))?;
            Ok((r.workspace_id, r.status))
        }
        "Contract" => {
            let r = contract_repo::get(conn, entity_id)?.ok_or_else(|| missing("Contract"))?;
            Ok((r.workspace_id, r.status))
        }
        "Task" => {
            let r = task_repo::get(conn, entity_id)?.ok_or_else(|| missing("Task"))?;
            Ok((r.workspace_id, r.status))
        }
        "Product" => {
            let r = product_repo::get(conn, entity_id)?.ok_or_else(|| missing("Product"))?;
            Ok((r.workspace_id, if r.is_active { "true".into() } else { "false".into() }))
        }
        other => {
            // Not a built-in type - try it as a custom object record.
            // custom_record_repo doesn't need `other` to already be known
            // active/valid here: an orphaned or deactivated object's
            // existing records still need their custom field values
            // readable (just not writable to new ones, which
            // custom_record_service::create/update already gate).
            match crate::repositories::custom_record_repo::get(conn, entity_id)? {
                Some(r) if r.object_key == other => Ok((r.workspace_id, r.status)),
                _ => Err(AppError::Validation(format!("Unsupported custom field entity type '{other}'"))),
            }
        }
    }
}

/// Validates and persists custom field values for one record - required-
/// field enforcement happens here, server-side, not only in the form,
/// since this is called by the same command any client (including a
/// direct API call) would use (ADM-BR makes the same argument for business
/// rules; the reasoning is identical here). Also the one integration point
/// business rules hook into, since every entity's save flow already calls
/// this unconditionally, even when it has no custom field values of its
/// own to save (see each feature screen's save mutation).
///
/// Returns any non-blocking `show_message` texts that fired, for the
/// caller to display - everything else a rule can do either mutates the
/// values being saved (`set_default`/`set_value`), rejects the save
/// (`require`/`block_save`, as an `Err`), or is purely cosmetic and
/// client-enforced (`hide`/`lock`, carried in `field_effects` but not
/// consulted here).
pub fn set_entity_values(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    values: &CustomFieldValues,
    actor_user_id: Option<&str>,
) -> AppResult<Vec<String>> {
    let (workspace_id, builtin_value) = resolve_entity_workspace(conn, entity_type, entity_id)?;
    let definitions = list_definitions(conn, &workspace_id, entity_type, true)?;

    let mut trigger_context: CustomFieldValues = values.clone();
    trigger_context.insert(builtin_trigger_field_for(entity_type).to_string(), builtin_value);
    let evaluation = business_rule_service::evaluate(conn, &workspace_id, entity_type, &trigger_context)?;

    if let Some(reason) = &evaluation.blocked {
        return Err(AppError::Validation(reason.clone()));
    }

    let mut effective_values = values.clone();
    for (key, value) in &evaluation.set_values {
        effective_values.insert(key.clone(), value.clone());
    }

    for def in &definitions {
        if evaluation.field_effects.get(&def.key).map(|e| e.as_str()) == Some("hide") {
            continue;
        }
        let value = effective_values.get(&def.key).map(|s| s.trim()).unwrap_or("");
        let required_by_rule = evaluation.field_effects.get(&def.key).map(|e| e.as_str()) == Some("require");
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
    Ok(evaluation.messages)
}

pub fn get_entity_values(conn: &Connection, entity_id: &str) -> AppResult<CustomFieldValues> {
    Ok(custom_field_repo::get_values(conn, entity_id)?)
}
