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
  collectRunDataRefs,
  computeReadsWrites,
  CONDITION_OPERATORS,
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

  it("condition operators come from the schema list", () => {
    expect(CONDITION_OPERATORS).toContain("equals");
    expect(CONDITION_OPERATORS).toContain("is_not_empty");
    expect(CONDITION_OPERATORS).toContain("on_or_after");
    expect(CONDITION_OPERATORS).toHaveLength(9);
  });
});
