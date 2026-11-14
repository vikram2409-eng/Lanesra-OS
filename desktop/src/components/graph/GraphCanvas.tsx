/**
 * PR 2 — Desktop Ontology Graph: main canvas renderer.
 *
 * React port of the demo's `graphRender` (app.js:4886) + pan/zoom.
 * Renders:
 * - `<svg>` with dashed cluster `<ellipse>`s and edge `<line>`s
 * - Absolutely-positioned `PillNode`s
 * - Edge labels as pills (for edges touching the inspected node, or all)
 *
 * Zoom/pan: wheel to zoom (at cursor), drag to pan, pinch on touch.
 * `Fit` frames the content bounds (pill extents included, see
 * `graphView.ts`) and re-runs on mount, model change, and resize;
 * pan is clamped on every pan/zoom/fit so the graph can never be
 * panned completely off-screen. `Reset` restores 1:1 (clamped).
 *
 * The canvas is height:100% with no min-height: the parent (the
 * Ontology Graph's fixed-height div, Object 360's flex:1 wrap) owns
 * the height. A min-height here would overflow short parents and the
 * wrap would clip the canvas's bottom rows.
 */
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { cn } from "../ui/cn";
import { GRAPH_CLUSTER_ORDER, SYSTEM_NODE_TYPE_COLORS, type GraphModel, type GraphNode } from "./graphTypes";
import { layoutGraphPositions, type GraphPosition } from "./useGraphLayout";
import { useHoverDimming, type HoverDimming } from "./useHoverDimming";
import { PillNode } from "./PillNode";
import { ClusterBoundary } from "./ClusterBoundary";
import {
  clampPan,
  contentBounds,
  estimateEdgeLabelSize,
  estimatePillSize,
  expandBounds,
  fitView,
  separateOverlappingPills,
  type ContentBounds,
  type FitInsets,
} from "./graphView";
import type { SystemNodeType } from "../../lib/types";

export interface GraphCanvasProps {
  model: GraphModel;
  rootKey: string | null;
  hiddenTypes: Set<SystemNodeType>;
  /** Currently inspected node key (outline ring + edge labels). */
  inspectingKey?: string | null;
  /** Collapsed cluster types (rendered as dashed-circle pills). */
  collapsedTypes?: Set<SystemNodeType>;
  onNodeClick?: (node: GraphNode) => void;
  onNodeDoubleClick?: (node: GraphNode) => void;
  onClusterToggle?: (type: SystemNodeType) => void;
  /** Exposed for Fit/Reset buttons in the toolbar. */
  zoomApiRef?: React.MutableRefObject<{ zoomIn: () => void; zoomOut: () => void; fit: () => void; reset: () => void; zoom: number } | null>;
  /** Hover spotlight state. If omitted, the canvas owns its own. */
  hover?: HoverDimming;
  /** Fires when zoom changes (for toolbar readout). */
  onZoomChange?: (zoom: number) => void;
  /** Full labels (wrap instead of truncating at 18 chars). Object 360
   *  enables this; the dense Ontology Graph keeps the default so its
   *  pills stay compact. */
  fullLabels?: boolean;
  /** Viewport edges to reserve when fitting (e.g. the Object 360
   *  inspector card over the canvas's right edge). */
  fitInset?: FitInsets;
  className?: string;
}

const MIN_ZOOM = 0.4;
const MAX_ZOOM = 3;

export const GraphCanvas = memo(function GraphCanvas(props: GraphCanvasProps) {
  const {
    model,
    rootKey,
    hiddenTypes,
    inspectingKey,
    collapsedTypes,
    onNodeClick,
    onNodeDoubleClick,
    onClusterToggle,
    zoomApiRef,
    hover: hoverProp,
    onZoomChange,
    fullLabels,
    fitInset,
    className,
  } = props;

  const containerRef = useRef<HTMLDivElement>(null);
  // Pre-measure fallback only: the effect below reads the real size
  // synchronously on mount, and ResizeObserver corrects it on resize —
  // the layout never visibly rests on this default.
  const [containerSize, setContainerSize] = useState({ w: 1200, h: 800 });
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null);
  const pinchRef = useRef<{ dist: number; zoom: number } | null>(null);

  // Measure container for layout dimensions — synchronously on mount
  // (so the first layout already uses the real size), then on resize.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const read = () => {
      const rect = el.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        // Floor is only a degenerate-size guard (100px): the old 400px
        // floor made the layout believe the canvas was taller/wider
        // than it is on short viewports, so the fit could clip the
        // bottom rows the canvas doesn't actually have.
        setContainerSize({ w: Math.max(100, rect.width), h: Math.max(100, rect.height) });
      }
    };
    read();
    const ro = new ResizeObserver((entries) => {
      const rect = entries[0].contentRect;
      if (rect.width > 0 && rect.height > 0) {
        setContainerSize({ w: Math.max(100, rect.width), h: Math.max(100, rect.height) });
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const W = containerSize.w;
  const H = containerSize.h;

  // Layout (deterministic, memoized).
  const positions = useMemo(
    () => layoutGraphPositions(model, { rootKey: rootKey ?? "", clusterOrder: GRAPH_CLUSTER_ORDER, W, H }),
    [model, rootKey, W, H],
  );

  // Display positions: with full labels, pills are up to 260px wide and
  // the force layout (which parks centers without knowing pill widths)
  // stacks several of them on the same spot — e.g. on the W-34 clamp
  // wall — covering each other's labels. Relax those apart; edges and
  // cluster ellipses follow the moved centers. The dense Ontology
  // Graph keeps the raw layout (its truncated pills are narrow).
  const displayPositions = useMemo(() => {
    if (!fullLabels) return positions;
    const items: { key: string; x: number; y: number; width: number; height: number }[] = [];
    model.nodes.forEach((n) => {
      if (hiddenTypes.has(n.type)) return;
      const p = positions.get(n.key);
      if (!p) return;
      const size = estimatePillSize(n.label, { isRoot: n.isRoot, fullLabels: true, count: n.count });
      items.push({ key: n.key, x: p.x, y: p.y, width: size.width, height: size.height });
    });
    const separated = separateOverlappingPills(items, { w: W, h: H });
    const next = new Map(positions);
    separated.forEach((pos, key) => next.set(key, pos));
    return next;
  }, [positions, model.nodes, hiddenTypes, fullLabels, W, H]);

  // Hover spotlight — owned here unless the parent passes one in
  // (parents pass it in when the legend needs the same state).
  const ownHover = useHoverDimming(model);
  const hover = hoverProp ?? ownHover;

  // Group visible nodes by type for cluster boundaries.
  const clusters = useMemo(() => {
    const map = new Map<SystemNodeType, GraphNode[]>();
    model.nodes.forEach((n) => {
      if (n.id === "__cluster__") return;
      if (hiddenTypes.has(n.type)) return;
      if (collapsedTypes?.has(n.type)) return;
      const list = map.get(n.type) ?? [];
      list.push(n);
      map.set(n.type, list);
    });
    return map;
  }, [model.nodes, hiddenTypes, collapsedTypes]);

  const nodeByKey = useMemo(() => new Map(model.nodes.map((n) => [n.key, n])), [model.nodes]);

  // Visible edges (both endpoints visible).
  const visibleEdges = useMemo(
    () =>
      model.edges.filter((e) => {
        const a = nodeByKey.get(e.fromKey);
        const b = nodeByKey.get(e.toKey);
        return a && b && !hiddenTypes.has(a.type) && !hiddenTypes.has(b.type);
      }),
    [model.edges, nodeByKey, hiddenTypes],
  );

  // Edge label positions (midpoint).
  const edgeLabelPos = useCallback(
    (fromKey: string, toKey: string): GraphPosition | null => {
      const a = displayPositions.get(fromKey);
      const b = displayPositions.get(toKey);
      if (!a || !b) return null;
      return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    },
    [displayPositions],
  );

  // --- Zoom / pan ---

  const clampZoom = useCallback((z: number) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z)), []);

  // Content bounds INCLUDING pill extents (and edge labels), so fitting
  // never half-clips the pills at the extremes. Cluster ellipses extend
  // PAD_X/PAD_Y (58/44 in ClusterBoundary) past their members, so the
  // bounds grow by that overhang whenever clusters render.
  const bounds = useMemo<ContentBounds | null>(() => {
    const items: { x: number; y: number; width: number; height: number }[] = [];
    model.nodes.forEach((n) => {
      if (hiddenTypes.has(n.type)) return;
      const p = displayPositions.get(n.key);
      if (!p) return;
      const size = estimatePillSize(n.label, { isRoot: n.isRoot, fullLabels, count: n.count });
      items.push({ x: p.x, y: p.y, width: size.width, height: size.height });
    });
    visibleEdges.forEach((e) => {
      const pos = edgeLabelPos(e.fromKey, e.toKey);
      if (!pos) return;
      const size = estimateEdgeLabelSize(e.label, e.access);
      items.push({ x: pos.x, y: pos.y, width: size.width, height: size.height });
    });
    const b = contentBounds(items);
    if (!b) return null;
    return clusters.size > 0 ? expandBounds(b, 58, 58) : b;
  }, [model.nodes, displayPositions, hiddenTypes, fullLabels, visibleEdges, clusters]);

  const viewport = useMemo(() => ({ w: W, h: H }), [W, H]);

  /** Apply a view, clamping the pan so content stays reachable. */
  const applyView = useCallback(
    (view: { zoom: number; pan: { x: number; y: number } }) => {
      setZoom(view.zoom);
      setPan(bounds ? clampPan(view.pan, view.zoom, bounds, viewport) : view.pan);
    },
    [bounds, viewport],
  );

  const zoomIn = useCallback(() => {
    const nz = clampZoom(zoom * 1.25);
    setZoom(nz);
    setPan((p) => (bounds ? clampPan(p, nz, bounds, viewport) : p));
  }, [clampZoom, zoom, bounds, viewport]);

  const zoomOut = useCallback(() => {
    const nz = clampZoom(zoom / 1.25);
    setZoom(nz);
    setPan((p) => (bounds ? clampPan(p, nz, bounds, viewport) : p));
  }, [clampZoom, zoom, bounds, viewport]);

  const fit = useCallback(() => {
    if (!bounds) return;
    applyView(fitView(bounds, viewport, { padding: 40, maxZoom: 1.5, minZoom: MIN_ZOOM, insets: fitInset }));
  }, [bounds, viewport, fitInset, applyView]);

  const reset = useCallback(() => {
    setZoom(1);
    setPan(bounds ? clampPan({ x: 0, y: 0 }, 1, bounds, viewport) : { x: 0, y: 0 });
    hover.onLeave();
  }, [hover, bounds, viewport]);

  // Expose zoom API to parent (toolbar).
  useEffect(() => {
    if (zoomApiRef) {
      zoomApiRef.current = { zoomIn, zoomOut, fit, reset, zoom };
    }
    onZoomChange?.(zoom);
  }, [zoomApiRef, zoomIn, zoomOut, fit, reset, zoom, onZoomChange]);

  // Auto-fit on mount and whenever the layout changes — model change
  // (legend toggle, pivot/recenter, data load), container resize, or
  // depth change all recompute `displayPositions`, so the graph
  // re-frames itself instead of resting on the one first fit. Pan/zoom
  // state alone never retriggers this: `displayPositions` is memoized
  // on [model, rootKey, W, H] only.
  useEffect(() => {
    if (displayPositions.size > 0) fit();
  }, [displayPositions, fit]);

  // Wheel zoom (at cursor).
  const onWheel = useCallback(
    (e: React.WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
      const rect = containerRef.current?.getBoundingClientRect();
      if (!rect) return;
      const cx = e.clientX - rect.left;
      const cy = e.clientY - rect.top;
      const nz = clampZoom(zoom * (e.deltaY < 0 ? 1.12 : 1 / 1.12));
      // Keep the cursor point stable, then clamp so the zoom can't push
      // the content off-screen.
      const raw = {
        x: cx - ((cx - pan.x) / zoom) * nz,
        y: cy - ((cy - pan.y) / zoom) * nz,
      };
      setZoom(nz);
      setPan(bounds ? clampPan(raw, nz, bounds, viewport) : raw);
    },
    [clampZoom, zoom, pan, bounds, viewport],
  );

  // Drag pan (clamped — the graph can never be panned off-screen).
  const onPointerDown = useCallback((e: React.PointerEvent) => {
    // Don't start pan on pill / button / label.
    if ((e.target as HTMLElement).closest(".og-pill,.og-cluster-label,button")) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY, origX: pan.x, origY: pan.y };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }, [pan]);

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    const d = dragRef.current;
    if (!d) return;
    const raw = { x: d.origX + (e.clientX - d.startX), y: d.origY + (e.clientY - d.startY) };
    setPan(bounds ? clampPan(raw, zoom, bounds, viewport) : raw);
  }, [bounds, zoom, viewport]);

  const onPointerUp = useCallback(() => {
    dragRef.current = null;
  }, []);

  // Touch pinch zoom.
  const onTouchStart = useCallback((e: React.TouchEvent) => {
    if (e.touches.length === 2) {
      const d = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY,
      );
      pinchRef.current = { dist: d, zoom };
    }
  }, [zoom]);

  const onTouchMove = useCallback(
    (e: React.TouchEvent) => {
      const p = pinchRef.current;
      if (!p || e.touches.length !== 2) return;
      e.preventDefault();
      const d = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY,
      );
      const nz = clampZoom(p.zoom * (d / p.dist));
      setZoom(nz);
      setPan((prev) => (bounds ? clampPan(prev, nz, bounds, viewport) : prev));
    },
    [clampZoom, bounds, viewport],
  );

  const onTouchEnd = useCallback(() => {
    pinchRef.current = null;
  }, []);

  const transform = `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`;

  return (
    <div
      ref={containerRef}
      className={cn("ontology-graph-canvas", hover.dimming && "graph-hover-dim", className)}
      style={{ width: "100%", height: "100%", minHeight: 0, position: "relative", overflow: "hidden", touchAction: "pan-x pan-y", userSelect: "none" }}
      onWheel={onWheel}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onTouchStart={onTouchStart}
      onTouchMove={onTouchMove}
      onTouchEnd={onTouchEnd}
      onMouseLeave={hover.onLeave}
    >
      <div className="og-pan-inner" style={{ position: "absolute", top: 0, left: 0, width: W, height: H, transform, transformOrigin: "0 0" }}>
        <svg width={W} height={H} style={{ position: "absolute", top: 0, left: 0, pointerEvents: "none" }}>
          {/* Cluster boundaries */}
          {Array.from(clusters.entries()).map(([type, members]) => (
            <ClusterBoundary
              key={`cluster-${type}`}
              type={type}
              members={members}
              positions={displayPositions}
              hoverOn={hover.hoveredClusterType === type}
              spotlightActive={hover.dimming}
              onCollapse={onClusterToggle}
              onMouseEnter={hover.onClusterEnter}
              onMouseLeave={hover.onLeave}
            />
          ))}
          {/* Edges */}
          {visibleEdges.map((e, i) => {
            const a = displayPositions.get(e.fromKey);
            const b = displayPositions.get(e.toKey);
            if (!a || !b) return null;
            const edgeOn = hover.dimming && (hover.connectedSet.has(e.fromKey) && hover.connectedSet.has(e.toKey));
            // When spotlight is active, only edges touching the hovered node stay bright.
            const isTouchingHover =
              hover.hoveredKey != null && (e.fromKey === hover.hoveredKey || e.toKey === hover.hoveredKey);
            const dimmed = hover.dimming && hover.hoveredKey != null && !isTouchingHover;
            return (
              <line
                key={`edge-${i}`}
                data-edge-from={e.fromKey}
                data-edge-to={e.toKey}
                x1={a.x}
                y1={a.y}
                x2={b.x}
                y2={b.y}
                stroke={SYSTEM_NODE_TYPE_COLORS[nodeByKey.get(e.fromKey)?.type ?? "custom_object"] ?? "#cbd5e1"}
                strokeOpacity={dimmed ? 0.06 : 0.5}
                strokeWidth={1.5}
                strokeDasharray={e.access ? "6 4" : undefined}
                className={cn(edgeOn && "graph-hover-on")}
              >
                <title>
                  {nodeByKey.get(e.fromKey)?.label} — {e.label} → {nodeByKey.get(e.toKey)?.label}
                </title>
              </line>
            );
          })}
        </svg>

        {/* Edge labels */}
        {visibleEdges.map((e, i) => {
          const pos = edgeLabelPos(e.fromKey, e.toKey);
          if (!pos) return null;
          // Demo's labelMode 'neighborhood': only edges touching inspected node.
          // We show all for now (labelMode 'all' is the demo default).
          const isTouchingHover =
            hover.hoveredKey != null && (e.fromKey === hover.hoveredKey || e.toKey === hover.hoveredKey);
          return (
            <div
              key={`elabel-${i}`}
              className={cn("ontology-edge-label", e.access && "ontology-edge-label-access", isTouchingHover && "graph-hover-on")}
              data-edge-from={e.fromKey}
              data-edge-to={e.toKey}
              style={{ position: "absolute", left: pos.x, top: pos.y }}
              title={`${nodeByKey.get(e.fromKey)?.label} — ${e.label} → ${nodeByKey.get(e.toKey)?.label}`}
            >
              {e.access ? "🔒 " : ""}
              {e.label}
            </div>
          );
        })}

        {/* Pill nodes */}
        {model.nodes.map((n) => {
          const p = displayPositions.get(n.key);
          if (!p) return null;
          if (hiddenTypes.has(n.type)) return null;
          const isDimmed = hiddenTypes.has(n.type);
          const hoverOn = hover.dimming && hover.connectedSet.has(n.key);
          return (
            <PillNode
              key={n.key}
              node={n}
              x={p.x}
              y={p.y}
              inspecting={inspectingKey === n.key}
              dimmed={isDimmed}
              hoverOn={hoverOn}
              spotlightActive={hover.dimming}
              fullLabels={fullLabels}
              onClick={onNodeClick}
              onDoubleClick={onNodeDoubleClick}
              onMouseEnter={hover.onPillEnter}
              onMouseLeave={hover.onLeave}
            />
          );
        })}
      </div>
    </div>
  );
});
