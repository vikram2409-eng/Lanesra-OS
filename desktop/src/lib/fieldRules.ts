import type { FieldRule } from "./types";

/**
 * Client-side mirror of `field_rule_service::effects_for` in the Rust core
 * (core/src/services/field_rule_service.rs) - kept deliberately identical so
 * the UI's hide/require behavior never disagrees with what the server will
 * actually enforce. This is a UX nicety only: the server is still the
 * single source of truth and re-validates `require` independently (FR-RUL-06
 * / FR-RUL-07), so a bug here can only ever make the form *stricter or
 * looser looking*, never actually bypass server-side validation.
 *
 * `rules` should already be filtered to `is_active` and the target entity
 * type, and ordered by `sort_order` ascending (as `listFieldRules` returns
 * them). Later (higher sort_order) rules win ties on the same target field.
 */
export function effectsFor(
  rules: FieldRule[],
  triggerContext: Record<string, string>,
): Record<string, string> {
  const effects: Record<string, string> = {};
  for (const rule of rules) {
    if (!ruleMatches(rule, triggerContext)) continue;
    effects[rule.target_field_key] = rule.effect;
  }
  return effects;
}

function ruleMatches(rule: FieldRule, triggerContext: Record<string, string>): boolean {
  const actual = triggerContext[rule.trigger_field_key] ?? "";
  switch (rule.operator) {
    case "equals":
      return actual === rule.trigger_value;
    case "not_equals":
      return actual !== rule.trigger_value;
    default:
      return false;
  }
}
