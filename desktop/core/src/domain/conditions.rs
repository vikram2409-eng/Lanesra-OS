//! Shared AND/OR condition evaluation, used by both Business Rules (Phase
//! C) and Workflow Automation (Phase D) so the two engines' "IF" halves
//! never drift apart. A condition is always (field_source, field_key,
//! operator, value) evaluated against a trigger context map of the
//! record's current built-in status/is_active plus its custom field
//! values - each engine builds that context its own way, but matching is
//! identical once it exists.

use std::collections::HashMap;

pub const CONDITION_OPERATORS: &[&str] = &[
    "equals", "not_equals", "contains", "not_contains", "is_empty", "is_not_empty",
    "greater_than", "less_than", "on_or_after", "on_or_before",
];
pub const TRIGGER_SOURCES: &[&str] = &["builtin", "custom"];
pub const MATCH_TYPES: &[&str] = &["all", "any"];

pub fn condition_matches(field_key: &str, operator: &str, value: &str, ctx: &HashMap<String, String>) -> bool {
    let actual = ctx.get(field_key).map(|s| s.as_str()).unwrap_or("");
    match operator {
        "equals" => actual == value,
        "not_equals" => actual != value,
        "contains" => !value.is_empty() && actual.contains(value),
        "not_contains" => value.is_empty() || !actual.contains(value),
        "is_empty" => actual.is_empty(),
        "is_not_empty" => !actual.is_empty(),
        "greater_than" => matches!((actual.parse::<f64>(), value.parse::<f64>()), (Ok(a), Ok(b)) if a > b),
        "less_than" => matches!((actual.parse::<f64>(), value.parse::<f64>()), (Ok(a), Ok(b)) if a < b),
        "on_or_after" => !actual.is_empty() && actual >= value,
        "on_or_before" => !actual.is_empty() && actual <= value,
        _ => false,
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
