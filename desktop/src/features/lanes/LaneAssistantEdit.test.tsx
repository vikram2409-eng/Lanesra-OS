/**
 * The blueprint page's "Ask Assistant to edit" section (WP-0723,
 * REQ-LANE-022).
 *
 * Renders LaneBlueprintPage with the api module mocked (same pattern as
 * LanePublish.test.tsx): the Receive Shipment definition from the shared
 * example, and an edit that returns a revised draft version. No backend
 * is needed.
 *
 * Covers: REQ-LANE-022 (WP-0723 AC-2, AC-3).
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LaneBlueprintPage } from "./LaneBlueprint";
import { api } from "../../lib/api";
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
    editLaneFromDescription: vi.fn(),
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

/// The revised draft version the edit returns: the same lane with one
/// added form step, version 2, still a draft.
function revisedVersion() {
  const revised = JSON.parse(JSON.stringify(definition)) as LaneDefinition;
  revised.version = 2;
  revised.stages[0].steps.push({
    id: "record-carrier",
    name: "Record carrier",
    kind: "form",
    role: "role_wh_receiver",
    required: true,
    description: "Record the carrier name.",
    fields: [
      {
        key: "carrier_name",
        label: "Carrier name",
        type: "string",
        required: true,
        binds_to: "run_data.carrier",
      },
    ],
  } as never);
  return {
    id: "v2",
    lane_id: "lane-wh-receiving",
    workspace_id: "ws",
    version: 2,
    status: "draft",
    definition_json: JSON.stringify({ format_version: "1.0.0", lane: revised }),
    graph_id: null,
    created_at: "",
    created_by: null,
    updated_at: "",
    updated_by: null,
  };
}

function renderPage(props: {
  onOpenEditedLane?: (laneKey: string, definition: LaneDefinition) => void;
} = {}) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return render(<LaneBlueprintPage onOpenEditedLane={props.onOpenEditedLane} />, { wrapper });
}

async function openEditSection() {
  // The default waitFor timeout (1s) is too tight under full-suite load,
  // where the mocked queries can take longer to settle.
  await waitFor(
    () => expect(screen.getByRole("heading", { name: "Ask Assistant to edit" })).toBeVisible(),
    { timeout: 5000 },
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.listLaneBlueprints).mockResolvedValue([
    {
      lane_key: "lane-wh-receiving",
      name: "Receive Shipment",
      object_key: "purchase_order",
      latest_version: 1,
      latest_status: "draft",
    },
  ] as never);
  vi.mocked(api.getLaneBlueprint).mockResolvedValue(blueprint() as never);
  vi.mocked(api.listCustomRecords).mockResolvedValue([]);
  vi.mocked(api.checkLanePublish).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    version: 1,
    refused: false,
    checks: [],
  } as never);
  vi.mocked(api.dryRunLane).mockResolvedValue({} as never);
});

describe("Ask Assistant to edit", () => {
  it("asks the Assistant to revise the lane and opens the revision in the builder", async () => {
    const onOpenEditedLane = vi.fn();
    vi.mocked(api.editLaneFromDescription).mockResolvedValue(revisedVersion() as never);
    renderPage({ onOpenEditedLane });

    await openEditSection();
    fireEvent.change(screen.getByLabelText("Change to make"), {
      target: { value: "Add a step where the receiver records the carrier name." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ask Assistant to edit" }));

    await waitFor(() =>
      expect(api.editLaneFromDescription).toHaveBeenCalledWith(
        "lane-wh-receiving",
        "Add a step where the receiver records the carrier name.",
      ),
    );
    // The result opens in the builder for review: the revised definition
    // carries the added step, version 2, still a draft.
    await waitFor(() => expect(onOpenEditedLane).toHaveBeenCalled());
    const [laneKey, revised] = onOpenEditedLane.mock.calls[0] as [string, LaneDefinition];
    expect(laneKey).toBe("lane-wh-receiving");
    expect(revised.version).toBe(2);
    expect(revised.status).toBe("draft");
    expect(revised.stages[0].steps.map((s) => s.id)).toContain("record-carrier");
  });

  it("shows the refusal when the Assistant cannot revise the lane", async () => {
    const onOpenEditedLane = vi.fn();
    vi.mocked(api.editLaneFromDescription).mockRejectedValue(
      new Error("step 'close-po' uses Action Type 'purchase_order.close_receipt' - not in the adopted catalog"),
    );
    renderPage({ onOpenEditedLane });

    await openEditSection();
    fireEvent.change(screen.getByLabelText("Change to make"), {
      target: { value: "Close the PO as RECEIVED." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ask Assistant to edit" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("not in the adopted catalog"),
    );
    expect(onOpenEditedLane).not.toHaveBeenCalled();
  });

  it("the ask button stays disabled until a change is described", async () => {
    renderPage();
    await openEditSection();
    const button = screen.getByRole("button", { name: "Ask Assistant to edit" });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  });
});
