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
 * `Fit` computes the bounding box and fits; `Reset` restores 1:1.
 */
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { cn } from "../ui/cn";
import { GRAPH_CLUSTER_ORDER, SYSTEM_NODE_TYPE_COLORS, type GraphModel, type GraphNode } from "./graphTypes";
import { layoutGraphPositions, type GraphPosition } from "./useGraphLayout";
import { useHoverDimming, type HoverDimming } from "./useHoverDimming";
import { PillNode } from "./PillNode";
import { ClusterBoundary } from "./ClusterBoundary";
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
    className,
  } = props;

  const containerRef = useRef<HTMLDivElement>(null);
  const [containerSize, setContainerSize] = useState({ w: 1200, h: 800 });
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null);
  const pinchRef = useRef<{ dist: number; zoom: number } | null>(null);

  // Measure container for layout dimensions.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      const rect = entries[0].contentRect;
      if (rect.width > 0 && rect.height > 0) {
        setContainerSize({ w: Math.max(400, rect.width), h: Math.max(400, rect.height) });
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

  // --- Zoom / pan ---

  const clampZoom = useCallback((z: number) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z)), []);

  const zoomIn = useCallback(() => setZoom((z) => clampZoom(z * 1.25)), [clampZoom]);
  const zoomOut = useCallback(() => setZoom((z) => clampZoom(z / 1.25)), [clampZoom]);

  const fit = useCallback(() => {
    if (!positions.size) return;
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    positions.forEach((p) => {
      minX = Math.min(minX, p.x);
      maxX = Math.max(maxX, p.x);
      minY = Math.min(minY, p.y);
      maxY = Math.max(maxY, p.y);
    });
    const pad = 80;
    const bw = maxX - minX + pad * 2;
    const bh = maxY - minY + pad * 2;
    const z = clampZoom(Math.min(W / bw, H / bh, 1.5));
    setZoom(z);
    setPan({
      x: W / 2 - ((minX + maxX) / 2) * z,
      y: H / 2 - ((minY + maxY) / 2) * z,
    });
  }, [positions, W, H, clampZoom]);

  const reset = useCallback(() => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
    hover.onLeave();
  }, [hover]);

  // Expose zoom API to parent (toolbar).
  useEffect(() => {
    if (zoomApiRef) {
      zoomApiRef.current = { zoomIn, zoomOut, fit, reset, zoom };
    }
    onZoomChange?.(zoom);
  }, [zoomApiRef, zoomIn, zoomOut, fit, reset, zoom, onZoomChange]);

  // Auto-fit on first layout.
  const fittedRef = useRef(false);
  useEffect(() => {
    if (!fittedRef.current && positions.size > 0) {
      fittedRef.current = true;
      fit();
    }
  }, [positions, fit]);

  // Wheel zoom (at cursor).
  const onWheel = useCallback(
    (e: React.WheelEvent) => {
      if (e.ctrlKey || e.metaKey) e.preventDefault();
      const rect = containerRef.current?.getBoundingClientRect();
      if (!rect) return;
      const cx = e.clientX - rect.left;
      const cy = e.clientY - rect.top;
      setZoom((z) => {
        const nz = clampZoom(z * (e.deltaY < 0 ? 1.12 : 1 / 1.12));
        // Keep the cursor point stable.
        setPan((p) => ({
          x: cx - ((cx - p.x) / z) * nz,
          y: cy - ((cy - p.y) / z) * nz,
        }));
        return nz;
      });
    },
    [clampZoom],
  );

  // Drag pan.
  const onPointerDown = useCallback((e: React.PointerEvent) => {
    // Don't start pan on pill / button / label.
    if ((e.target as HTMLElement).closest(".og-pill,.og-cluster-label,button")) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY, origX: pan.x, origY: pan.y };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }, [pan]);

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    const d = dragRef.current;
    if (!d) return;
    setPan({ x: d.origX + (e.clientX - d.startX), y: d.origY + (e.clientY - d.startY) });
  }, []);

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
      setZoom(clampZoom(p.zoom * (d / p.dist)));
    },
    [clampZoom],
  );

  const onTouchEnd = useCallback(() => {
    pinchRef.current = null;
  }, []);

  // Edge label positions (midpoint).
  const edgeLabelPos = useCallback(
    (fromKey: string, toKey: string): GraphPosition | null => {
      const a = positions.get(fromKey);
      const b = positions.get(toKey);
      if (!a || !b) return null;
      return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    },
    [positions],
  );

  const transform = `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`;

  return (
    <div
      ref={containerRef}
      className={cn("ontology-graph-canvas", hover.dimming && "graph-hover-dim", className)}
      style={{ width: "100%", height: "100%", minHeight: 480, position: "relative", overflow: "hidden", touchAction: "pan-x pan-y" }}
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
              positions={positions}
              hoverOn={hover.hoveredClusterType === type}
              spotlightActive={hover.dimming}
              onCollapse={onClusterToggle}
              onMouseEnter={hover.onClusterEnter}
              onMouseLeave={hover.onLeave}
            />
          ))}
          {/* Edges */}
          {visibleEdges.map((e, i) => {
            const a = positions.get(e.fromKey);
            const b = positions.get(e.toKey);
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
          const p = positions.get(n.key);
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
