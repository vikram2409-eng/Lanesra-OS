//! Admin extensibility Phase C (spec §22/ADM-BR): a richer IF (AND/OR) /
//! THEN business rule engine, replacing the original single-condition
//! field_rules (require/hide only). A rule now has any number of
//! conditions (matched as AND or OR), and actions beyond require/hide:
//! lock (read-only), set a default or forced value, block the whole save
//! with a custom message, or show a non-blocking message.
//!
//! Enforcement happens at the same integration point the original engine
//! used - `custom_field_service::set_entity_values`, which every entity's
//! save flow already calls unconditionally (see that module's own
//! comment). `lock`/`hide` stay purely cosmetic (client-enforced, nothing
//! to validate server-side); `require`/`block_save` reject the save;
//! `set_default`/`set_value` mutate the values being saved before
//! validation runs; `show_message` is surfaced back to the caller as a
//! non-blocking string so the UI can display it.

use std::collections::HashMap;

use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

use crate::domain::{AppError, AppResult};
use crate::models::business_rule::{
    builtin_trigger_field_for, BusinessRule, BusinessRuleInput, BusinessRuleUpdate, ACTION_TYPES, CONDITION_OPERATORS,
    FIELD_TARGETED_ACTIONS, MATCH_TYPES, MESSAGE_ACTIONS, TRIGGER_SOURCES,
};
use crate::repositories::{business_rule_repo, custom_field_repo, user_repo};
use crate::services::{custom_object_service, entity_registry};

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    let actor_id = actor_user_id.ok_or_else(|| AppError::Validation("Not authenticated".into()))?;
    let roles = user_repo::roles_for_user(conn, actor_id)?;
    if !roles.iter().any(|r| r == "Administrator") {
        return Err(AppError::Validation("Only an Administrator can manage business rules".into()));
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

fn validate_conditions(conn: &Connection, workspace_id: &str, entity_type: &str, conditions: &[crate::models::business_rule::BusinessRuleConditionInput]) -> AppResult<()> {
    if conditions.is_empty() {
        return Err(AppError::Validation("A rule needs at least one condition".into()));
    }
    let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
    for c in conditions {
        if !TRIGGER_SOURCES.contains(&c.field_source.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition source '{}'", c.field_source)));
        }
        if !CONDITION_OPERATORS.contains(&c.operator.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition operator '{}'", c.operator)));
        }
        if c.field_source == "builtin" {
            let expected = builtin_trigger_field_for(entity_type);
            if c.field_key != expected {
                return Err(AppError::Validation(format!(
                    "'{}' is not a supported built-in condition field for {entity_type} (expected '{expected}')", c.field_key
                )));
            }
        } else if !defs.iter().any(|d| d.key == c.field_key && d.is_active) {
            return Err(AppError::Validation(format!("'{}' is not an active custom field to trigger on", c.field_key)));
        }
    }
    Ok(())
}

fn validate_actions(conn: &Connection, workspace_id: &str, entity_type: &str, actions: &[crate::models::business_rule::BusinessRuleActionInput]) -> AppResult<()> {
    if actions.is_empty() {
        return Err(AppError::Validation("A rule needs at least one action".into()));
    }
    let defs = custom_field_repo::list_definitions(conn, workspace_id, entity_type)?;
    for a in actions {
        if !ACTION_TYPES.contains(&a.action_type.as_str()) {
            return Err(AppError::Validation(format!("Invalid action type '{}'", a.action_type)));
        }
        if FIELD_TARGETED_ACTIONS.contains(&a.action_type.as_str()) {
            let target = a.target_field_key.as_deref().unwrap_or("");
            if !defs.iter().any(|d| d.key == target && d.is_active) {
                return Err(AppError::Validation(format!("'{target}' is not an active custom field to target")));
            }
            if matches!(a.action_type.as_str(), "set_default" | "set_value") && a.action_value.is_none() {
                return Err(AppError::Validation("A value is required for this action".into()));
            }
        }
        if MESSAGE_ACTIONS.contains(&a.action_type.as_str()) && a.message.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::Validation("A message is required for this action".into()));
        }
    }
    Ok(())
}

fn validate_shape(conn: &Connection, workspace_id: &str, entity_type: &str, input: &BusinessRuleInput) -> AppResult<()> {
    require_valid_entity_type(conn, workspace_id, entity_type)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Rule name is required".into()));
    }
    if !MATCH_TYPES.contains(&input.match_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid match type '{}'", input.match_type)));
    }
    validate_conditions(conn, workspace_id, entity_type, &input.conditions)?;
    validate_actions(conn, workspace_id, entity_type, &input.actions)?;
    Ok(())
}

pub fn create_rule(conn: &Connection, workspace_id: &str, input: &BusinessRuleInput, actor_user_id: Option<&str>) -> AppResult<BusinessRule> {
    require_admin(conn, actor_user_id)?;
    validate_shape(conn, workspace_id, &input.entity_type, input)?;
    let id = crate::domain::ids::new_uuid();
    Ok(business_rule_repo::create(conn, &id, workspace_id, input, actor_user_id)?)
}

/// Any authenticated user can list active rules (the form needs them to
/// evaluate live via `test_rules`/save-time enforcement); only an
/// Administrator sees inactive ones, via the admin screen.
pub fn list_rules(conn: &Connection, workspace_id: &str, entity_type: &str, active_only: bool) -> AppResult<Vec<BusinessRule>> {
    let all = business_rule_repo::list(conn, workspace_id, entity_type)?;
    Ok(if active_only { all.into_iter().filter(|r| r.is_active).collect() } else { all })
}

pub fn update_rule(conn: &Connection, id: &str, input: &BusinessRuleUpdate, actor_user_id: Option<&str>) -> AppResult<BusinessRule> {
    require_admin(conn, actor_user_id)?;
    let existing = business_rule_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Business rule".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This rule is protected by the system and cannot be modified".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Rule name is required".into()));
    }
    if !MATCH_TYPES.contains(&input.match_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid match type '{}'", input.match_type)));
    }
    validate_conditions(conn, &existing.workspace_id, &existing.entity_type, &input.conditions)?;
    validate_actions(conn, &existing.workspace_id, &existing.entity_type, &input.actions)?;
    Ok(business_rule_repo::update(conn, id, input, actor_user_id)?)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuleEvaluation {
    /// target_field_key -> "require" | "hide" | "lock"
    pub field_effects: HashMap<String, String>,
    /// target_field_key -> value to apply before validation
    pub set_values: HashMap<String, String>,
    /// Some(message) means the whole save is rejected with this message.
    pub blocked: Option<String>,
    /// Non-blocking messages to surface to the user.
    pub messages: Vec<String>,
}

/// Delegates to the shared AND/OR matcher (`domain::conditions`) also used
/// by workflow_service, so the two engines' "IF" halves can never drift
/// apart.
fn rule_matches(rule: &BusinessRule, ctx: &HashMap<String, String>) -> bool {
    crate::domain::conditions::conditions_match(
        &rule.match_type,
        rule.conditions.iter().map(|c| (c.field_key.as_str(), c.operator.as_str(), c.value.as_str())),
        ctx,
    )
}

fn is_effective_today(rule: &BusinessRule, today: &str) -> bool {
    if let Some(start) = &rule.effective_start_date {
        if today < start.as_str() {
            return false;
        }
    }
    if let Some(end) = &rule.effective_end_date {
        if today > end.as_str() {
            return false;
        }
    }
    true
}

/// Evaluates every active, currently-effective rule for `entity_type`
/// against `ctx` (built-in status/is_active plus whatever custom field
/// values are being evaluated), in priority order. Where two rules
/// disagree about the same target field's require/hide/lock effect, the
/// later-evaluated (higher priority number) rule wins - the same "last one
/// wins" rule the original engine documented, now scoped per rule instead
/// of per condition.
pub fn evaluate(conn: &Connection, workspace_id: &str, entity_type: &str, ctx: &HashMap<String, String>) -> AppResult<RuleEvaluation> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let rules = business_rule_repo::list(conn, workspace_id, entity_type)?;
    let mut result = RuleEvaluation::default();

    for rule in rules.iter().filter(|r| r.is_active && is_effective_today(r, &today)) {
        if !rule_matches(rule, ctx) {
            continue;
        }
        for action in &rule.actions {
            match action.action_type.as_str() {
                "require" | "hide" | "lock" => {
                    if let Some(t) = &action.target_field_key {
                        result.field_effects.insert(t.clone(), action.action_type.clone());
                    }
                }
                "set_default" => {
                    if let (Some(t), Some(v)) = (&action.target_field_key, &action.action_value) {
                        let currently_empty = ctx.get(t).map(|s| s.trim().is_empty()).unwrap_or(true);
                        if currently_empty && !result.set_values.contains_key(t) {
                            result.set_values.insert(t.clone(), v.clone());
                        }
                    }
                }
                "set_value" => {
                    if let (Some(t), Some(v)) = (&action.target_field_key, &action.action_value) {
                        result.set_values.insert(t.clone(), v.clone());
                    }
                }
                "block_save" => {
                    if result.blocked.is_none() {
                        result.blocked = Some(action.message.clone().unwrap_or_else(|| format!("'{}' blocks saving this record", rule.name)));
                    }
                }
                "show_message" => {
                    if let Some(m) = &action.message {
                        result.messages.push(m.clone());
                    }
                }
                _ => {}
            }
        }
    }

    Ok(result)
}

/// ADM-BR-09 (Should): lets an admin try a hypothetical set of field
/// values against every active rule for an entity type before relying on
/// it - the same `evaluate` real saves use, just against caller-supplied
/// values instead of a persisted record, so nothing here writes anything.
pub fn test_rules(conn: &Connection, workspace_id: &str, entity_type: &str, ctx: &HashMap<String, String>, actor_user_id: Option<&str>) -> AppResult<RuleEvaluation> {
    require_admin(conn, actor_user_id)?;
    evaluate(conn, workspace_id, entity_type, ctx)
}
