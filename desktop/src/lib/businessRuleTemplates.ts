import { builtinFieldsFor, builtinTriggerFieldFor, statusesForEntity } from "./types";
import type { BusinessRuleActionInput, BusinessRuleConditionInput, MatchType } from "./types";

/** Only key/label/field_type/options are used here - same slice
 * `BusinessRulesAdmin.tsx`'s own `CustomFieldLite` names. */
type CustomFieldLite = { key: string; label: string; field_type: string; options?: string[] | null };

/** Business Rule Board 2.0 (issue #194): 5 starter templates an admin can
 * begin a new rule from instead of a blank IF/THEN board - each pre-fills a
 * condition on the entity's own built-in trigger field (status/stage/
 * is_active, always present) and one action on the first field that
 * actually fits the template's action type, so the seeded rule is always
 * immediately valid (passes `validate_conditions`/`validate_actions`) and
 * only needs the specific field/value swapped in, same honest "saves
 * clicks, not a turnkey rule" scope `workflowTemplates.ts` already
 * documents for its own 5 templates. */
export type BusinessRuleTemplate = {
  key: string;
  label: string;
  description: string;
  build: (
    entityType: string,
    customFields: CustomFieldLite[],
  ) => {
    name: string;
    match_type: MatchType;
    conditions: BusinessRuleConditionInput[];
    actions: BusinessRuleActionInput[];
  };
};

function triggerCondition(entityType: string): BusinessRuleConditionInput {
  const values = statusesForEntity(entityType);
  return {
    field_source: "builtin",
    field_key: builtinTriggerFieldFor(entityType),
    operator: "equals",
    value: values[0] ?? "",
    compare_field_source: null,
    compare_field_key: null,
    group_id: null,
    relationship_definition_id: null,
  };
}

/** First active custom field, else first actionable built-in field - same
 * fallback order `emptyAction` (BusinessRulesAdmin.tsx) and
 * `default_branch_action` (business_rule_service.rs) already use, kept in
 * sync by hand since each lives on its own side of the language boundary.
 * `selectOnly` narrows to select-typed fields, for `restrict_choices`. */
function firstTarget(entityType: string, customFields: CustomFieldLite[], selectOnly: boolean): { key: string; source: "builtin" | "custom" } | null {
  const custom = customFields.find((f) => !selectOnly || f.field_type === "select");
  if (custom) return { key: custom.key, source: "custom" };
  const builtin = builtinFieldsFor(entityType).find((f) => f.actionable && (!selectOnly || f.field_type === "select"));
  if (builtin) return { key: builtin.key, source: "builtin" };
  return null;
}

function fieldAction(target: { key: string; source: "builtin" | "custom" } | null, actionType: BusinessRuleActionInput["action_type"], actionValue: string | null = null): BusinessRuleActionInput {
  return { action_type: actionType, target_field_key: target?.key ?? null, target_field_source: target?.source ?? "custom", action_value: actionValue, message: null };
}

export const BUSINESS_RULE_TEMPLATES: BusinessRuleTemplate[] = [
  {
    key: "conditional_required",
    label: "Conditional Required",
    description: "Make a field mandatory only once the record reaches a status you pick.",
    build: (entityType, customFields) => ({
      name: "Conditional Required",
      match_type: "all",
      conditions: [triggerCondition(entityType)],
      actions: [fieldAction(firstTarget(entityType, customFields, false), "require")],
    }),
  },
  {
    key: "conditional_visibility",
    label: "Conditional Visibility",
    description: "Hide a field until the record reaches a status you pick - pair with a second rule to show it again later.",
    build: (entityType, customFields) => ({
      name: "Conditional Visibility",
      match_type: "all",
      conditions: [triggerCondition(entityType)],
      actions: [fieldAction(firstTarget(entityType, customFields, false), "hide")],
    }),
  },
  {
    key: "validation",
    label: "Validation",
    description: "Block saving the record with a custom message when a condition you pick is met.",
    build: (entityType) => ({
      name: "Validation",
      match_type: "all",
      conditions: [triggerCondition(entityType)],
      actions: [{ action_type: "block_save", target_field_key: null, target_field_source: "custom", action_value: null, message: "This record cannot be saved in its current state." }],
    }),
  },
  {
    key: "conditional_choices",
    label: "Conditional Choices",
    description: "Narrow a select field's choices once the record reaches a status you pick - choose which options stay selectable after creating this.",
    build: (entityType, customFields) => ({
      name: "Conditional Choices",
      match_type: "all",
      conditions: [triggerCondition(entityType)],
      actions: [fieldAction(firstTarget(entityType, customFields, true), "restrict_choices", "")],
    }),
  },
  {
    key: "field_lock",
    label: "Field Lock",
    description: "Make a field read-only once the record reaches a status you pick.",
    build: (entityType, customFields) => ({
      name: "Field Lock",
      match_type: "all",
      conditions: [triggerCondition(entityType)],
      actions: [fieldAction(firstTarget(entityType, customFields, false), "lock")],
    }),
  },
];
