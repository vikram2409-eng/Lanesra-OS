/**
 * The lane runner screen (WP-0707, WP-0720, REQ-LANE-014 / REQ-LANE-003).
 *
 * Renders from the mocked run view + the Receive Shipment definition, so
 * no backend is needed: the swim lanes, the current step's work area, the
 * anchor-record and written-summaries, Back and Complete step, the locked
 * other-role step (REQ-LANE-011), the approval decision with its routes
 * (WP-0720 AC-1) and the pause/resume/handoff/reopen/cancel controls
 * (WP-0720 AC-2) are all proven here. The page-level test below proves
 * the same screen loads through the real `api` module.
 *
 * Covers: REQ-LANE-014 (WP-0707 AC-1, AC-2, AC-4), REQ-LANE-003 (WP-0707 AC-1), REQ-LANE-011 (WP-0707 AC-1), REQ-LANE-012 (WP-0720 AC-1), REQ-LANE-009 (WP-0720 AC-2).
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
    resolveLaneApproval: vi.fn(),
    pauseLaneRun: vi.fn(),
    resumeLaneRun: vi.fn(),
    handoffLaneStep: vi.fn(),
    reopenLaneStep: vi.fn(),
    cancelLaneRun: vi.fn(),
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

  it("swim lanes show every stage with current and completed marked", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-2, WP-0720 AC-4)
    // WP-0720 replaced the stage bar with swim lanes (one column per
    // stage, each carrying its own steps), like the demo runner.
    const { container } = renderView(runViewStepTwo());
    const lanes = container.querySelector('[aria-label="Stages and their steps"]')!;
    const text = lanes.textContent ?? "";
    for (const name of ["Verify", "Record receipt", "Hold & review", "Stage & put away", "Confirm"]) {
      expect(text).toContain(name);
    }
    // Stage 1 is current (the run is still in Verify).
    expect(lanes.querySelector('[aria-label="Stage 1 of 5: Verify, current"]')).not.toBeNull();
  });

  it("marks a finished stage done once the run moves on", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-2, WP-0720 AC-4)
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
    const lanes = container.querySelector('[aria-label="Stages and their steps"]')!;
    expect(lanes.querySelector('[aria-label="Stage 2 of 5: Record receipt, current"]')).not.toBeNull();
    expect(lanes.querySelector('[aria-label="Stage 1 of 5: Verify, done"]')).not.toBeNull();
    expect(lanes.textContent).toContain("✓");
  });

  it("each swim lane carries its own stage's steps", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1, WP-0720 AC-4)
    renderView(runView());
    const verify = screen.getByLabelText("Stage 1 of 5: Verify, current");
    expect(verify.textContent).toContain("Confirm shipment arrived");
    expect(verify.textContent).toContain("Match lines to the delivery note");
    expect(verify.textContent).toContain("Verify the supplier");
    const receipt = screen.getByLabelText("Stage 2 of 5: Record receipt, not reached");
    expect(receipt.textContent).toContain("Enter received quantities");
    expect(receipt.textContent).toContain("Discrepancy check");
    const putaway = screen.getByLabelText("Stage 4 of 5: Stage & put away, not reached");
    expect(putaway.textContent).toContain("Stage the goods");
  });

  it("marks done, current and locked steps in the list", () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1, WP-0720 AC-4)
    renderView(runViewStepTwo());
    const verify = screen.getByLabelText("Stage 1 of 5: Verify, current");
    expect(within(verify).getByRole("button", { name: /Confirm shipment arrived/ }).textContent).toContain("✓");
    expect(within(verify).getByRole("button", { name: /Match lines to the delivery note/ }).textContent).toContain("▶");
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

  it("an approval step shows who it waits on, the routes, and Approve/Reject", async () => {
    // Covers: REQ-LANE-012 (WP-0720 AC-1)
    // WP-0720 supersedes the WP-0707 inbox state: the approval is decided
    // on the runner, with the declared way out of each decision.
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
        can_complete: true,
        kind: "approval",
        fields: [],
      },
    });
    const onResolveApproval = vi.fn();
    renderView(view, { onResolveApproval });
    expect(screen.getByText(/Waiting on/)).toBeInTheDocument();
    expect(screen.getByText(/role_wh_warehouse_manager/)).toBeInTheDocument();
    expect(screen.getByText("The decision routes the run")).toBeInTheDocument();
    expect(screen.getByText(/Approved/)).toBeInTheDocument();
    expect(screen.getByText(/Stage & put away · Stage the goods/)).toBeInTheDocument();
    expect(screen.getByText(/Rejected/)).toBeInTheDocument();
    expect(screen.getByText(/Hold & review · Record hold/)).toBeInTheDocument();
    expect(screen.getByText(/purchase_order\.set_status/)).toBeInTheDocument();
    expect(screen.queryByText(/Continue in the approvals inbox/)).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Complete step" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve" }));
    await waitFor(() => expect(onResolveApproval).toHaveBeenCalledWith(true, ""));
  });

  it("an approval for another role shows who it waits on, locked", () => {
    // Covers: REQ-LANE-012 (WP-0720 AC-1)
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
    renderView(view, { onResolveApproval: vi.fn() });
    expect(screen.getByText(/Waiting on role_wh_warehouse_manager/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Approve" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Reject/ })).toBeDisabled();
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
    // Every step in the swim lanes is a real button: reachable by Tab,
    // operable by keyboard, and the locked ones are disabled, not fake text.
    const verify = screen.getByLabelText("Stage 1 of 5: Verify, current");
    const buttons = within(verify).getAllByRole("button");
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

describe("LaneRunnerView anytime actions (WP-0711)", () => {
  // Covers: REQ-LANE-018 (WP-0711 AC-1, AC-2, AC-3)
  function anytimeView(): LaneRunView {
    return runView({
      notes: [
        {
          id: "note-1",
          run_id: "run-1",
          step_id: "confirm-arrival",
          author_user_id: "user-casey",
          author_display_name: "Casey Counter",
          body: "Checked the count against the pick list.",
          created_at: "2026-11-18T10:00:00Z",
        },
      ],
      attachments: [
        {
          id: "att-1",
          run_id: "run-1",
          file_name: "count-sheet.txt",
          mime_type: "text/plain",
          size_bytes: 2048,
          created_at: "2026-11-18T10:05:00Z",
        },
      ],
      events: [
        {
          event_type: "lane_run_note_added",
          summary: "Added a note to run run-1 (step confirm-arrival)",
          user_id: "user-casey",
          occurred_at: "2026-11-18T10:00:00Z",
        },
        {
          event_type: "lane_step_reassigned",
          summary: "Reassigned step confirm-arrival to Casey Counter",
          user_id: "user-admin",
          occurred_at: "2026-11-18T10:10:00Z",
        },
      ],
      reassign_candidates: [
        { user_id: "user-casey", display_name: "Casey Counter" },
        { user_id: "user-robin", display_name: "Robin Receiver" },
      ],
    });
  }

  it("shows notes, attached files and the audit events on the run", () => {
    // Covers: REQ-LANE-018 (WP-0711 AC-1)
    renderView(anytimeView(), { onAddNote: vi.fn(), onAttachFile: vi.fn(), onReassign: vi.fn(), onDownloadAttachment: vi.fn() });
    const card = screen.getByRole("group", { name: "Notes, attachments and reassign" });
    expect(card.textContent).toContain("Checked the count against the pick list.");
    expect(card.textContent).toContain("Casey Counter");
    expect(card.textContent).toContain("count-sheet.txt");
    const trail = screen.getByRole("group", { name: "Audit trail" });
    expect(trail.textContent).toContain("Added a note to run run-1");
    expect(trail.textContent).toContain("Reassigned step confirm-arrival to Casey Counter");
  });

  it("adding a note calls back with the entered text", async () => {
    // Covers: REQ-LANE-018 (WP-0711 AC-1)
    const onAddNote = vi.fn();
    renderView(anytimeView(), { onAddNote });
    fireEvent.change(screen.getByLabelText("Add a note"), { target: { value: "Pallet 3 is short two units." } });
    fireEvent.click(screen.getByRole("button", { name: "Add note" }));
    await waitFor(() => expect(onAddNote).toHaveBeenCalledWith("Pallet 3 is short two units."));
  });

  it("attaching a file calls back with its name, type and base64 content", async () => {
    // Covers: REQ-LANE-018 (WP-0711 AC-1)
    const onAttachFile = vi.fn();
    renderView(anytimeView(), { onAttachFile });
    const file = new File(["count sheet"], "count-sheet.txt", { type: "text/plain" });
    fireEvent.change(screen.getByLabelText("Attach a file"), { target: { files: [file] } });
    await waitFor(() =>
      expect(onAttachFile).toHaveBeenCalledWith("count-sheet.txt", "text/plain", "Y291bnQgc2hlZXQ="),
    );
  });

  it("reassign lists the role holders and calls back with the person and reason", async () => {
    // Covers: REQ-LANE-018 (WP-0711 AC-2)
    const onReassign = vi.fn();
    renderView(anytimeView(), { onReassign });
    const select = screen.getByLabelText("Reassign this step") as HTMLSelectElement;
    expect(select.textContent).toContain("Casey Counter");
    expect(select.textContent).toContain("Robin Receiver");
    fireEvent.change(select, { target: { value: "user-casey" } });
    fireEvent.change(screen.getByLabelText("Reason for reassigning"), { target: { value: "Casey is on shift" } });
    fireEvent.click(screen.getByRole("button", { name: "Reassign" }));
    await waitFor(() => expect(onReassign).toHaveBeenCalledWith("user-casey", "Casey is on shift"));
  });

  it("downloading an attachment calls back with that file", () => {
    // Covers: REQ-LANE-018 (WP-0711 AC-3)
    const onDownloadAttachment = vi.fn();
    renderView(anytimeView(), { onDownloadAttachment });
    fireEvent.click(screen.getByRole("button", { name: "Download" }));
    expect(onDownloadAttachment).toHaveBeenCalledWith(
      expect.objectContaining({ id: "att-1", file_name: "count-sheet.txt" }),
    );
  });
});

describe("LaneRunnerView second cut (WP-0720)", () => {
  // Covers: REQ-LANE-012 (WP-0720 AC-1), REQ-LANE-009 (WP-0720 AC-2), REQ-LANE-014 (WP-0720 AC-3, AC-4)
  it("shows Pause run while waiting and Resume run while paused", () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const { rerender } = renderView(runView(), { onPause: vi.fn(), onResume: vi.fn(), onCancel: vi.fn() });
    expect(screen.getByRole("button", { name: "Pause run" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Resume run" })).not.toBeInTheDocument();
    rerender(
      <LaneRunnerView
        run={runView({ status: "paused" })}
        blueprint={definition}
        recordLabel="PO-2026-001"
        recordSummary={[]}
        onSubmit={vi.fn()}
        onPause={vi.fn()}
        onResume={vi.fn()}
        onCancel={vi.fn()}
      />,
    );
    expect(screen.getByText("This run is paused")).toBeInTheDocument();
    expect(screen.getByText(/Paused at Confirm shipment arrived/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Resume run" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Pause run" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Complete step" })).not.toBeInTheDocument();
  });

  it("pausing calls back with no reason", async () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const onPause = vi.fn();
    renderView(runView(), { onPause });
    fireEvent.click(screen.getByRole("button", { name: "Pause run" }));
    await waitFor(() => expect(onPause).toHaveBeenCalledWith(null));
  });

  it("cancel asks once and lists the writes already posted", async () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const onCancel = vi.fn();
    renderView(runView({ writes: ["Posted RECEIPT move for HG-1001 × 40"] }), { onCancel });
    fireEvent.click(screen.getByRole("button", { name: "Cancel run" }));
    const prompt = screen.getByRole("group", { name: "Cancel this run?" });
    expect(prompt.textContent).toContain("Posted RECEIPT move for HG-1001 × 40");
    expect(prompt.textContent).toContain("stays in place");
    fireEvent.change(screen.getByLabelText("Reason for cancelling"), {
      target: { value: "Supplier never delivered" },
    });
    fireEvent.click(within(prompt).getByRole("button", { name: "Cancel run" }));
    await waitFor(() => expect(onCancel).toHaveBeenCalledWith("Supplier never delivered", false));
  });

  it("cancel with a waiting child offers cancel-child or keep-child", () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    renderView(
      runView({
        status: "waiting_child",
        child: { child_run_id: "run-child", lane_id: "lane-wh-supplier-check", stage: "Verify", status: "waiting_person" },
      }),
      { onCancel: vi.fn() },
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel run" }));
    expect(screen.getByRole("button", { name: "Cancel child too" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Keep child running" })).toBeInTheDocument();
  });

  it("handoff lists the lane's roles and calls back with the role and reason", async () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const onHandoff = vi.fn();
    renderView(runView(), { onHandoff });
    const card = screen.getByRole("group", { name: "Hand off this run" });
    const select = within(card).getByLabelText("Hand off to role") as HTMLSelectElement;
    expect(select.textContent).toContain("role_wh_receiver");
    expect(select.textContent).toContain("role_wh_warehouse_manager");
    fireEvent.change(select, { target: { value: "role_wh_warehouse_manager" } });
    fireEvent.change(within(card).getByLabelText("Reason for handing off"), {
      target: { value: "Manager needs to take it" },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Hand off" }));
    await waitFor(() =>
      expect(onHandoff).toHaveBeenCalledWith("role_wh_warehouse_manager", "Manager needs to take it"),
    );
  });

  it("reopen offers a completed step in the current stage, with a reason", async () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const onReopenStep = vi.fn();
    renderView(runViewStepTwo(), { onReopenStep });
    fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(screen.getByRole("heading", { name: "Confirm shipment arrived" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reopen this step" }));
    fireEvent.change(screen.getByLabelText("Why reopen?"), {
      target: { value: "Wrong carrier entered" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Confirm reopen" }));
    await waitFor(() => expect(onReopenStep).toHaveBeenCalledWith("confirm-arrival", "Wrong carrier entered"));
  });

  it("a completed step in an earlier stage cannot be reopened", () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2)
    const view = runView({
      current_stage_id: "record-receipt",
      current_stage_name: "Record receipt",
      current_step_id: "enter-quantities",
      current_step_name: "Enter received quantities",
      steps: ALL_STEP_IDS.map((id) =>
        progress(
          id,
          stageOf(id),
          ["confirm-arrival", "match-delivery-note", "verify-supplier"].includes(id)
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
    renderView(view, { onReopenStep: vi.fn() });
    fireEvent.click(screen.getByRole("button", { name: /Confirm shipment arrived/ }));
    expect(screen.getByRole("heading", { name: "Confirm shipment arrived" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Reopen this step" })).not.toBeInTheDocument();
    expect(screen.getByText(/cannot be reopened/)).toBeInTheDocument();
  });

  it("a branch stage is marked Branch and named as one to a screen reader", () => {
    // Covers: REQ-LANE-014 (WP-0720 AC-3)
    renderView(runView());
    const hold = screen.getByLabelText("Stage 3 of 5: Hold & review, not reached, branch stage");
    expect(hold.textContent).toContain("Branch");
    const verify = screen.getByLabelText("Stage 1 of 5: Verify, current");
    expect(verify.getAttribute("aria-label")).not.toContain("branch stage");
    expect(verify.textContent).not.toContain("Branch");
  });
});

describe("LaneRunnerPage second cut (WP-0720)", () => {
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
    vi.mocked(api.pauseLaneRun).mockResolvedValue({ id: "run-1" } as never);
    vi.mocked(api.resolveLaneApproval).mockResolvedValue({ id: "run-1" } as never);
  });

  it("pausing from the page calls the api and reloads the run", async () => {
    // Covers: REQ-LANE-009 (WP-0720 AC-2, AC-5)
    render(
      <Providers>
        <LaneRunnerPage runId="run-1" recordLabel="PO-2026-001" />
      </Providers>,
    );
    await screen.findByRole("button", { name: "Pause run" });
    fireEvent.click(screen.getByRole("button", { name: "Pause run" }));
    await waitFor(() => expect(api.pauseLaneRun).toHaveBeenCalledWith("run-1", null));
    await waitFor(() => {
      expect(vi.mocked(api.getLaneRunView).mock.calls.length).toBeGreaterThan(1);
    });
  });

  it("deciding an approval from the page calls the api", async () => {
    // Covers: REQ-LANE-012 (WP-0720 AC-1, AC-5)
    vi.mocked(api.getLaneRunView).mockResolvedValue(
      runView({
        status: "waiting_approval",
        current_stage_id: "hold-review",
        current_stage_name: "Hold & review",
        current_step_id: "review-discrepancy",
        current_step_name: "Review discrepancy",
        waiting: { step_id: "review-discrepancy", step_name: "Review discrepancy", kind: "approval" },
        current: {
          stage_id: "hold-review",
          stage_name: "Hold & review",
          step_id: "review-discrepancy",
          step_name: "Review discrepancy",
          role: "role_wh_warehouse_manager",
          waiting_on: "role_wh_warehouse_manager",
          can_complete: true,
          kind: "approval",
          fields: [],
        },
      }) as never,
    );
    render(
      <Providers>
        <LaneRunnerPage runId="run-1" recordLabel="PO-2026-001" />
      </Providers>,
    );
    await screen.findByRole("button", { name: "Approve" });
    fireEvent.change(screen.getByLabelText("Note with the decision (optional)"), {
      target: { value: "Quantities check out" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Approve" }));
    await waitFor(() =>
      expect(api.resolveLaneApproval).toHaveBeenCalledWith("run-1", true, "Quantities check out"),
    );
  });
});
