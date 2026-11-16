/**
 * Pure builder helpers (WP-0718, REQ-LANE-024).
 * Covers: REQ-LANE-024 (WP-0718 AC-1, AC-2).
 *
 * The Receive Shipment definition under test is the CURRENT 13-step
 * definition (program/spec/schemas/examples/lane-wh-receiving.json),
 * copied to ./lane-wh-receiving.example.json. The copy in
 * ../lanes/lane-wh-receiving.example.json is the older 12-step one and
 * must NOT be touched (LaneBlueprint.test.tsx asserts twelve steps
 * against it); the builder tests the 13-step shape including the
 * verify-supplier child-lane step (1.3).
 */
import { describe, expect, it } from "vitest";

import example from "./lane-wh-receiving.example.json";
import {
  blankLaneDefinition,
  checkLastWarnings,
  collapseNotEvaluatedChecks,
  collectRunDataRefs,
  computeReadsWrites,
  CONDITION_OPERATORS,
  countPerChildFixtures,
  DEFAULT_CHECK,
  defaultBranchFor,
  deleteStageFromDefinition,
  deleteStepFromDefinition,
  findStepSettingField,
  flattenSteps,
  localPublishChecks,
  normalizeDefinition,
  plumbingFromLocal,
  repeatingRecordStepsWithoutFilter,
  stepCount,
} from "./builderUtils";
import type { LaneDefinition } from "../../lib/types";

const definition = example.lane as unknown as LaneDefinition;

describe("builderUtils", () => {
  it("AC-1: Receiving is 13 steps in definition order, numbered 1..13", () => {
    expect(stepCount(definition)).toBe(13);
    const flat = flattenSteps(definition);
    expect(flat.map((f) => f.order)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
    // The child-lane step is step 3 (1.3), in the Verify stage.
    expect(flat[2].step.id).toBe("verify-supplier");
    expect(flat[2].step.child_lane?.lane_id).toBe("lane-wh-supplier-check");
    expect(flat[2].step.child_lane?.anchor_record).toBe("$record.supplier_id");
    expect(flat[2].stage.id).toBe("verify");
  });

  it("AC-1: round-trip is byte-identical modulo key order", () => {
    const parsed = JSON.parse(normalizeDefinition(definition)) as LaneDefinition;
    expect(normalizeDefinition(parsed)).toBe(normalizeDefinition(definition));
    expect(stepCount(parsed)).toBe(13);
  });

  it("blank lane starts from a blank definition with one stage", () => {
    const blank = blankLaneDefinition();
    expect(blank.stages).toHaveLength(1);
    expect(blank.stages[0].steps).toHaveLength(0);
    expect(blank.status).toBe("draft");
  });

  it("finds the step that sets a run-data field", () => {
    // carrier is set by step 1.1's form fields (binds_to run_data.carrier).
    expect(findStepSettingField(definition, "carrier")).toBe("confirm-arrival");
    expect(findStepSettingField(definition, "no-such-field")).toBeNull();
  });

  it("collects $run_data references out of a condition", () => {
    const refs = collectRunDataRefs({
      match: "all",
      conditions: [
        { field: "$run_data.damage_approved", operator: "equals", value: true },
        { field: "$item.qty_picked", operator: "equals", compare_field: "$item.qty_reserved" },
      ],
    });
    expect(refs).toEqual(["damage_approved"]);
  });

  it("computes the per-step reads/writes map", () => {
    const plumbing = computeReadsWrites(definition);
    const carrier = plumbing.fields.find((f) => f.key === "carrier");
    expect(carrier?.writes).toContain("confirm-arrival");
    // staging_bin is written by assign-staging-bin and read by no later step.
    const bin = plumbing.fields.find((f) => f.key === "staging_bin");
    expect(bin?.writes).toContain("assign-staging-bin");
    expect(plumbing.fields.length).toBe(definition.run_data?.fields.length);
  });

  it("flags a check that is not the last step of its stage (P-10)", () => {
    const warnings = checkLastWarnings(definition);
    // discrepancy-check is the last step of record-receipt, so no warning;
    // a check moved earlier must warn and name the step that runs on pass.
    const moved: LaneDefinition = JSON.parse(JSON.stringify(definition));
    const stage = moved.stages.find((s) => s.id === "record-receipt")!;
    stage.steps.reverse();
    const movedWarnings = checkLastWarnings(moved);
    expect(movedWarnings).toHaveLength(1);
    expect(movedWarnings[0].stepId).toBe("discrepancy-check");
    expect(movedWarnings[0].nextStepName).toBe("Enter received quantities");
    expect(warnings).toHaveLength(0);
  });

  it("flags repeating record steps with no filter (Count Q-34 warning)", () => {
    const count = {
      id: "lane-wh-count",
      name: "Count",
      object_key: "cycle_count",
      version: 1,
      status: "draft",
      stages: [
        {
          id: "post-adjustments",
          name: "Post adjustments",
          steps: [
            {
              id: "post-count-adjustments",
              name: "Post count adjustments",
              kind: "record",
              repeat_over: { field: "lines", item_id_field: "id" },
              writes: [{ action_type: "warehouse.post_stock_move", inputs: {} }],
            },
          ],
        },
      ],
    } as unknown as LaneDefinition;
    expect(repeatingRecordStepsWithoutFilter(count)).toEqual(["post-count-adjustments"]);
    // Receiving has two repeating record steps of its own — the warning
    // is per step, not per lane, so they are flagged too.
    expect(repeatingRecordStepsWithoutFilter(definition)).toEqual([
      "post-receipt-moves",
      "post-putaway-moves",
    ]);
  });

  it("AC-2: local publish checks cover P-05, P-06, P-09, P-10, P-11, P-12", () => {
    const broken: LaneDefinition = JSON.parse(JSON.stringify(definition));
    // Approval with no writes (P-05) and a check with no fail branch (P-09).
    const approval = broken.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === "review-discrepancy")!;
    approval.writes = [];
    const check = broken.stages.flatMap((s) => s.steps).find((s) => s.id === "discrepancy-check")!;
    check.branches = check.branches!.filter((b) => b.on === "pass");
    // A branch to a step that does not exist (P-06).
    check.branches[0].target = { stage: "nope", step: "nope" };
    // An undeclared $run_data reference (P-11).
    check.check!.condition = { field: "$run_data.ghost", operator: "equals", value: true };
    // A provisional Action Type (P-12).
    const record = broken.stages.flatMap((s) => s.steps).find((s) => s.id === "post-receipt-moves")!;
    record.writes = [{ action_type: "warehouse.post_stock_move", inputs: {} }];
    const checks = localPublishChecks(broken, []);
    const byId = (id: string) => checks.filter((c) => c.id === id);
    expect(byId("P-05").some((c) => c.status === "refuse")).toBe(true);
    expect(byId("P-06").some((c) => c.status === "refuse")).toBe(true);
    expect(byId("P-09").some((c) => c.status === "refuse")).toBe(true);
    expect(byId("P-11").some((c) => c.status === "refuse")).toBe(true);
    expect(byId("P-12").some((c) => c.status === "refuse")).toBe(true);
    // The clean 13-step definition has no refuse from the local checks
    // except P-12 (its Action Types are provisional until WP-0705 adopts
    // them) — P-05/P-06/P-09/P-11 must all pass on it.
    const clean = localPublishChecks(definition, []);
    expect(clean.filter((c) => ["P-05", "P-06", "P-09", "P-11"].includes(c.id) && c.status === "refuse")).toEqual([]);
    expect(clean.some((c) => c.id === "P-12" && c.status === "refuse")).toBe(true);
  });

  it("plumbingFromLocal renders the service's plumbing-table shape", () => {
    const report = plumbingFromLocal(definition);
    expect(report.lane_key).toBe("lane-wh-receiving");
    const carrier = report.plumbing.find((r) => r.field === "carrier");
    expect(carrier?.field_type).toBe("string");
    expect(carrier?.steps).toContainEqual({ step_id: "confirm-arrival", access: "write" });
    expect(report.plumbing.length).toBe(definition.run_data?.fields.length);
  });

  it("a repeating record step WITH skip_when no longer warns (Count Q-34)", () => {
    const count = {
      id: "lane-wh-count",
      name: "Count",
      object_key: "cycle_count",
      version: 1,
      status: "draft",
      stages: [
        {
          id: "post-adjustments",
          name: "Post adjustments",
          steps: [
            {
              id: "post-count-adjustments",
              name: "Post count adjustments",
              kind: "record",
              repeat_over: { field: "lines", item_id_field: "id" },
              skip_when: { field: "$item.variance", operator: "equals", value: 0 },
              writes: [{ action_type: "warehouse.post_stock_move", inputs: {} }],
            },
          ],
        },
      ],
    } as unknown as LaneDefinition;
    expect(repeatingRecordStepsWithoutFilter(count)).toEqual([]);
  });

  it("kind-change default check matches the demo exactly", () => {
    expect(DEFAULT_CHECK).toEqual({
      scope: "anchor",
      condition: { field: "", operator: "equals", value: "" },
      message: "",
    });
  });

  it("deleting a step removes the branches pointing at it and counts them", () => {
    const { definition: edited, removedBranches } = deleteStepFromDefinition(
      definition,
      "review-discrepancy",
    );
    expect(removedBranches).toBe(1);
    expect(edited.stages.flatMap((s) => s.steps).some((s) => s.id === "review-discrepancy")).toBe(false);
    const check = edited.stages.flatMap((s) => s.steps).find((s) => s.id === "discrepancy-check");
    expect(check?.branches?.map((b) => b.on)).toEqual(["pass"]);
    // Deleting a step nothing points at removes no branches.
    expect(deleteStepFromDefinition(definition, "close-po").removedBranches).toBe(0);
  });

  it("deleting a stage deletes its steps and branches pointing at them", () => {
    const edited = deleteStageFromDefinition(definition, "record-receipt");
    expect(edited.stages.some((s) => s.id === "record-receipt")).toBe(false);
    expect(edited.stages.flatMap((s) => s.steps).some((s) => s.id === "discrepancy-check")).toBe(false);
    expect(edited.stages.flatMap((s) => s.steps)).toHaveLength(11);
  });

  it("a new branch defaults to the next step, else end held", () => {
    const next = defaultBranchFor(definition, "confirm-arrival", "fail");
    expect(next).toEqual({
      on: "fail",
      target: { stage: "verify", step: "match-delivery-note" },
    });
    const last = defaultBranchFor(definition, "close-po", "fail");
    expect(last).toEqual({ on: "fail", end: { outcome: "held" } });
  });

  it("collapses consecutive not-evaluated entries into one line naming the blocker", () => {
    const checks = [
      { id: "P-03", step: "check-it", status: "refuse" as const, reason: "step 'check-it' needs a role" },
      ...["P-04", "P-05", "P-06"].map((id) => ({
        id,
        step: null,
        status: "warn" as const,
        reason: "not evaluated: validate_definition stopped at P-03 - step 'check-it' needs a role",
      })),
      { id: "P-12", step: null, status: "pass" as const, reason: null },
    ];
    const collapsed = collapseNotEvaluatedChecks(checks);
    expect(collapsed).toHaveLength(3);
    expect(collapsed[0].id).toBe("P-03");
    expect(collapsed[1].collapsedCount).toBe(3);
    expect(collapsed[1].reason).toBe(
      "3 checks not evaluated — fix P-03 first: step 'check-it' needs a role",
    );
    expect(collapsed[2].id).toBe("P-12");
    // A lone not-evaluated entry still collapses to one line.
    const single = collapseNotEvaluatedChecks([
      { id: "P-04", step: null, status: "warn", reason: "not evaluated: validate_definition stopped at P-02 - bad object" },
    ]);
    expect(single).toHaveLength(1);
    expect(single[0].reason).toBe("1 check not evaluated — fix P-02 first: bad object");
  });

  it("Count per-line edits become per-child dry-run fixtures", () => {
    expect(
      countPerChildFixtures([
        { line: "HG-1001", expected: 10, counted: 8 },
        { line: "HG-1003", expected: 6, counted: 7 },
      ]),
    ).toEqual({ "HG-1001": { counted_qty: 8 }, "HG-1003": { counted_qty: 7 } });
  });

  it("condition operators come from the schema list", () => {
    expect(CONDITION_OPERATORS).toContain("equals");
    expect(CONDITION_OPERATORS).toContain("is_not_empty");
    expect(CONDITION_OPERATORS).toContain("on_or_after");
    expect(CONDITION_OPERATORS).toHaveLength(9);
  });
});
