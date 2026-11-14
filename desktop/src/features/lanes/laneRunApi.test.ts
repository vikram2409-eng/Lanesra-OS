/**
 * Lane-run API wrappers (WP-0707).
 *
 * The frontend runs in two editions from the same code: inside Tauri the
 * wrappers go through `invoke`, in Team Workspace through the HTTP
 * transport (`/api/invoke/<command>` for sync reads, `/api/admin/...` for
 * the genuinely async admin actions). jsdom has no Tauri bridge, so these
 * tests pin the HTTP transport: the command name, the HTTP path and the
 * request body. The Tauri command names are the same strings.
 *
 * Covers: REQ-LANE-014 (WP-0707 AC-1), REQ-LANE-015 (WP-0707 AC-3).
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

describe("lane-run api wrappers (HTTP transport)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("getLaneRunView reads through the sync get_process_lane_run_view command", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = { run_id: "run-1", status: "waiting_person" };
    const view = await api.getLaneRunView("run-1");
    expect(view).toEqual({ run_id: "run-1", status: "waiting_person" });
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/invoke/get_process_lane_run_view");
    expect(bodyOf(calls[0])).toEqual({ runId: "run-1" });
  });

  it("startLaneRun posts to the async admin route with the anchor record id", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = { id: "run-1", status: "waiting_person" };
    const run = await api.startLaneRun("lane-wh-receiving", "po-1");
    expect(run).toEqual({ id: "run-1", status: "waiting_person" });
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/admin/process-lanes/lane-wh-receiving/runs");
    expect(calls[0].init?.method).toBe("POST");
    expect(bodyOf(calls[0])).toEqual({ anchor_record_id: "po-1" });
  });

  it("submitLaneStep posts the entered values to the async admin route", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = { id: "run-1", status: "waiting_person" };
    await api.submitLaneStep("run-1", { received_date: "2026-11-14" });
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/admin/process-lane-runs/run-1/submit-step");
    expect(calls[0].init?.method).toBe("POST");
    expect(bodyOf(calls[0])).toEqual({ values: { received_date: "2026-11-14" } });
  });

  it("skipLaneStep posts the reason to the async admin route", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = { id: "run-1", status: "waiting_person" };
    await api.skipLaneStep("run-1", "Nothing to check on this order");
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/admin/process-lane-runs/run-1/skip-step");
    expect(bodyOf(calls[0])).toEqual({ reason: "Nothing to check on this order" });
  });

  it("listLaneRunsForRecord reads through list_lane_runs_for_record and unwraps {runs}", async () => {
    // Covers: REQ-LANE-015 (WP-0707 AC-3)
    const { calls, fetchMock } = mockFetch();
    fetchMock.nextData = {
      runs: [
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
      ],
    };
    const runs = await api.listLaneRunsForRecord("purchase_order", "po-1");
    expect(runs).toHaveLength(1);
    expect(runs[0].lane_name).toBe("Receive Shipment");
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("/api/invoke/list_lane_runs_for_record");
    expect(bodyOf(calls[0])).toEqual({ objectKey: "purchase_order", recordId: "po-1" });
  });

  it("listLaneRunsForRecord returns an empty list when the record has no runs", async () => {
    // Covers: REQ-LANE-015 (WP-0707 AC-3)
    const { fetchMock } = mockFetch();
    fetchMock.nextData = { runs: [] };
    await expect(api.listLaneRunsForRecord("purchase_order", "po-9")).resolves.toEqual([]);
  });

  it("a failed call surfaces the server error, not a silent empty result", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const fetchMock = vi.fn(async () => ({
      json: async () => ({
        ok: false,
        error: { kind: "validation", message: "lane 'lane-wh-receiving' already has an active run (run-1) on this record" },
      }),
    })) as unknown as typeof fetch;
    vi.stubGlobal("fetch", fetchMock);
    await expect(api.startLaneRun("lane-wh-receiving", "po-1")).rejects.toBeInstanceOf(ApiError);
    await expect(api.startLaneRun("lane-wh-receiving", "po-1")).rejects.toThrow(/already has an active run/);
  });
});
