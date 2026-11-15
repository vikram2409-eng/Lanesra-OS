/**
 * Object 360 pan/fit fix — unit tests for the pure view math in
 * `graphView.ts` (fit centers + scales to bounds, clamp never lets
 * content leave the viewport entirely, clamp is a no-op for fitted
 * small content, zoom changes keep content reachable).
 */
import { describe, expect, it } from "vitest";

import {
  clampPan,
  contentBounds,
  estimateEdgeLabelSize,
  estimatePillSize,
  expandBounds,
  fitView,
  separateOverlappingPills,
  type ContentBounds,
} from "./graphView";

const VIEWPORT = { w: 800, h: 600 };

describe("fitView", () => {
  it("centers content and zooms in up to maxZoom", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 400, maxY: 200 };
    const view = fitView(bounds, VIEWPORT, { padding: 0 });
    // zoom = min(800/400, 600/200, 1.5) = 1.5; center (200,100) -> viewport center (400,300)
    expect(view.zoom).toBe(1.5);
    expect(view.pan).toEqual({ x: 400 - 200 * 1.5, y: 300 - 100 * 1.5 });
  });

  it("scales large content down to fit", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 2000, maxY: 1000 };
    const view = fitView(bounds, VIEWPORT, { padding: 0 });
    expect(view.zoom).toBeCloseTo(0.4, 10);
    // Content spans x [0, 800] and y [100, 500]: fully inside the viewport.
    expect(view.pan.x + 0 * view.zoom).toBeCloseTo(0, 10);
    expect(view.pan.x + 2000 * view.zoom).toBeCloseTo(800, 10);
    expect(view.pan.y + 0 * view.zoom).toBeGreaterThanOrEqual(0);
    expect(view.pan.y + 1000 * view.zoom).toBeLessThanOrEqual(600);
  });

  it("respects padding around the content", () => {
    const bounds: ContentBounds = { minX: 100, minY: 100, maxX: 500, maxY: 300 };
    const view = fitView(bounds, VIEWPORT, { padding: 50 });
    // bw = 400 + 100 = 500, bh = 200 + 100 = 300 -> zoom = min(800/500, 600/300, 1.5) = 1.5
    expect(view.zoom).toBe(1.5);
    // 50px of viewport padding on each side at zoom 1.5: left edge at 400 - 300*1.5 + ... check directly:
    const left = view.pan.x + 100 * view.zoom;
    const right = view.pan.x + 500 * view.zoom;
    expect(left).toBeGreaterThanOrEqual(50);
    expect(right).toBeLessThanOrEqual(800 - 50);
  });

  it("centers content in the usable area when insets reserve an edge", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 400, maxY: 200 };
    const view = fitView(bounds, VIEWPORT, { padding: 0, insets: { right: 200 } });
    // Usable width is [0, 600]; zoom = min(600/400, 600/200, 1.5) = 1.5.
    expect(view.zoom).toBe(1.5);
    expect(view.pan.x + 0 * view.zoom).toBe(0);
    expect(view.pan.x + 400 * view.zoom).toBe(600);
  });

  it("returns the identity view for empty content", () => {
    expect(fitView(null, VIEWPORT)).toEqual({ zoom: 1, pan: { x: 0, y: 0 } });
  });

  it("never zooms out below minZoom", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 10000, maxY: 10000 };
    const view = fitView(bounds, VIEWPORT, { padding: 0 });
    expect(view.zoom).toBe(0.4);
  });
});

describe("clampPan", () => {
  it("is a no-op for fitted content smaller than the viewport", () => {
    const bounds: ContentBounds = { minX: 100, minY: 100, maxX: 300, maxY: 200 };
    const view = fitView(bounds, VIEWPORT, { padding: 40 });
    expect(clampPan(view.pan, view.zoom, bounds, VIEWPORT)).toEqual(view.pan);
    // And for a hand-centered view at zoom 1.
    const centered = { x: (800 - 400) / 2 - 100, y: (600 - 300) / 2 - 100 };
    expect(clampPan(centered, 1, bounds, VIEWPORT)).toEqual(centered);
  });

  it("never lets content leave the viewport entirely when panned far right/down", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 400, maxY: 300 };
    const clamped = clampPan({ x: 10000, y: 10000 }, 1, bounds, VIEWPORT);
    // At least `margin` (80) px of the content stays visible on each axis.
    expect(clamped.x + 400).toBeGreaterThanOrEqual(80);
    expect(clamped.x).toBeLessThanOrEqual(800 - 80);
    expect(clamped.y + 300).toBeGreaterThanOrEqual(80);
    expect(clamped.y).toBeLessThanOrEqual(600 - 80);
  });

  it("never lets content leave the viewport entirely when panned far left/up", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 400, maxY: 300 };
    const clamped = clampPan({ x: -10000, y: -10000 }, 1, bounds, VIEWPORT);
    expect(clamped.x).toBe(80 - 400);
    expect(clamped.y).toBe(80 - 300);
    // The visible sliver is exactly the margin.
    expect(clamped.x + 400).toBe(80);
    expect(clamped.y + 300).toBe(80);
  });

  it("clamps content wider than the viewport to the range edge, keeping a margin visible", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 2000, maxY: 300 };
    // Wide content can still slide, but never past the point where only
    // the margin sliver remains on screen.
    expect(clampPan({ x: 10000, y: 0 }, 1, bounds, VIEWPORT).x).toBe(800 - 80 - 0);
    expect(clampPan({ x: -10000, y: 0 }, 1, bounds, VIEWPORT).x).toBe(80 - 2000);
    // Vertically the content is smaller: the pan is untouched.
    expect(clampPan({ x: 10000, y: 0 }, 1, bounds, VIEWPORT).y).toBe(0);
  });

  it("centers content when the viewport is too small for a clamp range", () => {
    // Viewport narrower than 2 * margin: lo > hi, so center the content.
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 50, maxY: 50 };
    const clamped = clampPan({ x: 10000, y: -10000 }, 1, bounds, { w: 100, h: 100 });
    expect(clamped).toEqual({ x: (100 - 50) / 2, y: (100 - 50) / 2 });
  });

  it("keeps content reachable after a zoom change", () => {
    const bounds: ContentBounds = { minX: 0, minY: 0, maxX: 400, maxY: 300 };
    const fitted = fitView(bounds, VIEWPORT, { padding: 40 });
    // Zoom in 2x around the fitted pan, then clamp: content must still
    // overlap the viewport by at least the margin on both axes.
    const clamped = clampPan(fitted.pan, fitted.zoom * 2, bounds, VIEWPORT);
    const left = clamped.x;
    const right = clamped.x + 400 * fitted.zoom * 2;
    const top = clamped.y;
    const bottom = clamped.y + 300 * fitted.zoom * 2;
    expect(right).toBeGreaterThanOrEqual(80);
    expect(left).toBeLessThanOrEqual(800 - 80);
    expect(bottom).toBeGreaterThanOrEqual(80);
    expect(top).toBeLessThanOrEqual(600 - 80);
  });

  it("returns the pan unchanged when there is no content", () => {
    expect(clampPan({ x: 5, y: 6 }, 1, null, VIEWPORT)).toEqual({ x: 5, y: 6 });
  });
});

describe("contentBounds / estimatePillSize", () => {
  it("includes pill extents, not just centers", () => {
    const bounds = contentBounds([
      { x: 100, y: 100, width: 200, height: 30 },
      { x: 300, y: 200, width: 100, height: 30 },
    ]);
    expect(bounds).toEqual({ minX: 0, minY: 85, maxX: 350, maxY: 215 });
  });

  it("returns null for no items", () => {
    expect(contentBounds([])).toBeNull();
  });

  it("expands bounds on every side", () => {
    expect(expandBounds({ minX: 0, minY: 0, maxX: 10, maxY: 10 }, 5, 7)).toEqual({
      minX: -5,
      minY: -7,
      maxX: 15,
      maxY: 17,
    });
  });

  it("sizes a short truncated pill on one line", () => {
    const size = estimatePillSize("Annual Revenue");
    expect(size.height).toBeCloseTo(14.4 + 12, 10);
    expect(size.width).toBeCloseTo(14 * 7 + 24 + 14, 10);
  });

  it("caps a truncated pill at 18 chars", () => {
    const size = estimatePillSize("PO received — put away to staging");
    expect(size.width).toBeCloseTo(18 * 7 + 24 + 14, 10);
    expect(size.height).toBeCloseTo(14.4 + 12, 10);
  });

  it("wraps a long full label at the max width and grows taller", () => {
    const size = estimatePillSize("PO received — put away to staging", { fullLabels: true });
    expect(size.width).toBe(260);
    expect(size.height).toBeGreaterThan(14.4 + 12);
    // 33 chars * 7px = 231px of text over 222px available -> 2 lines.
    expect(size.height).toBeCloseTo(2 * 14.4 + 12, 10);
  });

  it("shows a short full label on one line at its natural width", () => {
    const size = estimatePillSize("BrightPath Logistics", { fullLabels: true, isRoot: true });
    expect(size.width).toBeCloseTo(20 * 7.5 + 32, 10);
    expect(size.height).toBeCloseTo(15.6 + 16, 10);
  });

  it("sizes edge labels smaller than pills", () => {
    const size = estimateEdgeLabelSize("applies to");
    expect(size.width).toBeLessThan(estimatePillSize("applies to").width);
    expect(size.height).toBe(18);
  });
});

describe("separateOverlappingPills", () => {
  const area = { w: 1000, h: 600 };

  function rectsOverlap(a: { x: number; y: number; width: number; height: number }, b: { x: number; y: number; width: number; height: number }, gap = 8): boolean {
    return (
      Math.abs(a.x - b.x) < (a.width + b.width) / 2 + gap &&
      Math.abs(a.y - b.y) < (a.height + b.height) / 2 + gap
    );
  }

  it("separates two overlapping pills", () => {
    const items = [
      { key: "a", x: 500, y: 300, width: 200, height: 30 },
      { key: "b", x: 520, y: 305, width: 200, height: 30 },
    ];
    const out = separateOverlappingPills(items, area);
    const a = { ...items[0], ...out.get("a")! };
    const b = { ...items[1], ...out.get("b")! };
    expect(rectsOverlap(a, b)).toBe(false);
  });

  it("is a no-op when pills already clear each other", () => {
    const items = [
      { key: "a", x: 200, y: 300, width: 200, height: 30 },
      { key: "b", x: 700, y: 300, width: 200, height: 30 },
    ];
    const out = separateOverlappingPills(items, area);
    expect(out.get("a")).toEqual({ x: 200, y: 300 });
    expect(out.get("b")).toEqual({ x: 700, y: 300 });
  });

  it("spreads a stack of pills parked on the same point", () => {
    const items = Array.from({ length: 6 }, (_, i) => ({
      key: `p${i}`,
      x: 966,
      y: 52,
      width: 200,
      height: 26,
    }));
    const out = separateOverlappingPills(items, area);
    const placed = items.map((item) => ({ ...item, ...out.get(item.key)! }));
    for (let i = 0; i < placed.length; i++) {
      for (let j = i + 1; j < placed.length; j++) {
        expect(rectsOverlap(placed[i], placed[j])).toBe(false);
      }
      // Every pill stays inside the area.
      expect(placed[i].x - placed[i].width / 2).toBeGreaterThanOrEqual(0);
      expect(placed[i].x + placed[i].width / 2).toBeLessThanOrEqual(1000);
      expect(placed[i].y - placed[i].height / 2).toBeGreaterThanOrEqual(0);
      expect(placed[i].y + placed[i].height / 2).toBeLessThanOrEqual(600);
    }
  });

  it("is deterministic", () => {
    const items = [
      { key: "a", x: 500, y: 300, width: 200, height: 30 },
      { key: "b", x: 500, y: 300, width: 180, height: 30 },
      { key: "c", x: 510, y: 295, width: 220, height: 30 },
    ];
    const first = separateOverlappingPills(items, area);
    const second = separateOverlappingPills(items, area);
    expect([...second.entries()]).toEqual([...first.entries()]);
  });
});
