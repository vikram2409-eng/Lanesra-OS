#!/usr/bin/env python3
"""Deploy Desktop Ontology Graph (PR 2)."""

import os

REPO = "."

def write_file(path, content):
    full = os.path.join(REPO, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w", encoding="utf-8").write(content)
    print(f"Wrote {path}")

# File: desktop/src/components/graph/graphTypes.ts
write_file("desktop/src/components/graph/graphTypes.ts", """/**
 * PR 2 — Desktop Ontology Graph: shared graph type definitions.
 *
 * Ports the demo's graph model (`graphRender`'s `model`/`ui` contract)
 * into TypeScript. Colors are ported verbatim from the demo's
 * `SYSTEM_NODE_TYPE_COLORS` (app.js line 4636) for exact mockup parity.
 */
import type { SystemNodeType } from "../../lib/types";

/** A single node in the rendered graph. */
export interface GraphNode {
  /** Unique key: `${type}:${id}` (mirrors demo's `depNodeKey`). */
  key: string;
  /** Component id (backend identifier). */
  id: string;
  /** System node type. */
  type: SystemNodeType;
  /** Display label. */
  label: string;
  /** Node count (only set on `__cluster__` collapse nodes). */
  count?: number;
  /** True for the centered root node. */
  isRoot?: boolean;
}

/** A single directed edge in the rendered graph. */
export interface GraphEdge {
  fromKey: string;
  toKey: string;
  /** Raw edge type string (e.g. "has field", "depends_on"). */
  edgeType: string;
  /** Human-readable label. */
  label: string;
  /** True for access/permission edges (rendered dashed + 🔒). */
  access?: boolean;
}

/** The full model handed to the layout + renderer. */
export interface GraphModel {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/** Verbatim port of the demo's `SYSTEM_NODE_TYPE_COLORS` (app.js:4636). */
export const SYSTEM_NODE_TYPE_COLORS: Record<SystemNodeType, string> = {
  custom_object: "#4F46E5",
  custom_field: "#64748B",
  relationship: "#D97706",
  business_rule: "#B45309",
  workflow: "#0F766E",
  screen_layout: "#475569",
  page_layout: "#475569",
  ai_agent: "#9333EA",
  execution_graph: "#0F766E",
  business_glossary_term: "#E11D48",
  metric_definition: "#0284C7",
  test_case_definition: "#9333EA",
  action_type_definition: "#475569",
};

/** Verbatim port of the demo's `SYSTEM_NODE_TYPE_LABELS` (app.js:3754). */
export const SYSTEM_NODE_TYPE_LABELS: Record<SystemNodeType, string> = {
  custom_object: "Object",
  custom_field: "Field",
  relationship: "Relationship",
  business_rule: "Business Rule",
  workflow: "Workflow",
  screen_layout: "Screen Layout",
  page_layout: "Page Layout",
  ai_agent: "AI Agent",
  execution_graph: "Agent Team",
  business_glossary_term: "Glossary Term",
  metric_definition: "Metric",
  test_case_definition: "Test Case",
  action_type_definition: "Action Type",
};

/** Verbatim port of the demo's `DEP_CLUSTER_ORDER` (app.js:4653). */
export const GRAPH_CLUSTER_ORDER: SystemNodeType[] = [
  "custom_object",
  "custom_field",
  "relationship",
  "business_rule",
  "workflow",
  "screen_layout",
  "page_layout",
  "ai_agent",
  "execution_graph",
  "business_glossary_term",
  "metric_definition",
  "test_case_definition",
  "action_type_definition",
];

/** Builds a node's stable key from type + id (mirrors demo's `depNodeKey`). */
export function graphNodeKey(nodeType: SystemNodeType, id: string): string {
  return `${nodeType}:${id}`;
}

/** Edge types the demo treats as access/permission edges (dashed + 🔒). */
const ACCESS_EDGE_TYPES = new Set(["grants_access_to", "can_read", "can_write", "can_execute"]);

/** True when the edge should render as an access edge. */
export function isAccessEdge(edgeType: string): boolean {
  return ACCESS_EDGE_TYPES.has(edgeType);
}
""")

# File: desktop/src/components/graph/useGraphLayout.ts
write_file("desktop/src/components/graph/useGraphLayout.ts", """/**
 * PR 2 — Desktop Ontology Graph: force-directed layout with per-type
 * cluster anchors.
 *
 * Line-for-line port of the demo's `graphLayoutPositions`
 * (app.js:4791). Deterministic: positions derive from a seeded RNG over
 * the sorted node keys, so the same model always lays out identically.
 */

import type { GraphModel } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";

/** Port of demo's `depHashStr` (FNV-1a, app.js:4656). */
function hashStr(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Port of demo's `depSeededRandom` (mulberry32-ish, app.js:4657). */
function seededRandom(seed: number): () => number {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface GraphLayoutOptions {
  /** Key of the node pinned to canvas center. */
  rootKey: string;
  /** Per-type cluster anchor order (GRAPH_CLUSTER_ORDER). */
  clusterOrder: SystemNodeType[];
  W: number;
  H: number;
}

export interface GraphPosition {
  x: number;
  y: number;
}

/**
 * Computes node positions. Port of `graphLayoutPositions` — keep the
 * constants in sync with the demo (K=82, LINK_LEN=185).
 */
export function layoutGraphPositions(
  model: GraphModel,
  layout: GraphLayoutOptions,
): Map<string, GraphPosition> {
  const { W, H } = layout;
  const visible = model.nodes;
  const rng = seededRandom(hashStr(visible.map((n) => n.key).sort().join("|")));

  const anchors: Record<string, GraphPosition> = {};
  (layout.clusterOrder || []).forEach((t, i) => {
    const a = (i / layout.clusterOrder.length) * Math.PI * 2 - Math.PI / 2;
    anchors[t] = { x: W / 2 + Math.cos(a) * W * 0.36, y: H / 2 + Math.sin(a) * H * 0.36 };
  });

  const pos = new Map<string, GraphPosition>();
  const rootKey = layout.rootKey;
  visible.forEach((n) => {
    if (n.key === rootKey) {
      pos.set(n.key, { x: W / 2, y: H / 2 });
      return;
    }
    const a = anchors[n.type] || { x: W / 2, y: H / 2 };
    pos.set(n.key, { x: a.x + (rng() - 0.5) * 90, y: a.y + (rng() - 0.5) * 90 });
  });

  const TICKS = visible.length > 260 ? 60 : 110;
  const K = 82;
  const LINK_LEN = 185;

  for (let t = 0; t < TICKS; t++) {
    const disp = new Map(visible.map((n) => [n.key, { x: 0, y: 0 }] as [string, GraphPosition]));

    // Repulsion between all pairs.
    for (let i = 0; i < visible.length; i++) {
      for (let j = i + 1; j < visible.length; j++) {
        const a = visible[i];
        const b = visible[j];
        const pa = pos.get(a.key)!;
        const pb = pos.get(b.key)!;
        let dx = pa.x - pb.x;
        let dy = pa.y - pb.y;
        let d2 = dx * dx + dy * dy;
        if (d2 < 1) {
          dx = rng() - 0.5;
          dy = rng() - 0.5;
          d2 = 1;
        }
        const d = Math.sqrt(d2);
        const f = Math.min((K * K) / d, 9);
        const fx = (f * dx) / d;
        const fy = (f * dy) / d;
        const da = disp.get(a.key)!;
        const db = disp.get(b.key)!;
        da.x += fx;
        da.y += fy;
        db.x -= fx;
        db.y -= fy;
      }
    }

    // Spring attraction along edges.
    model.edges.forEach((e) => {
      const pa = pos.get(e.fromKey);
      const pb = pos.get(e.toKey);
      if (!pa || !pb) return;
      const dx = pb.x - pa.x;
      const dy = pb.y - pa.y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      const f = (d - LINK_LEN) * 0.055;
      const fx = (f * dx) / d;
      const fy = (f * dy) / d;
      const da = disp.get(e.fromKey)!;
      const db = disp.get(e.toKey)!;
      da.x += fx;
      da.y += fy;
      db.x -= fx;
      db.y -= fy;
    });

    // Pull toward per-type cluster anchors.
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const p = pos.get(n.key)!;
      const d = disp.get(n.key)!;
      const a = anchors[n.type] || { x: W / 2, y: H / 2 };
      d.x += (a.x - p.x) * 0.14 + (W / 2 - p.x) * 0.006;
      d.y += (a.y - p.y) * 0.14 + (H / 2 - p.y) * 0.006;
    });

    // PR4 intra-cluster cohesion: same-type nodes pull toward their
    // type centroid each tick so every cluster reads as one tight clump.
    const centroids = new Map<string, { x: number; y: number; k: number }>();
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const c = centroids.get(n.type) || { x: 0, y: 0, k: 0 };
      const pp = pos.get(n.key)!;
      c.x += pp.x;
      c.y += pp.y;
      c.k++;
      centroids.set(n.type, c);
    });
    centroids.forEach((c) => {
      c.x /= c.k;
      c.y /= c.k;
    });
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const c = centroids.get(n.type);
      if (!c || c.k < 2) return;
      const pp = pos.get(n.key)!;
      const dd = disp.get(n.key)!;
      dd.x += (c.x - pp.x) * 0.06;
      dd.y += (c.y - pp.y) * 0.06;
    });

    // Integrate + clamp to canvas.
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const p = pos.get(n.key)!;
      const d = disp.get(n.key)!;
      p.x = Math.max(34, Math.min(W - 34, p.x + d.x * 0.5));
      p.y = Math.max(52, Math.min(H - 30, p.y + d.y * 0.5));
    });
  }

  return pos;
}
""")

# File: desktop/src/components/graph/useGraphModel.ts
write_file("desktop/src/components/graph/useGraphModel.ts", """/**
 * PR 2 — Desktop Ontology Graph: react-query data hook that assembles a
 * `GraphModel` from the system-graph endpoints.
 *
 * Mirrors the demo's model assembly: the root node plus its neighborhood
 * (1-hop dependencies/dependents, or the transitive impact/lineage
 * closures when those toggles are on). Hidden types are filtered out of
 * the returned model so layout + rendering never see them.
 */
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { SystemGraphHit, SystemNodeType } from "../../lib/types";
import {
  graphNodeKey,
  isAccessEdge,
  type GraphEdge,
  type GraphModel,
  type GraphNode,
} from "./graphTypes";

export type GraphDepth = "1" | "2" | "3" | "all";

export interface UseGraphModelOptions {
  rootType: SystemNodeType | null;
  rootId: string | null;
  depth: GraphDepth;
  showImpact: boolean;
  showLineage: boolean;
  hiddenTypes: Set<SystemNodeType>;
}

export interface UseGraphModelResult {
  model: GraphModel;
  rootKey: string | null;
  isLoading: boolean;
  error: unknown;
}

function hitToNode(hit: SystemGraphHit): GraphNode {
  return {
    key: graphNodeKey(hit.node.node_type, hit.node.component_id),
    id: hit.node.component_id,
    type: hit.node.node_type,
    label: hit.node.label,
  };
}

/** Human-readable edge label (mirrors demo's `edgeTypeLabel`). */
function edgeLabel(edgeType: string): string {
  return edgeType
    .split("_")
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(" ");
}

export function useGraphModel(opts: UseGraphModelOptions): UseGraphModelResult {
  const { rootType, rootId, depth, showImpact, showLineage, hiddenTypes } = opts;
  const enabled = !!rootType && !!rootId;
  const rootKey = enabled ? graphNodeKey(rootType!, rootId!) : null;

  const rootNode = useQuery({
    queryKey: ["systemNode", rootType, rootId],
    queryFn: () => api.getSystemNode(rootType!, rootId!),
    enabled,
  });

  const dependents = useQuery({
    queryKey: ["systemNodeDependents", rootType, rootId],
    queryFn: () => api.getSystemNodeDependents(rootType!, rootId!),
    enabled,
  });

  const dependencies = useQuery({
    queryKey: ["systemNodeDependencies", rootType, rootId],
    queryFn: () => api.getSystemNodeDependencies(rootType!, rootId!),
    enabled,
  });

  const impact = useQuery({
    queryKey: ["systemNodeImpact", rootType, rootId],
    queryFn: () => api.getSystemNodeImpact(rootType!, rootId!),
    enabled: enabled && showImpact,
  });

  const lineage = useQuery({
    queryKey: ["systemNodeLineage", rootType, rootId],
    queryFn: () => api.getSystemNodeLineage(rootType!, rootId!),
    enabled: enabled && showLineage,
  });

  const model = useMemo<GraphModel>(() => {
    if (!enabled || !rootNode.data) return { nodes: [], edges: [] };

    const maxDepth = depth === "all" ? Infinity : parseInt(depth, 10);
    const nodeMap = new Map<string, GraphNode>();
    const edges: GraphEdge[] = [];

    // Root node is always present.
    const root: GraphNode = {
      key: rootKey!,
      id: rootNode.data.component_id,
      type: rootNode.data.node_type,
      label: rootNode.data.label,
      isRoot: true,
    };
    nodeMap.set(root.key, root);

    const addHit = (hit: SystemGraphHit, direction: "dependents" | "dependencies") => {
      if (hit.depth > maxDepth) return;
      if (hiddenTypes.has(hit.node.node_type)) return;
      const node = hitToNode(hit);
      if (!nodeMap.has(node.key)) nodeMap.set(node.key, node);
      // Edge direction: dependents point AT the root; dependencies are
      // pointed at BY the root.
      const fromKey = direction === "dependents" ? node.key : rootKey!;
      const toKey = direction === "dependents" ? rootKey! : node.key;
      edges.push({
        fromKey,
        toKey,
        edgeType: hit.edge_type,
        label: edgeLabel(hit.edge_type),
        access: isAccessEdge(hit.edge_type),
      });
    };

    if (showImpact || showLineage) {
      // Transitive closures carry their own depth; filter by depth here.
      // Direction is ambiguous in the closure, so edges point toward the
      // root for impact (upstream) and away for lineage (downstream) —
      // matching the demo's cloud semantics.
      (showImpact ? impact.data ?? [] : []).forEach((h) => addHit(h, "dependents"));
      (showLineage ? lineage.data ?? [] : []).forEach((h) => addHit(h, "dependencies"));
    } else {
      (dependents.data ?? []).forEach((h) => addHit(h, "dependents"));
      (dependencies.data ?? []).forEach((h) => addHit(h, "dependencies"));
    }

    return { nodes: Array.from(nodeMap.values()), edges };
  }, [
    enabled,
    rootKey,
    rootNode.data,
    dependents.data,
    dependencies.data,
    impact.data,
    lineage.data,
    depth,
    showImpact,
    showLineage,
    hiddenTypes,
  ]);

  return {
    model,
    rootKey,
    isLoading:
      rootNode.isLoading ||
      dependents.isLoading ||
      dependencies.isLoading ||
      (showImpact && impact.isLoading) ||
      (showLineage && lineage.isLoading),
    error: rootNode.error ?? dependents.error ?? dependencies.error ?? impact.error ?? lineage.error,
  };
}
""")

# File: desktop/src/components/graph/useHoverDimming.ts
write_file("desktop/src/components/graph/useHoverDimming.ts", """/**
 * PR 2 — Desktop Ontology Graph: hover spotlight state.
 *
 * React port of the demo's `__graphHoverDelegated` IIFE (app.js:5022).
 * The demo uses document-level mouseover/mouseout delegation; here we
 * use React state (cleaner in React, same visual result via CSS classes).
 *
 * Hovering a pill computes the connected set (the node + its 1-hop
 * neighbors from the rendered edges) and exposes it; `GraphCanvas`
 * derives `graph-hover-dim` / `graph-hover-on` classes from it.
 * Hovering a cluster label highlights the whole type group instead.
 */
import { useCallback, useMemo, useState } from "react";

import type { GraphEdge, GraphModel } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";

export interface HoverDimming {
  /** Key of the hovered pill, if any. */
  hoveredKey: string | null;
  /** Type of the hovered cluster label, if any. */
  hoveredClusterType: SystemNodeType | null;
  /** Node keys that stay bright (hovered + 1-hop neighbors). */
  connectedSet: Set<string>;
  /** Node types present in the connected set (for legend sync). */
  connectedTypes: Set<SystemNodeType>;
  /** Call on pill mouse-enter. */
  onPillEnter: (key: string) => void;
  /** Call on cluster-label mouse-enter. */
  onClusterEnter: (type: SystemNodeType) => void;
  /** Call on mouse-leave of pill / label / canvas. */
  onLeave: () => void;
  /** True while any spotlight is active. */
  dimming: boolean;
}

export function useHoverDimming(model: GraphModel): HoverDimming {
  const [hoveredKey, setHoveredKey] = useState<string | null>(null);
  const [hoveredClusterType, setHoveredClusterType] = useState<SystemNodeType | null>(null);

  // Adjacency from rendered edges.
  const adjacency = useMemo(() => {
    const map = new Map<string, Set<string>>();
    const link = (a: string, b: string) => {
      if (!map.has(a)) map.set(a, new Set());
      map.get(a)!.add(b);
    };
    model.edges.forEach((e: GraphEdge) => {
      link(e.fromKey, e.toKey);
      link(e.toKey, e.fromKey);
    });
    return map;
  }, [model]);

  const nodeTypeByKey = useMemo(() => {
    const map = new Map<string, SystemNodeType>();
    model.nodes.forEach((n) => map.set(n.key, n.type));
    return map;
  }, [model]);

  const connectedSet = useMemo(() => {
    if (hoveredClusterType) {
      // Whole type group stays bright.
      const s = new Set<string>();
      model.nodes.forEach((n) => {
        if (n.type === hoveredClusterType) s.add(n.key);
      });
      return s;
    }
    if (!hoveredKey) return new Set<string>();
    const s = new Set<string>([hoveredKey]);
    adjacency.get(hoveredKey)?.forEach((k) => s.add(k));
    return s;
  }, [hoveredKey, hoveredClusterType, adjacency, model.nodes]);

  const connectedTypes = useMemo(() => {
    const s = new Set<SystemNodeType>();
    connectedSet.forEach((k) => {
      const t = nodeTypeByKey.get(k);
      if (t) s.add(t);
    });
    return s;
  }, [connectedSet, nodeTypeByKey]);

  const onPillEnter = useCallback((key: string) => {
    setHoveredClusterType(null);
    setHoveredKey(key);
  }, []);

  const onClusterEnter = useCallback((type: SystemNodeType) => {
    setHoveredKey(null);
    setHoveredClusterType(type);
  }, []);

  const onLeave = useCallback(() => {
    setHoveredKey(null);
    setHoveredClusterType(null);
  }, []);

  return {
    hoveredKey,
    hoveredClusterType,
    connectedSet,
    connectedTypes,
    onPillEnter,
    onClusterEnter,
    onLeave,
    dimming: hoveredKey !== null || hoveredClusterType !== null,
  };
}
""")

# File: desktop/src/components/graph/GraphCanvas.tsx
write_file("desktop/src/components/graph/GraphCanvas.tsx", """/**
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
""")

# File: desktop/src/components/graph/PillNode.tsx
write_file("desktop/src/components/graph/PillNode.tsx", """/**
 * PR 2 — Desktop Ontology Graph: pill node.
 *
 * React port of the demo's `og-pill` renderer (app.js ~4951-4960).
 * Renders as an absolutely-positioned pill: type dot + label (root
 * pills are solid type-color with white text). Carries `data-node-key`
 * and `data-og-type` for hover/legend wiring.
 */
import { memo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphNode } from "./graphTypes";

export interface PillNodeProps {
  node: GraphNode;
  x: number;
  y: number;
  /** Currently inspected node key (outline ring). */
  inspecting?: boolean;
  /** Dimmed by hidden-type filter. */
  dimmed?: boolean;
  /** In the hover spotlight's connected set. */
  hoverOn?: boolean;
  /** Spotlight active on the canvas. */
  spotlightActive?: boolean;
  onClick?: (node: GraphNode) => void;
  onDoubleClick?: (node: GraphNode) => void;
  onMouseEnter?: (key: string) => void;
  onMouseLeave?: () => void;
}

function truncate(label: string, max = 18): string {
  return label.length > max ? label.slice(0, max - 1) + "…" : label;
}

export const PillNode = memo(function PillNode(props: PillNodeProps) {
  const { node, x, y, inspecting, dimmed, hoverOn, spotlightActive, onClick, onDoubleClick, onMouseEnter, onMouseLeave } = props;
  const color = SYSTEM_NODE_TYPE_COLORS[node.type] ?? "#888";
  const typeLabel = SYSTEM_NODE_TYPE_LABELS[node.type] ?? node.type;

  const cls = cn(
    "og-pill",
    node.isRoot && "og-pill-root",
    inspecting && "og-pill-inspecting",
    dimmed && "og-pill-dimmed",
    hoverOn && "graph-hover-on",
    node.id === "__cluster__" && "og-pill-cluster",
  );

  const style: React.CSSProperties = {
    left: x,
    top: y,
    ...(node.isRoot ? { background: color, borderColor: color } : {}),
    // When the spotlight is active but this pill isn't connected, the
    // CSS `.graph-hover-dim .og-pill:not(.graph-hover-on)` handles the
    // fade — no inline opacity needed.
    ...(spotlightActive && !hoverOn ? {} : {}),
  };

  return (
    <div
      className={cls}
      data-node-key={node.key}
      data-og-type={node.type}
      style={style}
      title={`${typeLabel}: ${node.label}`}
      role="button"
      tabIndex={0}
      onClick={() => onClick?.(node)}
      onDoubleClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        onDoubleClick?.(node);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick?.(node);
        }
      }}
      onMouseEnter={() => onMouseEnter?.(node.key)}
      onMouseLeave={() => onMouseLeave?.()}
    >
      {!node.isRoot && <span className="og-pill-dot" style={{ background: color }} />}
      <span className="og-pill-label">
        {truncate(node.label)}
        {node.id === "__cluster__" && node.count != null ? ` · ${node.count}` : ""}
      </span>
    </div>
  );
});
""")

# File: desktop/src/components/graph/ClusterBoundary.tsx
write_file("desktop/src/components/graph/ClusterBoundary.tsx", """/**
 * PR 2 — Desktop Ontology Graph: dashed-ellipse cluster boundary.
 *
 * React port of the demo's `__clusterBoundaries` ellipse renderer
 * (app.js ~4903-4911). Geometry ported verbatim: padX=58, padY=44,
 * min rx=70/ry=48, stroke-dasharray "8 6", type color at 60% opacity,
 * 4% fill. The label button sits at the top-center of the ellipse.
 */
import { memo, useMemo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphNode } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";
import type { GraphPosition } from "./useGraphLayout";

export interface ClusterBoundaryProps {
  type: SystemNodeType;
  members: GraphNode[];
  positions: Map<string, GraphPosition>;
  /** In the hover spotlight's connected set. */
  hoverOn?: boolean;
  /** Spotlight active on the canvas. */
  spotlightActive?: boolean;
  onCollapse?: (type: SystemNodeType) => void;
  onMouseEnter?: (type: SystemNodeType) => void;
  onMouseLeave?: () => void;
}

const PAD_X = 58;
const PAD_Y = 44;
const MIN_RX = 70;
const MIN_RY = 48;

export const ClusterBoundary = memo(function ClusterBoundary(props: ClusterBoundaryProps) {
  const { type, members, positions, hoverOn, spotlightActive, onCollapse, onMouseEnter, onMouseLeave } = props;

  const geom = useMemo(() => {
    const pts = members
      .map((n) => positions.get(n.key))
      .filter((p): p is GraphPosition => !!p);
    if (!pts.length) return null;
    const xs = pts.map((p) => p.x);
    const ys = pts.map((p) => p.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return {
      cx: (minX + maxX) / 2,
      cy: (minY + maxY) / 2,
      rx: Math.max(MIN_RX, (maxX - minX) / 2 + PAD_X),
      ry: Math.max(MIN_RY, (maxY - minY) / 2 + PAD_Y),
      labelX: (minX + maxX) / 2,
      labelY: minY - 14,
    };
  }, [members, positions]);

  if (!geom) return null;

  const color = SYSTEM_NODE_TYPE_COLORS[type] ?? "#888";
  const label = `${SYSTEM_NODE_TYPE_LABELS[type] ?? type} · ${members.length}`;

  // Dim non-hovered clusters while the spotlight is active (demo sets
  // inline opacity 0.25 on unrelated ellipses).
  const dimmed = spotlightActive && !hoverOn;

  return (
    <>
      <ellipse
        data-cluster-type={type}
        cx={geom.cx}
        cy={geom.cy}
        rx={geom.rx}
        ry={geom.ry}
        fill={color}
        fillOpacity={0.04}
        stroke={color}
        strokeOpacity={0.6}
        strokeWidth={1.5}
        strokeDasharray="8 6"
        style={{ opacity: dimmed ? 0.25 : undefined, transition: "opacity 0.2s ease" }}
        className={cn(hoverOn && "graph-hover-on")}
      />
      <button
        type="button"
        className="ontology-halo-label og-cluster-label"
        data-cluster-type={type}
        style={{ left: geom.labelX, top: geom.labelY, color }}
        title={`Collapse ${label} into a single node`}
        onClick={() => onCollapse?.(type)}
        onMouseEnter={() => onMouseEnter?.(type)}
        onMouseLeave={() => onMouseLeave?.()}
      >
        {label} ▾
      </button>
    </>
  );
});
""")

# File: desktop/src/components/graph/GraphToolbar.tsx
write_file("desktop/src/components/graph/GraphToolbar.tsx", """/**
 * PR 2 — Desktop Ontology Graph: compact 8-group toolbar.
 *
 * React port of the demo's `og-toolbar-card` (app.js toolbar template +
 * styles.css). All controls in one card, 1–2 slim rows on desktop via
 * flex-wrap; stacks on mobile.
 *
 * Groups: Type | Component | Search | Find path | Zoom | Depth |
 * Impact/Lineage | Fields/Simulate/Copy link
 */
import { memo, useState } from "react";

import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Select } from "../ui/select";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import { SYSTEM_NODE_TYPE_LABELS } from "./graphTypes";
import { GRAPH_CLUSTER_ORDER } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";
import type { GraphDepth } from "./useGraphModel";

export interface ComponentOption {
  id: string;
  label: string;
}

export interface GraphToolbarProps {
  // Type + Component
  nodeType: SystemNodeType;
  onNodeTypeChange: (t: SystemNodeType) => void;
  componentId: string;
  onComponentChange: (id: string) => void;
  componentOptions: ComponentOption[];
  // Search
  searchQuery: string;
  onSearchQueryChange: (q: string) => void;
  searchResults: ComponentOption[];
  onSearchSelect: (opt: ComponentOption) => void;
  // Find path
  pathMode: boolean;
  onTogglePathMode: () => void;
  pathFromLabel?: string | null;
  pathToLabel?: string | null;
  onClearPath?: () => void;
  // Zoom
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
  onReset: () => void;
  // Depth
  depth: GraphDepth;
  onDepthChange: (d: GraphDepth) => void;
  // Impact / Lineage
  showImpact: boolean;
  onToggleImpact: () => void;
  showLineage: boolean;
  onToggleLineage: () => void;
  // Fields / Simulate / Copy link
  showFields: boolean;
  onToggleFields: () => void;
  onSimulate?: () => void;
  onCopyLink?: () => void;
}

export const GraphToolbar = memo(function GraphToolbar(props: GraphToolbarProps) {
  const {
    nodeType, onNodeTypeChange,
    componentId, onComponentChange, componentOptions,
    searchQuery, onSearchQueryChange, searchResults, onSearchSelect,
    pathMode, onTogglePathMode, pathFromLabel, pathToLabel, onClearPath,
    zoom, onZoomIn, onZoomOut, onFit, onReset,
    depth, onDepthChange,
    showImpact, onToggleImpact, showLineage, onToggleLineage,
    showFields, onToggleFields, onSimulate, onCopyLink,
  } = props;

  const [searchFocused, setSearchFocused] = useState(false);

  return (
    <div className="og-toolbar-card" role="toolbar" aria-label="Graph tools">
      {/* Type */}
      <div className="tool-group">
        <Label className="tool-inline">
          Type{" "}
          <Select
            value={nodeType}
            onValueChange={(v) => onNodeTypeChange(v as SystemNodeType)}
            aria-label="Node type"
            options={GRAPH_CLUSTER_ORDER.map((t) => ({ value: t, label: SYSTEM_NODE_TYPE_LABELS[t] }))}
          />
        </Label>
      </div>

      {/* Component */}
      <div className="tool-group">
        <Label className="tool-inline">
          Component{" "}
          <Select
            value={componentId}
            onValueChange={(v) => onComponentChange(v)}
            aria-label="Component"
            placeholder="Select…"
            options={componentOptions.map((o) => ({ value: o.id, label: o.label }))}
          />
        </Label>
      </div>

      {/* Search */}
      <div className="tool-group">
        <div className="search-wrap" style={{ position: "relative", minWidth: 200 }}>
          <Input
            type="search"
            value={searchQuery}
            onChange={(e) => onSearchQueryChange(e.target.value)}
            onFocus={() => setSearchFocused(true)}
            onBlur={() => setTimeout(() => setSearchFocused(false), 150)}
            placeholder="Search components…"
            aria-label="Search components"
          />
          {searchFocused && searchResults.length > 0 && (
            <div className="ontology-search-results">
              {searchResults.map((r) => (
                <button key={r.id} type="button" onClick={() => onSearchSelect(r)}>
                  {r.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Find path */}
      <div className="tool-group">
        <Button
          variant={pathMode ? "default" : "secondary"}
          size="sm"
          onClick={onTogglePathMode}
          title="Click two nodes on the canvas to trace the shortest path between them"
        >
          Find path
        </Button>
        {(pathFromLabel || pathToLabel) && (
          <span className="path-fields" style={{ fontSize: 12, color: "var(--muted,#64748b)" }}>
            {pathFromLabel ?? "…"} → {pathToLabel ?? "…"}
            {onClearPath && (
              <Button variant="ghost" size="sm" onClick={onClearPath} aria-label="Clear path">
                ✕
              </Button>
            )}
          </span>
        )}
      </div>

      {/* Zoom */}
      <div className="tool-group">
        <div className="zoom-row" role="group" aria-label="Viewport" style={{ display: "flex", gap: 4 }}>
          <Button variant="outline" size="sm" onClick={onZoomOut} title="Zoom out" aria-label="Zoom out">
            −
          </Button>
          <Button variant="outline" size="sm" onClick={onFit} title="Fit graph to viewport">
            Fit
          </Button>
          <Button variant="outline" size="sm" onClick={onZoomIn} title="Zoom in" aria-label="Zoom in">
            +
          </Button>
          <Button variant="outline" size="sm" onClick={onReset} title="Reset zoom, pan, path and simulation">
            Reset
          </Button>
        </div>
        <span className="zoom-readout" data-zoom-readout>
          {Math.round(zoom * 100)}%
        </span>
      </div>

      {/* Depth */}
      <div className="tool-group">
        <Label className="tool-inline">
          Depth{" "}
          <Select
            value={depth}
            onValueChange={(v) => onDepthChange(v as GraphDepth)}
            aria-label="Depth"
            options={(["1", "2", "3", "all"] as GraphDepth[]).map((d) => ({
              value: d,
              label: d === "all" ? "All" : d,
            }))}
          />
        </Label>
      </div>

      {/* Impact / Lineage */}
      <div className="tool-group">
        <button
          type="button"
          className={`toggle-chip${showImpact ? " active" : ""}`}
          onClick={onToggleImpact}
          aria-pressed={showImpact}
        >
          Impact
        </button>
        <button
          type="button"
          className={`toggle-chip${showLineage ? " active" : ""}`}
          onClick={onToggleLineage}
          aria-pressed={showLineage}
        >
          Lineage
        </button>
      </div>

      {/* Fields / Simulate / Copy link */}
      <div className="tool-group">
        <Label className="ontology-fields-toggle">
          <Checkbox checked={showFields} onCheckedChange={() => onToggleFields()} />
          Fields
        </Label>
        {onSimulate && (
          <Button variant="default" size="sm" onClick={onSimulate} title="Simulate a change">
            Simulate change
          </Button>
        )}
        {onCopyLink && (
          <Button variant="secondary" size="sm" onClick={onCopyLink} title="Copy a shareable link to this view">
            Copy link
          </Button>
        )}
      </div>
    </div>
  );
});
""")

# File: desktop/src/components/graph/GraphLegend.tsx
write_file("desktop/src/components/graph/GraphLegend.tsx", """/**
 * PR 2 — Desktop Ontology Graph: type legend with counts.
 *
 * React port of the demo's legend (app.js:4641). Chips double as
 * toggle-filters (click dims the type) and as the spotlight mirror:
 * `highlightedTypes` (from `useHoverDimming`) adds `legend-highlight`
 * / `legend-dim` classes, porting the demo's `__syncLegend`.
 */
import { memo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphModel } from "./graphTypes";
import { GRAPH_CLUSTER_ORDER } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";

export interface GraphLegendProps {
  model: GraphModel;
  hiddenTypes: Set<SystemNodeType>;
  onToggleType: (t: SystemNodeType) => void;
  /** Types in the hover spotlight's neighborhood (legend sync). */
  highlightedTypes?: Set<SystemNodeType>;
  /** Spotlight active (any hover). */
  spotlightActive?: boolean;
}

export const GraphLegend = memo(function GraphLegend(props: GraphLegendProps) {
  const { model, hiddenTypes, onToggleType, highlightedTypes, spotlightActive } = props;

  // Count visible nodes per type.
  const counts = new Map<SystemNodeType, number>();
  model.nodes.forEach((n) => {
    if (n.id === "__cluster__") return;
    counts.set(n.type, (counts.get(n.type) ?? 0) + 1);
  });

  // Only show types present in the model (plus keep cluster order).
  const types = GRAPH_CLUSTER_ORDER.filter((t) => counts.has(t));

  if (!types.length) return null;

  return (
    <div className="ontology-legend" role="group" aria-label="Node type legend">
      {types.map((t) => {
        const hidden = hiddenTypes.has(t);
        const highlighted = spotlightActive && highlightedTypes?.has(t);
        const dimmed = spotlightActive && highlightedTypes && !highlightedTypes.has(t);
        return (
          <button
            key={t}
            type="button"
            className={cn(
              "ontology-legend-item",
              hidden && "ontology-legend-item-off",
              highlighted && "legend-highlight",
              dimmed && "legend-dim",
            )}
            data-legend-toggle={t}
            onClick={() => onToggleType(t)}
            title={`${hidden ? "Show" : "Hide"} ${SYSTEM_NODE_TYPE_LABELS[t]}${counts.get(t) ? ` (${counts.get(t)} shown)` : ""}`}
          >
            <span
              style={{
                display: "inline-block",
                flexShrink: 0,
                width: 10,
                height: 10,
                borderRadius: "50%",
                background: SYSTEM_NODE_TYPE_COLORS[t],
              }}
            />
            {SYSTEM_NODE_TYPE_LABELS[t]}
            {counts.get(t) ? <span className="ontology-legend-count">{counts.get(t)}</span> : null}
          </button>
        );
      })}
    </div>
  );
});
""")

# File: desktop/src/components/graph/graph.css
write_file("desktop/src/components/graph/graph.css", """/**
 * PR 2 — Desktop Ontology Graph: styles.
 *
 * Ported from the demo's styles.css (og-pill, og-toolbar-card,
 * og-cluster-label, hover spotlight, legend sync). Uses CSS variables
 * with fallbacks so it works with or without the PR 1 token theme.
 */

/* Canvas */
.ontology-graph-canvas {
  position: relative;
  background-color: var(--canvas, #fbfcfe);
  background-image: radial-gradient(rgba(100, 116, 139, 0.18) 1px, transparent 1px);
  background-size: 20px 20px;
  border-radius: 12px;
  overflow: hidden;
  will-change: transform;
}
.og-pan-inner {
  will-change: transform;
}

/* Toolbar card */
.og-toolbar-card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  padding: 12px 16px;
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 16px;
  margin-bottom: 12px;
}
.og-toolbar-card .tool-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.og-toolbar-card .tool-group + .tool-group {
  padding-left: 16px;
  border-left: 1px solid #f3f4f6;
}
.og-toolbar-card .tool-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
}
.zoom-readout,
[data-zoom-readout] {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 700;
  color: #64748b;
  min-width: 44px;
  text-align: center;
}
.toggle-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  padding: 0 12px;
  height: 32px;
  border: 1px solid #e2e8f0;
  border-radius: 999px;
  font-weight: 600;
  font-size: 12px;
  background: #fff;
  color: inherit;
  transition: all 0.18s ease;
}
.toggle-chip::before {
  content: "";
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #cbd5e1;
  transition: background 0.18s ease;
}
.toggle-chip.active {
  color: #3730a3;
  background: #eef2ff;
  border-color: #a5b4fc;
}
.toggle-chip.active::before {
  background: #4f46e5;
}
.toggle-chip:hover {
  border-color: #cbd5e1;
}
.ontology-fields-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  font-size: 12px;
  white-space: nowrap;
  cursor: pointer;
}
.ontology-search-results {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 50;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  margin-top: 4px;
  max-height: 240px;
  overflow: auto;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);
}
.ontology-search-results button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  border: 0;
  background: none;
  font-size: 12px;
  cursor: pointer;
}
.ontology-search-results button:hover {
  background: #f1f5f9;
}

/* Pill nodes */
.og-pill {
  position: absolute;
  transform: translate(-50%, -50%);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 9999px;
  background: #fff;
  border: 1.5px solid #e5e7eb;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  font-size: 12px;
  font-weight: 500;
  color: #1f2937;
  white-space: nowrap;
  cursor: pointer;
  transition: opacity 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
  z-index: 2;
}
.og-pill:hover {
  transform: translate(-50%, -50%) scale(1.08);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 10;
}
.og-pill-root {
  background: #4f46e5 !important;
  border-color: #4f46e5 !important;
  color: #fff !important;
  font-weight: 600;
  font-size: 13px;
  padding: 8px 16px;
  box-shadow: 0 2px 8px rgba(79, 70, 229, 0.3);
}
.og-pill-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.og-pill-label {
  line-height: 1.2;
}
.og-pill-cluster {
  border-style: dashed;
}
.og-pill-inspecting {
  outline: 2px solid #4f46e5;
  outline-offset: 2px;
}
.og-pill-path {
  background: #eef2ff !important;
  border-color: #4f46e5 !important;
}
.og-pill-dimmed {
  opacity: 0.25;
}

/* Hover spotlight */
.ontology-graph-canvas.graph-hover-dim .og-pill:not(.graph-hover-on) {
  opacity: 0.18;
}
.og-pill.graph-hover-on {
  opacity: 1 !important;
}
.ontology-graph-canvas.graph-hover-dim svg line:not(.graph-hover-on) {
  stroke-opacity: 0.06 !important;
}
.ontology-graph-canvas.graph-hover-dim .ontology-edge-label:not(.graph-hover-on) {
  opacity: 0.05;
}
.ontology-graph-canvas svg line {
  transition: opacity 0.2s ease, stroke 0.2s ease;
}

/* Cluster boundaries */
.ontology-halo-label {
  position: absolute;
  transform: translate(-50%, -50%);
  border: 1px solid #e2e8f0;
  background: #fff;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 11px;
  cursor: pointer;
  z-index: 2;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.06);
  white-space: nowrap;
  transition: transform 0.18s ease, border-color 0.18s ease;
}
.ontology-halo-label:hover {
  transform: translate(-50%, -50%) scale(1.05);
}
.og-cluster-label {
  border: 1px solid color-mix(in srgb, currentColor 35%, transparent) !important;
  background: color-mix(in srgb, currentColor 8%, #fff) !important;
  font-size: 10px !important;
  font-weight: 700 !important;
  letter-spacing: 0.02em;
}
.og-cluster-label:hover {
  border-color: currentColor !important;
  transform: translate(-50%, -50%) scale(1.05);
}

/* Edge labels */
.ontology-edge-label {
  position: absolute;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #e5e7eb;
  border-radius: 9999px;
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 600;
  color: #6b7280;
  transform: translate(-50%, -50%);
  white-space: nowrap;
  pointer-events: none;
  transition: opacity 0.2s ease;
  z-index: 1;
}
.ontology-edge-label-access {
  border-color: #fbbf24;
  color: #92400e;
}

/* Legend */
.ontology-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 10px 12px;
  border-top: 1px solid #e2e8f0;
  background: #fff;
}
.ontology-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid #e2e8f0;
  background: #fff;
  color: #64748b;
  border-radius: 999px;
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}
.ontology-legend-item:hover {
  border-color: #a5b4fc;
  color: #3730a3;
}
.ontology-legend-item-off {
  opacity: 0.4;
  text-decoration: line-through;
}
.ontology-legend-count {
  background: #f1f5f9;
  border-radius: 4px;
  padding: 1px 6px;
  font-size: 10px;
  font-weight: 700;
  color: #475569;
}
.ontology-legend-item.legend-dim {
  opacity: 0.3;
}
.ontology-legend-item.legend-highlight {
  box-shadow: 0 0 0 2px rgba(79, 70, 229, 0.35);
  border-color: #4f46e5;
}

/* Stage header */
.og-stage-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 8px;
}
.og-stage-title {
  font-size: 20px;
  font-weight: 600;
  color: #111827;
  margin: 0;
}
.og-stage-chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.og-pill-chip {
  background: #f3f4f6;
  border-radius: 9999px;
  padding: 4px 12px;
  font-size: 12px;
  color: #4b5563;
  font-weight: 500;
}

/* Responsive */
@media (max-width: 768px) {
  .og-toolbar-card {
    padding: 12px;
  }
  .og-toolbar-card .tool-group {
    width: 100%;
  }
  .og-toolbar-card .tool-group + .tool-group {
    border-left: 0;
    border-top: 1px solid #f3f4f6;
    padding-left: 0;
    padding-top: 8px;
  }
  .og-pill {
    font-size: 11px;
    padding: 4px 10px;
    max-width: 44vw;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .og-pill-root {
    max-width: 54vw;
  }
}
""")

# File: desktop/src/components/graph/index.ts
write_file("desktop/src/components/graph/index.ts", """/**
 * PR 2 — Desktop Ontology Graph: shared graph engine.
 *
 * One module used by BOTH the Ontology Graph (Dependency Explorer) and
 * Object 360 (PR 3), mirroring the demo's shared `graphRender`.
 */
export {
  SYSTEM_NODE_TYPE_COLORS,
  SYSTEM_NODE_TYPE_LABELS,
  GRAPH_CLUSTER_ORDER,
  graphNodeKey,
  isAccessEdge,
  type GraphNode,
  type GraphEdge,
  type GraphModel,
} from "./graphTypes";
export { useGraphModel, type GraphDepth, type UseGraphModelOptions, type UseGraphModelResult } from "./useGraphModel";
export { layoutGraphPositions, type GraphLayoutOptions, type GraphPosition } from "./useGraphLayout";
export { useHoverDimming, type HoverDimming } from "./useHoverDimming";
export { GraphCanvas, type GraphCanvasProps } from "./GraphCanvas";
export { PillNode, type PillNodeProps } from "./PillNode";
export { ClusterBoundary, type ClusterBoundaryProps } from "./ClusterBoundary";
export { GraphToolbar, type GraphToolbarProps, type ComponentOption } from "./GraphToolbar";
export { GraphLegend, type GraphLegendProps } from "./GraphLegend";
""")

# File: desktop/src/features/settings/DependencyExplorerAdmin.tsx
write_file("desktop/src/features/settings/DependencyExplorerAdmin.tsx", """import { useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { SystemGraphHit, SystemNodeType } from "../../lib/types";
import { SYSTEM_NODE_TYPE_META, SYSTEM_NODE_TYPES } from "../../components/visualBuilder/systemGraphNodeMeta";
// PR 2: shared pill-node graph engine (replaces the radial dot-graph).
import {
  graphNodeKey,
  isAccessEdge,
  SYSTEM_NODE_TYPE_LABELS,
  type GraphEdge,
  type GraphModel,
  type GraphNode,
} from "../../components/graph/graphTypes";
import { GraphCanvas } from "../../components/graph/GraphCanvas";
import { GraphLegend } from "../../components/graph/GraphLegend";
import { GraphToolbar, type ComponentOption } from "../../components/graph/GraphToolbar";
import { useHoverDimming } from "../../components/graph/useHoverDimming";
import type { GraphDepth } from "../../components/graph/useGraphModel";
import "../../components/graph/graph.css";

type RootRef = { nodeType: SystemNodeType; componentId: string };

// Ontology Explorer visual language (issue #339): which admin tab owns
// each System Graph node type - the inspector's primary "Open in..."
// action, reusing the exact same `onOpenTab` cross-link convention every
// other admin screen in this program already uses (see
// `ObjectTypesAdmin.tsx`'s own cross-link row, whose `onOpenTab` prop is
// typed as a plain string for the same reason this one is: avoiding an
// import of `Settings.tsx`'s own `AdminTab` union into a screen it itself
// renders). A plain lookup, not a new navigation concept.
const NODE_TYPE_ADMIN_TAB: Record<SystemNodeType, string> = {
  custom_object: "objectTypes",
  custom_field: "fields",
  relationship: "relationships",
  business_rule: "rules",
  workflow: "workflow",
  screen_layout: "layouts",
  page_layout: "pageBuilder",
  ai_agent: "aiAgents",
  execution_graph: "agentTeams",
  business_glossary_term: "businessGlossary",
  metric_definition: "metricDefinitions",
  test_case_definition: "testEval",
  // Action Types have no admin screen of their own - they live inside
  // the Object Types screen (Admin -> Ontology -> Object Types -> Action
  // Types), same place `ai_agent`'s own Ontology-scope cross-link
  // already points an agent's scoped object types to.
  action_type_definition: "objectTypes",
};

function TypeDot({ nodeType, size = 10 }: { nodeType: SystemNodeType; size?: number }) {
  const meta = SYSTEM_NODE_TYPE_META[nodeType];
  return (
    <span
      style={{
        display: "inline-block",
        flexShrink: 0,
        width: size,
        height: size,
        borderRadius: "50%",
        background: meta.color ?? `var(--chart-${meta.colorIndex})`,
      }}
    />
  );
}

// Groups a mixed dependents+dependencies hit list by edge type - the
// floating inspector's "Campaign · came from" / "User · belongs to"
// style groups from the reference video, rather than this screen's old
// flat two-column "Depends on this" / "This depends on" lists.
function groupByEdgeType(hits: { hit: SystemGraphHit; direction: "dependents" | "dependencies" }[]) {
  const groups = new Map<string, { hit: SystemGraphHit; direction: "dependents" | "dependencies" }[]>();
  hits.forEach((h) => {
    const key = h.hit.edge_type;
    const list = groups.get(key) ?? [];
    list.push(h);
    groups.set(key, list);
  });
  return Array.from(groups.entries());
}

/** BFS shortest path over the loaded model (mirrors demo's Find path). */
function useBfsPathfinder(model: GraphModel) {
  return useMemo(() => {
    const adj = new Map<string, Set<string>>();
    const link = (a: string, b: string) => {
      if (!adj.has(a)) adj.set(a, new Set());
      adj.get(a)!.add(b);
    };
    model.edges.forEach((e) => {
      link(e.fromKey, e.toKey);
      link(e.toKey, e.fromKey);
    });
    return (from: string, to: string): string[] | null => {
      if (from === to) return [from];
      const prev = new Map<string, string>();
      const seen = new Set<string>([from]);
      const queue: string[] = [from];
      while (queue.length) {
        const cur = queue.shift()!;
        for (const nb of adj.get(cur) ?? []) {
          if (seen.has(nb)) continue;
          seen.add(nb);
          prev.set(nb, cur);
          if (nb === to) {
            const path = [to];
            let c = to;
            while (c !== from) {
              c = prev.get(c)!;
              path.unshift(c);
            }
            return path;
          }
          queue.push(nb);
        }
      }
      return null;
    };
  }, [model]);
}

function edgeLabel(edgeType: string): string {
  return edgeType
    .split("_")
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(" ");
}

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
 * Dependency Explorer - pick any synced component, see its direct
 * dependents (what points at it) and dependencies (what it points at),
 * click a neighbor to inspect it.
 *
 * PR 2 rebuild: the radial dot-graph is replaced by the shared
 * pill-node graph engine (`GraphCanvas`): pill nodes, hover spotlight
 * dimming, dashed-ellipse cluster boundaries, legend sync, and the
 * compact 8-group toolbar. The inspector panel, admin-tab cross-links,
 * and all data wiring are unchanged.
 */
export function DependencyExplorerAdmin({ initialRoot, onOpenTab }: { initialRoot?: RootRef | null; onOpenTab?: (tab: string) => void }) {
  const [nodeType, setNodeType] = useState<SystemNodeType>(initialRoot?.nodeType ?? "custom_object");
  const [root, setRoot] = useState<RootRef | null>(initialRoot ?? null);
  const [inspecting, setInspecting] = useState<RootRef | null>(initialRoot ?? null);
  // Both default to true: the graph shows the combined multi-hop
  // upstream+downstream neighborhood out of the box, with no click required.
  const [showImpact, setShowImpact] = useState(true);
  const [showLineage, setShowLineage] = useState(true);
  const [hiddenTypes, setHiddenTypes] = useState<Set<SystemNodeType>>(() => new Set());
  const [depth, setDepth] = useState<GraphDepth>("all");
  const [pathMode, setPathMode] = useState(false);
  const [pathFrom, setPathFrom] = useState<string | null>(null);
  const [pathTo, setPathTo] = useState<string | null>(null);
  const [pathKeys, setPathKeys] = useState<string[] | null>(null);
  const [showFields, setShowFields] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [collapsedTypes, setCollapsedTypes] = useState<Set<SystemNodeType>>(() => new Set());
  const [cardOffset, setCardOffset] = useState<{ left: number; top: number } | null>(null);
  const [zoom, setZoom] = useState(1);
  const dragStateRef = useRef<{ pointerId: number; startX: number; startY: number; origLeft: number; origTop: number } | null>(null);
  const zoomApiRef = useRef<{ zoomIn: () => void; zoomOut: () => void; fit: () => void; reset: () => void; zoom: number } | null>(null);

  function startCardDrag(e: ReactPointerEvent<HTMLElement>) {
    const card = (e.currentTarget as HTMLElement).closest(".ontology-inspector-card") as HTMLElement | null;
    if (!card) return;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    dragStateRef.current = { pointerId: e.pointerId, startX: e.clientX, startY: e.clientY, origLeft: card.offsetLeft, origTop: card.offsetTop };
    e.preventDefault();
  }
  function onCardDragMove(e: ReactPointerEvent<HTMLElement>) {
    const d = dragStateRef.current;
    if (!d || d.pointerId !== e.pointerId) return;
    setCardOffset({ left: d.origLeft + (e.clientX - d.startX), top: d.origTop + (e.clientY - d.startY) });
  }
  function endCardDrag(e: ReactPointerEvent<HTMLElement>) {
    if (dragStateRef.current?.pointerId === e.pointerId) dragStateRef.current = null;
  }

  function toggleTypeVisibility(t: SystemNodeType) {
    setHiddenTypes((prev) => {
      const next = new Set(prev);
      if (next.has(t)) next.delete(t);
      else next.add(t);
      return next;
    });
  }

  function toggleCollapse(t: SystemNodeType) {
    setCollapsedTypes((prev) => {
      const next = new Set(prev);
      if (next.has(t)) next.delete(t);
      else next.add(t);
      return next;
    });
  }

  // Ontology Layer epic, issue #345: lets Object Types deep-link straight
  // into a specific object's node here instead of landing on this
  // screen's own default (first custom_object) - same
  // `initialTopicSlug`/`HelpAdmin` pattern this file's sibling screens
  // already use for a cross-admin-screen deep link.
  useEffect(() => {
    if (initialRoot) {
      setNodeType(initialRoot.nodeType);
      setRoot(initialRoot);
      setInspecting(initialRoot);
      setShowImpact(true);
      setShowLineage(true);
      setCardOffset(null);
      setPathKeys(null);
      setPathFrom(null);
      setPathTo(null);
    }
  }, [initialRoot]);

  const options = useQuery({ queryKey: ["systemNodes", nodeType], queryFn: () => api.listSystemNodes(nodeType) });

  const rootNode = useQuery({
    queryKey: ["systemNode", root?.nodeType, root?.componentId],
    queryFn: () => api.getSystemNode(root!.nodeType, root!.componentId),
    enabled: !!root,
  });
  const dependents = useQuery({
    queryKey: ["systemNodeDependents", root?.nodeType, root?.componentId],
    queryFn: () => api.getSystemNodeDependents(root!.nodeType, root!.componentId),
    enabled: !!root,
  });
  const dependencies = useQuery({
    queryKey: ["systemNodeDependencies", root?.nodeType, root?.componentId],
    queryFn: () => api.getSystemNodeDependencies(root!.nodeType, root!.componentId),
    enabled: !!root,
  });
  const impactQuery = useQuery({
    queryKey: ["systemNodeImpact", root?.nodeType, root?.componentId],
    queryFn: () => api.getSystemNodeImpact(root!.nodeType, root!.componentId),
    enabled: !!root && showImpact,
  });
  const lineageQuery = useQuery({
    queryKey: ["systemNodeLineage", root?.nodeType, root?.componentId],
    queryFn: () => api.getSystemNodeLineage(root!.nodeType, root!.componentId),
    enabled: !!root && showLineage,
  });

  // The floating inspector targets `inspecting`, which starts out as
  // `root` but can be a not-yet-centered neighbor the admin just clicked
  // on - its own dependents/dependencies are fetched lazily, only once
  // something other than the root is actually being inspected.
  const inspectingIsRoot = !!root && !!inspecting && inspecting.nodeType === root.nodeType && inspecting.componentId === root.componentId;
  const inspectingNode = useQuery({
    queryKey: ["systemNode", inspecting?.nodeType, inspecting?.componentId],
    queryFn: () => api.getSystemNode(inspecting!.nodeType, inspecting!.componentId),
    enabled: !!inspecting && !inspectingIsRoot,
  });
  const inspectingDependents = useQuery({
    queryKey: ["systemNodeDependents", inspecting?.nodeType, inspecting?.componentId],
    queryFn: () => api.getSystemNodeDependents(inspecting!.nodeType, inspecting!.componentId),
    enabled: !!inspecting && !inspectingIsRoot,
  });
  const inspectingDependencies = useQuery({
    queryKey: ["systemNodeDependencies", inspecting?.nodeType, inspecting?.componentId],
    queryFn: () => api.getSystemNodeDependencies(inspecting!.nodeType, inspecting!.componentId),
    enabled: !!inspecting && !inspectingIsRoot,
  });
  const inspectorNodeLabel = inspectingIsRoot ? rootNode.data?.label : inspectingNode.data?.label;
  const inspectorDependents = inspectingIsRoot ? dependents.data : inspectingDependents.data;
  const inspectorDependencies = inspectingIsRoot ? dependencies.data : inspectingDependencies.data;

  // A node's plain-English Business Glossary definition (FND-02) - only
  // meaningful for `custom_object` nodes, same restriction
  // `ontology_service`'s own description resolution already enforces
  // (a whole-object mapping only, never a field-level one); reuses that
  // exact backend computation (`ObjectTypeSummary.description`) rather
  // than re-deriving it client-side from semantic mappings a second time.
  const inspectingObjectType = useQuery({
    queryKey: ["ontologyObjectTypeDetail", inspecting?.componentId],
    queryFn: () => api.getObjectTypeDetail(inspecting!.componentId),
    enabled: !!inspecting && inspecting.nodeType === "custom_object",
  });

  // --- PR 2: assemble the GraphModel from the existing queries ---
  const rootKey = root ? graphNodeKey(root.nodeType, root.componentId) : null;

  const model = useMemo<GraphModel>(() => {
    if (!root || !rootNode.data) return { nodes: [], edges: [] };
    const maxDepth = depth === "all" ? Infinity : parseInt(depth, 10);
    const nodeMap = new Map<string, GraphNode>();
    const edges: GraphEdge[] = [];

    const rootN: GraphNode = {
      key: rootKey!,
      id: rootNode.data.component_id,
      type: rootNode.data.node_type,
      label: rootNode.data.label,
      isRoot: true,
    };
    nodeMap.set(rootN.key, rootN);

    const addHit = (hit: SystemGraphHit, direction: "dependents" | "dependencies") => {
      if (hit.depth > maxDepth) return;
      if (hiddenTypes.has(hit.node.node_type)) return;
      if (!showFields && hit.node.node_type === "custom_field") return;
      const key = graphNodeKey(hit.node.node_type, hit.node.component_id);
      if (!nodeMap.has(key)) {
        nodeMap.set(key, {
          key,
          id: hit.node.component_id,
          type: hit.node.node_type,
          label: hit.node.label,
        });
      }
      const fromKey = direction === "dependents" ? key : rootKey!;
      const toKey = direction === "dependents" ? rootKey! : key;
      edges.push({
        fromKey,
        toKey,
        edgeType: hit.edge_type,
        label: edgeLabel(hit.edge_type),
        access: isAccessEdge(hit.edge_type),
      });
    };

    if (showImpact || showLineage) {
      (showImpact ? impactQuery.data ?? [] : []).forEach((h) => addHit(h, "dependents"));
      (showLineage ? lineageQuery.data ?? [] : []).forEach((h) => addHit(h, "dependencies"));
    } else {
      (dependents.data ?? []).forEach((h) => addHit(h, "dependents"));
      (dependencies.data ?? []).forEach((h) => addHit(h, "dependencies"));
    }

    // Collapsed types: replace members with a single __cluster__ node.
    collapsedTypes.forEach((t) => {
      const members = Array.from(nodeMap.values()).filter((n) => n.type === t && !n.isRoot);
      if (members.length === 0) return;
      members.forEach((m) => nodeMap.delete(m.key));
      const clusterKey = `__cluster__:${t}`;
      nodeMap.set(clusterKey, {
        key: clusterKey,
        id: "__cluster__",
        type: t,
        label: `${SYSTEM_NODE_TYPE_LABELS[t]} (${members.length})`,
        count: members.length,
      });
      // Rewire edges touching removed members to the cluster node.
      edges.forEach((e) => {
        if (members.some((m) => m.key === e.fromKey)) e.fromKey = clusterKey;
        if (members.some((m) => m.key === e.toKey)) e.toKey = clusterKey;
      });
    });

    return { nodes: Array.from(nodeMap.values()), edges };
  }, [
    root,
    rootKey,
    rootNode.data,
    dependents.data,
    dependencies.data,
    impactQuery.data,
    lineageQuery.data,
    depth,
    showImpact,
    showLineage,
    showFields,
    hiddenTypes,
    collapsedTypes,
  ]);

  const nodeByKey = useMemo(() => new Map(model.nodes.map((n) => [n.key, n])), [model.nodes]);

  // Find-path: BFS over the loaded model.
  const findPath = useBfsPathfinder(model);

  // Hover spotlight — shared between the canvas and the legend so the
  // legend chips mirror the graph spotlight (demo's __syncLegend).
  const hover = useHoverDimming(model);

  function selectRoot(nt: SystemNodeType, componentId: string) {
    setRoot({ nodeType: nt, componentId });
    setInspecting({ nodeType: nt, componentId });
    setShowImpact(true);
    setShowLineage(true);
    setPathKeys(null);
    setPathFrom(null);
    setPathTo(null);
  }

  function inspect(nt: SystemNodeType, componentId: string) {
    setInspecting({ nodeType: nt, componentId });
  }

  function handleNodeClick(node: GraphNode) {
    if (node.id === "__cluster__") {
      toggleCollapse(node.type);
      return;
    }
    if (pathMode) {
      const key = node.key;
      if (!pathFrom) {
        setPathFrom(key);
      } else if (!pathTo && key !== pathFrom) {
        setPathTo(key);
        const path = findPath(pathFrom, key);
        setPathKeys(path);
      } else {
        // Restart path selection.
        setPathFrom(key);
        setPathTo(null);
        setPathKeys(null);
      }
      return;
    }
    inspect(node.type, node.id);
  }

  function handleNodeDoubleClick(node: GraphNode) {
    if (node.id === "__cluster__") return;
    // Double-click re-centers on the node (loads its neighborhood).
    selectRoot(node.type, node.id);
  }

  function handleCopyLink() {
    if (!root) return;
    const url = `${window.location.origin}${window.location.pathname}#/admin/ontology?type=${root.nodeType}&id=${root.componentId}`;
    navigator.clipboard?.writeText(url).catch(() => {});
  }

  function handleSimulate() {
    // Simulate is wired to impact highlight in the demo; here we ensure
    // impact is on so the blast radius is visible.
    setShowImpact(true);
    setShowLineage(false);
  }

  const componentOptions: ComponentOption[] = useMemo(
    () => (options.data ?? []).map((n) => ({ id: n.component_id, label: n.label })),
    [options.data],
  );

  const searchResults = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return [];
    return componentOptions.filter((o) => o.label.toLowerCase().includes(q)).slice(0, 8);
  }, [searchQuery, componentOptions]);

  const inspectingKey = inspecting ? graphNodeKey(inspecting.nodeType, inspecting.componentId) : null;
  const pathFromLabel = pathFrom ? nodeByKey.get(pathFrom)?.label ?? null : null;
  const pathToLabel = pathTo ? nodeByKey.get(pathTo)?.label ?? null : null;

  return (
    <div>
      <div className="og-stage-head">
        <h3 className="og-stage-title">Dependency Explorer</h3>
        <div className="og-stage-chips">
          {root && <span className="og-pill-chip">{SYSTEM_NODE_TYPE_LABELS[root.nodeType]}</span>}
          <span className="og-pill-chip">
            {model.nodes.length} nodes · {model.edges.length} relationships
          </span>
        </div>
      </div>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Pick any synced component to see what depends on it and what it depends on. Covers {SYSTEM_NODE_TYPES.length} component
        types for now (objects, fields, relationships, business rules, workflows, screen/page layouts, AI agents, agent teams,
        glossary terms, metrics, test cases) - the rest of the platform's components are a documented follow-up, not silently missing.
      </p>

      {root && rootNode.data ? (
        <>
          <GraphToolbar
            nodeType={nodeType}
            onNodeTypeChange={(t) => {
              setNodeType(t);
              setRoot(null);
              setInspecting(null);
            }}
            componentId={root.componentId}
            onComponentChange={(id) => {
              if (id) selectRoot(nodeType, id);
              else {
                setRoot(null);
                setInspecting(null);
              }
            }}
            componentOptions={componentOptions}
            searchQuery={searchQuery}
            onSearchQueryChange={setSearchQuery}
            searchResults={searchResults}
            onSearchSelect={(opt) => {
              selectRoot(nodeType, opt.id);
              setSearchQuery("");
            }}
            pathMode={pathMode}
            onTogglePathMode={() => {
              setPathMode((v) => !v);
              setPathFrom(null);
              setPathTo(null);
              setPathKeys(null);
            }}
            pathFromLabel={pathFromLabel}
            pathToLabel={pathToLabel}
            onClearPath={() => {
              setPathFrom(null);
              setPathTo(null);
              setPathKeys(null);
              setPathMode(false);
            }}
            zoom={zoom}
            onZoomIn={() => zoomApiRef.current?.zoomIn()}
            onZoomOut={() => zoomApiRef.current?.zoomOut()}
            onFit={() => zoomApiRef.current?.fit()}
            onReset={() => {
              zoomApiRef.current?.reset();
              setPathKeys(null);
              setPathFrom(null);
              setPathTo(null);
              setPathMode(false);
            }}
            depth={depth}
            onDepthChange={setDepth}
            showImpact={showImpact}
            onToggleImpact={() => setShowImpact((v) => !v)}
            showLineage={showLineage}
            onToggleLineage={() => setShowLineage((v) => !v)}
            showFields={showFields}
            onToggleFields={() => setShowFields((v) => !v)}
            onSimulate={handleSimulate}
            onCopyLink={handleCopyLink}
          />

          <div className="graph-canvas-wrap ontology-canvas-wrap" style={{ position: "relative" }}>
            <div style={{ height: 560 }}>
              <GraphCanvas
                model={model}
                rootKey={rootKey}
                hiddenTypes={hiddenTypes}
                inspectingKey={pathKeys ? null : inspectingKey}
                collapsedTypes={collapsedTypes}
                onNodeClick={handleNodeClick}
                onNodeDoubleClick={handleNodeDoubleClick}
                onClusterToggle={toggleCollapse}
                zoomApiRef={zoomApiRef}
                hover={hover}
                onZoomChange={setZoom}
              />
            </div>

            {pathKeys && pathKeys.length > 0 && (
              <div className="graph-path-banner">
                Path: {pathKeys.map((k) => nodeByKey.get(k)?.label ?? k).join(" → ")} ({pathKeys.length - 1} hops)
                <button
                  type="button"
                  onClick={() => {
                    setPathKeys(null);
                    setPathFrom(null);
                    setPathTo(null);
                    setPathMode(false);
                  }}
                >
                  Clear
                </button>
              </div>
            )}

            {inspecting && (inspectingIsRoot ? rootNode.data : inspectingNode.data) && (
              <div className="ontology-inspector-card" style={cardOffset ? { left: cardOffset.left, top: cardOffset.top, right: "auto" } : undefined}>
                <div
                  className="ontology-inspector-head"
                  style={{ cursor: "grab", touchAction: "none" }}
                  onPointerDown={startCardDrag}
                  onPointerMove={onCardDragMove}
                  onPointerUp={endCardDrag}
                  onPointerCancel={endCardDrag}
                >
                  <TypeDot nodeType={inspecting.nodeType} size={14} />
                  <div>
                    <strong>{inspectorNodeLabel}</strong>
                    <div className="ontology-inspector-subtitle">{SYSTEM_NODE_TYPE_META[inspecting.nodeType].label}</div>
                  </div>
                </div>

                {inspecting.nodeType === "custom_object" && inspectingObjectType.data?.object_type.description && (
                  <p className="ontology-inspector-definition">{inspectingObjectType.data.object_type.description}</p>
                )}

                <div className="ontology-inspector-actions">
                  <button className="btn btn-primary" onClick={() => onOpenTab?.(NODE_TYPE_ADMIN_TAB[inspecting.nodeType])} disabled={!onOpenTab}>
                    Open in its admin screen
                  </button>
                  <button className="btn btn-secondary" onClick={() => selectRoot(inspecting.nodeType, inspecting.componentId)} disabled={inspectingIsRoot}>
                    Show what's around it
                  </button>
                </div>

                <div className="ontology-inspector-groups">
                  {groupByEdgeType([
                    ...(inspectorDependents ?? []).map((hit) => ({ hit, direction: "dependents" as const })),
                    ...(inspectorDependencies ?? []).map((hit) => ({ hit, direction: "dependencies" as const })),
                  ]).map(([edgeType, members]) => (
                    <div key={edgeType} className="ontology-inspector-group">
                      <div className="ontology-inspector-group-label">{edgeType}</div>
                      {members.map((m, i) => (
                        <button
                          key={i}
                          type="button"
                          className="ontology-inspector-bullet"
                          onClick={() => selectRoot(m.hit.node.node_type, m.hit.node.component_id)}
                          title={m.direction === "dependents" ? "Depends on this" : "This depends on"}
                        >
                          <TypeDot nodeType={m.hit.node.node_type} />
                          {m.hit.node.label}
                        </button>
                      ))}
                    </div>
                  ))}
                  {(inspectorDependents ?? []).length === 0 && (inspectorDependencies ?? []).length === 0 && (
                    <p style={{ color: "var(--text-muted)", fontSize: 12 }}>Nothing else synced depends on or is depended on by this.</p>
                  )}
                </div>
              </div>
            )}
          </div>

          <GraphLegend
            model={model}
            hiddenTypes={hiddenTypes}
            onToggleType={toggleTypeVisibility}
            highlightedTypes={hover.connectedTypes}
            spotlightActive={hover.dimming}
          />
        </>
      ) : (
        <>
          {/* No root selected yet: keep the type/component pickers visible. */}
          <GraphToolbar
            nodeType={nodeType}
            onNodeTypeChange={(t) => {
              setNodeType(t);
              setRoot(null);
              setInspecting(null);
            }}
            componentId=""
            onComponentChange={(id) => {
              if (id) selectRoot(nodeType, id);
            }}
            componentOptions={componentOptions}
            searchQuery={searchQuery}
            onSearchQueryChange={setSearchQuery}
            searchResults={searchResults}
            onSearchSelect={(opt) => {
              selectRoot(nodeType, opt.id);
              setSearchQuery("");
            }}
            pathMode={false}
            onTogglePathMode={() => {}}
            zoom={1}
            onZoomIn={() => {}}
            onZoomOut={() => {}}
            onFit={() => {}}
            onReset={() => {}}
            depth={depth}
            onDepthChange={setDepth}
            showImpact={showImpact}
            onToggleImpact={() => setShowImpact((v) => !v)}
            showLineage={showLineage}
            onToggleLineage={() => setShowLineage((v) => !v)}
            showFields={showFields}
            onToggleFields={() => setShowFields((v) => !v)}
          />
          <p style={{ color: "var(--text-muted)" }}>Nothing synced yet for this type, or none selected.</p>
        </>
      )}
    </div>
  );
}
""")

# File: desktop/src/components/visualBuilder/systemGraphNodeMeta.ts
write_file("desktop/src/components/visualBuilder/systemGraphNodeMeta.ts", """// Next-Gen program, Domain A, FND-01/FND-02: per-node-type metadata for
// the Dependency Explorer - pure data, no JSX/hooks, same split
// `graphNodeMeta.ts` already establishes. Scoped to exactly the node
// types `system_graph.rs`'s `NODE_TYPES` syncs.
//
// Ontology Explorer visual language (issue #339): `colorIndex` replaces
// this map's old hardcoded hex `color` field - every node now renders
// through Theme Studio's own categorical `--chart-1`..`--chart-6` tokens
// (the same `ontologyColorIndex`-style convention Object Types' own
// per-type color already uses), so a Theme Studio preset switch re-colors
// this graph too instead of a second, untouchable palette.
import type { SystemNodeType } from "../../lib/types";

export const SYSTEM_NODE_TYPE_META: Record<SystemNodeType, { label: string; colorIndex: number; color: string }> = {
  custom_object: { label: "Object", colorIndex: 1, color: "#4F46E5" },
  custom_field: { label: "Field", colorIndex: 2, color: "#64748B" },
  relationship: { label: "Relationship", colorIndex: 3, color: "#D97706" },
  business_rule: { label: "Business Rule", colorIndex: 4, color: "#B45309" },
  workflow: { label: "Workflow", colorIndex: 5, color: "#0F766E" },
  screen_layout: { label: "Screen Layout", colorIndex: 6, color: "#475569" },
  page_layout: { label: "Page Layout", colorIndex: 1, color: "#475569" },
  ai_agent: { label: "AI Agent", colorIndex: 2, color: "#9333EA" },
  execution_graph: { label: "Agent Team", colorIndex: 3, color: "#0F766E" },
  business_glossary_term: { label: "Glossary Term", colorIndex: 4, color: "#E11D48" },
  metric_definition: { label: "Metric", colorIndex: 5, color: "#0284C7" },
  test_case_definition: { label: "Test Case", colorIndex: 6, color: "#9333EA" },
  action_type_definition: { label: "Action Type", colorIndex: 1, color: "#475569" },
};

export const SYSTEM_NODE_TYPES: SystemNodeType[] = [
  "custom_object", "custom_field", "relationship", "business_rule", "workflow", "screen_layout", "page_layout", "ai_agent", "execution_graph",
  "business_glossary_term", "metric_definition", "test_case_definition", "action_type_definition",
];
""")

print("Done")
