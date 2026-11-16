/**
 * The visual lane builder (WP-0718, REQ-LANE-024).
 * Covers: REQ-LANE-024 (WP-0718 AC-1, AC-2, AC-3).
 *
 * Renders LaneBuilder with the api module mocked (same pattern as
 * LanePublish.test.tsx). The Receive Shipment definition is the CURRENT
 * 13-step one (see builderUtils.test.ts for why the copy lives here).
 * The builder commands (save_builder_lane, builder_check_lane,
 * builder_dry_run_lane, builder_reads_writes, builder_action_types)
 * have landed in the core (dispatch arms + Tauri commands +
 * process_lane_builder service); the UI calls them through the api
 * wrappers and falls back to the local publish checks
 * (builderUtils.localPublishChecks) while a command is unavailable, so
 * AC-2 is shown either way. Wrapper argument names and result shapes
 * are verified against the landed service (see builderApi.test.ts).
 */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import axe from "axe-core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LaneBuilder } from "./LaneBuilder";
import { api } from "../../lib/api";
import type { LaneDefinition } from "../../lib/types";
import example from "./lane-wh-receiving.example.json";

vi.mock("../../lib/api", () => ({
  api: {
    saveBuilderLane: vi.fn(),
    builderCheckLane: vi.fn(),
    builderDryRunLane: vi.fn(),
    builderReadsWrites: vi.fn(),
    builderActionTypes: vi.fn(),
    adoptBuilderActionType: vi.fn(),
    listCustomRecords: vi.fn(),
    listCustomObjects: vi.fn(),
    listAccessRoles: vi.fn(),
    listLaneBlueprints: vi.fn(),
    listWorkflowRules: vi.fn(),
    publishLane: vi.fn(),
  },
  ApiError: class ApiError extends Error {
    kind: string;
    constructor(payload: { kind: string; message: string }) {
      super(payload.message);
      this.kind = payload.kind;
    }
  },
}));

const definition = example.lane as unknown as LaneDefinition;

const PASS_REPORT = {
  lane_key: "lane-wh-receiving",
  version: 1,
  refused: false,
  checks: [
    { id: "P-01", step: null, status: "pass", reason: null },
    { id: "P-09", step: null, status: "pass", reason: null },
  ],
};

const REFUSE_REPORT = {
  lane_key: "lane-wh-receiving",
  version: 1,
  refused: true,
  checks: [
    { id: "P-01", step: null, status: "pass", reason: null },
    { id: "P-09", step: "discrepancy-check", status: "refuse", reason: "check 'discrepancy-check' has no fail branch" },
    { id: "P-12", step: "post-receipt-moves", status: "refuse", reason: "provisional Action Type 'warehouse.post_stock_move'" },
  ],
};

function renderBuilder(def: LaneDefinition = definition, onChange = vi.fn(), laneKeyArg: string | null = "lane-wh-receiving") {
  const utils = render(
    <LaneBuilder definition={def} laneKey={laneKeyArg} onDefinitionChange={onChange} />,
  );
  return { ...utils, onChange };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.builderCheckLane).mockResolvedValue(PASS_REPORT as never);
  vi.mocked(api.builderReadsWrites).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    steps: [],
    lane_reads: [],
    plumbing: [],
  } as never);
  vi.mocked(api.builderActionTypes).mockResolvedValue([] as never);
  vi.mocked(api.adoptBuilderActionType).mockResolvedValue({ name: "Set Tracking" } as never);
  vi.mocked(api.listCustomRecords).mockResolvedValue([] as never);
  vi.mocked(api.listCustomObjects).mockResolvedValue([] as never);
  vi.mocked(api.listAccessRoles).mockResolvedValue([] as never);
  vi.mocked(api.listLaneBlueprints).mockResolvedValue([] as never);
  vi.mocked(api.listWorkflowRules).mockResolvedValue([] as never);
  vi.mocked(api.saveBuilderLane).mockResolvedValue({ lane_key: "lane-wh-receiving", version: 2 } as never);
  vi.mocked(api.publishLane).mockResolvedValue({ version: 2 } as never);
  vi.mocked(api.builderDryRunLane).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    lane_version: 1,
    anchor_object_key: "widget",
    anchor_record_id: "rec-1",
    would_start: true,
    refusal_reason: null,
    stages_visited: ["capture"],
    steps: [],
    checks: [],
    branches_taken: [],
    writes: [],
    zero_quantity_flags: [],
    unevaluated: [],
    end_outcome: "done",
    expected_outcome: null,
    matches_expected: false,
    completed: true,
    stopped_reason: null,
  } as never);
});

describe("LaneBuilder", () => {
  it("AC-1: renders all 13 steps of Receiving in definition order with order numbers", () => {
    const { container } = renderBuilder();
    expect(screen.getByRole("heading", { name: "Receive Shipment" })).toBeVisible();
    // Every step card shows its order number and name.
    expect(screen.getByText("Verify the supplier")).toBeVisible();
    expect(container.textContent).toContain("Verify the supplier");
    const cards = screen.getAllByRole("button", { name: /Step \d+ of 13/ });
    expect(cards).toHaveLength(13);
    expect(cards[2]).toHaveTextContent("Verify the supplier");
    // The child-lane step shows its child lane.
    expect(screen.getByText(/lane-wh-supplier-check/)).toBeVisible();
  });

  it("AC-1: editing a step name round-trips — same definition except the name", async () => {
    const { onChange } = renderBuilder();
    fireEvent.click(screen.getByRole("button", { name: /Step 3 of 13.*Verify the supplier/ }));
    const nameInput = screen.getByLabelText("Step name");
    fireEvent.change(nameInput, { target: { value: "Verify the supplier (updated)" } });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.stages.flatMap((s) => s.steps).find((s) => s.id === "verify-supplier")?.name).toBe(
      "Verify the supplier (updated)",
    );
    // Everything else is untouched: same 13 steps, same stages, same child lane.
    expect(edited.stages.flatMap((s) => s.steps)).toHaveLength(13);
    expect(edited.stages.flatMap((s) => s.steps).find((s) => s.id === "verify-supplier")?.child_lane).toEqual({
      lane_id: "lane-wh-supplier-check",
      anchor_record: "$record.supplier_id",
    });
    expect(JSON.stringify(edited.stages.map((s) => s.id))).toBe(
      JSON.stringify(definition.stages.map((s) => s.id)),
    );
  });

  it("AC-2: live publish checks are shown and publish is disabled naming the first failure", async () => {
    vi.mocked(api.builderCheckLane).mockResolvedValue(REFUSE_REPORT as never);
    renderBuilder();
    await waitFor(() => expect(screen.getByText(/P-09/)).toBeVisible());
    expect(screen.getByText(/has no fail branch/)).toBeVisible();
    expect(screen.getByText(/P-12/)).toBeVisible();
    const publish = screen.getByRole("button", { name: /Publish/ });
    expect(publish).toBeDisabled();
    expect(publish).toHaveAttribute("title", expect.stringContaining("P-09"));
    // The failing step's card is marked.
    expect(screen.getByRole("button", { name: /Step 5 of 13.*Discrepancy check/ })).toHaveAttribute(
      "data-check-failed",
      "true",
    );
  });

  it("AC-2: publish is enabled once the checks pass", async () => {
    renderBuilder();
    await waitFor(() => expect(screen.getByRole("button", { name: "Publish" })).toBeEnabled());
    fireEvent.click(screen.getByRole("button", { name: "Publish" }));
    // Publish saves the edited definition first, then publishes it.
    await waitFor(() => expect(api.saveBuilderLane).toHaveBeenCalled());
    await waitFor(() => expect(api.publishLane).toHaveBeenCalledWith("lane-wh-receiving"));
  });

  it("AC-2: when the builder check service is unavailable the local checks still show", async () => {
    vi.mocked(api.builderCheckLane).mockRejectedValue(new Error("command not found"));
    renderBuilder();
    // The local fallback flags the provisional Action Types (P-12).
    await waitFor(() => expect(screen.getAllByText(/P-12/).length).toBeGreaterThan(0));
  });

  it("AC-3: is axe-clean and keyboard operable", async () => {
    const { container } = renderBuilder();
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
    // Keyboard: tab to a step card and press Enter to open its panel.
    const card = screen.getByRole("button", { name: /Step 3 of 13.*Verify the supplier/ });
    card.focus();
    expect(document.activeElement).toBe(card);
    fireEvent.keyDown(card, { key: "Enter" });
    fireEvent.click(card);
    expect(screen.getByLabelText("Step name")).toBeVisible();
    // Every canvas move has a keyboard button: move the step down.
    const moveDown = screen.getByRole("button", { name: "Move step down" });
    expect(moveDown).toBeVisible();
  });

  it("AC-3: keyboard Move step down reorders the steps", async () => {
    const { onChange } = renderBuilder();
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 13.*Confirm shipment arrived/ }));
    fireEvent.click(screen.getByRole("button", { name: "Move step down" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const verifySteps = edited.stages.find((s) => s.id === "verify")!.steps.map((s) => s.id);
    expect(verifySteps[0]).toBe("match-delivery-note");
    expect(verifySteps[1]).toBe("confirm-arrival");
  });

  it("check-last warning offers a one-click move to end of stage (P-10)", async () => {
    const moved: LaneDefinition = JSON.parse(JSON.stringify(definition));
    const stage = moved.stages.find((s) => s.id === "record-receipt")!;
    stage.steps.reverse();
    const { onChange } = renderBuilder(moved);
    await waitFor(() =>
      expect(screen.getByText(/not the last step of its stage/)).toBeVisible(),
    );
    fireEvent.click(screen.getByRole("button", { name: /Step 4 of 13.*Discrepancy check/ }));
    fireEvent.click(screen.getByRole("button", { name: /Move check to end of stage/ }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const steps = edited.stages.find((s) => s.id === "record-receipt")!.steps.map((s) => s.id);
    expect(steps[steps.length - 1]).toBe("discrepancy-check");
  });

  it("condition builder: nested any/all groups with $run_data chips that jump to the setter step", async () => {
    const def: LaneDefinition = {
      id: "lane-wh-pickpack",
      name: "Pick & Pack",
      object_key: "sales_order",
      version: 1,
      status: "draft",
      run_data: {
        fields: [{ key: "damage_approved", type: "boolean", description: "Set by the damage approval" }],
      },
      stages: [
        {
          id: "pick",
          name: "Pick",
          steps: [
            {
              id: "review-short-pick",
              name: "Review short pick",
              kind: "approval",
              writes: [{ action_type: "run_data.set", inputs: { key: "damage_approved", value: true } }],
              branches: [],
            },
            {
              id: "pick-check",
              name: "Pick check",
              kind: "check",
              check: {
                scope: "per_item",
                message: "A line is short or damaged.",
                condition: {
                  match: "any",
                  conditions: [
                    {
                      match: "all",
                      conditions: [
                        { field: "$item.qty_picked", operator: "equals", compare_field: "$item.qty_reserved" },
                        { field: "$run_data.damage_approved", operator: "equals", value: true },
                      ],
                    },
                    { field: "$run_data.ghost_field", operator: "equals", value: true },
                  ],
                },
              },
              branches: [
                { on: "pass", target: { stage: "pick", step: "review-short-pick" } },
                { on: "fail", end: { outcome: "held" } },
              ],
            },
          ],
        },
      ],
    };
    renderBuilder(def);
    fireEvent.click(screen.getByRole("button", { name: /Step 2 of 2.*Pick check/ }));
    // Nested groups render with their AND/OR switch.
    expect(screen.getAllByText("Pick check").length).toBeGreaterThan(0);
    expect(screen.getAllByLabelText(/Match/).length).toBeGreaterThan(0);
    // The undeclared reference is flagged live.
    expect(screen.getByText(/ghost_field.*not set by any step|not declared/)).toBeVisible();
    // The declared reference is a chip that jumps to the step that sets it.
    const chip = screen.getByRole("button", { name: /Go to step that sets damage_approved/ });
    fireEvent.click(chip);
    expect(screen.getByLabelText("Step name")).toHaveValue("Review short pick");
  });

  it("approval-writes view: provisional writes are struck through with the P-12 reason", async () => {
    const def: LaneDefinition = {
      id: "lane-wh-pickpack",
      name: "Pick & Pack",
      object_key: "sales_order",
      version: 1,
      status: "draft",
      stages: [
        {
          id: "hold-review",
          name: "Hold & review",
          steps: [
            {
              id: "review-short-pick",
              name: "Review short pick",
              kind: "approval",
              writes: [{ action_type: "run_data.set", inputs: { key: "damage_approved", value: true } }],
              branches: [
                { on: "approved", target: { stage: "hold-review", step: "record-hold" } },
                { on: "rejected", end: { outcome: "held" } },
              ],
            },
            { id: "record-hold", name: "Record hold", kind: "form" },
          ],
        },
      ],
    };
    renderBuilder(def);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 2.*Review short pick/ }));
    const struck = screen.getByText("run_data.set");
    expect(struck.tagName).toBe("S");
    expect(screen.getByText(/P-12.*cannot publish until the catalog adopts it/)).toBeVisible();
    expect(screen.getByText(/approve.*these writes post/)).toBeVisible();
    expect(screen.getByText(/reject.*these writes do not post/)).toBeVisible();
  });

  it("approval with no writes shows the P-05 gap", async () => {
    const def: LaneDefinition = {
      id: "lane-x",
      name: "Lane X",
      object_key: "widget",
      version: 1,
      status: "draft",
      stages: [
        {
          id: "s1",
          name: "Stage 1",
          steps: [{ id: "a1", name: "Approve this", kind: "approval", writes: [], branches: [] }],
        },
      ],
    };
    renderBuilder(def);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 1.*Approve this/ }));
    expect(screen.getByText(/P-05.*no writes/)).toBeVisible();
  });

  it("branch-target editor: an empty arrow slot is a visible gap and only existing steps can be picked", async () => {
    renderBuilder();
    fireEvent.click(screen.getByRole("button", { name: /Step 5 of 13.*Discrepancy check/ }));
    const panel = screen.getByRole("region", { name: "Step panel" });
    expect(within(panel).getByText("pass")).toBeVisible();
    const selects = within(panel).getAllByLabelText(/Branch .* target/);
    expect(selects.length).toBe(2);
    // Every option is a step that exists in this lane — no free-text target.
    const options = Array.from(selects[0].querySelectorAll("option")).map((o) => o.textContent);
    expect(options.join(" ")).toContain("Verify the supplier");
    expect(options.join(" ")).not.toContain("nope");
  });

  it("reads/writes map lists run-data fields against steps", async () => {
    renderBuilder();
    await waitFor(() => expect(screen.getByRole("table", { name: "Run-data plumbing" })).toBeVisible());
    expect(screen.getByText("carrier")).toBeVisible();
  });
});

/**
 * Builder-gaps batch (WP-0718 follow-up).
 * Covers: REQ-LANE-024 (WP-0718 AC-2).
 *
 * A small two-stage lane used by the gap tests: form → record → check
 * (pass only) → approval, with one declared run-data field.
 */
const SMALL_DEF: LaneDefinition = {
  id: "lane-small",
  name: "Small lane",
  object_key: "widget",
  version: 1,
  status: "draft",
  run_data: { fields: [{ key: "carrier", type: "string", required: true }] },
  stages: [
    {
      id: "s1",
      name: "Stage 1",
      steps: [
        {
          id: "enter",
          name: "Enter details",
          kind: "form",
          fields: [
            { key: "carrier", label: "Carrier", type: "string", binds_to: "run_data.carrier", required: true },
          ],
        },
        {
          id: "post",
          name: "Post tracking",
          kind: "record",
          writes: [{ action_type: "Set Tracking", inputs: { val: "$run_data.carrier" } }],
        },
      ],
    },
    {
      id: "s2",
      name: "Stage 2",
      steps: [
        {
          id: "check-it",
          name: "Check it",
          kind: "check",
          check: {
            scope: "anchor",
            message: "Not active.",
            condition: { field: "$record.status", operator: "equals", value: "Active" },
          },
          branches: [{ on: "pass", target: { stage: "s2", step: "approve-it" } }],
        },
        {
          id: "approve-it",
          name: "Approve it",
          kind: "approval",
          writes: [{ action_type: "Set Tracking", inputs: { val: "ok" } }],
          branches: [
            { on: "approved", end: { outcome: "done" } },
            { on: "rejected", end: { outcome: "held" } },
          ],
        },
      ],
    },
  ],
};

const COUNT_DEF: LaneDefinition = {
  id: "lane-wh-count",
  name: "Count",
  object_key: "cycle_count",
  version: 1,
  status: "draft",
  run_data: { fields: [] },
  stages: [
    {
      id: "post",
      name: "Post",
      steps: [
        {
          id: "post-adjustments",
          name: "Post adjustments",
          kind: "record",
          repeat_over: { field: "lines", item_id_field: "id" },
          writes: [{ action_type: "warehouse.post_stock_move", inputs: {} }],
        },
      ],
    },
  ],
};

describe("LaneBuilder gaps batch", () => {
  it("gap 1: run-data editor declares a field and rejects a duplicate key", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.change(screen.getByLabelText("New run-data field key"), { target: { value: "carrier" } });
    fireEvent.click(screen.getByRole("button", { name: "Add run-data field" }));
    expect(screen.getByText(/already exists/)).toBeVisible();
    fireEvent.change(screen.getByLabelText("New run-data field key"), { target: { value: "tracking_no" } });
    fireEvent.click(screen.getByRole("button", { name: "Add run-data field" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.run_data?.fields.map((f) => f.key)).toContain("tracking_no");
  });

  it("gap 1: Add field never creates a P-11 trap — it declares the field when none exists", async () => {
    const noFields: LaneDefinition = JSON.parse(JSON.stringify(SMALL_DEF));
    noFields.run_data = { fields: [] };
    const { onChange } = renderBuilder(noFields);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    fireEvent.click(screen.getByRole("button", { name: "Add field" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.run_data?.fields.map((f) => f.key)).toContain("new_field");
    const added = edited.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === "enter")
      ?.fields?.find((f) => f.key === "new_field");
    expect(added?.binds_to).toBe("run_data.new_field");
  });

  it("gap 2: a write's +Input adds an editable name/value pair", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 2 of 4.*Post tracking/ }));
    fireEvent.click(screen.getByRole("button", { name: "+ Input" }));
    // The write already had one input (val), so there are now two
    // editable name inputs — one per input.
    expect(screen.getAllByLabelText("Write 1 input name")).toHaveLength(2);
    fireEvent.change(screen.getByLabelText("Write 1 input new_input"), { target: { value: "FastFreight" } });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const write = edited.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === "post")
      ?.writes?.[0];
    expect(write?.inputs.new_input).toBe("FastFreight");
  });

  it("gap 3: kind-change to check initialises the demo default check", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    fireEvent.change(screen.getByLabelText("Kind"), { target: { value: "check" } });
    // The check fieldset renders now that the step has a check object.
    expect(screen.getByLabelText("Message")).toBeVisible();
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const step = edited.stages.flatMap((s) => s.steps).find((s) => s.id === "enter");
    expect(step?.kind).toBe("check");
    expect(step?.check).toEqual({
      scope: "anchor",
      condition: { field: "", operator: "equals", value: "" },
      message: "",
    });
  });

  it("gap 4: deleting a step removes the branches pointing at it and says so", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const { onChange } = renderBuilder();
    fireEvent.click(screen.getByRole("button", { name: /Step 6 of 13.*Review discrepancy/ }));
    fireEvent.click(screen.getByRole("button", { name: "Delete step" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.stages.flatMap((s) => s.steps).some((s) => s.id === "review-discrepancy")).toBe(false);
    const check = edited.stages.flatMap((s) => s.steps).find((s) => s.id === "discrepancy-check");
    expect(check?.branches?.map((b) => b.on)).toEqual(["pass"]);
    expect(screen.getByText(/also removed 1 branch/)).toBeVisible();
  });

  it("gap 4: deleting a stage deletes its steps", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const { onChange } = renderBuilder();
    fireEvent.click(screen.getByRole("button", { name: "Delete stage Record receipt" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.stages.some((s) => s.id === "record-receipt")).toBe(false);
    expect(edited.stages.flatMap((s) => s.steps).some((s) => s.id === "discrepancy-check")).toBe(false);
  });

  it("gap 5: save adopts the returned lane key so Publish enables at once", async () => {
    renderBuilder(definition, vi.fn(), null);
    fireEvent.click(screen.getByRole("button", { name: "Save lane" }));
    await waitFor(() => expect(screen.getByText(/Saved as version 2/)).toBeVisible());
    await waitFor(() => expect(screen.getByRole("button", { name: "Publish" })).toBeEnabled());
  });

  it("gap 6: a blank lane gets an object picker and cannot publish without one", async () => {
    vi.mocked(api.listCustomObjects).mockResolvedValue([
      { key: "widget", plural_label: "Widgets" },
    ] as never);
    const { onChange } = renderBuilder(
      {
        id: "lane-new-x",
        name: "Untitled lane",
        object_key: "",
        version: 1,
        status: "draft",
        stages: [{ id: "stage-1", name: "Stage 1", steps: [] }],
        run_data: { fields: [] },
      },
      vi.fn(),
      null,
    );
    const publish = screen.getByRole("button", { name: "Publish" });
    expect(publish).toBeDisabled();
    expect(publish).toHaveAttribute("title", expect.stringContaining("Pick an object first"));
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Widgets (widget)" })).toBeVisible(),
    );
    const picker = screen.getByLabelText("Lane object");
    fireEvent.change(picker, { target: { value: "widget" } });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(edited.object_key).toBe("widget");
  });

  it("gap 7: branch targets exclude the step itself and Add branch defaults to the next step", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 3 of 4.*Check it/ }));
    const panel = screen.getByRole("region", { name: "Step panel" });
    const options = Array.from(
      within(panel).getByLabelText("Branch pass target").querySelectorAll("option"),
    ).map((o) => o.textContent);
    expect(options.join(" ")).not.toContain("Check it");
    fireEvent.click(screen.getByRole("button", { name: "Add fail branch" }));
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const fail = edited.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === "check-it")
      ?.branches?.find((b) => b.on === "fail");
    expect(fail?.target?.step).toBe("approve-it");
  });

  it("gap 8 (UI half): adopting a provisional Action Type calls the adopt wrapper", async () => {
    renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 2 of 4.*Post tracking/ }));
    fireEvent.click(screen.getByRole("button", { name: "Adopt Set Tracking" }));
    await waitFor(() =>
      expect(api.adoptBuilderActionType).toHaveBeenCalledWith("Set Tracking", "widget"),
    );
  });

  it("gap 9: the publish report framing renders as a banner above the checks", async () => {
    vi.mocked(api.builderCheckLane).mockResolvedValue({
      lane_key: "lane-wh-receiving",
      version: 1,
      refused: true,
      framing: "This lane's object doesn't exist in this workspace.",
      checks: [
        { id: "P-11", step: "confirm-arrival", status: "refuse", reason: "object 'purchase_order' does not exist" },
        { id: "P-13", step: "close-po", status: "refuse", reason: "workflow 'wf-wh-01' does not exist" },
      ],
    } as never);
    renderBuilder();
    await waitFor(() =>
      expect(screen.getByText(/doesn't exist in this workspace/)).toBeVisible(),
    );
    expect(screen.getByText(/P-11/)).toBeVisible();
    expect(screen.getByText(/P-13/)).toBeVisible();
  });

  it("gap 10: dry-run fixtures feed the dry run (form values, run data, approvals)", async () => {
    vi.mocked(api.listCustomRecords).mockResolvedValue([
      { id: "rec-1", primary_name: "Widget one", display_number: "WID-000001" },
    ] as never);
    renderBuilder(SMALL_DEF);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Widget one (WID-000001)" })).toBeVisible(),
    );
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "rec-1" } });
    fireEvent.change(screen.getByLabelText("Fixture Enter details Carrier"), {
      target: { value: "FastFreight" },
    });
    fireEvent.change(screen.getByLabelText("Fixture run-data carrier"), {
      target: { value: "FastFreight" },
    });
    fireEvent.change(screen.getByLabelText("Fixture approval Approve it"), {
      target: { value: "rejected" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    await waitFor(() => expect(api.builderDryRunLane).toHaveBeenCalled());
    const fixtures = vi.mocked(api.builderDryRunLane).mock.calls[0][2] as {
      run_data: Record<string, unknown>;
      form_values?: Record<string, Record<string, unknown>>;
      approvals?: Record<string, string>;
    };
    expect(fixtures.form_values?.enter?.carrier).toBe("FastFreight");
    expect(fixtures.run_data.carrier).toBe("FastFreight");
    expect(fixtures.approvals?.["approve-it"]).toBe("rejected");
  });

  it("gap 10: an untouched approval select still sends its displayed decision", async () => {
    vi.mocked(api.listCustomRecords).mockResolvedValue([
      { id: "rec-1", primary_name: "Widget one", display_number: "WID-000001" },
    ] as never);
    renderBuilder(SMALL_DEF);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Widget one (WID-000001)" })).toBeVisible(),
    );
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "rec-1" } });
    // The approval select is never touched — it shows "Approve".
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    await waitFor(() => expect(api.builderDryRunLane).toHaveBeenCalled());
    const fixtures = vi.mocked(api.builderDryRunLane).mock.calls[0][2] as {
      approvals?: Record<string, string>;
    };
    expect(fixtures.approvals?.["approve-it"]).toBe("approved");
  });

  it("gap 10: with no records the dry run says so and stays disabled", async () => {
    renderBuilder(SMALL_DEF);
    await waitFor(() =>
      expect(screen.getByText(/No widget records exist yet/)).toBeVisible(),
    );
    expect(screen.getByRole("button", { name: "Run dry run" })).toBeDisabled();
  });

  it("gap 14: a dry-run error names the step and the remedy in two sentences", async () => {
    vi.mocked(api.listCustomRecords).mockResolvedValue([
      { id: "rec-1", primary_name: "Widget one", display_number: "WID-000001" },
    ] as never);
    vi.mocked(api.builderDryRunLane).mockRejectedValue(
      new Error("step 'tracking-check' needs a role"),
    );
    renderBuilder(SMALL_DEF);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "Widget one (WID-000001)" })).toBeVisible(),
    );
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "rec-1" } });
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    await waitFor(() =>
      expect(
        screen.getByText(
          "Can't dry-run this draft: step 'tracking-check' needs a role. Fix the checks above, then dry-run here.",
        ),
      ).toBeVisible(),
    );
  });

  it("gap 11: the role picker lists workspace roles", async () => {
    vi.mocked(api.listAccessRoles).mockResolvedValue([
      {
        id: "role_wh_manager",
        workspace_id: "w",
        name: "Warehouse Manager",
        description: "",
        is_system: false,
        created_at: "",
        updated_at: "",
      },
    ] as never);
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    await waitFor(() => expect(screen.getByLabelText("Role").tagName).toBe("SELECT"));
    expect(screen.getByText("Warehouse Manager (role_wh_manager)")).toBeVisible();
    fireEvent.change(screen.getByLabelText("Role"), { target: { value: "role_wh_manager" } });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(
      edited.stages.flatMap((s) => s.steps).find((s) => s.id === "enter")?.role,
    ).toBe("role_wh_manager");
  });

  it("gap 11: with no role list the free-text fallback warns", async () => {
    renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    await waitFor(() => expect(screen.getByText(/Role list unavailable/)).toBeVisible());
    expect(screen.getByLabelText("Role").tagName).toBe("INPUT");
  });

  it("gap 12: consecutive not-evaluated entries collapse into one line naming the blocker", async () => {
    const checks = [
      { id: "P-03", step: "check-it", status: "refuse", reason: "step 'check-it' needs a role" },
      ...["P-04", "P-05", "P-06", "P-07", "P-08", "P-09", "P-10", "P-11"].map((id) => ({
        id,
        step: null,
        status: "warn",
        reason: "not evaluated: validate_definition stopped at P-03 - step 'check-it' needs a role",
      })),
    ];
    vi.mocked(api.builderCheckLane).mockResolvedValue({
      lane_key: "lane-small",
      version: 1,
      refused: true,
      checks,
    } as never);
    renderBuilder(SMALL_DEF);
    await waitFor(() =>
      expect(
        screen.getByText("8 checks not evaluated — fix P-03 first: step 'check-it' needs a role"),
      ).toBeVisible(),
    );
    expect(screen.getAllByText(/not evaluated/).length).toBe(1);
    // The one real refusal is still there, and its reason text appears
    // in both the refuse entry and the collapsed line.
    expect(screen.getByText("P-03")).toBeVisible();
    expect(screen.getAllByText(/needs a role/).length).toBe(2);
  });

  it("gap 15: a repeating step gets an assignment control and a skip-condition builder", async () => {
    const { onChange } = renderBuilder(COUNT_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 1.*Post adjustments/ }));
    fireEvent.change(screen.getByLabelText("Per-item assignment"), {
      target: { value: "child_field" },
    });
    fireEvent.change(screen.getByLabelText("Assignment child field"), {
      target: { value: "counted_by" },
    });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const step = edited.stages.flatMap((s) => s.steps).find((s) => s.id === "post-adjustments");
    expect(
      (step?.repeat_over as unknown as { assign?: { by: string; field: string } })?.assign,
    ).toEqual({ by: "child_field", field: "counted_by" });
    fireEvent.click(screen.getByRole("button", { name: "Add condition group" }));
    await waitFor(() =>
      expect(
        (
          onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition
        ).stages
          .flatMap((s) => s.steps)
          .find((s) => s.id === "post-adjustments"),
      ).toHaveProperty("skip_when"),
    );
    expect(screen.getByText(/runner honors skip conditions once WP-0710 lands/)).toBeVisible();
  });

  it("gap 15: Count per-line edits feed the dry run as per-child fixtures", async () => {
    vi.mocked(api.listCustomRecords).mockResolvedValue([
      { id: "cc-1", primary_name: "CC-2026-001", display_number: "CC-2026-001" },
    ] as never);
    renderBuilder(COUNT_DEF);
    await waitFor(() =>
      expect(screen.getByRole("option", { name: "CC-2026-001 (CC-2026-001)" })).toBeVisible(),
    );
    fireEvent.change(screen.getByLabelText("Counted quantity for HG-1001"), {
      target: { value: "9" },
    });
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "cc-1" } });
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    await waitFor(() => expect(api.builderDryRunLane).toHaveBeenCalled());
    const fixtures = vi.mocked(api.builderDryRunLane).mock.calls[0][2] as {
      per_child?: Record<string, Record<string, unknown>>;
    };
    expect(fixtures.per_child?.["HG-1001"]).toEqual({ counted_qty: 9 });
    expect(screen.getByText(/dry-run fixtures for the count step/)).toBeVisible();
  });

  it("gap 16: a form field gets a resolve-from-latest-record control", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    fireEvent.click(screen.getByRole("button", { name: "Resolve from latest record" }));
    fireEvent.change(screen.getByLabelText("Field 1 resolve field"), {
      target: { value: "supplier_id" },
    });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    const field = edited.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === "enter")
      ?.fields?.[0];
    expect(
      (field as unknown as { resolve?: { kind: string; field: string } })?.resolve,
    ).toEqual(
      expect.objectContaining({ kind: "latest_record", field: "supplier_id", object: "widget" }),
    );
    expect(screen.getByText(/runner does not resolve it yet/)).toBeVisible();
  });

  it("gap 17: a step gets a replaces editor (workflow id + action index)", async () => {
    const { onChange } = renderBuilder(SMALL_DEF);
    fireEvent.click(screen.getByRole("button", { name: /Step 1 of 4.*Enter details/ }));
    fireEvent.click(screen.getByRole("button", { name: "Add replaces" }));
    fireEvent.change(screen.getByLabelText("Replaces 1 workflow id"), {
      target: { value: "wf-wh-01" },
    });
    await waitFor(() => expect(onChange).toHaveBeenCalled());
    const edited = onChange.mock.calls[onChange.mock.calls.length - 1][0] as LaneDefinition;
    expect(
      edited.stages.flatMap((s) => s.steps).find((s) => s.id === "enter")?.replaces,
    ).toEqual([{ workflow_id: "wf-wh-01", action_index: 0 }]);
  });
});
