/**
 * PR 3 — GraphCanvas regression tests.
 *
 * Verifies the shared pill-node renderer: one pill per node, root pill
 * styling, dashed-ellipse cluster boundaries per type, edges with
 * data-edge-from/to, edge labels, inspecting ring, and hidden-type
 * filtering.
 */
import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphCanvas } from "../components/graph/GraphCanvas";
import { fixtureModel } from "./fixtures";

function renderCanvas(extra: Partial<Parameters<typeof GraphCanvas>[0]> = {}) {
  return render(
    <GraphCanvas
      model={fixtureModel}
      rootKey="custom_object:companies"
      hiddenTypes={new Set()}
      {...extra}
    />,
  );
}

describe("GraphCanvas", () => {
  it("renders one pill per node with data-node-key", () => {
    const { container } = renderCanvas();
    const pills = container.querySelectorAll(".og-pill");
    expect(pills).toHaveLength(fixtureModel.nodes.length);
    const keys = Array.from(pills).map((p) => p.getAttribute("data-node-key"));
    fixtureModel.nodes.forEach((n) => expect(keys).toContain(n.key));
  });

  it("renders the root pill with the root class", () => {
    const { container } = renderCanvas();
    const root = container.querySelector('[data-node-key="custom_object:companies"]');
    expect(root).toHaveClass("og-pill-root");
  });

  it("renders dashed-ellipse cluster boundaries per type", () => {
    const { container } = renderCanvas();
    // custom_object, custom_field, workflow, ai_agent, metric_definition
    expect(container.querySelectorAll("ellipse").length).toBe(5);
  });

  it("renders edges with data-edge-from/to and the access edge dashed", () => {
    const { container } = renderCanvas();
    const lines = container.querySelectorAll("line[data-edge-from]");
    expect(lines).toHaveLength(fixtureModel.edges.length);
    const access = container.querySelector('line[data-edge-from="ai_agent:a1"]');
    expect(access?.getAttribute("stroke-dasharray")).toBeTruthy();
  });

  it("renders an edge label per edge", () => {
    const { container } = renderCanvas();
    expect(container.querySelectorAll(".ontology-edge-label")).toHaveLength(fixtureModel.edges.length);
  });

  it("marks the inspected pill with the inspecting ring", () => {
    const { container } = renderCanvas({ inspectingKey: "custom_field:name" });
    const pill = container.querySelector('[data-node-key="custom_field:name"]');
    expect(pill).toHaveClass("og-pill-inspecting");
  });

  it("hides nodes of hidden types", () => {
    const { container } = renderCanvas({ hiddenTypes: new Set(["workflow"]) });
    expect(container.querySelector('[data-node-key="workflow:wf1"]')).toBeNull();
    // other pills remain
    expect(container.querySelector('[data-node-key="ai_agent:a1"]')).not.toBeNull();
  });

  it("fires onNodeClick with the clicked node", () => {
    const onNodeClick = vi.fn();
    const { container } = renderCanvas({ onNodeClick });
    const pill = container.querySelector('[data-node-key="ai_agent:a1"]')!;
    fireEvent.click(pill);
    expect(onNodeClick).toHaveBeenCalledTimes(1);
    expect(onNodeClick.mock.calls[0][0]).toMatchObject({ key: "ai_agent:a1", label: "Research Analyst" });
  });
});
