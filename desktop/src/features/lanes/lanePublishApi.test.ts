/**
 * Lane publish-check and dry-run API wrappers (WP-0714).
 *
 * Same dual-transport pinning as laneRunApi.test.ts: jsdom has no Tauri
 * bridge, so these tests pin the HTTP transport - the command name, the
 * HTTP path and the request body. The Tauri command names are the same
 * strings, and the server dispatch arms mirror the Tauri commands 1:1.
 *
 * The wrappers are written against the WP-0714 API contract, verified
 * against the Rust shapes in
 * `core::services::process_lane_publish` (PublishReport,
 * PublishCheckEntry, DryRunReport et al): `check_process_lane_publish`
 * returns one entry per P-01..P-16 ({id, step, status pass/refuse/warn,
 * reason: string | null}) with `refused` and a nullable `version`;
 * `dry_run_process_lane` takes (lane_key, anchor_record_id,
 * fixtures {run_data, per_child[child_id][field], approvals[step_id]:
 * "approved" | "rejected"}, expected_outcome?) and returns the walk
 * report (steps/checks/branches/writes carry ids, not names; a branch
 * target is "stage/step" or "end:<outcome>"); `publish_process_lane`
 * is the existing WP-0703 command, whose refusal lists every failed
 * check.
 *
 * Covers: REQ-LANE-021 (WP-0714 AC-1, AC-2, AC-4).
 */
import { afterEach, describe, expect, it, vi } from "vitest";

import { api, ApiError } from "../../lib/api";

type FetchCall = { url: string; init: RequestInit | undefined };

function mockFetch() {
  const calls: FetchCall[] = [];
  const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
    calls.push({ url, init });
    return {
      json: async () => ({ ok: true, data: fetchMock.nextData }),
    } as Response;
  }) as unknown as typeof fetch & { nextData: unknown };
  fetchMock.nextData = null;
  vi.stubGlobal("fetch", fetchMock);
  return { calls, fetchMock };
}

function bodyOf(call: FetchCall): Record<string, unknown> {
  return JSON.parse(String(call.init?.body ?? "{}")) as Record<string, unknown>;
}

const PUBLISH_REPORT = {
  lane_key: "lane-wh-receiving",
  version: 2,
  refused: true,
  checks: [
    { id: "P-01", step: null, status: "pass", reason: null },
    { id: "P-09", step: "discrepancy-check", status: "refuse", reason: "check 'discrepancy-check' has no fail branch" },
    { id: "P-10", step: "discrepancy-check", status: "warn", reason: "check 'discrepancy-check' is not the last step of stage 'record-receipt' — the step after it runs on the pass path; move the check to the end of the stage, or confirm the next step is the pass continuation" },
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

describe("lane publish-check and dry-run api wrappers (HTTP transport)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("checkLanePublish reads through the sync check_process_lane_publish command", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = PUBLISH_REPORT;
    const report = await api.checkLanePublish("lane-wh-receiving");
    expect(report).toEqual(PUBLISH_REPORT);
    expect(report.refused).toBe(true);
    expect(report.checks).toHaveLength(4);
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/invoke/check_process_lane_publish");
    expect(bodyOf(calls[0])).toEqual({ laneKey: "lane-wh-receiving" });
  });

  it("publishLane calls the existing publish_process_lane command", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = {
      id: "ver-2",
      lane_id: "lane-wh-receiving",
      workspace_id: "ws-1",
      version: 2,
      status: "published",
      definition_json: "{}",
      graph_id: "lane-wh-receiving@v2",
      created_at: "2026-11-14T10:00:00Z",
      created_by: null,
      updated_at: "2026-11-14T10:00:00Z",
      updated_by: null,
    };
    const version = await api.publishLane("lane-wh-receiving");
    expect(version.status).toBe("published");
    expect(version.version).toBe(2);
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/invoke/publish_process_lane");
    expect(bodyOf(calls[0])).toEqual({ laneKey: "lane-wh-receiving" });
  });

  it("dryRunLane posts lane, anchor record, fixtures and expected outcome", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = DRY_RUN_REPORT;
    const fixtures = {
      run_data: { carrier: "FastFreight", delivery_note_number: "DN-8812" },
      per_child: { "line-1": { delivery_note_qty: 40 }, "line-2": { delivery_note_qty: 30 } },
      approvals: { "review-discrepancy": "approved" as const },
    };
    const report = await api.dryRunLane("lane-wh-receiving", "po-2026-001", fixtures, "received");
    expect(report).toEqual(DRY_RUN_REPORT);
    expect(report.end_outcome).toBe("received");
    expect(report.matches_expected).toBe(true);
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/invoke/dry_run_process_lane");
    expect(bodyOf(calls[0])).toEqual({
      laneKey: "lane-wh-receiving",
      anchorRecordId: "po-2026-001",
      fixtures,
      expectedOutcome: "received",
    });
  });

  it("dryRunLane sends a null expected outcome when the author names none", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = { ...DRY_RUN_REPORT, expected_outcome: null, matches_expected: true };
    await api.dryRunLane("lane-wh-receiving", "po-2026-001", { run_data: { carrier: "FastFreight" } });
    expect(calls).toHaveLength(1);
    expect(bodyOf(calls[0]).expectedOutcome).toBeNull();
  });

  it("a publish refusal surfaces the server error with every failed check named", async () => {
    // Covers: REQ-LANE-021 (WP-0714 AC-2)
    const fetchMock = vi.fn(async () => ({
      json: async () => ({
        ok: false,
        error: {
          kind: "validation",
          message:
            "lane 'lane-wh-receiving' cannot publish: check 'discrepancy-check' has no fail branch; step 'post-receipt-moves' uses provisional Action Type 'warehouse.post_stock_move' — not in the adopted catalog",
        },
      }),
    })) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);
    await expect(api.publishLane("lane-wh-receiving")).rejects.toBeInstanceOf(ApiError);
    await expect(api.publishLane("lane-wh-receiving")).rejects.toThrow(/has no fail branch/);
    await expect(api.publishLane("lane-wh-receiving")).rejects.toThrow(/provisional Action Type/);
  });
});
