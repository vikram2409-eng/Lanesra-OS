/**
 * "Start lane run" entry points (WP-0707, REQ-LANE-014 / REQ-LANE-010).
 *
 * One active run per lane per record: starting when one already exists
 * opens the existing run instead of a second. The button lists the
 * published lanes for the record's object; with one lane it starts
 * directly, with several it lets the person pick.
 *
 * Covers: REQ-LANE-014 (WP-0707 AC-1), REQ-LANE-010 (WP-0707 AC-1).
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StartLaneRunButton } from "./StartLaneRun";
import { api } from "../../lib/api";
import type { LaneRunListItem } from "../../lib/types";

vi.mock("../../lib/api", () => ({
  api: {
    listLaneBlueprints: vi.fn(),
    listLaneRunsForRecord: vi.fn(),
    startLaneRun: vi.fn(),
  },
}));

const blueprints = [
  { lane_key: "lane-wh-receiving", name: "Receive Shipment", object_key: "purchase_order", latest_version: 1, latest_status: "published" },
  { lane_key: "lane-wh-pick-pack", name: "Pick and Pack", object_key: "purchase_order", latest_version: 1, latest_status: "published" },
  { lane_key: "lane-draft", name: "Draft lane", object_key: "purchase_order", latest_version: 1, latest_status: "draft" },
];

function activeRun(): LaneRunListItem {
  return {
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
  };
}

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("StartLaneRunButton", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listLaneBlueprints).mockResolvedValue(blueprints as never);
    vi.mocked(api.listLaneRunsForRecord).mockResolvedValue([] as never);
    vi.mocked(api.startLaneRun).mockResolvedValue({ id: "run-2" } as never);
  });

  function renderButton(props: Partial<Parameters<typeof StartLaneRunButton>[0]> = {}) {
    const onOpenRun = vi.fn();
    const utils = render(
      <Providers>
        <StartLaneRunButton
          objectKey="purchase_order"
          recordId="po-1"
          recordLabel="PO-2026-001"
          onOpenRun={onOpenRun}
          {...props}
        />
      </Providers>,
    );
    return { onOpenRun, ...utils };
  }

  /** The button is disabled while its lane/run queries load. */
  async function enabledButton(name: string | RegExp) {
    const button = await screen.findByRole("button", { name });
    await waitFor(() => expect(button).not.toBeDisabled());
    return button;
  }

  it("starts the only published lane for the object and opens the new run", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    vi.mocked(api.listLaneBlueprints).mockResolvedValue([blueprints[0]] as never);
    const { onOpenRun } = renderButton();
    fireEvent.click(await enabledButton("Start lane run"));
    await waitFor(() => {
      expect(api.startLaneRun).toHaveBeenCalledWith("lane-wh-receiving", "po-1");
    });
    await waitFor(() => {
      expect(onOpenRun).toHaveBeenCalledWith("run-2", "PO-2026-001");
    });
  });

  it("opens the existing active run instead of starting a second", async () => {
    // Covers: REQ-LANE-010 (WP-0707 AC-1)
    vi.mocked(api.listLaneBlueprints).mockResolvedValue([blueprints[0]] as never);
    vi.mocked(api.listLaneRunsForRecord).mockResolvedValue([activeRun()] as never);
    const { onOpenRun } = renderButton();
    fireEvent.click(await enabledButton("Open lane run"));
    await waitFor(() => {
      expect(onOpenRun).toHaveBeenCalledWith("run-1", "PO-2026-001");
    });
    expect(api.startLaneRun).not.toHaveBeenCalled();
  });

  it("with several lanes it lets the person pick before starting", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    const { onOpenRun } = renderButton();
    fireEvent.click(await enabledButton("Start lane run"));
    expect(await screen.findByLabelText("Lane")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Lane"), { target: { value: "lane-wh-pick-pack" } });
    fireEvent.click(screen.getByRole("button", { name: "Start Pick and Pack" }));
    await waitFor(() => {
      expect(api.startLaneRun).toHaveBeenCalledWith("lane-wh-pick-pack", "po-1");
    });
    expect(onOpenRun).toHaveBeenCalledWith("run-2", "PO-2026-001");
  });

  it("only offers published lanes", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    renderButton();
    fireEvent.click(await enabledButton("Start lane run"));
    const select = await screen.findByLabelText("Lane");
    expect(select.textContent).not.toContain("Draft lane");
    expect(select.textContent).toContain("Receive Shipment");
    expect(select.textContent).toContain("Pick and Pack");
  });

  it("a start refused as a duplicate opens the existing run", async () => {
    // Covers: REQ-LANE-010 (WP-0707 AC-1)
    vi.mocked(api.listLaneBlueprints).mockResolvedValue([blueprints[0]] as never);
    vi.mocked(api.startLaneRun).mockRejectedValue(
      new Error("lane 'lane-wh-receiving' already has an active run (run-1) on this record"),
    );
    vi.mocked(api.listLaneRunsForRecord)
      .mockResolvedValueOnce([])
      .mockResolvedValue([activeRun()] as never);
    const { onOpenRun } = renderButton();
    fireEvent.click(await enabledButton("Start lane run"));
    await waitFor(() => {
      expect(onOpenRun).toHaveBeenCalledWith("run-1", "PO-2026-001");
    });
  });

  it("is disabled when no published lane runs on this object", async () => {
    // Covers: REQ-LANE-014 (WP-0707 AC-1)
    vi.mocked(api.listLaneBlueprints).mockResolvedValue([] as never);
    renderButton();
    const button = await screen.findByRole("button", { name: "Start lane run" });
    expect(button).toBeDisabled();
  });
});
