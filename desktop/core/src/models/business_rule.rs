use serde::{Deserialize, Serialize};

pub const MATCH_TYPES: &[&str] = &["all", "any"];
/// Re-exported from `domain::conditions` rather than kept as a second,
/// hand-copied list - this and `domain::conditions::CONDITION_OPERATORS`
/// used to be two separately maintained lists of the same 10 operators;
/// consolidated to one source of truth while adding the new operators
/// below, so there's nothing left to keep in sync by hand.
pub use crate::domain::conditions::CONDITION_OPERATORS;
pub const TRIGGER_SOURCES: &[&str] = &["builtin", "custom"];
/// `require`/`hide`/`show`/`lock`/`editable`/`set_default`/`set_value`/
/// `clear_value`/`restrict_choices` act on `target_field_key` (which must
/// be an actionable built-in field or an active custom field). `block_save`/
/// `show_error`/`show_warning` act on the whole record and use `message`
/// instead. `show_message` is kept only so a rule saved before the second
/// addendum round keeps evaluating exactly as it always did - the builder
/// no longer offers it for new rules, in favor of the two severities.
///
/// `show`/`editable` are the explicit counterparts to `hide`/`lock`: most
/// useful on a field flagged `is_hidden_by_default` on its definition (see
/// migration 0020), which otherwise never renders, or to override a
/// lower-priority rule's `hide`/`lock` - "last matching rule wins" per
/// target field, same as the original pair, so a later `show` correctly
/// beats an earlier `hide` on the same field. `clear_value` is `set_value`
/// with an empty value written unconditionally (unlike `set_default`,
/// which only fills a field that's currently empty). `restrict_choices`
/// only makes sense on a select-typed field; `action_value` holds the
/// pipe-delimited subset of its options that stays selectable while the
/// rule matches - same `LIST_SEPARATOR` convention `in_list`/`not_in_list`
/// condition values already use.
pub const ACTION_TYPES: &[&str] = &[
    "require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value",
    "restrict_choices", "block_save", "show_error", "show_warning", "show_message",
];
pub const FIELD_TARGETED_ACTIONS: &[&str] =
    &["require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value", "restrict_choices"];
pub const MESSAGE_ACTIONS: &[&str] = &["block_save", "show_error", "show_warning", "show_message"];
/// Actions a rule builder should still let an admin pick when creating a
/// *new* rule - `show_message` is excluded (legacy-only, see above).
pub const CURRENT_ACTION_TYPES: &[&str] = &[
    "require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value",
    "restrict_choices", "block_save", "show_error", "show_warning",
];
/// `set_value`, `set_default`, `restrict_choices` require a value; `clear_value`
/// deliberately doesn't (the whole point is writing empty).
pub const VALUE_REQUIRED_ACTIONS: &[&str] = &["set_default", "set_value", "restrict_choices"];

/// The one built-in, enum-like field each entity type exposes as a rule
/// trigger. Every entity has a `status` column except Product, which only
/// has `is_active` (stored/compared as the strings "true"/"false", the
/// same convention boolean custom field values already use).
pub fn builtin_trigger_field_for(entity_type: &str) -> &'static str {
    match entity_type {
        "Product" => "is_active",
        _ => "status",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRuleCondition {
    pub id: String,
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    /// Addendum §2.2 "field-to-field comparison": when both are `Some`,
    /// the condition's effective comparison value is this field's current
    /// value instead of the literal `value` above - resolved at evaluation
    /// time in each engine's service layer (`domain::conditions` itself
    /// never sees the difference). `None` for an ordinary literal-value
    /// condition, which is still the common case.
    pub compare_field_source: Option<String>,
    pub compare_field_key: Option<String>,
    /// See migration 0020 / `domain::conditions::conditions_match` - `None`
    /// for an ungrouped, top-level condition; `Some(group_id)` for one that
    /// belongs to an OR-group, sharing the id with its group siblings.
    pub group_id: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleConditionInput {
    pub field_source: String,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    #[serde(default)]
    pub compare_field_source: Option<String>,
    #[serde(default)]
    pub compare_field_key: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRuleAction {
    pub id: String,
    pub action_type: String,
    pub target_field_key: Option<String>,
    /// 'builtin' | 'custom' - which registry `target_field_key` was
    /// validated against (see `domain::builtin_fields`). Only meaningful
    /// for `FIELD_TARGETED_ACTIONS`; `None`-target actions (block_save/
    /// show_message) leave this at the column default ('custom') unused.
    pub target_field_source: String,
    pub action_value: Option<String>,
    pub message: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleActionInput {
    pub action_type: String,
    pub target_field_key: Option<String>,
    #[serde(default = "default_field_source")]
    pub target_field_source: String,
    pub action_value: Option<String>,
    pub message: Option<String>,
}

fn default_field_source() -> String {
    "custom".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessRule {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub is_protected: bool,
    pub created_at: String,
    pub updated_at: String,
    pub conditions: Vec<BusinessRuleCondition>,
    pub actions: Vec<BusinessRuleAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleInput {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub conditions: Vec<BusinessRuleConditionInput>,
    pub actions: Vec<BusinessRuleActionInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessRuleUpdate {
    pub name: String,
    pub description: Option<String>,
    pub match_type: String,
    pub priority: i64,
    pub is_active: bool,
    pub effective_start_date: Option<String>,
    pub effective_end_date: Option<String>,
    pub conditions: Vec<BusinessRuleConditionInput>,
    pub actions: Vec<BusinessRuleActionInput>,
}
