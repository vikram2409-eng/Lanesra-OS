//! FR-RUL: admin-defined conditional rules over custom fields - "require
//! Industry when Status = Prospect." Scoped to custom fields as the
//! trigger-or-target vocabulary this phase governs (see the migration's
//! header comment for why built-in fields are out of scope). The `hide`
//! effect is enforced client-side only (purely cosmetic - a hidden field
//! has nothing to validate); `require` is enforced here, server-side,
//! since that's the one with real data-integrity consequences.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::field_rule::{
    FieldRule, FieldRuleInput, FieldRuleUpdate, BUILTIN_TRIGGER_FIELDS, RULE_EFFECTS, RULE_OPERATORS, TRIGGER_SOURCES,
};
use crate::repositories::{custom_field_repo, field_rule_repo, user_repo};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage business rules".into()));
    }
    Ok(())
}

fn validate_shape(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    trigger_field_source: &str,
    trigger_field_key: &str,
    operator: &str,
    target_field_key: &str,
    effect: &str,
) -> AppResult<()> {
    if !RULE_OPERATORS.contains(&operator) {
        return Err(AppError::Validation(format!("Invalid operator '{operator}'")));
    }
    if !RULE_EFFECTS.contains(&effect) {
        return Err(AppError::Validation(format!("Invalid effect '{effect}'")));
    }
    if !TRIGGER_SOURCES.contains(&trigger_field_source) {
        return Err(AppError::Validation(format!("Invalid trigger source '{trigger_field_source}'")));
    }

    if trigger_field_source == "builtin" {
        if !BUILTIN_TRIGGER_FIELDS.contains(&trigger_field_key) {
            return Err(AppError::Validation(format!(
                "'{trigger_field_key}' is not a supported built-in trigger field"
            )));
        }
    } else {
        let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
        if !defs.iter().any(|d| d.key == trigger_field_key && d.is_active) {
            return Err(AppError::Validation(format!(
                "'{trigger_field_key}' is not an active custom field to trigger on"
            )));
        }
    }

    let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
    if !defs.iter().any(|d| d.key == target_field_key && d.is_active) {
        return Err(AppError::Validation(format!(
            "'{target_field_key}' is not an active custom field to target"
        )));
    }

    Ok(())
}

pub fn create_rule(
    conn: &Connection,
    workspace_id: &str,
    input: &FieldRuleInput,
    actor_user_id: Option<&str>,
) -> AppResult<FieldRule> {
    require_admin(conn, actor_user_id)?;
    validate_shape(
        conn, workspace_id, &input.entity_type, &input.trigger_field_source, &input.trigger_field_key,
        &input.operator, &input.target_field_key, &input.effect,
    )?;
    let id = crate::domain::ids::new_uuid();
    Ok(field_rule_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

/// Any authenticated user can list active rules (the form needs them to
/// evaluate live); only an Administrator sees inactive ones, via the
/// admin screen.
pub fn list_rules(conn: &Connection, workspace_id: &str, entity_type: &str, active_only: bool) -> AppResult<Vec<FieldRule>> {
    let all = field_rule_repo::list(conn, workspace_id, entity_type)?;
    Ok(if active_only { all.into_iter().filter(|r| r.is_active).collect() } else { all })
}

pub fn update_rule(conn: &Connection, id: &str, input: &FieldRuleUpdate, actor_user_id: Option<&str>) -> AppResult<FieldRule> {
    require_admin(conn, actor_user_id)?;
    let existing = field_rule_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Business rule".into()))?;
    validate_shape(
        conn, &existing.workspace_id, &existing.entity_type, &input.trigger_field_source, &input.trigger_field_key,
        &input.operator, &input.target_field_key, &input.effect,
    )?;
    Ok(field_rule_repo::update(conn, id, input, actor_user_id)?)
}

fn rule_matches(rule: &FieldRule, trigger_context: &HashMap<String, String>) -> bool {
    let actual = trigger_context.get(&rule.trigger_field_key).map(|s| s.as_str()).unwrap_or("");
    match rule.operator.as_str() {
        "equals" => actual == rule.trigger_value,
        "not_equals" => actual != rule.trigger_value,
        _ => false,
    }
}

/// Effective effect per target field key, given the entity's current
/// trigger values (built-in "status" plus whatever custom field values
/// are being evaluated). Rules are applied in sort_order; where two
/// active rules target the same field with different effects, the
/// higher-sort_order one wins, since it's applied last and simply
/// overwrites the map entry (FR-RUL-06).
pub fn effects_for(
    conn: &Connection,
    workspace_id: &str,
    entity_type: &str,
    trigger_context: &HashMap<String, String>,
) -> AppResult<HashMap<String, String>> {
    let rules = field_rule_repo::list(conn, workspace_id, entity_type)?;
    let mut effects = HashMap::new();
    for rule in rules.iter().filter(|r| r.is_active) {
        if rule_matches(rule, trigger_context) {
            effects.insert(rule.target_field_key.clone(), rule.effect.clone());
        }
    }
    Ok(effects)
}
