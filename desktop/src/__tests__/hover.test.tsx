/**
 * PR 3 — Hover spotlight regression tests.
 *
 * Verifies the demo-parity spotlight: hovering a pill dims everything
 * outside its 1-hop neighborhood (`graph-hover-dim` on the canvas,
 * `graph-hover-on` on connected pills), mouse-leave clears it, hovering
 * a cluster highlights the whole type, and the connected type set (the
 * legend-sync contract) is computed correctly.
 */
import { act, fireEvent, render, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphCanvas } from "../components/graph/GraphCanvas";
import { GraphLegend } from "../components/graph/GraphLegend";
import { useHoverDimming } from "../components/graph/useHoverDimming";
import { fixtureModel, ROOT_KEY as ROOT } from "./fixtures";

describe("useHoverDimming", () => {
  it("starts with no spotlight", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    expect(result.current.dimming).toBe(false);
    expect(result.current.connectedSet.size).toBe(0);
    expect(result.current.hoveredKey).toBeNull();
  });

  it("computes the 1-hop connected set on pill enter", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.dimming).toBe(true);
    // root + its 4 neighbors; the unconnected metric node stays out.
    expect(result.current.connectedSet.has(ROOT)).toBe(true);
    expect(result.current.connectedSet.has("custom_field:name")).toBe(true);
    expect(result.current.connectedSet.has("ai_agent:a1")).toBe(true);
    expect(result.current.connectedSet.has("metric_definition:m1")).toBe(false);
  });

  it("exposes connected types for legend sync", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.connectedTypes.has("custom_object")).toBe(true);
    expect(result.current.connectedTypes.has("custom_field")).toBe(true);
    expect(result.current.connectedTypes.has("workflow")).toBe(true);
    expect(result.current.connectedTypes.has("ai_agent")).toBe(true);
    expect(result.current.connectedTypes.has("metric_definition")).toBe(false);
  });

  it("clears the spotlight on leave", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.dimming).toBe(true);
    act(() => result.current.onLeave());
    expect(result.current.dimming).toBe(false);
    expect(result.current.connectedSet.size).toBe(0);
  });

  it("highlights the whole type on cluster enter", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onClusterEnter("custom_field"));
    expect(result.current.dimming).toBe(true);
    expect(result.current.connectedSet.has("custom_field:name")).toBe(true);
    expect(result.current.connectedSet.has("custom_field:revenue")).toBe(true);
    expect(result.current.connectedSet.has(ROOT)).toBe(false);
  });
});

describe("GraphCanvas hover spotlight", () => {
  it("dims the canvas and highlights the neighborhood on pill hover", () => {
    const { container } = render(
      <GraphCanvas model={fixtureModel} rootKey={ROOT} hiddenTypes={new Set()} />,
    );
    const canvas = container.querySelector(".ontology-graph-canvas")!;
    expect(canvas).not.toHaveClass("graph-hover-dim");

    const rootPill = container.querySelector(`[data-node-key="${ROOT}"]`)!;
    fireEvent.mouseEnter(rootPill);

    expect(canvas).toHaveClass("graph-hover-dim");
    expect(rootPill).toHaveClass("graph-hover-on");
    expect(container.querySelector('[data-node-key="custom_field:name"]')).toHaveClass("graph-hover-on");
    // Unconnected metric pill is dimmed (no hover-on class).
    expect(container.querySelector('[data-node-key="metric_definition:m1"]')).not.toHaveClass("graph-hover-on");

    fireEvent.mouseLeave(rootPill);
    expect(canvas).not.toHaveClass("graph-hover-dim");
  });
});

describe("GraphLegend sync", () => {
  it("highlights neighborhood types and dims the rest while spotlight is active", () => {
    const highlightedTypes = new Set(["custom_object", "custom_field", "workflow", "ai_agent"] as const);
    const { container } = render(
      <GraphLegend
        model={fixtureModel}
        hiddenTypes={new Set()}
        onToggleType={() => {}}
        highlightedTypes={highlightedTypes as unknown as Set<import("../lib/types").SystemNodeType>}
        spotlightActive
      />,
    );
    // Neighborhood types get the highlight ring…
    const chips = Array.from(container.querySelectorAll(".ontology-legend-item"));
    const highlighted = chips.filter((c) => c.classList.contains("legend-highlight"));
    expect(highlighted.length).toBe(4);
    // …and the out-of-neighborhood metric type is dimmed.
    const metricChip = chips.find((c) => c.getAttribute("data-legend-toggle") === "metric_definition");
    expect(metricChip).toHaveClass("legend-dim");
    expect(metricChip).not.toHaveClass("legend-highlight");
  });

  it("applies no sync classes when the spotlight is inactive", () => {
    const { container } = render(
      <GraphLegend model={fixtureModel} hiddenTypes={new Set()} onToggleType={() => {}} />,
    );
    expect(container.querySelectorAll(".legend-highlight,.legend-dim")).toHaveLength(0);
  });

  it("toggles a type off on chip click", () => {
    const onToggleType = vi.fn();
    const { container } = render(
      <GraphLegend model={fixtureModel} hiddenTypes={new Set()} onToggleType={onToggleType} />,
    );
    const chip = container.querySelector('[data-legend-toggle="workflow"]')!;
    fireEvent.click(chip);
    expect(onToggleType).toHaveBeenCalledWith("workflow");
  });
});
