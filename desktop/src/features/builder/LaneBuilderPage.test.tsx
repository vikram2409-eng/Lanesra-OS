/**
 * Object-first entry into the builder (WP-0722, REQ-LANE-024).
 * Covers: REQ-LANE-024 (WP-0722 AC-1).
 *
 * AC-1: from an object's page, "New lane" opens the builder with that
 * object set; publish and dry run are available without ever touching
 * the Object card. The page is handed the pre-filled definition
 * (App.tsx builds it from the guided start's choice); the proof at this
 * layer is that the Object card — the `<select aria-label="Lane
 * object">` — never renders for it, while it still renders for a blank
 * lane with no object. Same mock pattern as LaneBuilder.test.tsx, plus
 * a QueryClientProvider for the page's own useQuery.
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LaneBuilderPage } from "./LaneBuilderPage";
import { api } from "../../lib/api";
import type { LaneDefinition } from "../../lib/types";

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
    getLaneBlueprint: vi.fn(),
  },
  ApiError: class ApiError extends Error {
    kind: string;
    constructor(payload: { kind: string; message: string }) {
      super(payload.message);
      this.kind = payload.kind;
    }
  },
}));

const PASS_REPORT = {
  lane_key: "lane-new",
  version: 1,
  refused: false,
  checks: [{ id: "P-01", step: null, status: "pass", reason: null }],
};

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function blankOn(objectKey: string): LaneDefinition {
  return {
    id: "lane-new",
    name: "Count the bins",
    description: "",
    object_key: objectKey,
    version: 1,
    status: "draft",
    stages: [{ id: "stage-1", name: "Stage 1", steps: [] }],
    run_data: { fields: [] },
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.builderCheckLane).mockResolvedValue(PASS_REPORT as never);
  vi.mocked(api.builderReadsWrites).mockResolvedValue({
    lane_key: "lane-new",
    steps: [],
    lane_reads: [],
    plumbing: [],
  } as never);
  vi.mocked(api.builderActionTypes).mockResolvedValue([] as never);
  vi.mocked(api.adoptBuilderActionType).mockResolvedValue({ name: "Set Tracking" } as never);
  vi.mocked(api.listCustomRecords).mockResolvedValue([] as never);
  vi.mocked(api.listCustomObjects).mockResolvedValue([
    { key: "purchase_order", plural_label: "Purchase Orders" },
  ] as never);
  vi.mocked(api.listAccessRoles).mockResolvedValue([] as never);
  vi.mocked(api.listLaneBlueprints).mockResolvedValue([] as never);
  vi.mocked(api.listWorkflowRules).mockResolvedValue([] as never);
  vi.mocked(api.saveBuilderLane).mockResolvedValue({ lane_key: "lane-new", version: 1 } as never);
  vi.mocked(api.publishLane).mockResolvedValue({ version: 1 } as never);
});

describe("LaneBuilderPage object-first entry", () => {
  it("AC-1: a lane handed an object never shows the Object card, and publish is not blocked for want of one", async () => {
    render(
      <Providers>
        <LaneBuilderPage laneKey={null} initialDefinition={blankOn("purchase_order")} />
      </Providers>,
    );
    expect(screen.getByRole("heading", { name: "Count the bins" })).toBeVisible();
    await waitFor(() => expect(screen.queryByLabelText("Lane object")).not.toBeInTheDocument());
    expect(screen.getByText(/0 steps · 1 stages · purchase_order/)).toBeVisible();
    const publish = screen.getByRole("button", { name: "Publish" });
    expect(publish).not.toHaveAttribute("title", "Pick an object first — a lane with no object cannot publish.");
    // The dry-run section is present (it is blocked only by "no records",
    // not by "no object") — the run is available without the Object card.
    expect(screen.getByRole("heading", { name: "Dry-run preview" })).toBeVisible();
    expect(screen.getByText(/No purchase_order records exist yet/)).toBeVisible();
  });

  it("AC-1: a blank lane with no object still shows the Object card (the fallback)", async () => {
    render(
      <Providers>
        <LaneBuilderPage laneKey={null} initialDefinition={blankOn("")} />
      </Providers>,
    );
    await waitFor(() => expect(screen.getByLabelText("Lane object")).toBeInTheDocument());
    expect(screen.getByRole("button", { name: "Publish" })).toHaveAttribute(
      "title",
      "Pick an object first — a lane with no object cannot publish.",
    );
  });

  it("AC-1: the object can still be changed from the builder's name row — nothing is locked", async () => {
    render(
      <Providers>
        <LaneBuilderPage laneKey={null} initialDefinition={blankOn("purchase_order")} />
      </Providers>,
    );
    await waitFor(() => expect(screen.queryByLabelText("Lane object")).not.toBeInTheDocument());
    fireEvent.change(screen.getByLabelText("Lane name"), { target: { value: "Count the bins (v2)" } });
    expect(screen.getByRole("heading", { name: "Count the bins (v2)" })).toBeVisible();
  });
});
