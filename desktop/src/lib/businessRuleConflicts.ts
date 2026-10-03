import type { BusinessRule, TriggerSource } from "./types";

/**
 * Business Rule Board 2.0 (issue #194): "warn when two active rules on the
 * same object set incompatible required/visibility/editability states" -
 * net-new analysis, not an evaluation change. Deliberately conservative:
 * flags only the two field-effect pairs that are *always* a real problem
 * regardless of which rule's conditions end up true at runtime -
 * `hide`+`require` (a required field the user can never see to fill in) and
 * `lock`+`require` (a required field the user can never edit) on the same
 * target field across two different active rules. `show`/`hide` and
 * `lock`/`editable` disagreeing between two rules is *not* flagged - "last
 * matching rule wins" (business_rule_service::evaluate) is the documented,
 * intended way one rule overrides another's visibility/editability, not a
 * conflict.
 *
 * This can't prove the two rules' conditions can actually both be true for
 * the same record (that needs a SAT-style solver over arbitrary AND/OR
 * conditions, well beyond this issue's scope) - it flags every pair that
 * *could* conflict and lets the admin judge whether their conditions
 * actually overlap, same honest "may need review" framing the Workflow
 * graph editor's own dry-run already uses for branches it can't resolve.
 */
export type RuleConflictKind = "hide_vs_require" | "lock_vs_require";

export type RuleConflict = {
  fieldSource: TriggerSource;
  fieldKey: string;
  ruleAId: string;
  ruleAName: string;
  ruleBId: string;
  ruleBName: string;
  kind: RuleConflictKind;
};

const FIELD_EFFECT_ACTIONS = new Set(["require", "hide", "show", "lock", "editable"]);
const CONFLICTING_PAIRS: [string, string, RuleConflictKind][] = [
  ["hide", "require", "hide_vs_require"],
  ["lock", "require", "lock_vs_require"],
];

type Target = { fieldSource: TriggerSource; fieldKey: string; actionType: string };

function fieldTargets(rule: BusinessRule): Target[] {
  const out: Target[] = [];
  for (const a of rule.actions) {
    if (a.target_field_key && FIELD_EFFECT_ACTIONS.has(a.action_type)) {
      out.push({ fieldSource: a.target_field_source, fieldKey: a.target_field_key, actionType: a.action_type });
    }
  }
  return out;
}

export function findRuleConflicts(rules: BusinessRule[]): RuleConflict[] {
  const active = rules.filter((r) => r.is_active);
  const conflicts: RuleConflict[] = [];
  for (let i = 0; i < active.length; i++) {
    for (let j = i + 1; j < active.length; j++) {
      const a = active[i];
      const b = active[j];
      for (const ta of fieldTargets(a)) {
        for (const tb of fieldTargets(b)) {
          if (ta.fieldSource !== tb.fieldSource || ta.fieldKey !== tb.fieldKey) continue;
          for (const [typeX, typeY, kind] of CONFLICTING_PAIRS) {
            const matches = (ta.actionType === typeX && tb.actionType === typeY) || (ta.actionType === typeY && tb.actionType === typeX);
            if (matches) {
              conflicts.push({
                fieldSource: ta.fieldSource, fieldKey: ta.fieldKey,
                ruleAId: a.id, ruleAName: a.name, ruleBId: b.id, ruleBName: b.name, kind,
              });
            }
          }
        }
      }
    }
  }
  return conflicts;
}

export function describeConflict(c: RuleConflict, fieldLabel: string): string {
  const effect = c.kind === "hide_vs_require" ? "hidden by one rule and required by the other" : "locked read-only by one rule and required by the other";
  return `"${c.ruleAName}" and "${c.ruleBName}" disagree on ${fieldLabel} - it would be ${effect}.`;
}
