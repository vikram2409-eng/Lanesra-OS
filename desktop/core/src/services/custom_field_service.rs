//! FR-CFG: admin-defined custom fields on every major entity, via an
//! attribute side-table rather than a schema change per field. Also backs
//! custom fields on admin-defined custom objects (see
//! custom_object_service) - a custom object's key is just one more
//! entity_type string here, validated dynamically instead of against the
//! fixed CUSTOM_FIELD_ENTITY_TYPES list.

use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::custom_field::{
    CustomFieldDefinition, CustomFieldDefinitionInput, CustomFieldDefinitionUpdate, CustomFieldValues, CUSTOM_FIELD_TYPES,
};
use crate::repositories::{
    company_repo, contact_repo, contract_repo, custom_field_repo, invoice_repo, opportunity_repo, order_repo,
    product_repo, quote_repo, task_repo, user_repo,
};
use crate::services::{builtin_field_service, business_rule_service, workflow_service};

/// Non-blocking notices from `set_entity_values` - a rule's blocking
/// effects (`require`/`block_save`) are always an `Err`; these are the two
/// non-blocking severities from `show_error`/`show_warning` (plus legacy
/// `show_message`, folded into `warnings` - see `RuleEvaluation`'s doc
/// comment) for the caller to display.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SaveNotices {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Whether a field is effectively hidden on the form right now - either a
/// rule explicitly hides it, or it's flagged `is_hidden_by_default` and no
/// active rule's `show` action currently overrides that.
fn field_is_hidden(def: &CustomFieldDefinition, effect: Option<&str>) -> bool {
    match effect {
        Some("hide") => true,
        Some("show") => false,
        _ => def.is_hidden_by_default,
    }
}

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

/// ADM-CF-04: validates the optional validation settings themselves, not
/// values against them (that happens in `set_entity_values`) - min/max
/// only make sense for `number`, max_length/regex_pattern only for `text`,
/// and a malformed regex is rejected here rather than at every future save.
/// Addendum Phase 4 extends this with `is_unique` (rejected for `boolean` -
/// only two possible values, see this fn) and `default_value` (validated
/// against the field's own type/options/min/max/etc via
/// `validate_typed_value`, the same check a real save's value gets).
#[allow(clippy::too_many_arguments)]
fn validate_definition_extras(
    field_type: &str,
    options: &[String],
    min_value: Option<&str>,
    max_value: Option<&str>,
    max_length: Option<i64>,
    regex_pattern: Option<&str>,
    is_unique: bool,
    default_value: Option<&str>,
    label: &str,
) -> AppResult<()> {
    if let (Some(min), Some(max)) = (min_value, max_value) {
        if field_type == "number" {
            match (min.parse::<f64>(), max.parse::<f64>()) {
                (Ok(a), Ok(b)) if a > b => return Err(AppError::Validation("Minimum cannot be greater than maximum".into())),
                (Err(_), _) | (_, Err(_)) => return Err(AppError::Validation("Minimum/maximum must be numbers".into())),
                _ => {}
            }
        }
    }
    if (min_value.is_some() || max_value.is_some()) && field_type != "number" {
        return Err(AppError::Validation("Minimum/maximum only apply to number fields".into()));
    }
    if let Some(len) = max_length {
        if field_type != "text" {
            return Err(AppError::Validation("Maximum length only applies to text fields".into()));
        }
        if len <= 0 {
            return Err(AppError::Validation("Maximum length must be positive".into()));
        }
    }
    if let Some(pattern) = regex_pattern {
        if field_type != "text" {
            return Err(AppError::Validation("A pattern only applies to text fields".into()));
        }
        if !pattern.is_empty() && regex::Regex::new(pattern).is_err() {
            return Err(AppError::Validation("That pattern is not a valid regular expression".into()));
        }
    }
    if is_unique && field_type == "boolean" {
        return Err(AppError::Validation("Uniqueness doesn't apply to a yes/no field".into()));
    }
    if let Some(default) = default_value.filter(|d| !d.is_empty()) {
        validate_typed_value(field_type, options, min_value, max_value, max_length, regex_pattern, label, default)?;
    }
    Ok(())
}

/// The same per-type checks a real save's value gets in
/// `set_entity_values` - factored out so a definition's own
/// `default_value` can be validated against its field's rules at
/// definition-save time too, instead of only surfacing a bad default the
/// first time some record tries to fall back to it.
fn validate_typed_value(
    field_type: &str,
    options: &[String],
    min_value: Option<&str>,
    max_value: Option<&str>,
    max_length: Option<i64>,
    regex_pattern: Option<&str>,
    label: &str,
    value: &str,
) -> AppResult<()> {
    match field_type {
        "number" => match value.parse::<f64>() {
            Err(_) => return Err(AppError::Validation(format!("{label} must be a number"))),
            Ok(n) => {
                if let Some(min) = min_value.and_then(|m| m.parse::<f64>().ok()) {
                    if n < min {
                        return Err(AppError::Validation(format!("{label} must be at least {min}")));
                    }
                }
                if let Some(max) = max_value.and_then(|m| m.parse::<f64>().ok()) {
                    if n > max {
                        return Err(AppError::Validation(format!("{label} must be at most {max}")));
                    }
                }
            }
        },
        "text" => {
            if let Some(max_len) = max_length {
                if value.chars().count() as i64 > max_len {
                    return Err(AppError::Validation(format!("{label} must be {max_len} characters or fewer")));
                }
            }
            if let Some(pattern) = regex_pattern.filter(|p| !p.is_empty()) {
                let matches = regex::Regex::new(pattern).map(|re| re.is_match(value)).unwrap_or(true);
                if !matches {
                    return Err(AppError::Validation(format!("{label} does not match the required format")));
                }
            }
        }
        "select" if !options.iter().any(|o| o == value) => {
            return Err(AppError::Validation(format!("'{value}' is not a valid option for {label}")));
        }
        _ => {}
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
    validate_definition_extras(
        &input.field_type, &input.options, input.min_value.as_deref(), input.max_value.as_deref(), input.max_length,
        input.regex_pattern.as_deref(), input.is_unique, input.default_value.as_deref(), &input.label,
    )?;
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
    validate_definition_extras(
        &existing.field_type, &input.options, input.min_value.as_deref(), input.max_value.as_deref(), input.max_length,
        input.regex_pattern.as_deref(), input.is_unique, input.default_value.as_deref(), &input.label,
    )?;
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
        min_value: existing.min_value,
        max_value: existing.max_value,
        max_length: existing.max_length,
        regex_pattern: existing.regex_pattern,
        is_searchable: existing.is_searchable,
        is_filterable: existing.is_filterable,
        is_reportable: existing.is_reportable,
        default_value: existing.default_value,
        is_unique: existing.is_unique,
        help_text: existing.help_text,
        placeholder: existing.placeholder,
        is_hidden_by_default: existing.is_hidden_by_default,
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
/// Returns any non-blocking `show_error`/`show_warning` texts that fired
/// (see `SaveNotices`), for the caller to display - everything else a rule
/// can do either mutates the values being saved (`set_default`/`set_value`/
/// `clear_value`), rejects the save (`require`/`block_save`, as an `Err`),
/// or is purely cosmetic and client-enforced (`hide`/`show`/`lock`/
/// `editable`/`restrict_choices`, carried in `field_effects`/
/// `restricted_choices` but not consulted here) - except `hide` combined
/// with a field's own `is_hidden_by_default`, which this function does
/// enforce server-side (see `field_is_hidden`) since a hidden field must
/// never be required or validated.
pub fn set_entity_values(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    values: &CustomFieldValues,
    actor_user_id: Option<&str>,
) -> AppResult<SaveNotices> {
    let (workspace_id, _) = resolve_entity_workspace(conn, entity_type, entity_id)?;
    let definitions = list_definitions(conn, &workspace_id, entity_type, true)?;
    let before_values = custom_field_repo::get_values(conn, entity_id)?;

    // ADM-BR "any field" targeting: the trigger context is every built-in
    // field's current value (name, industry, owner, dates, ...) - not just
    // the one status/stage field the original engine knew about - merged
    // with the custom field values actually being saved, so a condition can
    // target either kind of field identically (see `domain::conditions`).
    let mut trigger_context: CustomFieldValues = builtin_field_service::field_values(conn, entity_type, entity_id)?;
    for (k, v) in values {
        trigger_context.insert(k.clone(), v.clone());
    }
    let evaluation = business_rule_service::evaluate(conn, &workspace_id, entity_type, &trigger_context)?;

    if let Some(reason) = &evaluation.blocked {
        return Err(AppError::Validation(reason.clone()));
    }

    let mut effective_values = values.clone();
    // Addendum Phase 4: a definition's own default_value is the baseline,
    // applied whenever the caller passed nothing (or blank) for this
    // field - filled in before business rules' set_values below so a
    // rule's own set_default/set_value (which check/overwrite the same
    // "currently empty" condition) still has the final say if both apply.
    for def in &definitions {
        let currently_empty = effective_values.get(&def.key).map(|s| s.trim().is_empty()).unwrap_or(true);
        if currently_empty {
            if let Some(default) = def.default_value.as_deref().filter(|d| !d.is_empty()) {
                effective_values.insert(def.key.clone(), default.to_string());
            }
        }
    }
    for (key, value) in &evaluation.set_values {
        effective_values.insert(key.clone(), value.clone());
    }

    let mut changed_keys = Vec::new();
    for def in &definitions {
        let effect = evaluation.field_effects.get(&def.key).map(|e| e.as_str());
        if field_is_hidden(def, effect) {
            continue;
        }
        let value = effective_values.get(&def.key).map(|s| s.trim()).unwrap_or("");
        let required_by_rule = effect == Some("require");
        if (def.required || required_by_rule) && value.is_empty() {
            return Err(AppError::Validation(format!("{} is required", def.label)));
        }
        if !value.is_empty() {
            validate_typed_value(&def.field_type, &def.options, def.min_value.as_deref(), def.max_value.as_deref(), def.max_length, def.regex_pattern.as_deref(), &def.label, value)?;
            if def.is_unique && custom_field_repo::value_exists_elsewhere(conn, &def.id, entity_id, value)? {
                return Err(AppError::Validation(format!("{} must be unique - '{value}' is already used", def.label)));
            }
        }
        if before_values.get(&def.key).map(|s| s.as_str()).unwrap_or("") != value {
            changed_keys.push(def.key.clone());
        }
        custom_field_repo::set_value(conn, &def.id, entity_id, value)?;
    }

    // ADM-WF: field_changed workflows fire from the same seam ADM-BR uses -
    // the one call site every entity's save flow already goes through
    // unconditionally, so no per-entity wiring is needed here either.
    workflow_service::fire_field_changed(conn, &workspace_id, entity_type, entity_id, "custom", &changed_keys, None, actor_user_id)?;

    // A business rule's set_default/set_value targeting a built-in field
    // has no in-flight built-in save to merge into the way a custom-field
    // target does (see `builtin_field_service`'s doc comment) - applied
    // here instead, as an immediate follow-up write through the entity's
    // own service, after everything above has confirmed nothing blocks
    // the save.
    for (field_key, value) in &evaluation.builtin_set_values {
        builtin_field_service::set_field(conn, &workspace_id, entity_type, entity_id, field_key, value, actor_user_id)?;
    }

    let _ = actor_user_id; // no audit entry per value write - the parent record's own create/update audit entry covers this edit
    Ok(SaveNotices { errors: evaluation.errors, warnings: evaluation.warnings })
}

pub fn get_entity_values(conn: &Connection, entity_id: &str) -> AppResult<CustomFieldValues> {
    Ok(custom_field_repo::get_values(conn, entity_id)?)
}
