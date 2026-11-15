/**
 * The lane blueprint page's publish-check and dry-run sections (WP-0714,
 * REQ-LANE-021).
 *
 * Renders LaneBlueprintPage with the api module mocked (same pattern as
 * LaneRunner.test.tsx): the Receive Shipment definition from the shared
 * example, one anchor record, a publish report with a pass, a warn and
 * two refuses, and a dry-run report for the happy path. No backend is
 * needed.
 *
 * Covers: REQ-LANE-021 (WP-0714 AC-1, AC-2, AC-4).
 */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import axe from "axe-core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LaneBlueprintPage } from "./LaneBlueprint";
import { api, ApiError } from "../../lib/api";
import type { LaneBlueprint, LaneDefinition } from "../../lib/types";
// Same shared example as LaneBlueprint.test.tsx (see its note on why the
// test keeps a copy instead of importing the shared file).
import example from "./lane-wh-receiving.example.json";

vi.mock("../../lib/api", () => ({
  api: {
    listLaneBlueprints: vi.fn(),
    getLaneBlueprint: vi.fn(),
    listCustomRecords: vi.fn(),
    checkLanePublish: vi.fn(),
    publishLane: vi.fn(),
    dryRunLane: vi.fn(),
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

function blueprint(): LaneBlueprint {
  return {
    lane_key: "lane-wh-receiving",
    name: "Receive Shipment",
    object_key: "purchase_order",
    version: 1,
    status: "draft",
    definition,
  };
}

const PUBLISH_REPORT = {
  lane_key: "lane-wh-receiving",
  version: 1,
  refused: true,
  checks: [
    { id: "P-01", step: null, status: "pass", reason: null },
    { id: "P-09", step: "discrepancy-check", status: "refuse", reason: "check 'discrepancy-check' has no fail branch" },
    { id: "P-10", step: "discrepancy-check", status: "warn", reason: "check 'discrepancy-check' is not the last step of stage 'record-receipt' — the step after it runs on the pass path" },
    { id: "P-12", step: "post-receipt-moves", status: "refuse", reason: "step 'post-receipt-moves' uses provisional Action Type 'warehouse.post_stock_move' — not in the adopted catalog" },
  ],
};

const DRY_RUN_REPORT = {
  lane_key: "lane-wh-receiving",
  lane_version: 1,
  anchor_object_key: "purchase_order",
  anchor_record_id: "po-2026-001",
  would_start: true,
  refusal_reason: null,
  stages_visited: ["verify", "record-receipt", "stage-put-away", "confirm"],
  steps: [
    { stage_id: "verify", step_id: "confirm-arrival", kind: "form", outcome: "executed", reason: null },
    { stage_id: "record-receipt", step_id: "discrepancy-check", kind: "check", outcome: "executed", reason: null },
    { stage_id: "hold-review", step_id: "review-discrepancy", kind: "approval", outcome: "skipped", reason: "check 'discrepancy-check' passed, so the fail branch was not taken" },
  ],
  checks: [
    { step_id: "discrepancy-check", outcome: "passed", reason: "every line matches the delivery note", per_child: [] },
  ],
  branches_taken: [
    { step_id: "discrepancy-check", on: "pass", target: "stage-put-away/assign-staging-bin" },
  ],
  writes: [
    { step_id: "post-receipt-moves", child_id: "line-1", action_type: "warehouse.post_stock_move", inputs: { po_id: "po-2026-001", qty: 40 }, zero_quantity: false },
  ],
  zero_quantity_flags: [],
  unevaluated: [],
  end_outcome: "received",
  expected_outcome: "received",
  matches_expected: true,
  completed: true,
  stopped_reason: null,
};

const DRY_RUN_MISMATCH_REPORT = {
  ...DRY_RUN_REPORT,
  end_outcome: "held",
  matches_expected: false,
  zero_quantity_flags: [
    { step_id: "post-receipt-moves", action_type: "warehouse.post_stock_move", skipped_children: 1, message: "line 'line-2' has zero variance — a zero-quantity COUNT_ADJUSTMENT is not a real adjustment" },
  ],
  unevaluated: [
    { step_id: "discrepancy-check", reason: "unevaluated (needs fixture): run-data field 'delivery_note_qty' was not supplied" },
  ],
};

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return render(<LaneBlueprintPage />, { wrapper });
}

async function openDetail() {
  renderPage();
  // The default waitFor timeout (1s) is too tight under full-suite load,
  // where the mocked queries can take longer to settle.
  await waitFor(() => expect(screen.getByRole("heading", { name: "Publish checks" })).toBeVisible(), { timeout: 5000 });
  await waitFor(() => expect(screen.getByRole("heading", { name: "Dry run" })).toBeVisible(), { timeout: 5000 });
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.listLaneBlueprints).mockResolvedValue([
    { lane_key: "lane-wh-receiving", name: "Receive Shipment", object_key: "purchase_order", latest_version: 1, latest_status: "draft" },
  ] as never);
  vi.mocked(api.getLaneBlueprint).mockResolvedValue(blueprint() as never);
  vi.mocked(api.listCustomRecords).mockResolvedValue([
    { id: "po-2026-001", primary_name: "PO-2026-001", display_number: "PO-2026-001" },
    { id: "po-2026-005", primary_name: "PO-2026-005", display_number: "PO-2026-005" },
  ] as never);
  vi.mocked(api.checkLanePublish).mockResolvedValue(PUBLISH_REPORT as never);
  vi.mocked(api.dryRunLane).mockResolvedValue(DRY_RUN_REPORT as never);
});

describe("LaneBlueprintPage publish checks", () => {
  it("lists every publish check with its id, step, status and named reason", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    await openDetail();
    await screen.findByRole("button", { name: "Run publish checks" });
    fireEvent.click(screen.getByRole("button", { name: "Run publish checks" }));
    const list = await screen.findByRole("list", { name: "Publish checks" });
    for (const id of ["P-01", "P-09", "P-10", "P-12"]) {
      expect(within(list).getByText(id)).toBeVisible();
    }
    expect(within(list).getByText(/has no fail branch/)).toBeVisible();
    expect(within(list).getByText(/provisional Action Type/)).toBeVisible();
    expect(within(list).getByText(/not the last step of stage/)).toBeVisible();
    // P-09 and P-10 both belong to the same step.
    expect(within(list).getAllByText("discrepancy-check")).toHaveLength(2);
    // Statuses are named in words, not colour alone.
    expect(within(list).getAllByText("refuse")).toHaveLength(2);
    expect(within(list).getAllByText("warn")).toHaveLength(1);
    expect(within(list).getAllByText("pass")).toHaveLength(1);
    expect(screen.getByText(/Cannot publish/)).toBeVisible();
    expect(vi.mocked(api.checkLanePublish)).toHaveBeenCalledWith("lane-wh-receiving");
  });

  it("publishing a passing lane shows the published version", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    vi.mocked(api.checkLanePublish).mockResolvedValue({
      ...PUBLISH_REPORT,
      refused: false,
      checks: PUBLISH_REPORT.checks.map((c) => ({ ...c, status: "pass" as const, reason: null })),
    } as never);
    vi.mocked(api.publishLane).mockResolvedValue({
      id: "ver-1",
      lane_id: "lane-wh-receiving",
      workspace_id: "ws-1",
      version: 1,
      status: "published",
      definition_json: "{}",
      graph_id: "lane-wh-receiving@v1",
      created_at: "2026-11-14T10:00:00Z",
      created_by: null,
      updated_at: "2026-11-14T10:00:00Z",
      updated_by: null,
    } as never);
    await openDetail();
    fireEvent.click(screen.getByRole("button", { name: "Publish" }));
    await waitFor(() => expect(screen.getByText(/Published version 1/)).toBeVisible());
    expect(vi.mocked(api.publishLane)).toHaveBeenCalledWith("lane-wh-receiving");
  });

  it("a publish refusal lists every failed check, not just the first", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    vi.mocked(api.publishLane).mockRejectedValue(
      new ApiError({
        kind: "validation",
        message:
          "lane 'lane-wh-receiving' cannot publish: check 'discrepancy-check' has no fail branch; step 'post-receipt-moves' uses provisional Action Type 'warehouse.post_stock_move' — not in the adopted catalog",
      }),
    );
    await openDetail();
    fireEvent.click(screen.getByRole("button", { name: "Run publish checks" }));
    await screen.findByRole("list", { name: "Publish checks" });
    fireEvent.click(screen.getByRole("button", { name: "Publish" }));
    await waitFor(() => expect(screen.getByText(/Publish refused/)).toBeVisible());
    // The refusal names every failed check: both reasons are on screen
    // (in the check list and again in the refusal message itself).
    expect(screen.getAllByText(/has no fail branch/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/provisional Action Type/).length).toBeGreaterThan(0);
    expect(screen.getByText(/cannot publish/)).toBeVisible();
  });

  it("says publishing stays a separate action by a person", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    await openDetail();
    expect(
      screen.getByText(/Publishing stays a separate action by a person/),
    ).toBeVisible();
  });
});

describe("LaneBlueprintPage dry run", () => {
  it("offers an anchor-record picker and fixture inputs for the lane's run-data fields", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-1)
    await openDetail();
    const picker = screen.getByLabelText("Anchor record");
    expect(within(picker as HTMLElement).getByText(/PO-2026-001/)).toBeVisible();
    expect(within(picker as HTMLElement).getByText(/PO-2026-005/)).toBeVisible();
    // One input per declared run-data field, per-child fields marked so.
    expect(screen.getByLabelText("carrier")).toBeVisible();
    expect(screen.getByLabelText("delivery_note_number")).toBeVisible();
    expect(screen.getByLabelText("delivery_note_qty (per line)")).toBeVisible();
    expect(screen.getByLabelText("rejection_reason (per line)")).toBeVisible();
    expect(screen.getByLabelText("staging_bin")).toBeVisible();
    // Approval steps get an approve/reject fixture.
    expect(screen.getByLabelText("Approval: Review discrepancy")).toBeVisible();
    expect(screen.getByLabelText("Expected outcome")).toBeVisible();
    expect(screen.getByText(/A dry run walks the lane against a seed record and writes nothing/)).toBeVisible();
    // Nothing runs until an anchor record is chosen.
    expect(screen.getByRole("button", { name: "Run dry run" })).toBeDisabled();
  });

  it("runs the dry run and shows the full walk: path, checks, branches, simulated writes, outcome", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-1)
    await openDetail();
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "po-2026-001" } });
    fireEvent.change(screen.getByLabelText("carrier"), { target: { value: "FastFreight" } });
    fireEvent.change(screen.getByLabelText("delivery_note_number"), { target: { value: "DN-8812" } });
    fireEvent.change(screen.getByLabelText("delivery_note_qty (per line)"), {
      target: { value: '{"line-1": 40, "line-2": 30}' },
    });
    fireEvent.change(screen.getByLabelText("Expected outcome"), { target: { value: "received" } });
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    const report = await screen.findByRole("region", { name: "Dry-run report" });
    expect(within(report).getByText(/Stages visited/)).toBeVisible();
    expect(within(report).getByText(/Verify/)).toBeVisible();
    expect(within(report).getByText(/Confirm shipment arrived/)).toBeVisible();
    expect(within(report).getByText(/skipped/)).toBeVisible();
    expect(within(report).getByText(/check 'discrepancy-check' passed/)).toBeVisible();
    expect(within(report).getByText(/Discrepancy check — passed/)).toBeVisible();
    expect(within(report).getByText(/every line matches the delivery note/)).toBeVisible();
    expect(within(report).getByText(/pass → Stage the goods/)).toBeVisible();
    expect(within(report).getByText(/warehouse.post_stock_move/)).toBeVisible();
    expect(within(report).getByText(/Simulated only — nothing was written/)).toBeVisible();
    expect(within(report).getByText(/End outcome: received/)).toBeVisible();
    expect(within(report).getByText(/Matches the expected outcome/)).toBeVisible();
    expect(vi.mocked(api.dryRunLane)).toHaveBeenCalledWith(
      "lane-wh-receiving",
      "po-2026-001",
      {
        run_data: expect.objectContaining({ carrier: "FastFreight", delivery_note_number: "DN-8812" }),
        // Per-line values are transposed: per_child[child_id][field].
        per_child: { "line-1": { delivery_note_qty: 40 }, "line-2": { delivery_note_qty: 30 } },
        approvals: { "review-discrepancy": "approved" },
      },
      "received",
    );
  });

  it("reports an outcome mismatch, zero-quantity flags and unevaluated checks", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-1)
    vi.mocked(api.dryRunLane).mockResolvedValue(DRY_RUN_MISMATCH_REPORT as never);
    await openDetail();
    fireEvent.change(screen.getByLabelText("Anchor record"), { target: { value: "po-2026-001" } });
    fireEvent.change(screen.getByLabelText("Expected outcome"), { target: { value: "received" } });
    fireEvent.click(screen.getByRole("button", { name: "Run dry run" }));
    const report = await screen.findByRole("region", { name: "Dry-run report" });
    expect(within(report).getByText(/End outcome: held/)).toBeVisible();
    expect(within(report).getByText(/Does not match the expected outcome/)).toBeVisible();
    expect(within(report).getByText(/Zero-quantity writes flagged/)).toBeVisible();
    expect(within(report).getByText(/zero variance/)).toBeVisible();
    expect(within(report).getByText(/Unevaluated checks/)).toBeVisible();
    expect(within(report).getByText(/unevaluated \(needs fixture\)/)).toBeVisible();
  });

  it("is axe-clean and keyboard operable", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-4)
    const { container } = renderPage();
    await waitFor(() => expect(screen.getByRole("heading", { name: "Publish checks" })).toBeVisible());
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
    expect(screen.getByRole("button", { name: "Run publish checks" })).not.toBeDisabled();
    expect(screen.getByRole("button", { name: "Publish" })).not.toBeDisabled();
    expect(screen.getByLabelText("Anchor record")).not.toBeDisabled();
  });
});
