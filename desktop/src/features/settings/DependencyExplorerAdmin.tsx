import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { SystemGraphHit, SystemNodeType } from "../../lib/types";
import { SYSTEM_NODE_TYPE_META, SYSTEM_NODE_TYPES } from "../../components/visualBuilder/systemGraphNodeMeta";
import { VisualBuilderCanvas, type CanvasEdgeView } from "../../components/visualBuilder/VisualBuilderCanvas";

const CARD_W = 200;
const CARD_H = 56;
const COL_GAP = 320;
const ROW_GAP = 72;

type RootRef = { nodeType: SystemNodeType; componentId: string };

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
 * Dependency Explorer - pick any synced component, see its direct
 * dependents (what points at it) and dependencies (what it points at),
 * click a neighbor to re-center on it. Read-only, incremental-reveal -
 * never renders the whole graph at once, the same way a real dependency
 * graph for a workspace this size would be unreadable as one drawing.
 * "Full impact"/"Full lineage" below give the transitive closure as a
 * plain grouped list instead, for exactly that reason.
 */
export function DependencyExplorerAdmin() {
  const [nodeType, setNodeType] = useState<SystemNodeType>("custom_object");
  const [root, setRoot] = useState<RootRef | null>(null);
  const [transitiveMode, setTransitiveMode] = useState<"impact" | "lineage" | null>(null);

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
  const transitive = useQuery({
    queryKey: ["systemNodeTransitive", transitiveMode, root?.nodeType, root?.componentId],
    queryFn: () => (transitiveMode === "impact" ? api.getSystemNodeImpact(root!.nodeType, root!.componentId) : api.getSystemNodeLineage(root!.nodeType, root!.componentId)),
    enabled: !!root && !!transitiveMode,
  });

  type LaidOutNode = { key: string; x: number; y: number; label: string; nodeType: SystemNodeType; componentId: string; edgeType?: string; isRoot?: boolean };

  const { nodes, edges } = useMemo<{ nodes: LaidOutNode[]; edges: CanvasEdgeView[] }>(() => {
    if (!root || !rootNode.data) return { nodes: [], edges: [] };
    const deps = dependents.data ?? [];
    const reqs = dependencies.data ?? [];
    const colHeight = Math.max(deps.length, reqs.length, 1) * ROW_GAP;
    const centerY = colHeight / 2;

    const laidOut: LaidOutNode[] = [{ key: "root", x: COL_GAP, y: centerY, label: rootNode.data.label, nodeType: rootNode.data.node_type, componentId: rootNode.data.component_id, isRoot: true }];
    deps.forEach((hit: SystemGraphHit, i: number) => {
      laidOut.push({ key: `dep-${i}`, x: 0, y: i * ROW_GAP, label: hit.node.label, nodeType: hit.node.node_type, componentId: hit.node.component_id, edgeType: hit.edge_type });
    });
    reqs.forEach((hit: SystemGraphHit, i: number) => {
      laidOut.push({ key: `req-${i}`, x: COL_GAP * 2, y: i * ROW_GAP, label: hit.node.label, nodeType: hit.node.node_type, componentId: hit.node.component_id, edgeType: hit.edge_type });
    });

    const edgeViews: CanvasEdgeView[] = [];
    deps.forEach((hit: SystemGraphHit, i: number) => {
      edgeViews.push({ key: `dep-edge-${i}`, fromX: 0 + CARD_W, fromY: i * ROW_GAP + CARD_H / 2, toX: COL_GAP, toY: centerY + CARD_H / 2, label: hit.edge_type });
    });
    reqs.forEach((hit: SystemGraphHit, i: number) => {
      edgeViews.push({ key: `req-edge-${i}`, fromX: COL_GAP + CARD_W, fromY: centerY + CARD_H / 2, toX: COL_GAP * 2, toY: i * ROW_GAP + CARD_H / 2, label: hit.edge_type });
    });
    return { nodes: laidOut, edges: edgeViews };
  }, [root, rootNode.data, dependents.data, dependencies.data]);

  const canvasWidth = COL_GAP * 2 + CARD_W + 40;
  const canvasHeight = Math.max(nodes.length ? Math.max(...nodes.map((n) => n.y)) + CARD_H + 40 : 200, 200);

  function selectRoot(nt: SystemNodeType, componentId: string) {
    setRoot({ nodeType: nt, componentId });
    setTransitiveMode(null);
  }

  return (
    <div>
      <h3>Dependency Explorer</h3>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Pick any synced component to see what depends on it and what it depends on. Covers {SYSTEM_NODE_TYPES.length} component
        types for now (objects, fields, relationships, business rules, workflows, screen/page layouts, AI agents, agent teams,
        glossary terms, metrics, test cases) - the rest of the platform's components are a documented follow-up, not silently missing.
      </p>

      <div className="form-grid" style={{ marginBottom: 16 }}>
        <div className="form-field">
          <label>Component type</label>
          <select
            value={nodeType}
            onChange={(e) => {
              setNodeType(e.target.value as SystemNodeType);
              setRoot(null);
            }}
          >
            {SYSTEM_NODE_TYPES.map((t) => (
              <option key={t} value={t}>
                {SYSTEM_NODE_TYPE_META[t].label}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Component</label>
          <select value={root?.componentId ?? ""} onChange={(e) => (e.target.value ? selectRoot(nodeType, e.target.value) : setRoot(null))}>
            <option value="">Select...</option>
            {(options.data ?? []).map((n) => (
              <option key={n.id} value={n.component_id}>
                {n.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      {!root && <p style={{ color: "var(--text-muted)" }}>Nothing synced yet for this type, or none selected.</p>}

      {root && rootNode.data && (
        <>
          <div className="toolbar" style={{ marginBottom: 8 }}>
            <button className={`btn ${transitiveMode === "impact" ? "btn-primary" : ""}`} onClick={() => setTransitiveMode(transitiveMode === "impact" ? null : "impact")}>
              Full impact (everything upstream)
            </button>
            <button className={`btn ${transitiveMode === "lineage" ? "btn-primary" : ""}`} onClick={() => setTransitiveMode(transitiveMode === "lineage" ? null : "lineage")}>
              Full lineage (everything downstream)
            </button>
          </div>

          {!transitiveMode && (
            <div className="graph-canvas-wrap">
              <VisualBuilderCanvas<LaidOutNode>
                nodes={nodes}
                edges={edges}
                zoom={1}
                canvasWidth={canvasWidth}
                canvasHeight={canvasHeight}
                editable={false}
                renderNode={(n) => {
                  const meta = SYSTEM_NODE_TYPE_META[n.nodeType];
                  return (
                    <div
                      key={n.key}
                      className="graph-node"
                      style={{ left: n.x, top: n.y, width: CARD_W, borderColor: n.isRoot ? meta.color : undefined, cursor: n.isRoot ? "default" : "pointer" }}
                      onClick={() => !n.isRoot && selectRoot(n.nodeType, n.componentId)}
                      title={n.isRoot ? undefined : `Edge: ${n.edgeType} - click to re-center`}
                    >
                      <div className="graph-node-head" style={{ background: `${meta.color}26`, color: meta.color }}>
                        {meta.label}
                      </div>
                      <div className="graph-node-body">
                        <strong>{n.label}</strong>
                      </div>
                    </div>
                  );
                }}
              />
            </div>
          )}

          {transitiveMode && (
            <div className="card">
              <h4 style={{ marginTop: 0 }}>{transitiveMode === "impact" ? "Full impact - everything that transitively depends on this" : "Full lineage - everything this transitively depends on"}</h4>
              {transitive.isLoading && <p>Loading...</p>}
              {transitive.data && transitive.data.length === 0 && <p style={{ color: "var(--text-muted)" }}>Nothing found.</p>}
              {transitive.data && transitive.data.length > 0 && (
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Depth</th>
                      <th>Type</th>
                      <th>Component</th>
                      <th>Via edge</th>
                    </tr>
                  </thead>
                  <tbody>
                    {transitive.data.map((hit: SystemGraphHit) => (
                      <tr key={hit.node.id} style={{ cursor: "pointer" }} onClick={() => selectRoot(hit.node.node_type, hit.node.component_id)}>
                        <td>{hit.depth}</td>
                        <td>{SYSTEM_NODE_TYPE_META[hit.node.node_type].label}</td>
                        <td>{hit.node.label}</td>
                        <td>{hit.edge_type}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}
        </>
      )}
    </div>
  );
}
