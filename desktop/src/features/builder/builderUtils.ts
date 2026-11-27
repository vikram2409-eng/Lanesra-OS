/**
 * Pure helpers for the visual lane builder (WP-0718, REQ-LANE-024).
 *
 * No React, no api calls: the canvas, the step panel, the condition
 * builder and the checks panel all read the same functions, and the
 * vitest suite proves them against the current 13-step Receive
 * Shipment definition. `localPublishChecks` is the client-side
 * fallback for AC-2 — the same P-05/P-06/P-09/P-10/P-11/P-12 reasons
 * the WP-0714 service returns — used while the builder_check_lane
 * command is still landing; the service result wins whenever it
 * answers.
 */
import type {
  BuilderPlumbingCell,
  BuilderReadsWrites,
  LaneBranch,
  LaneDefinition,
  LaneStep,
  PublishCheckResult,
} from "../../lib/types";

/// The demo's default check object (app.js `laneBuilderOnChange`,
/// kind→check): scope anchor, an empty single condition, empty
/// message. The desktop kind-change initialises exactly this, so a
/// check made by kind-change has a fieldset to render.
export const DEFAULT_CHECK = {
  scope: "anchor",
  condition: { field: "", operator: "equals", value: "" },
  message: "",
};

/// The schema's operator list (WP-0718 §2): operators come from the
/// schema, never free text.
export const CONDITION_OPERATORS = [
  "equals",
  "not_equals",
  "in_list",
  "is_empty",
  "is_not_empty",
  "greater_than",
  "less_than",
  "on_or_after",
  "on_or_before",
] as const;

export interface FlatStep {
  stage: { id: string; name: string };
  step: LaneStep;
  /** Definition order across the whole lane, 1-based. Order is definition
   * order; reordering renumbers nothing — the builder shows this number. */
  order: number;
}

export function flattenSteps(definition: LaneDefinition): FlatStep[] {
  const flat: FlatStep[] = [];
  let order = 0;
  for (const stage of definition.stages) {
    for (const step of stage.steps) {
      flat.push({ stage: { id: stage.id, name: stage.name }, step, order: ++order });
    }
  }
  return flat;
}

export function stepCount(definition: LaneDefinition): number {
  return definition.stages.reduce((n, s) => n + s.steps.length, 0);
}

/// Byte-identical modulo key order: sort object keys recursively, then
/// stringify. The AC-1 round-trip proof compares these, not raw bytes.
export function normalizeDefinition(definition: LaneDefinition): string {
  function sort(value: unknown): unknown {
    if (Array.isArray(value)) return value.map(sort);
    if (value && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value as Record<string, unknown>)
          .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
          .map(([k, v]) => [k, sort(v)]),
      );
    }
    return value;
  }
  return JSON.stringify(sort(definition));
}

export function blankLaneDefinition(): LaneDefinition {
  return {
    id: `lane-new-${Date.now().toString(36)}`,
    name: "Untitled lane",
    description: "",
    object_key: "",
    version: 1,
    status: "draft",
    stages: [{ id: "stage-1", name: "Stage 1", steps: [] }],
    run_data: { fields: [] },
  };
}

/// WP-0722 (REQ-LANE-024): the definition inside a drafted lane — the
/// guided start's "describe it" path hands the WP-0715 draft to the
/// builder, and the draft arrives as a stored lane whose first version
/// carries the definition as the `{ format_version, lane }` file shape.
/// Returns null when the reply has no parseable definition, so the
/// caller shows an error instead of opening an empty builder.
export function draftedLaneDefinition(drafted: {
  lane_key: string;
  versions?: { definition_json: string }[] | null;
}): { laneKey: string; definition: LaneDefinition } | null {
  const raw = drafted.versions?.[0]?.definition_json;
  if (!raw) return null;
  try {
    const file = JSON.parse(raw) as { lane?: LaneDefinition };
    const lane = file?.lane;
    if (!lane || typeof lane !== "object" || !lane.id || !(lane.name || "").trim()) return null;
    return { laneKey: drafted.lane_key, definition: lane };
  } catch {
    return null;
  }
}

/// WP-0723 (REQ-LANE-022): the definition inside an edited lane version —
/// the guided edit's "describe the change" path hands the WP-0723 edit
/// back as the stored draft version, whose definition is the same
/// `{ format_version, lane }` file shape. Returns null when the reply
/// has no parseable definition, so the caller shows an error instead of
/// opening an empty builder.
export function editedLaneDefinition(version: {
  definition_json: string;
}): LaneDefinition | null {
  try {
    const file = JSON.parse(version.definition_json) as { lane?: LaneDefinition };
    const lane = file?.lane;
    if (!lane || typeof lane !== "object" || !lane.id || !(lane.name || "").trim()) return null;
    return lane;
  } catch {
    return null;
  }
}

export function declaredRunDataKeys(definition: LaneDefinition): string[] {
  return (definition.run_data?.fields ?? []).map((f) => f.key);
}

/// The step that sets a run-data field: a form field bound to it, or a
/// `run_data.set` write naming it as its key. Null when no step sets it
/// — the condition builder flags that reference live (P-11).
export function findStepSettingField(definition: LaneDefinition, key: string): string | null {
  for (const stage of definition.stages) {
    for (const step of stage.steps) {
      if ((step.fields ?? []).some((f) => f.binds_to === `run_data.${key}`)) return step.id;
      if (
        (step.writes ?? []).some(
          (w) => w.action_type === "run_data.set" && w.inputs?.key === key,
        )
      ) {
        return step.id;
      }
    }
  }
  return null;
}

/// Every `$run_data.<key>` reference anywhere inside a value (condition,
/// validation, write inputs), in first-seen order, deduplicated.
export function collectRunDataRefs(value: unknown): string[] {
  const refs: string[] = [];
  function walk(v: unknown): void {
    if (typeof v === "string") {
      for (const m of v.matchAll(/\$run_data\.([A-Za-z0-9_]+)/g)) {
        if (!refs.includes(m[1])) refs.push(m[1]);
      }
      return;
    }
    if (Array.isArray(v)) {
      v.forEach(walk);
      return;
    }
    if (v && typeof v === "object") Object.values(v as Record<string, unknown>).forEach(walk);
  }
  walk(value);
  return refs;
}

export interface FieldPlumbing {
  key: string;
  reads: string[];
  writes: string[];
}

/// Lane-level plumbing (§4): for each declared run-data field, which
/// steps read it (conditions, validation, write inputs that reference
/// it) and which write it (a bound form field or a `run_data.set`).
export function computeReadsWrites(definition: LaneDefinition): { fields: FieldPlumbing[] } {
  const keys = declaredRunDataKeys(definition);
  const fields: FieldPlumbing[] = keys.map((key) => ({ key, reads: [], writes: [] }));
  const byKey = new Map(fields.map((f) => [f.key, f]));
  for (const stage of definition.stages) {
    for (const step of stage.steps) {
      for (const field of step.fields ?? []) {
        const m = field.binds_to.match(/^run_data\.([A-Za-z0-9_]+)$/);
        if (m && byKey.has(m[1])) byKey.get(m[1])!.writes.push(step.id);
      }
      for (const write of step.writes ?? []) {
        if (write.action_type === "run_data.set" && typeof write.inputs?.key === "string") {
          byKey.get(write.inputs.key as string)?.writes.push(step.id);
        }
        for (const ref of collectRunDataRefs(write.inputs)) {
          if (ref !== (write.inputs?.key as string | undefined)) {
            byKey.get(ref)?.reads.push(step.id);
          }
        }
      }
      for (const ref of collectRunDataRefs(step.check?.condition)) {
        byKey.get(ref)?.reads.push(step.id);
      }
      for (const ref of collectRunDataRefs(step.validation)) {
        byKey.get(ref)?.reads.push(step.id);
      }
    }
  }
  for (const f of fields) {
    f.reads = [...new Set(f.reads)];
    f.writes = [...new Set(f.writes)];
  }
  return { fields };
}

export interface CheckLastWarning {
  stepId: string;
  stageId: string;
  /** The step that would run on pass — the happy path runs it. */
  nextStepName: string;
}

/// P-10: a check that is not the last step of its stage. The runner
/// takes the next step in definition order on pass, so the steps after
/// the check are the hold branch's steps running on the happy path.
export function checkLastWarnings(definition: LaneDefinition): CheckLastWarning[] {
  const warnings: CheckLastWarning[] = [];
  for (const stage of definition.stages) {
    stage.steps.forEach((step, i) => {
      if (step.kind === "check" && i < stage.steps.length - 1) {
        warnings.push({
          stepId: step.id,
          stageId: stage.id,
          nextStepName: stage.steps[i + 1].name,
        });
      }
    });
  }
  return warnings;
}

/// The Q-34 warning (§8, Count): a repeating record step has no filter,
/// so it will post for every child — including the ones with nothing to
/// post. No schema change: this is a warning, not an editor. A step WITH
/// a `skip_when` condition no longer warns: the author has said which
/// children to skip (Q-34, accepted for this batch; the runner honors
/// it once WP-0710 lands).
export function repeatingRecordStepsWithoutFilter(definition: LaneDefinition): string[] {
  return definition.stages
    .flatMap((s) => s.steps)
    .filter((s) => s.kind === "record" && s.repeat_over && !(s as unknown as { skip_when?: unknown }).skip_when)
    .map((s) => s.id);
}

/// Delete a step: the step goes, and every branch that pointed at it
/// goes with it (a branch to a deleted step would dangle, P-06). The
/// caller reports how many branches were removed.
export function deleteStepFromDefinition(
  definition: LaneDefinition,
  stepId: string,
): { definition: LaneDefinition; removedBranches: number } {
  const next: LaneDefinition = JSON.parse(JSON.stringify(definition));
  let removedBranches = 0;
  for (const stage of next.stages) {
    for (const step of stage.steps) {
      const before = (step.branches ?? []).length;
      step.branches = (step.branches ?? []).filter(
        (b) => !(b.target && b.target.step === stepId),
      );
      removedBranches += before - (step.branches ?? []).length;
    }
    stage.steps = stage.steps.filter((s) => s.id !== stepId);
  }
  return { definition: next, removedBranches };
}

/// Delete a stage: its steps go with it, and branches anywhere in the
/// lane that pointed at one of those steps go too (same dangling-branch
/// rule as deleting a step).
export function deleteStageFromDefinition(definition: LaneDefinition, stageId: string): LaneDefinition {
  const stage = definition.stages.find((s) => s.id === stageId);
  if (!stage) return definition;
  const removed = new Set(stage.steps.map((s) => s.id));
  const next: LaneDefinition = JSON.parse(JSON.stringify(definition));
  for (const s of next.stages) {
    for (const step of s.steps) {
      step.branches = (step.branches ?? []).filter(
        (b) => !(b.target && removed.has(b.target.step)),
      );
    }
  }
  next.stages = next.stages.filter((s) => s.id !== stageId);
  return next;
}

/// The next step in definition order after `stepId`, or null when the
/// step is the last one. A new branch defaults to it (never to the
/// step itself — a self-loop publishes today and retries forever).
export function nextStepInDefinitionOrder(definition: LaneDefinition, stepId: string): FlatStep | null {
  const flat = flattenSteps(definition);
  const i = flat.findIndex((f) => f.step.id === stepId);
  if (i < 0 || i + 1 >= flat.length) return null;
  return flat[i + 1];
}

/// The default for a new branch on a step: the next step in definition
/// order if there is one, else an end with outcome `held`.
export function defaultBranchFor(definition: LaneDefinition, stepId: string, on: string): LaneBranch {
  const next = nextStepInDefinitionOrder(definition, stepId);
  if (next) return { on, target: { stage: next.stage.id, step: next.step.id } };
  return { on, end: { outcome: "held" } };
}

export interface CollapsedCheck {
  key: string;
  id: string;
  step: string | null;
  status: PublishCheckResult["status"];
  reason: string | null;
  /** How many `not evaluated` entries this one line stands for. */
  collapsedCount?: number;
}

/// Gap 12: one early refusal floods the list with `warn — not
/// evaluated` entries (`not evaluated: validate_definition stopped at
/// P-0Y - <message>`). Collapse each consecutive run of them into one
/// line that names the blocking check, so the one real refusal is not
/// drowned. Non-`not evaluated` entries pass through untouched.
export function collapseNotEvaluatedChecks(checks: PublishCheckResult[]): CollapsedCheck[] {
  const out: CollapsedCheck[] = [];
  let i = 0;
  while (i < checks.length) {
    const c = checks[i];
    if (c.status === "warn" && c.reason?.startsWith("not evaluated:")) {
      let j = i;
      while (
        j < checks.length &&
        checks[j].status === "warn" &&
        checks[j].reason?.startsWith("not evaluated:")
      ) {
        j++;
      }
      const n = j - i;
      const m = /validate_definition stopped at (P-\d+)(?: - (.*))?/.exec(c.reason ?? "");
      const reason = m
        ? `${n} check${n === 1 ? "" : "s"} not evaluated — fix ${m[1]} first: ${m[2] ?? "see the refusal above"}`
        : `${n} check${n === 1 ? "" : "s"} not evaluated — ${c.reason ?? ""}`;
      out.push({
        key: `not-evaluated-${i}`,
        id: "not-evaluated",
        step: null,
        status: "warn",
        reason,
        collapsedCount: n,
      });
      i = j;
    } else {
      out.push({
        key: `${c.id}:${c.step ?? ""}:${i}`,
        id: c.id,
        step: c.step,
        status: c.status,
        reason: c.reason,
      });
      i++;
    }
  }
  return out;
}

export interface CountLineRow {
  line: string;
  expected: number;
  counted: number;
}

/// Gap 15: the Count per-line table's edits are dry-run fixtures for
/// the count step (`per_child[line].counted_qty`), keyed the way
/// `DryRunFixtures.per_child` expects (child id, then field name).
/// `counted_qty` is record data, not lane data, so the edits never
/// touch the stored definition — they change what the next dry run
/// simulates, and the dry run previews the variances/adjustments.
export function countPerChildFixtures(rows: CountLineRow[]): Record<string, Record<string, unknown>> {
  return Object.fromEntries(rows.map((r) => [r.line, { counted_qty: r.counted }]));
}

/// Client-side subset of the WP-0714 publish checks, for the live panel
/// while builder_check_lane is still landing. `adoptedActionTypes` is
/// the WP-0705 catalog; until it lands every Action Type is provisional
/// and P-12 refuses.
export function localPublishChecks(
  definition: LaneDefinition,
  adoptedActionTypes: string[],
): PublishCheckResult[] {
  const checks: PublishCheckResult[] = [];
  const declared = new Set(declaredRunDataKeys(definition));
  const existing = new Set(
    definition.stages.flatMap((s) => s.steps.map((st) => `${s.id}/${st.id}`)),
  );
  const pass = (id: string, step: string | null): PublishCheckResult => ({
    id,
    step,
    status: "pass",
    reason: null,
  });
  const refuse = (id: string, step: string | null, reason: string): PublishCheckResult => ({
    id,
    step,
    status: "refuse",
    reason,
  });

  let p05 = false;
  let p06 = false;
  let p09 = false;
  let p11 = false;
  let p12 = false;
  for (const stage of definition.stages) {
    for (const step of stage.steps) {
      if (step.kind === "approval" && (step.writes ?? []).length === 0) {
        checks.push(refuse("P-05", step.id, `approval '${step.id}' has no writes`));
        p05 = true;
      }
      const expected = step.kind === "check" ? ["pass", "fail"] : step.kind === "approval" ? ["approved", "rejected"] : [];
      const have = new Set((step.branches ?? []).map((b) => b.on));
      for (const on of expected) {
        if (!have.has(on)) {
          checks.push(refuse("P-09", step.id, `${step.kind} '${step.id}' has no ${on} branch`));
          p09 = true;
        }
      }
      for (const branch of step.branches ?? []) {
        if (branch.target && !existing.has(`${branch.target.stage}/${branch.target.step}`)) {
          checks.push(
            refuse("P-06", step.id, `branch '${branch.on}' targets missing step ${branch.target.stage}/${branch.target.step}`),
          );
          p06 = true;
        }
      }
      for (const ref of [
        ...collectRunDataRefs(step.check?.condition),
        ...collectRunDataRefs(step.validation),
        ...collectRunDataRefs(step.writes),
      ]) {
        if (!declared.has(ref)) {
          checks.push(
            refuse("P-11", step.id, `$run_data.${ref} is not declared in lane.run_data.fields`),
          );
          p11 = true;
        }
      }
      for (const write of step.writes ?? []) {
        if (!adoptedActionTypes.includes(write.action_type)) {
          checks.push(
            refuse(
              "P-12",
              step.id,
              `step '${step.id}' uses provisional Action Type '${write.action_type}' — not in the adopted catalog`,
            ),
          );
          p12 = true;
        }
      }
    }
  }
  if (!p05) checks.push(pass("P-05", null));
  if (!p06) checks.push(pass("P-06", null));
  if (!p09) checks.push(pass("P-09", null));
  if (!p11) checks.push(pass("P-11", null));
  if (!p12) checks.push(pass("P-12", null));
  for (const w of checkLastWarnings(definition)) {
    checks.push({
      id: "P-10",
      step: w.stepId,
      status: "warn",
      reason: `check '${w.stepId}' is not the last step of stage '${w.stageId}' — '${w.nextStepName}' runs on the pass path`,
    });
  }
  checks.push(pass("P-01", null));
  return checks.sort((a, b) => a.id.localeCompare(b.id));
}

/// The local reads/writes map in the service's ReadsWritesReport shape
/// (builder_reads_writes), so the plumbing table renders identically
/// whether the service or the local computation supplied it.
export function plumbingFromLocal(definition: LaneDefinition): BuilderReadsWrites {
  const { fields } = computeReadsWrites(definition);
  const declared = definition.run_data?.fields ?? [];
  return {
    lane_key: definition.id,
    steps: [],
    lane_reads: [],
    plumbing: fields.map((f) => {
      const declaredField = declared.find((d) => d.key === f.key);
      const cells: BuilderPlumbingCell[] = [...new Set([...f.reads, ...f.writes])].map((id) => ({
        step_id: id,
        access:
          f.reads.includes(id) && f.writes.includes(id)
            ? "both"
            : f.writes.includes(id)
              ? "write"
              : "read",
      }));
      return {
        field: f.key,
        field_type: declaredField?.type ?? null,
        per_child: declaredField?.per_child ?? false,
        steps: cells,
      };
    }),
  };
}

/// The Action Types the four warehouse lanes use today. Until WP-0705
/// adopts them they are provisional (P-12) — the picker shows them
/// marked provisional so the author can draft with them.
export function provisionalActionTypes(definition: LaneDefinition): string[] {
  const ids = new Set<string>();
  for (const stage of definition.stages) {
    for (const step of stage.steps) {
      for (const write of step.writes ?? []) ids.add(write.action_type);
    }
  }
  return [...ids].sort();
}
