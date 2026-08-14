import type { BusinessRule, BusinessRuleCondition, ConditionOperator } from "./types";

/**
 * Client-side mirror of `business_rule_service::evaluate`'s condition/
 * match-type logic in the Rust core (core/src/services/business_rule_service.rs
 * and core/src/domain/conditions.rs) - kept deliberately identical so the
 * UI's require/hide/show/lock/editable hints never disagree with what the
 * server will actually enforce. This is a UX nicety only, scoped to the
 * purely-cosmetic field effects; `set_default`/`set_value`/`clear_value`/
 * `block_save`/`show_error`/`show_warning` are only ever resolved
 * server-side, at save time, since they mutate or reject the save itself
 * rather than just how the form looks. The server is still the single
 * source of truth and re-validates `require` independently, so a bug here
 * can only ever make the form look stricter or looser, never actually
 * bypass server-side validation.
 *
 * `rules` should already be filtered to `is_active` and the target entity
 * type, and ordered by `priority` ascending (as `listBusinessRules`
 * returns them). Later (higher-priority) rules win ties on the same
 * target field - so a later `show` correctly beats an earlier `hide` on
 * the same field, same for `editable` over `lock`.
 */
export function fieldEffectsFor(rules: BusinessRule[], triggerContext: Record<string, string>): Record<string, string> {
  const effects: Record<string, string> = {};
  for (const rule of rules) {
    if (!ruleMatches(rule, triggerContext)) continue;
    for (const action of rule.actions) {
      if (
        (action.action_type === "require" || action.action_type === "hide" || action.action_type === "show" ||
          action.action_type === "lock" || action.action_type === "editable") &&
        action.target_field_key
      ) {
        effects[action.target_field_key] = action.action_type;
      }
    }
  }
  return effects;
}

/** Client-side mirror of `restrict_choices` - target field key -> the
 * pipe-delimited (LIST_SEPARATOR) subset of options that stays selectable
 * while a matching rule's action is in effect. Same "later rule wins" tie
 * break as fieldEffectsFor. */
export function restrictedChoicesFor(rules: BusinessRule[], triggerContext: Record<string, string>): Record<string, string> {
  const choices: Record<string, string> = {};
  for (const rule of rules) {
    if (!ruleMatches(rule, triggerContext)) continue;
    for (const action of rule.actions) {
      if (action.action_type === "restrict_choices" && action.target_field_key && action.action_value) {
        choices[action.target_field_key] = action.action_value;
      }
    }
  }
  return choices;
}

const LIST_SEPARATOR = "|";

function conditionMatches(cond: BusinessRuleCondition, ctx: Record<string, string>): boolean {
  return operatorMatches(cond.operator, ctx[cond.field_key] ?? "", resolveComparand(cond, ctx));
}

/** A condition's effective comparison value - the compare-to field's live
 * value when field-to-field comparison is set (addendum §2.2), otherwise
 * the literal `value`. Mirrors each engine's service-layer resolution step
 * ahead of domain::conditions::condition_matches. */
function resolveComparand(cond: BusinessRuleCondition, ctx: Record<string, string>): string {
  if (cond.compare_field_source && cond.compare_field_key) {
    return ctx[cond.compare_field_key] ?? "";
  }
  return cond.value;
}

function operatorMatches(operator: ConditionOperator, actual: string, value: string): boolean {
  switch (operator) {
    case "equals":
      return actual === value;
    case "not_equals":
      return actual !== value;
    case "contains":
      return value !== "" && actual.includes(value);
    case "not_contains":
      return value === "" || !actual.includes(value);
    case "starts_with":
      return value !== "" && actual.startsWith(value);
    case "ends_with":
      return value !== "" && actual.endsWith(value);
    case "in_list":
      return value.split(LIST_SEPARATOR).includes(actual);
    case "not_in_list":
      return !value.split(LIST_SEPARATOR).includes(actual);
    case "is_empty":
      return actual === "";
    case "is_not_empty":
      return actual !== "";
    case "greater_than": {
      const a = Number(actual), b = Number(value);
      return actual !== "" && value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a > b;
    }
    case "less_than": {
      const a = Number(actual), b = Number(value);
      return actual !== "" && value !== "" && !Number.isNaN(a) && !Number.isNaN(b) && a < b;
    }
    case "on_or_after":
      return actual !== "" && actual >= value;
    case "on_or_before":
      return actual !== "" && actual <= value;
    default:
      return false;
  }
}

/** Mirrors `domain::conditions::conditions_match`: a condition with no
 * `group_id` participates directly in the rule's top-level match_type;
 * conditions sharing a `group_id` are OR'd together into one sub-unit
 * first, and that sub-unit's result then participates in the top-level
 * match_type alongside the ungrouped conditions - one level of nested
 * OR-grouping. */
function ruleMatches(rule: BusinessRule, ctx: Record<string, string>): boolean {
  if (rule.conditions.length === 0) return false;
  const units: boolean[] = [];
  const groups = new Map<string, boolean>();
  const groupOrder: string[] = [];
  for (const cond of rule.conditions) {
    const matched = conditionMatches(cond, ctx);
    if (cond.group_id) {
      const existing = groups.get(cond.group_id);
      if (existing === undefined) groupOrder.push(cond.group_id);
      groups.set(cond.group_id, (existing ?? false) || matched);
    } else {
      units.push(matched);
    }
  }
  for (const g of groupOrder) units.push(groups.get(g) as boolean);
  return rule.match_type === "any" ? units.some(Boolean) : units.every(Boolean);
}
