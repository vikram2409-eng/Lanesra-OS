//! Shared AND/OR condition evaluation, used by both Business Rules (Phase
//! C) and Workflow Automation (Phase D) so the two engines' "IF" halves
//! never drift apart. A condition is always (field_source, field_key,
//! operator, value) evaluated against a trigger context map of the
//! record's current built-in status/is_active plus its custom field
//! values - each engine builds that context its own way, but matching is
//! identical once it exists.
//!
//! Admin Automation & Customization addendum (Phase 1): a condition's
//! `value` can now be resolved from another field on the same record
//! instead of a fixed literal ("field-to-field comparison", spec §2.2).
//! That resolution happens in each engine's service layer before calling
//! `conditions_match` - looking up the compare-to field in the same `ctx`
//! and substituting it as the effective value - so this module's matching
//! logic never needs to know the difference between a literal and a
//! resolved field value.

use std::collections::HashMap;

pub const CONDITION_OPERATORS: &[&str] = &[
    "equals", "not_equals", "contains", "not_contains", "starts_with", "ends_with",
    "in_list", "not_in_list", "is_empty", "is_not_empty",
    "greater_than", "less_than", "on_or_after", "on_or_before",
];
pub const TRIGGER_SOURCES: &[&str] = &["builtin", "custom"];
pub const MATCH_TYPES: &[&str] = &["all", "any"];
/// Operators that don't compare against a value at all - `is_empty`/
/// `is_not_empty` only look at the field itself, so a condition using one
/// of these has no `value` (and no compare-to field) to resolve or show.
pub const VALUELESS_OPERATORS: &[&str] = &["is_empty", "is_not_empty"];
/// `in_list`/`not_in_list` split `value` on this separator - the same
/// convention `select`/`multi-select` custom field options already use, so
/// the same "Option A|Option B|Option C" editor pattern applies here too.
pub const LIST_SEPARATOR: char = '|';

pub fn condition_matches(field_key: &str, operator: &str, value: &str, ctx: &HashMap<String, String>) -> bool {
    let actual = ctx.get(field_key).map(|s| s.as_str()).unwrap_or("");
    match operator {
        "equals" => actual == value,
        "not_equals" => actual != value,
        "contains" => !value.is_empty() && actual.contains(value),
        "not_contains" => value.is_empty() || !actual.contains(value),
        "starts_with" => !value.is_empty() && actual.starts_with(value),
        "ends_with" => !value.is_empty() && actual.ends_with(value),
        "in_list" => value.split(LIST_SEPARATOR).any(|v| v == actual),
        "not_in_list" => !value.split(LIST_SEPARATOR).any(|v| v == actual),
        "is_empty" => actual.is_empty(),
        "is_not_empty" => !actual.is_empty(),
        "greater_than" => matches!((actual.parse::<f64>(), value.parse::<f64>()), (Ok(a), Ok(b)) if a > b),
        "less_than" => matches!((actual.parse::<f64>(), value.parse::<f64>()), (Ok(a), Ok(b)) if a < b),
        "on_or_after" => !actual.is_empty() && actual >= value,
        "on_or_before" => !actual.is_empty() && actual <= value,
        _ => false,
    }
}

/// True when `source`/`key` refers to a real field on `entity_type` - a
/// built-in field from the registry, or one of the caller's active custom
/// field keys. Shared by business_rule_service and workflow_service to
/// validate both a condition's primary field and (Phase 1) its optional
/// compare-to field identically.
pub fn field_ref_is_valid<'a>(entity_type: &str, source: &str, key: &str, active_custom_keys: impl Iterator<Item = &'a str>) -> bool {
    if source == "builtin" {
        crate::domain::builtin_fields::find_builtin_field(entity_type, key).is_some()
    } else {
        active_custom_keys.into_iter().any(|k| k == key)
    }
}

/// True when `conditions` (as AND when `match_type == "all"`, OR when
/// `"any"`) match `ctx`. Empty `conditions` never match - both engines
/// require at least one condition at creation time, so this is a defensive
/// default rather than an expected path.
pub fn conditions_match<'a>(
    match_type: &str,
    conditions: impl Iterator<Item = (&'a str, &'a str, &'a str)>,
    ctx: &HashMap<String, String>,
) -> bool {
    let mut any_seen = false;
    let mut all_match = true;
    let mut any_match = false;
    for (field_key, operator, value) in conditions {
        any_seen = true;
        let matched = condition_matches(field_key, operator, value, ctx);
        all_match &= matched;
        any_match |= matched;
    }
    if !any_seen {
        return false;
    }
    if match_type == "any" {
        any_match
    } else {
        all_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn starts_with_and_ends_with_match_prefixes_and_suffixes() {
        let c = ctx(&[("name", "Acme Corp")]);
        assert!(condition_matches("name", "starts_with", "Acme", &c));
        assert!(!condition_matches("name", "starts_with", "Corp", &c));
        assert!(condition_matches("name", "ends_with", "Corp", &c));
        assert!(!condition_matches("name", "ends_with", "Acme", &c));
        // An empty prefix/suffix never matches - same "meaningless empty
        // literal" guard `contains` already has.
        assert!(!condition_matches("name", "starts_with", "", &c));
        assert!(!condition_matches("name", "ends_with", "", &c));
    }

    #[test]
    fn in_list_and_not_in_list_split_on_the_pipe_separator() {
        let c = ctx(&[("status", "Prospect")]);
        assert!(condition_matches("status", "in_list", "Lead|Prospect|Customer", &c));
        assert!(!condition_matches("status", "in_list", "Lead|Customer", &c));
        assert!(!condition_matches("status", "not_in_list", "Lead|Prospect|Customer", &c));
        assert!(condition_matches("status", "not_in_list", "Lead|Customer", &c));
    }

    #[test]
    fn field_ref_is_valid_checks_builtin_and_custom_fields_correctly() {
        // "status" is a real built-in field on Company; "not_a_field" isn't.
        assert!(field_ref_is_valid("Company", "builtin", "status", std::iter::empty()));
        assert!(!field_ref_is_valid("Company", "builtin", "not_a_field", std::iter::empty()));
        let active = ["lead_source", "priority"];
        assert!(field_ref_is_valid("Company", "custom", "lead_source", active.iter().copied()));
        assert!(!field_ref_is_valid("Company", "custom", "unknown_key", active.iter().copied()));
    }
}
