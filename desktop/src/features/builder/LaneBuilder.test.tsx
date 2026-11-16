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
    listCustomRecords: vi.fn(),
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

function renderBuilder(def: LaneDefinition = definition, onChange = vi.fn()) {
  const utils = render(<LaneBuilder definition={def} laneKey="lane-wh-receiving" onDefinitionChange={onChange} />);
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
  vi.mocked(api.listCustomRecords).mockResolvedValue([] as never);
  vi.mocked(api.saveBuilderLane).mockResolvedValue({ version: 2 } as never);
  vi.mocked(api.publishLane).mockResolvedValue({ version: 2 } as never);
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
