/**
 * The lane runner screen (WP-0707, REQ-LANE-014 / REQ-LANE-003).
 *
 * Renders from the mocked run view + the Receive Shipment definition, so
 * no backend is needed: the stage bar, the current stage's step list, the
 * current step's work area, the anchor-record and written-summaries, Back
 * and Complete step, the locked other-role step (REQ-LANE-011) and the
 * approvals-inbox state (AC-2) are all proven here. The page-level test
 * below proves the same screen loads through the real `api` module.
 *
 * Covers: REQ-LANE-014 (WP-0707 AC-1, AC-2, AC-4), REQ-LANE-003 (WP-0707 AC-1), REQ-LANE-011 (WP-0707 AC-1).
 */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import axe from "axe-core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LaneRunnerPage, LaneRunnerView } from "./LaneRunner";
import { api } from "../../lib/api";
import type { LaneDefinition, LaneRunView, LaneStepProgress } from "../../lib/types";
// Same shared example as LaneBlueprint.test.tsx (see its note on why the
// test keeps a copy instead of importing the shared file).
import example from "./lane-wh-receiving.example.json";

vi.mock("../../lib/api", () => ({
  api: {
    getLaneRunView: vi.fn(),
    getLaneBlueprint: vi.fn(),
    getCustomRecord: vi.fn(),
    listLaneRunsForRecord: vi.fn(),
    submitLaneStep: vi.fn(),
    skipLaneStep: vi.fn(),
  },
}));

const definition = example.lane as unknown as LaneDefinition;
const ALL_STEP_IDS = definition.stages.flatMap((s) => s.steps.map((st) => st.id));

function progress(
  stepId: string,
  stageId: string,
  status: LaneStepProgress["status"],
  startedAt: string | null = null,
  finishedAt: string | null = null,
): LaneStepProgress {
  return { step_id: stepId, stage_id: stageId, status, started_at: startedAt, finished_at: finishedAt };
}

function stageOf(stepId: string): string {
  return definition.stages.find((s) => s.steps.some((st) => st.id === stepId))!.id;
}

function runView(overrides: Partial<LaneRunView> = {}): LaneRunView {
  return {
    run_id: "run-1",
    status: "waiting_person",
    anchor_object_key: "purchase_order",
    anchor_record_id: "po-1",
    lane_id: "lane-wh-receiving",
    lane_key: "lane-wh-receiving",
    lane_name: "Receive Shipment",
    current_stage_id: "verify",
    current_stage_name: "Verify",
    current_step_id: "confirm-arrival",
    current_step_name: "Confirm shipment arrived",
    run_data: {},
    steps: ALL_STEP_IDS.map((id) =>
      progress(id, stageOf(id), id === "confirm-arrival" ? "current" : "locked"),
    ),
    writes: [],
    waiting: { step_id: "confirm-arrival", step_name: "Confirm shipment arrived", kind: "person" },
    current: {
      stage_id: "verify",
      stage_name: "Verify",
      step_id: "confirm-arrival",
      step_name: "Confirm shipment arrived",
      role: "role_wh_receiver",
      waiting_on: "role_wh_receiver",
      can_complete: true,
      kind: "form",
      fields: [
        {
          key: "received_date",
          label: "Received date",
          type: "date",
          binds_to: "run_data.received_date",
          required: true,
          value: null,
          readable: true,
        },
        {
          key: "carrier",
          label: "Carrier",
          type: "text",
          binds_to: "run_data.carrier",
          required: false,
          value: "Kiln & Clay Co.",
          readable: true,
        },
      ],
    },
    ...overrides,
  };
}

/** Second step current, first step completed — the Back-review fixture. */
function runViewStepTwo(): LaneRunView {
  const view = runView({
    current_stage_id: "verify",
    current_stage_name: "Verify",
    current_step_id: "match-delivery-note",
    current_step_name: "Match lines to the delivery note",
    steps: ALL_STEP_IDS.map((id) =>
      progress(
        id,
        stageOf(id),
        id === "confirm-arrival" ? "completed" : id === "match-delivery-note" ? "current" : "locked",
        id === "confirm-arrival" ? "2026-11-14T10:00:00Z" : null,
        id === "confirm-arrival" ? "2026-11-14T10:05:00Z" : null,
      ),
    ),
    waiting: { step_id: "match-delivery-note", step_name: "Match lines to the delivery note", kind: "person" },
    current: {
      stage_id: "verify",
      stage_name: "Verify",
      step_id: "match-delivery-note",
      step_name: "Match lines to the delivery note",
      role: "role_wh_receiver",
      waiting_on: "role_wh_receiver",
      can_complete: true,
      kind: "form",
      fields: [
        {
          key: "delivery_note_qty",
          label: "Delivery note quantity",
          type: "number",
          binds_to: "run_data.delivery_note_qty",
          required: true,
          value: null,
          readable: true,
        },
      ],
    },
  });
  return view;
}

function renderView(view: LaneRunView, props: Partial<Parameters<typeof LaneRunnerView>[0]> = {}) {
  const onSubmit = vi.fn();
  const onSkip = vi.fn();
  const utils = render(
    <LaneRunnerView
      run={view}
      blueprint={definition}
      recordLabel="PO-2026-001"
      recordSummary={[
        { label: "Status", value: "OPEN" },
        { label: "Total", value: "$2,105.00" },
      ]}
      onSubmit={onSubmit}
      onSkip={onSkip}
      {...props}
    />,
  );
  return { onSubmit, onSkip, ...utils };
}

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("LaneRunnerView", () => {
  it("labels the run by record, lane and stage (REQ-LANE-003)", () => {
    // Covers: REQ-LANE-003 (WP-0707 AC-1)
    renderView(runView());
    expect(screen.getByRole("heading", { name: "PO-2026-001 · Receive Shipment · stage 1 of 5" })).toBeInTheDocument();
  });

  it("stage bar shows every stage with current and completed marked", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-2)
    const { container } = renderView(runViewStepTwo());
    const bar = container.querySelector('[aria-label="Stage progress"]')!;
    const text = bar.textContent ?? "";
    for (const name of ["Verify", "Record receipt", "Hold & review", "Stage & put away", "Confirm"]) {
      expect(text).toContain(name);
    }
    // Stage 1 is current (the run is still in Verify).
    expect(bar.querySelector('[aria-current="step"]')?.textContent).toContain("Verify");
  });

  it("marks a finished stage done once the run moves on", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-2)
    const view = runView({
      current_stage_id: "record-receipt",
      current_stage_name: "Record receipt",
      current_step_id: "enter-quantities",
      current_step_name: "Enter received quantities",
      steps: ALL_STEP_IDS.map((id) =>
        progress(
          id,
          stageOf(id),
          ["confirm-arrival", "match-delivery-note"].includes(id)
            ? "completed"
            : id === "enter-quantities"
              ? "current"
              : "locked",
        ),
      ),
      current: {
        stage_id: "record-receipt",
        stage_name: "Record receipt",
        step_id: "enter-quantities",
        step_name: "Enter received quantities",
        role: "role_wh_receiver",
        waiting_on: "role_wh_receiver",
        can_complete: true,
        kind: "form",
        fields: [],
      },
    });
    const { container } = renderView(view);
    const bar = container.querySelector('[aria-label="Stage progress"]')!;
    expect(bar.querySelector('[aria-current="step"]')?.textContent).toContain("Record receipt");
    expect(bar.textContent).toContain("✓");
  });

  it("step list shows the current stage's steps only", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    renderView(runView());
    const list = screen.getByRole("group", { name: "Steps — Verify" });
    expect(list.textContent).toContain("Confirm shipment arrived");
    expect(list.textContent).toContain("Match lines to the delivery note");
    expect(list.textContent).not.toContain("Enter received quantities");
    expect(list.textContent).not.toContain("Stage the goods");
  });

  it("marks done, current and locked steps in the list", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    renderView(runViewStepTwo());
    const list = screen.getByRole("group", { name: "Steps — Verify" });
    expect(within(list).getByRole("button", { name: /Confirm shipment arrived/ }).textContent).toContain("✓");
    expect(within(list).getByRole("button", { name: /Match lines to the delivery note/ }).textContent).toContain("▶");
  });

  it("form step renders its fields and completes with the entered values", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { onSubmit } = renderView(runView());
    fireEvent.change(screen.getByLabelText("Received date *"), { target: { value: "2026-11-14" } });
    fireEvent.change(screen.getByLabelText("Carrier"), { target: { value: "Fast Freight" } });
    fireEvent.click(screen.getByRole("button", { name: "Complete step" }));
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({ received_date: "2026-11-14", carrier: "Fast Freight" });
    });
  });

  it("a required field blocks completion with its reason", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { onSubmit } = renderView(runView());
    fireEvent.click(screen.getByRole("button", { name: "Complete step" }));
    expect(onSubmit).not.toHaveBeenCalled();
    expect(screen.getByText(/Received date is required/)).toBeInTheDocument();
  });

  it("a step for another role shows who it waits on, locked, Complete disabled", () => {
    // Covers: REQ-LANE-011 (WP-0707 AC-1)
    const view = runView({
      current: {
        stage_id: "verify",
        stage_name: "Verify",
        step_id: "confirm-arrival",
        step_name: "Confirm shipment arrived",
        role: "role_wh_warehouse_manager",
        waiting_on: "role_wh_warehouse_manager",
        can_complete: false,
        kind: "form",
        fields: [],
      },
    });
    renderView(view);
    expect(screen.getByText(/Waiting on role_wh_warehouse_manager/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Complete step" })).toBeDisabled();
  });

  it("a record-bound field the viewer cannot read shows value None", () => {
    // Covers: REQ-LANE-011 (WP-0707 AC-1)
    const view = runView({
      current: {
        stage_id: "verify",
        stage_name: "Verify",
        step_id: "confirm-arrival",
        step_name: "Confirm shipment arrived",
        role: null,
        waiting_on: null,
        can_complete: true,
        kind: "form",
        fields: [
          {
            key: "supplier_cost",
            label: "Supplier cost",
            type: "number",
            binds_to: "record.total",
            required: false,
            value: null,
            readable: false,
          },
        ],
      },
    });
    renderView(view);
    expect(screen.getByLabelText("Supplier cost")).toHaveTextContent("None");
  });

  it("record step lists its writes and posts with 'Post and complete step'", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const view = runView({
      current_stage_id: "stage-put-away",
      current_stage_name: "Stage & put away",
      current_step_id: "post-receipt-moves",
      current_step_name: "Post receipt moves",
      steps: ALL_STEP_IDS.map((id) =>
        progress(id, stageOf(id), id === "post-receipt-moves" ? "current" : "locked"),
      ),
      current: {
        stage_id: "stage-put-away",
        stage_name: "Stage & put away",
        step_id: "post-receipt-moves",
        step_name: "Post receipt moves",
        role: "role_wh_receiver",
        waiting_on: "role_wh_receiver",
        can_complete: true,
        kind: "record",
        writes: [{ action_type: "warehouse.post_stock_move", inputs: { move_type: "RECEIPT" } }],
        fields: [],
      },
    });
    const { onSubmit } = renderView(view);
    expect(screen.getByText("warehouse.post_stock_move")).toBeInTheDocument();
    expect(screen.getByText(/\{"move_type":"RECEIPT"\}/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Post and complete step" }));
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({});
    });
  });

  it("check step shows pass/fail and its reason, with no Complete button", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const view = runView({
      current_stage_id: "record-receipt",
      current_stage_name: "Record receipt",
      current_step_id: "discrepancy-check",
      current_step_name: "Discrepancy check",
      steps: ALL_STEP_IDS.map((id) =>
        progress(id, stageOf(id), id === "discrepancy-check" ? "current" : "locked"),
      ),
      current: {
        stage_id: "record-receipt",
        stage_name: "Record receipt",
        step_id: "discrepancy-check",
        step_name: "Discrepancy check",
        role: null,
        waiting_on: null,
        can_complete: true,
        kind: "check",
        check_passed: false,
        check_reason: "A line is short or has rejected units.",
        fields: [],
      },
    });
    renderView(view);
    expect(screen.getByText(/Failed/)).toBeInTheDocument();
    expect(screen.getByText(/A line is short or has rejected units/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Complete step" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Post and complete step" })).not.toBeInTheDocument();
  });

  it("an approval step shows the inbox state and no approval UI", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-2)
    const view = runView({
      status: "waiting_approval",
      current_stage_id: "hold-review",
      current_stage_name: "Hold & review",
      current_step_id: "review-discrepancy",
      current_step_name: "Review discrepancy",
      steps: ALL_STEP_IDS.map((id) =>
        progress(id, stageOf(id), id === "review-discrepancy" ? "waiting" : "locked"),
      ),
      waiting: { step_id: "review-discrepancy", step_name: "Review discrepancy", kind: "approval" },
      current: {
        stage_id: "hold-review",
        stage_name: "Hold & review",
        step_id: "review-discrepancy",
        step_name: "Review discrepancy",
        role: "role_wh_warehouse_manager",
        waiting_on: "role_wh_warehouse_manager",
        can_complete: false,
        kind: "approval",
        fields: [],
      },
    });
    renderView(view);
    expect(screen.getByText(/Continue in the approvals inbox/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Approve/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Reject/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Complete step" })).not.toBeInTheDocument();
  });

  it("Back reviews a finished step without changing the run", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { onSubmit } = renderView(runViewStepTwo());
    fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(screen.getByRole("heading", { name: "Confirm shipment arrived" })).toBeInTheDocument();
    expect(screen.getByText(/Back reviews a finished step without changing the run/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Complete step" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Back to current step" }));
    expect(screen.getByRole("button", { name: "Complete step" })).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("shows the anchor-record summary, what the run has written, and the audit trail", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const view = runView({
      writes: ["Posted RECEIPT move for HG-1001 × 40"],
      steps: ALL_STEP_IDS.map((id) =>
        progress(
          id,
          stageOf(id),
          id === "confirm-arrival" ? "completed" : id === "match-delivery-note" ? "current" : "locked",
          id === "confirm-arrival" ? "2026-11-14T10:00:00Z" : null,
          id === "confirm-arrival" ? "2026-11-14T10:05:00Z" : null,
        ),
      ),
    });
    renderView(view);
    expect(screen.getByText("PO-2026-001")).toBeInTheDocument();
    expect(screen.getByText("Status")).toBeInTheDocument();
    expect(screen.getByText("OPEN")).toBeInTheDocument();
    expect(screen.getByText("What this run has written")).toBeInTheDocument();
    expect(screen.getByText("Posted RECEIPT move for HG-1001 × 40")).toBeInTheDocument();
    const audit = screen.getByRole("group", { name: "Audit trail" });
    expect(audit.textContent).toContain("Confirm shipment arrived");
    expect(audit.textContent).toContain("completed");
  });

  it("an optional step offers Skip with a reason", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const view = runView({
      current: {
        stage_id: "verify",
        stage_name: "Verify",
        step_id: "confirm-arrival",
        step_name: "Confirm shipment arrived",
        role: "role_wh_receiver",
        waiting_on: "role_wh_receiver",
        can_complete: true,
        kind: "form",
        required: false,
        fields: [],
      },
    });
    const { onSkip } = renderView(view);
    fireEvent.click(screen.getByRole("button", { name: "Skip step" }));
    fireEvent.change(screen.getByLabelText("Reason for skipping"), {
      target: { value: "Already confirmed by phone" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Confirm skip" }));
    await waitFor(() => {
      expect(onSkip).toHaveBeenCalledWith("Already confirmed by phone");
    });
  });

  it("is axe-clean and keyboard operable", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-4)
    const { container } = renderView(runViewStepTwo());
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
    // Every step in the list is a real button: reachable by Tab, operable
    // by keyboard, and the locked ones are disabled, not fake text.
    const list = screen.getByRole("group", { name: "Steps — Verify" });
    const buttons = within(list).getAllByRole("button");
    // Three steps in Verify since WP-0716 AC-5 added the verify-supplier
    // child step (was two when this test was written).
    expect(buttons.length).toBe(3);
    expect(buttons[0]).not.toBeDisabled();
    expect(screen.getByRole("button", { name: "Complete step" })).not.toBeDisabled();
  });
});

describe("LaneRunnerPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.getLaneRunView).mockResolvedValue(runView() as never);
    vi.mocked(api.getLaneBlueprint).mockResolvedValue({
      lane_key: "lane-wh-receiving",
      name: "Receive Shipment",
      object_key: "purchase_order",
      version: 1,
      status: "published",
      definition,
    } as never);
    vi.mocked(api.getCustomRecord).mockResolvedValue({
      id: "po-1",
      primary_name: "PO-2026-001",
      display_number: "PO-2026-001",
      status: "OPEN",
    } as never);
    vi.mocked(api.listLaneRunsForRecord).mockResolvedValue([
      {
        run_id: "run-1",
        lane_id: "lane-wh-receiving",
        lane_name: "Receive Shipment",
        status: "waiting_person",
        current_stage_id: "verify",
        current_stage_name: "Verify",
        current_step_id: "confirm-arrival",
        current_step_name: "Confirm shipment arrived",
        started_at: "2026-11-14T10:00:00Z",
        finished_at: null,
      },
    ] as never);
    vi.mocked(api.submitLaneStep).mockResolvedValue({ id: "run-1" } as never);
  });

  it("loads the run view and blueprint and renders the runner", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    render(
      <Providers>
        <LaneRunnerPage runId="run-1" recordLabel="PO-2026-001" />
      </Providers>,
    );
    expect(await screen.findByRole("heading", { name: "PO-2026-001 · Receive Shipment · stage 1 of 5" })).toBeInTheDocument();
    expect(api.getLaneRunView).toHaveBeenCalledWith("run-1");
    expect(api.getLaneBlueprint).toHaveBeenCalledWith("lane-wh-receiving");
  });

  it("resolves the blueprint through the record's runs when the view names no lane", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    // The landed run view carries steps/writes/waiting but no lane key;
    // the page finds this run in the anchor record's run list, whose
    // lane_id IS the blueprint key (definition id == lane key).
    const { lane_key: _laneKey, lane_id: _laneId, lane_name: _laneName, ...viewWithoutLane } = runView();
    void _laneKey;
    void _laneId;
    void _laneName;
    vi.mocked(api.getLaneRunView).mockResolvedValue(viewWithoutLane as never);
    render(
      <Providers>
        <LaneRunnerPage runId="run-1" recordLabel="PO-2026-001" />
      </Providers>,
    );
    expect(await screen.findByRole("heading", { name: "PO-2026-001 · Receive Shipment · stage 1 of 5" })).toBeInTheDocument();
    expect(api.listLaneRunsForRecord).toHaveBeenCalledWith("purchase_order", "po-1");
    expect(api.getLaneBlueprint).toHaveBeenCalledWith("lane-wh-receiving");
  });

  it("submitting a step calls the api and reloads the run", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    render(
      <Providers>
        <LaneRunnerPage runId="run-1" recordLabel="PO-2026-001" />
      </Providers>,
    );
    await screen.findByRole("button", { name: "Complete step" });
    fireEvent.change(screen.getByLabelText("Received date *"), { target: { value: "2026-11-14" } });
    fireEvent.click(screen.getByRole("button", { name: "Complete step" }));
    await waitFor(() => {
      expect(api.submitLaneStep).toHaveBeenCalledWith("run-1", {
        received_date: "2026-11-14",
        carrier: "Kiln & Clay Co.",
      });
    });
    await waitFor(() => {
      expect(vi.mocked(api.getLaneRunView).mock.calls.length).toBeGreaterThan(1);
    });
  });

  it("reports a load failure instead of rendering an empty runner", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    vi.mocked(api.getLaneRunView).mockRejectedValue(new Error("Run not found"));
    render(
      <Providers>
        <LaneRunnerPage runId="run-9" recordLabel="PO-2026-009" />
      </Providers>,
    );
    expect(await screen.findByText(/Could not load this run/)).toBeInTheDocument();
  });
});
