/**
 * Object 360 pan/fit fix — GraphCanvas wiring tests.
 *
 * The pure fit/clamp math lives in graphView.test.ts; these tests prove
 * the canvas actually uses it: full labels render in full (Object 360)
 * while the default still truncates (Ontology Graph), and a drag that
 * would fling the graph off-screen is clamped to the reachable range.
 */
import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { GraphCanvas } from "./GraphCanvas";
import { GRAPH_CLUSTER_ORDER } from "./graphTypes";
import { layoutGraphPositions } from "./useGraphLayout";
import {
  clampPan,
  contentBounds,
  estimateEdgeLabelSize,
  estimatePillSize,
  expandBounds,
  fitView,
  separateOverlappingPills,
} from "./graphView";
import { fixtureModel, ROOT_KEY } from "../../__tests__/fixtures";

const LONG_LABEL = "PO received — put away to staging";

function longLabelModel() {
  return {
    nodes: [
      { key: "custom_object:po", id: "po", type: "custom_object" as const, label: "PO-2026-001", isRoot: true },
      { key: "workflow:wf1", id: "wf1", type: "workflow" as const, label: LONG_LABEL },
    ],
    edges: [{ fromKey: "workflow:wf1", toKey: "custom_object:po", edgeType: "applies_to", label: "applies to" }],
  };
}

function transformOf(container: HTMLElement): { x: number; y: number; zoom: number } {
  const inner = container.querySelector(".og-pan-inner") as HTMLElement;
  const m = /translate\((-?[\d.]+)px, (-?[\d.]+)px\) scale\(([\d.]+)\)/.exec(inner.style.transform);
  if (!m) throw new Error(`unexpected transform: ${inner.style.transform}`);
  return { x: Number(m[1]), y: Number(m[2]), zoom: Number(m[3]) };
}

describe("GraphCanvas fullLabels", () => {
  it("renders the full label, wrapping, when fullLabels is set", () => {
    const { container } = render(
      <GraphCanvas model={longLabelModel()} rootKey="custom_object:po" hiddenTypes={new Set()} fullLabels />,
    );
    const pill = container.querySelector('[data-node-key="workflow:wf1"]')!;
    expect(pill).toHaveClass("og-pill-full");
    expect(pill.textContent).toContain(LONG_LABEL);
    expect(pill.textContent).not.toContain("…");
  });

  it("still truncates at 18 chars by default (Ontology Graph unchanged)", () => {
    const { container } = render(
      <GraphCanvas model={longLabelModel()} rootKey="custom_object:po" hiddenTypes={new Set()} />,
    );
    const pill = container.querySelector('[data-node-key="workflow:wf1"]')!;
    expect(pill).not.toHaveClass("og-pill-full");
    expect(pill.textContent).not.toContain(LONG_LABEL);
    expect(pill.textContent).toContain("…");
  });

  it("separates overlapping full-label pills instead of stacking them", () => {
    const { container } = render(
      <GraphCanvas model={fixtureModel} rootKey={ROOT_KEY} hiddenTypes={new Set()} fullLabels />,
    );
    const W = 1200;
    const H = 800;
    const raw = layoutGraphPositions(fixtureModel, { rootKey: ROOT_KEY, clusterOrder: GRAPH_CLUSTER_ORDER, W, H });
    const items = fixtureModel.nodes.map((n) => {
      const p = raw.get(n.key)!;
      const size = estimatePillSize(n.label, { isRoot: n.isRoot, fullLabels: true });
      return { key: n.key, x: p.x, y: p.y, width: size.width, height: size.height };
    });
    const separated = separateOverlappingPills(items, { w: W, h: H });

    let moved = 0;
    fixtureModel.nodes.forEach((n) => {
      const pill = container.querySelector(`[data-node-key="${n.key}"]`) as HTMLElement;
      const expected = separated.get(n.key)!;
      expect(parseFloat(pill.style.left)).toBeCloseTo(expected.x, 5);
      expect(parseFloat(pill.style.top)).toBeCloseTo(expected.y, 5);
      const rawPos = raw.get(n.key)!;
      if (Math.abs(rawPos.x - expected.x) > 0.01 || Math.abs(rawPos.y - expected.y) > 0.01) moved++;
    });
    // The fixture really does collide without separation, so this test
    // is not vacuous: at least one pill must have been moved.
    expect(moved).toBeGreaterThan(0);
  });
});

describe("GraphCanvas drag clamp", () => {
  it("clamps a drag that would pan the graph completely off-screen", () => {
    const { container } = render(
      <GraphCanvas model={fixtureModel} rootKey={ROOT_KEY} hiddenTypes={new Set()} />,
    );
    const canvas = container.querySelector(".ontology-graph-canvas") as HTMLElement;

    // Expected fitted view, computed the same way the canvas does
    // (jsdom never measures a real size, so the canvas uses its
    // 1200x800 pre-measure fallback).
    const W = 1200;
    const H = 800;
    const positions = layoutGraphPositions(fixtureModel, { rootKey: ROOT_KEY, clusterOrder: GRAPH_CLUSTER_ORDER, W, H });
    const items = fixtureModel.nodes
      .map((n) => {
        const p = positions.get(n.key)!;
        const size = estimatePillSize(n.label, { isRoot: n.isRoot });
        return { x: p.x, y: p.y, width: size.width, height: size.height };
      })
      .concat(
        fixtureModel.edges.map((e) => {
          const a = positions.get(e.fromKey)!;
          const b = positions.get(e.toKey)!;
          const size = estimateEdgeLabelSize(e.label, e.access);
          return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, width: size.width, height: size.height };
        }),
      );
    const bounds = expandBounds(contentBounds(items)!, 58, 58);
    const fitted = fitView(bounds, { w: W, h: H }, { padding: 40, maxZoom: 1.5, minZoom: 0.4 });

    // The canvas auto-fits on mount: the starting transform is the fit.
    const start = transformOf(container);
    expect(start.zoom).toBeCloseTo(fitted.zoom, 5);
    expect(start.x).toBeCloseTo(fitted.pan.x, 5);
    expect(start.y).toBeCloseTo(fitted.pan.y, 5);

    // Drag 10,000px left — without the clamp the graph would vanish.
    fireEvent.pointerDown(canvas, { clientX: 600, clientY: 400, pointerId: 1 });
    fireEvent.pointerMove(canvas, { clientX: 600 - 10000, clientY: 400, pointerId: 1 });
    fireEvent.pointerUp(canvas, { pointerId: 1 });

    const after = transformOf(container);
    const expected = clampPan(
      { x: fitted.pan.x - 10000, y: fitted.pan.y },
      fitted.zoom,
      bounds,
      { w: W, h: H },
    );
    expect(after.x).toBeCloseTo(expected.x, 5);
    expect(after.y).toBeCloseTo(expected.y, 5);
    // And the content is still on screen: its right edge is >= the
    // 80px margin inside the viewport's left edge.
    expect(after.x + bounds.maxX * after.zoom).toBeGreaterThanOrEqual(80);
  });
});
