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
    BusinessRule, BusinessRuleActionInput, BusinessRuleConditionInput, BusinessRuleInput, BusinessRuleUpdate,
    BusinessRuleVersion, ACTION_TYPES, CONDITION_OPERATORS, FIELD_TARGETED_ACTIONS, MATCH_TYPES, MESSAGE_ACTIONS,
    TRIGGER_SOURCES,
};
use crate::repositories::{business_rule_repo, custom_field_repo, relationship_repo};
use crate::services::{access_service, builtin_field_service, custom_object_service, entity_registry};

/// Administrator always passes (unchanged); a non-Administrator additionally
/// passes with an explicit Access Role grant on "BusinessRule" - see
/// `access_service::require_admin_or_explicit_update`'s own doc comment.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "BusinessRule", "Only an Administrator can manage business rules")
}

/// Per-app scoped automation: `app_id`, if set, must reference an existing
/// App Builder app (migration 0025) in the same workspace - same "exists
/// and is in this workspace, else 404-shaped Validation error" check
/// `app_service::validate_object_keys_and_dashboard` already uses for
/// `dashboard_id`. `None` (workspace-wide) always passes.
fn require_valid_app_id(conn: &Connection, workspace_id: &str, app_id: Option<&str>) -> AppResult<()> {
    let Some(app_id) = app_id else { return Ok(()) };
    let app = super::app_service::get(conn, app_id).map_err(|_| AppError::Validation("App not found".into()))?;
    if app.workspace_id != workspace_id {
        return Err(AppError::Validation("App not found".into()));
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

/// The related type a condition's `relationship_definition_id` resolves
/// against, for `entity_type` - only relationships where `entity_type` has
/// at most one linked record are eligible (the `many_to_one` "many" side,
/// or either side of a `one_to_one`), mirroring exactly the same
/// eligibility `resolve_related_field` checks at evaluation time. Kept in
/// sync with that function rather than shared, since one resolves a type
/// (authoring time) and the other a value (evaluation time).
fn related_field_owner_type(conn: &Connection, entity_type: &str, relationship_definition_id: &str) -> AppResult<String> {
    let def = relationship_repo::get_definition(conn, relationship_definition_id)?
        .ok_or_else(|| AppError::Validation("Selected relationship does not exist".into()))?;
    if def.source_entity_type == entity_type && matches!(def.relationship_type.as_str(), "many_to_one" | "one_to_one") {
        Ok(def.target_entity_type)
    } else if def.target_entity_type == entity_type && def.relationship_type == "one_to_one" {
        Ok(def.source_entity_type)
    } else {
        Err(AppError::Validation(
            "Selected relationship doesn't have a single determinate related record from this object - only a many-to-one's \"many\" side or either side of a one-to-one can be used".into(),
        ))
    }
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
        // A condition reading a related record's field (relationship_
        // definition_id set) validates field_key against *that* type's
        // fields instead of the triggering entity_type's own.
        if let Some(rel_id) = &c.relationship_definition_id {
            let related_type = related_field_owner_type(conn, entity_type, rel_id)?;
            let related_defs = custom_field_repo::list_definitions(conn, workspace_id, &related_type)?;
            let related_active_keys: Vec<&str> = related_defs.iter().filter(|d| d.is_active).map(|d| d.key.as_str()).collect();
            if !crate::domain::conditions::field_ref_is_valid(&related_type, &c.field_source, &c.field_key, related_active_keys.iter().copied()) {
                return Err(AppError::Validation(format!("'{}' is not a valid field on the related {related_type} record", c.field_key)));
            }
        } else if !crate::domain::conditions::field_ref_is_valid(entity_type, &c.field_source, &c.field_key, active_keys.iter().copied()) {
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
    require_valid_app_id(conn, workspace_id, input.app_id.as_deref())?;
    Ok(())
}

pub fn create_rule(conn: &Connection, workspace_id: &str, input: &BusinessRuleInput, actor_user_id: Option<&str>) -> AppResult<BusinessRule> {
    require_admin(conn, actor_user_id)?;
    validate_shape(conn, workspace_id, &input.entity_type, input)?;
    let id = crate::domain::ids::new_uuid();
    let created = business_rule_repo::create(conn, &id, workspace_id, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "business_rule", &created.id, actor_user_id)?;
    Ok(created)
}

/// Any authenticated user can list active rules (the form needs them to
/// evaluate live via `test_rules`/save-time enforcement); only an
/// Administrator sees inactive ones, via the admin screen.
pub fn list_rules(conn: &Connection, workspace_id: &str, entity_type: &str, active_only: bool) -> AppResult<Vec<BusinessRule>> {
    let all = business_rule_repo::list(conn, workspace_id, entity_type)?;
    Ok(if active_only { all.into_iter().filter(|r| r.is_active).collect() } else { all })
}

/// See `custom_object_service::get`'s doc comment - same export use case.
pub fn get_rule(conn: &Connection, id: &str) -> AppResult<Option<BusinessRule>> {
    Ok(business_rule_repo::get(conn, id)?)
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
    require_valid_app_id(conn, &existing.workspace_id, input.app_id.as_deref())?;
    // Admin UX polish (spec §10): snapshot the pre-edit state before it's
    // overwritten, so an admin can review or restore it later. `existing`
    // is always serializable (plain String/bool/i64/Vec fields), so this
    // can't fail the way parsing untrusted JSON can.
    let snapshot_json = serde_json::to_string(&existing).expect("BusinessRule is always serializable");
    business_rule_repo::insert_version(conn, id, &snapshot_json)?;
    Ok(business_rule_repo::update(conn, id, input, actor_user_id)?)
}

/// Admin UX polish (spec §10): every saved-version snapshot for a rule,
/// newest first. `require_admin` first, then confirm the rule itself
/// exists, so a bad id reads as 404 rather than an empty list.
pub fn list_versions(conn: &Connection, rule_id: &str, actor_user_id: Option<&str>) -> AppResult<Vec<BusinessRuleVersion>> {
    require_admin(conn, actor_user_id)?;
    business_rule_repo::get(conn, rule_id)?.ok_or_else(|| AppError::NotFound("Business rule".into()))?;
    business_rule_repo::list_version_rows(conn, rule_id)?
        .into_iter()
        .map(|(version_id, snapshot_json, saved_at)| {
            let snapshot: BusinessRule =
                serde_json::from_str(&snapshot_json).map_err(|e| AppError::Validation(format!("Corrupt rule version snapshot: {e}")))?;
            Ok(BusinessRuleVersion { id: version_id, business_rule_id: rule_id.to_string(), snapshot, saved_at })
        })
        .collect()
}

/// Admin UX polish (spec §10): re-submits an earlier snapshot through the
/// normal update path, so a restore is validated exactly like a hand-edited
/// save and (since `update_rule` always snapshots first) the state it
/// replaces is itself recoverable.
pub fn restore_version(conn: &Connection, rule_id: &str, version_id: &str, actor_user_id: Option<&str>) -> AppResult<BusinessRule> {
    require_admin(conn, actor_user_id)?;
    let (_, snapshot_json, _) = business_rule_repo::list_version_rows(conn, rule_id)?
        .into_iter()
        .find(|(vid, _, _)| vid == version_id)
        .ok_or_else(|| AppError::NotFound("Rule version".into()))?;
    let snapshot: BusinessRule =
        serde_json::from_str(&snapshot_json).map_err(|e| AppError::Validation(format!("Corrupt rule version snapshot: {e}")))?;
    let update = BusinessRuleUpdate {
        name: snapshot.name,
        description: snapshot.description,
        match_type: snapshot.match_type,
        priority: snapshot.priority,
        is_active: snapshot.is_active,
        effective_start_date: snapshot.effective_start_date,
        effective_end_date: snapshot.effective_end_date,
        app_id: snapshot.app_id,
        conditions: snapshot
            .conditions
            .into_iter()
            .map(|c| BusinessRuleConditionInput {
                field_source: c.field_source,
                field_key: c.field_key,
                operator: c.operator,
                value: c.value,
                compare_field_source: c.compare_field_source,
                compare_field_key: c.compare_field_key,
                group_id: c.group_id,
                relationship_definition_id: c.relationship_definition_id,
            })
            .collect(),
        actions: snapshot
            .actions
            .into_iter()
            .map(|a| BusinessRuleActionInput {
                action_type: a.action_type,
                target_field_key: a.target_field_key,
                target_field_source: a.target_field_source,
                action_value: a.action_value,
                message: a.message,
            })
            .collect(),
    };
    update_rule(conn, rule_id, &update, actor_user_id)
}

/// Admin UX polish (spec §10): copies a rule's full condition/action shape
/// into a new, inactive draft (named "<name> (Copy)" so it's distinguishable
/// in the list without opening it) - lets an admin build a close variant
/// starting from a known-good rule instead of from scratch. Created active
/// (via the normal `create` path) then immediately flipped inactive, rather
/// than adding a second insert variant just for that one flag.
pub fn duplicate_rule(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<BusinessRule> {
    require_admin(conn, actor_user_id)?;
    let existing = business_rule_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Business rule".into()))?;
    let input = BusinessRuleInput {
        entity_type: existing.entity_type.clone(),
        name: format!("{} (Copy)", existing.name),
        description: existing.description.clone(),
        match_type: existing.match_type.clone(),
        priority: existing.priority,
        effective_start_date: existing.effective_start_date.clone(),
        effective_end_date: existing.effective_end_date.clone(),
        app_id: existing.app_id.clone(),
        conditions: existing
            .conditions
            .iter()
            .map(|c| BusinessRuleConditionInput {
                field_source: c.field_source.clone(),
                field_key: c.field_key.clone(),
                operator: c.operator.clone(),
                value: c.value.clone(),
                compare_field_source: c.compare_field_source.clone(),
                compare_field_key: c.compare_field_key.clone(),
                group_id: c.group_id.clone(),
                relationship_definition_id: c.relationship_definition_id.clone(),
            })
            .collect(),
        actions: existing
            .actions
            .iter()
            .map(|a| BusinessRuleActionInput {
                action_type: a.action_type.clone(),
                target_field_key: a.target_field_key.clone(),
                target_field_source: a.target_field_source.clone(),
                action_value: a.action_value.clone(),
                message: a.message.clone(),
            })
            .collect(),
    };
    let new_id = crate::domain::ids::new_uuid();
    let created = business_rule_repo::create(conn, &new_id, &existing.workspace_id, &input, actor_user_id)?;
    let deactivate = BusinessRuleUpdate {
        name: created.name.clone(),
        description: created.description.clone(),
        match_type: created.match_type.clone(),
        priority: created.priority,
        is_active: false,
        effective_start_date: created.effective_start_date.clone(),
        effective_end_date: created.effective_end_date.clone(),
        app_id: created.app_id.clone(),
        conditions: input.conditions,
        actions: input.actions,
    };
    Ok(business_rule_repo::update(conn, &created.id, &deactivate, actor_user_id)?)
}

/// Admin UX polish (spec §10): human-readable descriptions of every active
/// rule on `entity_type` that reads (a condition's field or comparison
/// field) or writes (an action's target field) `field_key` - called by
/// `custom_field_service` before letting an admin deactivate a custom
/// field, since a rule referencing a deactivated field just silently stops
/// finding it and never fires that clause again.
pub fn describe_active_rules_referencing_field(conn: &Connection, workspace_id: &str, entity_type: &str, field_key: &str) -> AppResult<Vec<String>> {
    let rules = business_rule_repo::list(conn, workspace_id, entity_type)?;
    Ok(rules
        .iter()
        .filter(|r| r.is_active && rule_references_field(r, field_key))
        .map(|r| format!("Business rule \"{}\"", r.name))
        .collect())
}

fn rule_references_field(rule: &BusinessRule, field_key: &str) -> bool {
    rule.conditions.iter().any(|c| c.field_key == field_key || c.compare_field_key.as_deref() == Some(field_key))
        || rule.actions.iter().any(|a| a.target_field_key.as_deref() == Some(field_key))
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

/// Migration 0049, "cross-record validation": the actual value of
/// `field_key`/`field_source` on the record linked to `entity_id` through
/// `relationship_definition_id`, not `entity_id`'s own field. Scoped to
/// relationships shaped so `entity_type` has *at most one* linked record -
/// the "many" side of a `many_to_one`, or either side of a `one_to_one` -
/// since a `many_to_many` or the "one" side of a `many_to_one` has no
/// single determinate record to read. Returns `None` (never an error) when
/// the relationship doesn't apply, isn't eligibly shaped, or nothing is
/// linked yet - the condition then evaluates against an empty value, same
/// as any other unset field.
fn resolve_related_field(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    relationship_definition_id: &str,
    field_source: &str,
    field_key: &str,
) -> AppResult<Option<String>> {
    let Some(def) = relationship_repo::get_definition(conn, relationship_definition_id)? else {
        return Ok(None);
    };
    let (related_id, other_type) = if def.source_entity_type == entity_type && matches!(def.relationship_type.as_str(), "many_to_one" | "one_to_one") {
        (relationship_repo::list_instances_where_source(conn, &def.id, entity_id)?.into_iter().next().map(|i| i.target_id), def.target_entity_type)
    } else if def.target_entity_type == entity_type && def.relationship_type == "one_to_one" {
        (relationship_repo::list_instances_where_target(conn, &def.id, entity_id)?.into_iter().next().map(|i| i.source_id), def.source_entity_type)
    } else {
        (None, String::new())
    };
    let Some(related_id) = related_id else { return Ok(None) };
    let values = if field_source == "builtin" {
        builtin_field_service::field_values(conn, &other_type, &related_id)?
    } else {
        custom_field_repo::get_values(conn, &related_id)?
    };
    Ok(values.get(field_key).cloned())
}

/// Delegates to the shared AND/OR matcher (`domain::conditions`) also used
/// by workflow_service, so the two engines' "IF" halves can never drift
/// apart. When any of `rule`'s conditions reads a related record's field
/// (`relationship_definition_id` set), resolves those into a per-rule
/// scratch copy of `ctx` first - the shared base `ctx` passed in from
/// `evaluate` below is never mutated, since a sibling rule's own related-
/// field conditions (if any) may need a different resolved value under the
/// same field key.
fn rule_matches(conn: &Connection, entity_type: &str, entity_id: &str, rule: &BusinessRule, ctx: &HashMap<String, String>) -> AppResult<bool> {
    if !rule.conditions.iter().any(|c| c.relationship_definition_id.is_some()) {
        return Ok(crate::domain::conditions::conditions_match(
            &rule.match_type,
            rule.conditions.iter().map(|c| (c.group_id.as_deref(), c.field_key.as_str(), c.operator.as_str(), resolve_condition_value(c, ctx))),
            ctx,
        ));
    }
    let mut scratch = ctx.clone();
    for c in &rule.conditions {
        if let Some(rel_id) = &c.relationship_definition_id {
            let value = resolve_related_field(conn, entity_type, entity_id, rel_id, &c.field_source, &c.field_key)?.unwrap_or_default();
            scratch.insert(c.field_key.clone(), value);
        }
    }
    Ok(crate::domain::conditions::conditions_match(
        &rule.match_type,
        rule.conditions.iter().map(|c| (c.group_id.as_deref(), c.field_key.as_str(), c.operator.as_str(), resolve_condition_value(c, &scratch))),
        &scratch,
    ))
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
pub fn evaluate(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str, ctx: &HashMap<String, String>) -> AppResult<RuleEvaluation> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let rules = business_rule_repo::list(conn, workspace_id, entity_type)?;
    let mut result = RuleEvaluation::default();

    for rule in rules.iter().filter(|r| r.is_active && is_effective_today(r, &today)) {
        if !rule_matches(conn, entity_type, entity_id, rule, ctx)? {
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
/// There's no real record id to resolve a "cross-record validation"
/// condition's relationship from - an empty `entity_id` finds no linked
/// record (same as any other never-yet-linked record would), so such a
/// condition evaluates against an empty related value here, same as every
/// other hypothetical field this tester can't fully simulate.
pub fn test_rules(conn: &Connection, workspace_id: &str, entity_type: &str, ctx: &HashMap<String, String>, actor_user_id: Option<&str>) -> AppResult<RuleEvaluation> {
    require_admin(conn, actor_user_id)?;
    evaluate(conn, workspace_id, entity_type, "", ctx)
}
