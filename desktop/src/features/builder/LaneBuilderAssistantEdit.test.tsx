/**
 * The builder's own "Ask Assistant to edit" section (WP-0723,
 * REQ-LANE-022).
 *
 * Renders LaneBuilder with the api module mocked (same pattern as
 * LaneBuilder.test.tsx). The edit returns a revised draft version; the
 * builder swaps its definition in place and says so.
 *
 * Covers: REQ-LANE-022 (WP-0723 AC-3).
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.builderCheckLane).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    version: 1,
    refused: false,
    checks: [],
  } as never);
  vi.mocked(api.builderReadsWrites).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    steps: [],
    lane_reads: [],
    plumbing: [],
  } as never);
  vi.mocked(api.builderActionTypes).mockResolvedValue([] as never);
  vi.mocked(api.listCustomRecords).mockResolvedValue([] as never);
  vi.mocked(api.listCustomObjects).mockResolvedValue([] as never);
  vi.mocked(api.listAccessRoles).mockResolvedValue([] as never);
  vi.mocked(api.listLaneBlueprints).mockResolvedValue([] as never);
  vi.mocked(api.listWorkflowRules).mockResolvedValue([] as never);
  vi.mocked(api.saveBuilderLane).mockResolvedValue({
    lane_key: "lane-wh-receiving",
    version: 2,
  } as never);
  vi.mocked(api.publishLane).mockResolvedValue({ version: 2 } as never);
});

describe("LaneBuilder Ask Assistant to edit", () => {
  it("revises the lane in place and names the new draft version", async () => {
    const onChange = vi.fn();
    vi.mocked(api.editLaneFromDescription).mockResolvedValue(revisedVersion() as never);
    render(<LaneBuilder definition={definition} laneKey="lane-wh-receiving" onDefinitionChange={onChange} />);

    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Ask Assistant to edit" })).toBeVisible(),
    );
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
    // The revised definition replaces the one on screen - the added
    // step is listed (twice: the step card and the reads/writes map) -
    // and the message names the draft version.
    await waitFor(() => expect(screen.getByText(/Revised as draft version 2/)).toBeVisible());
    expect(screen.getAllByText("Record carrier").length).toBeGreaterThan(0);
    expect(onChange).toHaveBeenCalled();
  });

  it("shows the refusal and keeps the current definition", async () => {
    const onChange = vi.fn();
    vi.mocked(api.editLaneFromDescription).mockRejectedValue(
      new Error("step 'close-po' uses Action Type 'purchase_order.close_receipt' - not in the adopted catalog"),
    );
    render(<LaneBuilder definition={definition} laneKey="lane-wh-receiving" onDefinitionChange={onChange} />);

    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Ask Assistant to edit" })).toBeVisible(),
    );
    fireEvent.change(screen.getByLabelText("Change to make"), {
      target: { value: "Close the PO as RECEIVED." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ask Assistant to edit" }));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("not in the adopted catalog"),
    );
    // No revised definition was ever applied.
    expect(screen.queryByText("Record carrier")).toBeNull();
    expect(onChange).not.toHaveBeenCalled();
  });

  it("the section is absent for a lane that has no key yet", () => {
    render(<LaneBuilder definition={definition} laneKey={null} onDefinitionChange={vi.fn()} />);
    expect(screen.queryByRole("heading", { name: "Ask Assistant to edit" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Ask Assistant to edit" })).toBeNull();
  });
});
