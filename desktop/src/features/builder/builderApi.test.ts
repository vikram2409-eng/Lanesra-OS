/**
 * Builder API wrappers (WP-0718, REQ-LANE-024).
 * Covers: REQ-LANE-024 (WP-0718 AC-4).
 *
 * AC-4: desktop and Team Workspace behave the same from the same code.
 * Every builder wrapper goes through the shared dual-transport `call<T>`
 * in lib/api.ts — no transport-specific branches anywhere in the builder
 * surface — except `draftLaneFromDescription` (WP-0722): the WP-0715
 * draft is genuinely async, so like the other async admin actions it
 * goes through `callAdminAction` with its own HTTP route
 * (`/api/admin/process-lanes/drafts`) instead of the uniform
 * `/api/invoke/<command>` dispatcher. In jsdom there is no Tauri bridge, so `call` takes the HTTP
 * path; these tests assert the exact command name and arguments that
 * path sends to `/api/invoke/<command>`, which is the same command the
 * Tauri path invokes. The Tauri commands / server arms
 * (save_builder_lane, builder_check_lane, builder_dry_run_lane,
 * builder_reads_writes, builder_action_types) are the WP-0718
 * commands, verified against the landed core service
 * (core::services::process_lane_builder + the dispatch arms and Tauri
 * commands, 2026-11-15): argument names are the dispatch's camelCase
 * keys (`laneKey`, `definitionJson`, `anchorRecordId`, `fixtures`,
 * `expectedOutcome`, `objectKey`), and the results are the service's
 * shapes (BuilderSaveResult, PublishReport, DryRunReport,
 * ReadsWritesReport, BuilderActionType).
 */
import { beforeEach, describe, expect, it, vi } from "vitest";

import { api } from "../../lib/api";
import type { LaneDefinition } from "../../lib/types";

const definition = {
  id: "lane-wh-receiving",
  name: "Receive Shipment",
  object_key: "purchase_order",
  version: 1,
  status: "draft",
  stages: [],
} as LaneDefinition;

function mockFetchOnce(payload: unknown) {
  const fetchMock = vi.fn().mockResolvedValue({
    json: () => Promise.resolve({ ok: true, data: payload }),
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

function lastBody(fetchMock: ReturnType<typeof vi.fn>) {
  return JSON.parse(String(fetchMock.mock.calls[0][1].body));
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

describe("builder api wrappers", () => {
  it("saveBuilderLane posts save_builder_lane with the lane key and definition JSON", async () => {
    const fetchMock = mockFetchOnce({ lane_key: "lane-wh-receiving", version: 2, status: "draft" });
    await api.saveBuilderLane(definition, "lane-wh-receiving");
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/invoke/save_builder_lane",
      expect.objectContaining({ method: "POST" }),
    );
    const body = lastBody(fetchMock);
    expect(body.laneKey).toBe("lane-wh-receiving");
    expect(JSON.parse(body.definitionJson).lane.id).toBe("lane-wh-receiving");
  });

  it("saveBuilderLane sends a null lane key for a brand-new lane (the import path)", async () => {
    const fetchMock = mockFetchOnce({ lane_key: "lane-wh-receiving", version: 1, status: "draft" });
    await api.saveBuilderLane(definition);
    expect(lastBody(fetchMock).laneKey).toBeNull();
  });

  it("builderCheckLane posts builder_check_lane with the definition JSON", async () => {
    const fetchMock = mockFetchOnce({ lane_key: "lane-wh-receiving", refused: false, checks: [] });
    await api.builderCheckLane(definition);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/invoke/builder_check_lane",
      expect.objectContaining({ method: "POST" }),
    );
    expect(JSON.parse(lastBody(fetchMock).definitionJson).lane.id).toBe("lane-wh-receiving");
  });

  it("builderDryRunLane posts builder_dry_run_lane with definition, anchor and fixtures", async () => {
    const fetchMock = mockFetchOnce({ lane_key: "lane-wh-receiving" });
    await api.builderDryRunLane(definition, "po-2026-001", { run_data: { carrier: "Acme" } }, "received");
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/invoke/builder_dry_run_lane",
      expect.objectContaining({ method: "POST" }),
    );
    const body = lastBody(fetchMock);
    expect(body.anchorRecordId).toBe("po-2026-001");
    expect(body.fixtures).toEqual({ run_data: { carrier: "Acme" } });
    expect(body.expectedOutcome).toBe("received");
  });

  it("builderReadsWrites posts builder_reads_writes with the definition JSON", async () => {
    const fetchMock = mockFetchOnce({ fields: [] });
    await api.builderReadsWrites(definition);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/invoke/builder_reads_writes",
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("builderActionTypes posts builder_action_types with the object key", async () => {
    const fetchMock = mockFetchOnce([]);
    await api.builderActionTypes("purchase_order");
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/invoke/builder_action_types",
      expect.objectContaining({ method: "POST" }),
    );
    expect(lastBody(fetchMock).objectKey).toBe("purchase_order");
  });

  it("AC-4: no wrapper branches on the transport — the same call serves both", async () => {
    // The wrappers above all went through the one shared `call<T>`; none
    // of them reads `window.__TAURI_INTERNALS__` or builds its own URL.
    // Proof at this layer: every call hit the uniform /api/invoke/<name>
    // dispatcher, never a per-command HTTP path.
    const fetchMock = mockFetchOnce([]);
    await api.builderActionTypes();
    const url = String(fetchMock.mock.calls[0][0]);
    expect(url).toBe("/api/invoke/builder_action_types");
  });

  // WP-0722 (REQ-LANE-024, AC-2): the guided start's "describe it" path.
  // The draft is genuinely async, so — like the other async admin
  // actions (test_ai_key and friends) — it has its own HTTP route
  // instead of the /api/invoke dispatcher: POST
  // /api/admin/process-lanes/drafts (server/src/admin_actions.rs), with
  // the route's snake_case body, while the Tauri command takes the
  // camelCase args (Tauri v2 command convention). The result is always
  // a draft; the wrapper has no publish path.
  it("draftLaneFromDescription posts the description and object to the drafts route", async () => {
    const fetchMock = mockFetchOnce({ lane_key: "lane-draft-receive", versions: [] });
    await api.draftLaneFromDescription("Receive a purchase order.", "purchase_order");
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/admin/process-lanes/drafts",
      expect.objectContaining({ method: "POST" }),
    );
    expect(lastBody(fetchMock)).toEqual({
      description: "Receive a purchase order.",
      object_key: "purchase_order",
    });
  });
});
