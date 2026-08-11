import type { BusinessRule, BusinessRuleCondition } from "./types";

/**
 * Client-side mirror of `business_rule_service::evaluate`'s condition/
 * match-type logic in the Rust core (core/src/services/business_rule_service.rs)
 * - kept deliberately identical so the UI's require/hide/lock hints never
 * disagree with what the server will actually enforce. This is a UX nicety
 * only, scoped to the three purely-cosmetic field effects (require/hide/
 * lock); `set_default`/`set_value`/`block_save`/`show_message` are only
 * ever resolved server-side, at save time, since they mutate or reject the
 * save itself rather than just how the form looks. The server is still the
 * single source of truth and re-validates `require` independently, so a
 * bug here can only ever make the form look stricter or looser, never
 * actually bypass server-side validation.
 *
 * `rules` should already be filtered to `is_active` and the target entity
 * type, and ordered by `priority` ascending (as `listBusinessRules`
 * returns them). Later (higher-priority) rules win ties on the same
 * target field.
 */
export function fieldEffectsFor(rules: BusinessRule[], triggerContext: Record<string, string>): Record<string, string> {
  const effects: Record<string, string> = {};
  for (const rule of rules) {
    if (!ruleMatches(rule, triggerContext)) continue;
    for (const action of rule.actions) {
      if ((action.action_type === "require" || action.action_type === "hide" || action.action_type === "lock") && action.target_field_key) {
        effects[action.target_field_key] = action.action_type;
      }
    }
  }
  return effects;
}

function conditionMatches(cond: BusinessRuleCondition, ctx: Record<string, string>): boolean {
  const actual = ctx[cond.field_key] ?? "";
  switch (cond.operator) {
    case "equals":
      return actual === cond.value;
    case "not_equals":
      return actual !== cond.value;
    case "contains":
      return cond.value !== "" && actual.includes(cond.value);
    case "not_contains":
      return cond.value === "" || !actual.includes(cond.value);
    case "is_empty":
      return actual === "";
    case "is_not_empty":
      return actual !== "";
    case "greater_than": {
      const a = Number(actual), b = Number(cond.value);
      return actual !== "" && cond.value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a > b;
    }
    case "less_than": {
      const a = Number(actual), b = Number(cond.value);
      return actual !== "" && cond.value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a < b;
    }
    case "on_or_after":
      return actual !== "" && actual >= cond.value;
    case "on_or_before":
      return actual !== "" && actual <= cond.value;
    default:
      return false;
  }
}

function ruleMatches(rule: BusinessRule, ctx: Record<string, string>): boolean {
  if (rule.conditions.length === 0) return false;
  return rule.match_type === "any" ? rule.conditions.some((c) => conditionMatches(c, ctx)) : rule.conditions.every((c) => conditionMatches(c, ctx));
}
