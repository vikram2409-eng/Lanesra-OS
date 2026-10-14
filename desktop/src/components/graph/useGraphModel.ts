/**
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
