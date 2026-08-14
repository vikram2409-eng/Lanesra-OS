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

use crate::domain::{builtin_fields, AppError, AppResult};
use crate::models::business_rule::{
    BusinessRule, BusinessRuleInput, BusinessRuleUpdate, ACTION_TYPES, CONDITION_OPERATORS,
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
    let active_keys: Vec<&str> = defs.iter().filter(|d| d.is_active).map(|d| d.key.as_str()).collect();
    for c in conditions {
        if !TRIGGER_SOURCES.contains(&c.field_source.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition source '{}'", c.field_source)));
        }
        if !CONDITION_OPERATORS.contains(&c.operator.as_str()) {
            return Err(AppError::Validation(format!("Invalid condition operator '{}'", c.operator)));
        }
        if !crate::domain::conditions::field_ref_is_valid(entity_type, &c.field_source, &c.field_key, active_keys.iter().copied()) {
            return Err(AppError::Validation(format!("'{}' is not a valid field to trigger on", c.field_key)));
        }
        // Addendum §2.2: field-to-field comparison - either both compare_*
        // are set (validated the same way as the primary field) or both
        // are absent (an ordinary literal-value condition).
        match (&c.compare_field_source, &c.compare_field_key) {
            (Some(src), Some(key)) => {
                if !TRIGGER_SOURCES.contains(&src.as_str()) {
                    return Err(AppError::Validation(format!("Invalid comparison field source '{src}'")));
                }
                if !crate::domain::conditions::field_ref_is_valid(entity_type, src, key, active_keys.iter().copied()) {
                    return Err(AppError::Validation(format!("'{key}' is not a valid field to compare against")));
                }
            }
            (None, None) => {}
            _ => return Err(AppError::Validation("A comparison field needs both a source and a key".into())),
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
            if !TRIGGER_SOURCES.contains(&a.target_field_source.as_str()) {
                return Err(AppError::Validation(format!("Invalid target field source '{}'", a.target_field_source)));
            }
            if a.target_field_source == "builtin" {
                if !builtin_fields::is_actionable_builtin_field(entity_type, target) {
                    return Err(AppError::Validation(format!("'{target}' is not an actionable built-in field on {entity_type}")));
                }
            } else if !defs.iter().any(|d| d.key == target && d.is_active) {
                return Err(AppError::Validation(format!("'{target}' is not an active custom field to target")));
            }
            if crate::models::business_rule::VALUE_REQUIRED_ACTIONS.contains(&a.action_type.as_str()) && a.action_value.is_none() {
                return Err(AppError::Validation("A value is required for this action".into()));
            }
            if a.action_type == "restrict_choices" {
                let is_select = if a.target_field_source == "builtin" {
                    builtin_fields::find_builtin_field(entity_type, target).is_some_and(|f| f.field_type == "select")
                } else {
                    defs.iter().any(|d| d.key == target && d.field_type == "select")
                };
                if !is_select {
                    return Err(AppError::Validation(format!("'{target}' is not a select field with a fixed set of options")));
                }
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
    /// custom target_field_key -> "require" | "hide" | "show" | "lock" | "editable"
    /// - "last matching rule wins" per target field (a later `show`
    /// correctly overrides an earlier `hide` on the same field, same for
    /// `editable` over `lock`), same map-insert-overwrite semantics the
    /// original require/hide/lock pair already had.
    pub field_effects: HashMap<String, String>,
    /// custom target_field_key -> value to apply before validation
    /// (set_default/set_value/clear_value all land here - clear_value just
    /// writes an empty string, unconditionally like set_value).
    pub set_values: HashMap<String, String>,
    /// Same as `field_effects`/`set_values` but for built-in-field targets
    /// - kept in separate maps (rather than merged by key) so a custom
    /// field that happens to share a key with a built-in one (e.g. an
    /// admin-defined "Notes" custom field on an entity that already has a
    /// built-in `notes` column) can never collide with it.
    pub builtin_field_effects: HashMap<String, String>,
    /// Applied immediately via `builtin_field_service::set_field` once
    /// evaluation completes and nothing blocked the save - unlike
    /// `set_values`, which the caller merges into the same custom-field
    /// save that's already in flight, there is no in-flight built-in save
    /// to merge into (see `builtin_field_service`'s doc comment).
    pub builtin_set_values: HashMap<String, String>,
    /// custom or built-in target_field_key -> pipe-delimited subset of a
    /// select field's options that stays selectable while the rule
    /// matches - same map for both sources since a select field is never
    /// ambiguous between them the way a plain string key could be.
    pub restricted_choices: HashMap<String, String>,
    /// Some(message) means the whole save is rejected with this message.
    pub blocked: Option<String>,
    /// Non-blocking, higher-severity messages from `show_error` actions.
    pub errors: Vec<String>,
    /// Non-blocking messages from `show_warning` actions - legacy
    /// `show_message` rows (saved before the second addendum round) also
    /// land here, since a plain message reads closest to a warning.
    pub warnings: Vec<String>,
}

/// A condition's effective comparison value: the compare-to field's
/// current value in `ctx` when one is set (Addendum §2.2 field-to-field
/// comparison), otherwise the condition's own literal `value`. `ctx` is a
/// single flat map regardless of field_source (built-in and custom values
/// share one namespace once merged - see `custom_field_service::
/// set_entity_values`'s trigger_context), so this is the same lookup
/// `condition_matches` already does for the primary field, just applied to
/// the compare-to field instead of a literal.
fn resolve_condition_value<'a>(c: &'a crate::models::business_rule::BusinessRuleCondition, ctx: &'a HashMap<String, String>) -> &'a str {
    match &c.compare_field_key {
        Some(key) => ctx.get(key).map(|s| s.as_str()).unwrap_or(""),
        None => c.value.as_str(),
    }
}

/// Delegates to the shared AND/OR matcher (`domain::conditions`) also used
/// by workflow_service, so the two engines' "IF" halves can never drift
/// apart.
fn rule_matches(rule: &BusinessRule, ctx: &HashMap<String, String>) -> bool {
    crate::domain::conditions::conditions_match(
        &rule.match_type,
        rule.conditions.iter().map(|c| (c.group_id.as_deref(), c.field_key.as_str(), c.operator.as_str(), resolve_condition_value(c, ctx))),
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
            let is_builtin = action.target_field_source == "builtin";
            match action.action_type.as_str() {
                // "show"/"editable" are the explicit counterparts to
                // "hide"/"lock" - same map, same last-matching-rule-wins
                // overwrite, so a later show/editable correctly beats an
                // earlier hide/lock on the same field.
                "require" | "hide" | "show" | "lock" | "editable" => {
                    if let Some(t) = &action.target_field_key {
                        let map = if is_builtin { &mut result.builtin_field_effects } else { &mut result.field_effects };
                        map.insert(t.clone(), action.action_type.clone());
                    }
                }
                "set_default" => {
                    if let (Some(t), Some(v)) = (&action.target_field_key, &action.action_value) {
                        let currently_empty = ctx.get(t).map(|s| s.trim().is_empty()).unwrap_or(true);
                        let map = if is_builtin { &mut result.builtin_set_values } else { &mut result.set_values };
                        if currently_empty && !map.contains_key(t) {
                            map.insert(t.clone(), v.clone());
                        }
                    }
                }
                "set_value" => {
                    if let (Some(t), Some(v)) = (&action.target_field_key, &action.action_value) {
                        let map = if is_builtin { &mut result.builtin_set_values } else { &mut result.set_values };
                        map.insert(t.clone(), v.clone());
                    }
                }
                // Same map as set_value, always writing empty - "clear"
                // needs no action_value at all.
                "clear_value" => {
                    if let Some(t) = &action.target_field_key {
                        let map = if is_builtin { &mut result.builtin_set_values } else { &mut result.set_values };
                        map.insert(t.clone(), String::new());
                    }
                }
                "restrict_choices" => {
                    if let (Some(t), Some(v)) = (&action.target_field_key, &action.action_value) {
                        result.restricted_choices.insert(t.clone(), v.clone());
                    }
                }
                "block_save" => {
                    if result.blocked.is_none() {
                        result.blocked = Some(action.message.clone().unwrap_or_else(|| format!("'{}' blocks saving this record", rule.name)));
                    }
                }
                "show_error" => {
                    if let Some(m) = &action.message {
                        result.errors.push(m.clone());
                    }
                }
                "show_warning" | "show_message" => {
                    if let Some(m) = &action.message {
                        result.warnings.push(m.clone());
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
