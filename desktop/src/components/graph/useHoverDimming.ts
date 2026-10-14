/**
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
