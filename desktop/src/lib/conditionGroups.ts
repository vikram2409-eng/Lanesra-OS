import type { MatchType } from "./types";

/**
 * One level of nested OR-grouping (migration 0020) shared by the Business
 * Rules and Workflow Automation builders - a condition with no `group_id`
 * participates directly in the rule's top-level match_type; conditions
 * sharing a `group_id` are OR'd together into one sub-unit first, and that
 * sub-unit then participates in the top-level match_type alongside the
 * ungrouped conditions. Mirrors `core::domain::conditions::conditions_match`
 * exactly - see that function's doc comment for the full explanation.
 *
 * Deliberately generic over the condition shape (just needs `group_id`) so
 * both admin screens' own condition types - typed against their own
 * BusinessRuleCondition(Input)/WorkflowCondition(Input) interfaces - can
 * share this one grouping algorithm instead of each hand-rolling it.
 */
export type ConditionUnit =
  | { kind: "single"; index: number }
  | { kind: "group"; groupId: string; indices: number[] };

/** Groups a flat conditions array into top-level units, in first-occurrence
 * order - used both by the builder (to render an OR-group as one bordered
 * box) and by read-only summaries (to know which conditions to parenthesize
 * together). */
export function groupConditionIndices(conditions: { group_id: string | null }[]): ConditionUnit[] {
  const units: ConditionUnit[] = [];
  const byGroup = new Map<string, ConditionUnit & { kind: "group" }>();
  conditions.forEach((c, i) => {
    if (c.group_id) {
      let g = byGroup.get(c.group_id);
      if (!g) {
        g = { kind: "group", groupId: c.group_id, indices: [] };
        byGroup.set(c.group_id, g);
        units.push(g);
      }
      g.indices.push(i);
    } else {
      units.push({ kind: "single", index: i });
    }
  });
  return units;
}

/** Plain-language description of a whole conditions list, honoring
 * OR-groups - `(a OR b) AND c` rather than the flat `a OR b AND c` a naive
 * join would produce. */
export function describeGroupedConditions<C extends { group_id: string | null }>(
  conditions: C[],
  matchType: MatchType,
  describeOne: (c: C) => string,
): string {
  const units = groupConditionIndices(conditions);
  const parts = units.map((u) =>
    u.kind === "single"
      ? describeOne(conditions[u.index])
      : `(${u.indices.map((i) => describeOne(conditions[i])).join(" OR ")})`,
  );
  return parts.join(matchType === "any" ? " OR " : " AND ");
}

/** A short, human-readable id for a new OR-group - never persisted or
 * parsed, just needs to be distinct within one rule's conditions array. */
export function newGroupId(): string {
  return `g${Math.random().toString(36).slice(2, 9)}`;
}
