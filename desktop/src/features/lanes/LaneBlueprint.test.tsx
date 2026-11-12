/**
 * The lane blueprint view (WP-0712, REQ-LANE-020).
 * Covers: REQ-LANE-020 (WP-0712 AC-1, AC-2, AC-3).
 *
 * The view renders from the shared lane schema example - the same
 * definition the demo and the core import - so the snapshot below IS the
 * Receive Shipment blueprint. The view takes only a LaneBlueprint prop:
 * it has no record input at all, which is the AC-3 guarantee at this
 * layer (the service layer proves the payload carries no run or anchor
 * data; see desktop/core/tests/process_lane_blueprint.rs).
 */
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LaneBlueprintView } from "./LaneBlueprint";
import type { LaneBlueprint, LaneDefinition } from "../../lib/types";
import example from "../../../../program/spec/schemas/examples/lane-wh-receiving.json";

function receiveShipmentBlueprint(): LaneBlueprint {
  return {
    lane_key: "lane-wh-receiving",
    name: "Receive Shipment",
    object_key: "purchase_order",
    version: 1,
    status: "draft",
    definition: example.lane as unknown as LaneDefinition,
  };
}

describe("LaneBlueprintView", () => {
  it("renders the Receive Shipment blueprint: all stages and steps", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);

    // Purpose + version + object.
    expect(container.textContent).toContain("Receive Shipment");
    expect(container.textContent).toContain("v1 · draft");
    expect(container.textContent).toContain("purchase_order");
    // Stage chevrons, in order.
    for (const name of ["Verify", "Record receipt", "Hold & review", "Stage & put away", "Confirm"]) {
      expect(container.textContent).toContain(name);
    }
    // All twelve steps.
    for (const name of [
      "Confirm shipment arrived",
      "Match lines to the delivery note",
      "Enter received quantities",
      "Discrepancy check",
      "Review discrepancy",
      "Record hold",
      "Stage the goods",
      "Post receipt moves",
      "Confirm put-away",
      "Post put-away moves",
      "Review summary",
      "Close the PO",
    ]) {
      expect(container.textContent).toContain(name);
    }
  });

  it("renders branches with their targets", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);
    expect(container.textContent).toContain("pass → Stage & put away · Stage the goods");
    expect(container.textContent).toContain("fail → Hold & review · Review discrepancy");
    expect(container.textContent).toContain("approved → Stage & put away · Stage the goods");
    expect(container.textContent).toContain("rejected → Hold & review · Record hold");
  });

  it("renders run-data fields and mapped record fields", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);
    // Run-data declarations.
    expect(container.textContent).toContain("carrier");
    expect(container.textContent).toContain("delivery_note_qty");
    expect(container.textContent).toContain("hold_reason");
    // Mapped record fields name the mapping, never a value.
    expect(container.textContent).toContain("record.lines[].qty_received");
    expect(container.textContent).toContain("record.lines[].qty_rejected");
    expect(container.textContent).toContain("run_data.rejection_reason");
  });

  it("renders writes, roles and the check message", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);
    expect(container.textContent).toContain("warehouse.post_stock_move");
    expect(container.textContent).toContain("purchase_order.close_receipt");
    expect(container.textContent).toContain("role_wh_receiver");
    expect(container.textContent).toContain("role_wh_warehouse_manager");
    expect(container.textContent).toContain("A line is short or has rejected units.");
    expect(container.textContent).toContain("Replaces workflow wf-wh-01, action 0.");
  });

  it("renders the same blueprint identically every time (the export matches the view)", () => {
    // AC-2: the print export renders this same component with this same
    // prop, so the export is the view. A deterministic render is the
    // precondition - no timestamps, no random ids, no per-render state.
    const blueprint = receiveShipmentBlueprint();
    const first = render(<LaneBlueprintView blueprint={blueprint} />);
    const second = render(<LaneBlueprintView blueprint={blueprint} />);
    expect(first.container.innerHTML).toBe(second.container.innerHTML);
  });

  it("matches the Receive Shipment blueprint snapshot", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);
    expect(container.firstChild).toMatchSnapshot();
  });

  it("takes only the definition: no run or anchor record is rendered", () => {
    const { container } = render(<LaneBlueprintView blueprint={receiveShipmentBlueprint()} />);
    // No run state, no anchor-record panel, no entered values anywhere in
    // the view's own chrome - the component receives a LaneBlueprint and
    // nothing else. (The definition's own authored prose names its worked
    // example; that text is part of the definition, not a record read.)
    expect(container.textContent).not.toContain("Anchor record:");
    expect(container.textContent).not.toContain("Run status");
    expect(container.textContent).not.toContain("Entered values");
  });
});
