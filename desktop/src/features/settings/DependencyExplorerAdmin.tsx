import { useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { SystemGraphHit, SystemNodeType } from "../../lib/types";
import { SYSTEM_NODE_TYPE_META, SYSTEM_NODE_TYPES } from "../../components/visualBuilder/systemGraphNodeMeta";
import { VisualBuilderCanvas, type CanvasEdgeView } from "../../components/visualBuilder/VisualBuilderCanvas";

const ROOT_SIZE = 26;
const NODE_SIZE = 15;
const RING_RADIUS = 170;
const CLOUD_RING_GAP = 110;

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
        background: `var(--chart-${meta.colorIndex})`,
      }}
    />
  );
}

// The legend doubles as a toggle-filter (issue #339's own design text:
// "identify/toggle what's showing") - clicking a type dims every node of
// that type in the graph below rather than removing it outright, so the
// root/edge layout never has to reflow around a hidden node.
function Legend({ hidden, onToggle }: { hidden: Set<SystemNodeType>; onToggle: (t: SystemNodeType) => void }) {
  return (
    <div className="ontology-legend">
      {SYSTEM_NODE_TYPES.map((t) => (
        <button
          key={t}
          type="button"
          className={`ontology-legend-item${hidden.has(t) ? " ontology-legend-item-off" : ""}`}
          onClick={() => onToggle(t)}
          title={hidden.has(t) ? `Show ${SYSTEM_NODE_TYPE_META[t].label}` : `Hide ${SYSTEM_NODE_TYPE_META[t].label}`}
        >
          <TypeDot nodeType={t} />
          {SYSTEM_NODE_TYPE_META[t].label}
        </button>
      ))}
    </div>
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

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-01: the
 * Dependency Explorer - pick any synced component, see its direct
 * dependents (what points at it) and dependencies (what it points at),
 * click a neighbor to re-center on it.
 *
 * Ontology Explorer visual language (issue #339): a radial dot-graph
 * (angle = index/count around the centered root, not the old fixed
 * left/center/right columns), a persistent type-color legend sourced
 * from Theme Studio's own categorical tokens, and a compact floating
 * inspector card on node selection (colored dot + name/type, an "Open
 * in..." cross-link and a "Show what's around it" re-center action,
 * relationships grouped by edge type, and - where one is mapped - the
 * node's Business Glossary one-liner, FND-02) in place of the old
 * node-is-the-only-detail-view pattern. "Full impact"/"Full lineage" now
 * render as a zoomed-out radial "cloud" of every node in that transitive
 * closure (grouped into concentric rings by depth) instead of a plain
 * table - deliberately position-only with no connecting lines, since
 * `SystemGraphHit` (depth + edge type from the root) has no per-hop
 * parent to draw an exact line from at depth > 1, and a precise one isn't
 * worth a new query just for this overview.
 */
export function DependencyExplorerAdmin({ initialRoot, onOpenTab }: { initialRoot?: RootRef | null; onOpenTab?: (tab: string) => void }) {
  const [nodeType, setNodeType] = useState<SystemNodeType>(initialRoot?.nodeType ?? "custom_object");
  const [root, setRoot] = useState<RootRef | null>(initialRoot ?? null);
  const [inspecting, setInspecting] = useState<RootRef | null>(initialRoot ?? null);
  // Both default to true: the graph shows the combined multi-hop
  // upstream+downstream cloud out of the box, with no click required.
  // Turning one off narrows to a single-direction cloud (this screen's
  // original "Full impact"/"Full lineage" behavior); turning both off
  // falls back to the direct 1-hop view with real SVG edge lines, for
  // when the admin wants that more focused, edge-labeled read instead.
  const [showImpact, setShowImpact] = useState(true);
  const [showLineage, setShowLineage] = useState(true);
  const [hiddenTypes, setHiddenTypes] = useState<Set<SystemNodeType>>(() => new Set());
  const [cardOffset, setCardOffset] = useState<{ left: number; top: number } | null>(null);
  const dragStateRef = useRef<{ pointerId: number; startX: number; startY: number; origLeft: number; origTop: number } | null>(null);

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

  type LaidOutNode = { key: string; x: number; y: number; label: string; nodeType: SystemNodeType; componentId: string; edgeType?: string; isRoot?: boolean; size: number };

  const { nodes, edges, canvasWidth, canvasHeight } = useMemo<{ nodes: LaidOutNode[]; edges: CanvasEdgeView[]; canvasWidth: number; canvasHeight: number }>(() => {
    if (!root || !rootNode.data) return { nodes: [], edges: [], canvasWidth: 200, canvasHeight: 200 };

    if (showImpact || showLineage) {
      const hits = [...(showImpact ? impactQuery.data ?? [] : []), ...(showLineage ? lineageQuery.data ?? [] : [])];
      const maxDepth = hits.reduce((m, h) => Math.max(m, h.depth), 0);
      const radius = Math.max(RING_RADIUS, (maxDepth + 1) * CLOUD_RING_GAP);
      const cx = radius + 60;
      const cy = radius + 60;
      const byDepth = new Map<number, SystemGraphHit[]>();
      hits.forEach((h) => byDepth.set(h.depth, [...(byDepth.get(h.depth) ?? []), h]));
      const laidOut: LaidOutNode[] = [{ key: "root", x: cx, y: cy, label: rootNode.data.label, nodeType: rootNode.data.node_type, componentId: rootNode.data.component_id, isRoot: true, size: ROOT_SIZE }];
      Array.from(byDepth.entries()).forEach(([depth, depthHits]) => {
        const r = depth * CLOUD_RING_GAP;
        const size = Math.max(8, NODE_SIZE - depth);
        depthHits.forEach((h, i) => {
          const angle = (i / depthHits.length) * Math.PI * 2;
          laidOut.push({
            key: `${depth}-${i}`,
            x: cx + r * Math.cos(angle),
            y: cy + r * Math.sin(angle),
            label: h.node.label,
            nodeType: h.node.node_type,
            componentId: h.node.component_id,
            edgeType: h.edge_type,
            size,
          });
        });
      });
      return { nodes: laidOut, edges: [], canvasWidth: cx * 2, canvasHeight: cy * 2 };
    }

    const deps = dependents.data ?? [];
    const reqs = dependencies.data ?? [];
    const cx = RING_RADIUS + 120;
    const cy = RING_RADIUS + 60;
    const laidOut: LaidOutNode[] = [{ key: "root", x: cx, y: cy, label: rootNode.data.label, nodeType: rootNode.data.node_type, componentId: rootNode.data.component_id, isRoot: true, size: ROOT_SIZE }];
    const edgeViews: CanvasEdgeView[] = [];
    // Dependents (what points at this) fan out across the left
    // semicircle, dependencies (what this points at) across the right -
    // a radial take on the old left/root/right column layout, not a
    // fully undirected cloud, so the existing "depends on" mental model
    // still reads at a glance.
    const place = (hits: SystemGraphHit[], startDeg: number, endDeg: number, keyPrefix: string) => {
      hits.forEach((hit, i) => {
        const deg = hits.length === 1 ? (startDeg + endDeg) / 2 : startDeg + ((endDeg - startDeg) * i) / (hits.length - 1);
        const rad = (deg * Math.PI) / 180;
        const x = cx + RING_RADIUS * Math.cos(rad);
        const y = cy + RING_RADIUS * Math.sin(rad);
        laidOut.push({ key: `${keyPrefix}-${i}`, x, y, label: hit.node.label, nodeType: hit.node.node_type, componentId: hit.node.component_id, edgeType: hit.edge_type, size: NODE_SIZE });
        edgeViews.push({ key: `${keyPrefix}-edge-${i}`, fromX: x, fromY: y, toX: cx, toY: cy, label: hit.edge_type });
      });
    };
    place(deps, 100, 260, "dep");
    place(reqs, -80, 80, "req");
    return { nodes: laidOut, edges: edgeViews, canvasWidth: cx + RING_RADIUS + 60, canvasHeight: cy + RING_RADIUS + 60 };
  }, [root, rootNode.data, dependents.data, dependencies.data, showImpact, showLineage, impactQuery.data, lineageQuery.data]);

  function selectRoot(nt: SystemNodeType, componentId: string) {
    setRoot({ nodeType: nt, componentId });
    setInspecting({ nodeType: nt, componentId });
    setShowImpact(true);
    setShowLineage(true);
  }

  function inspect(nt: SystemNodeType, componentId: string) {
    setInspecting({ nodeType: nt, componentId });
  }

  return (
    <div>
      <h3>Dependency Explorer</h3>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Pick any synced component to see what depends on it and what it depends on. Covers {SYSTEM_NODE_TYPES.length} component
        types for now (objects, fields, relationships, business rules, workflows, screen/page layouts, AI agents, agent teams,
        glossary terms, metrics, test cases) - the rest of the platform's components are a documented follow-up, not silently missing.
      </p>

      <Legend hidden={hiddenTypes} onToggle={toggleTypeVisibility} />

      <div className="form-grid" style={{ marginBottom: 16 }}>
        <div className="form-field">
          <label>Component type</label>
          <select
            value={nodeType}
            onChange={(e) => {
              setNodeType(e.target.value as SystemNodeType);
              setRoot(null);
              setInspecting(null);
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
          <select
            value={root?.componentId ?? ""}
            onChange={(e) => {
              if (e.target.value) {
                selectRoot(nodeType, e.target.value);
              } else {
                setRoot(null);
                setInspecting(null);
              }
            }}
          >
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
            <button className={`btn ${showImpact ? "btn-primary" : ""}`} onClick={() => setShowImpact((v) => !v)}>
              Full impact (everything upstream)
            </button>
            <button className={`btn ${showLineage ? "btn-primary" : ""}`} onClick={() => setShowLineage((v) => !v)}>
              Full lineage (everything downstream)
            </button>
          </div>

          <div className="graph-canvas-wrap ontology-canvas-wrap" style={{ position: "relative" }}>
            <VisualBuilderCanvas<LaidOutNode>
              nodes={nodes}
              edges={edges}
              zoom={1}
              canvasWidth={canvasWidth}
              canvasHeight={canvasHeight}
              editable={false}
              renderNode={(n) => {
                const meta = SYSTEM_NODE_TYPE_META[n.nodeType];
                const isInspecting = inspecting?.nodeType === n.nodeType && inspecting?.componentId === n.componentId;
                const isDimmed = !n.isRoot && hiddenTypes.has(n.nodeType);
                return (
                  <div
                    key={n.key}
                    className={`ontology-node-dot${n.isRoot ? " ontology-node-root" : ""}${isInspecting && !n.isRoot ? " ontology-node-inspecting" : ""}${isDimmed ? " ontology-node-dimmed" : ""}`}
                    style={{ left: n.x - n.size / 2, top: n.y - n.size / 2, width: n.size, height: n.size, background: `var(--chart-${meta.colorIndex})` }}
                    onClick={() => inspect(n.nodeType, n.componentId)}
                    title={`${meta.label}: ${n.label}${n.edgeType ? ` (${n.edgeType})` : ""}`}
                  />
                );
              }}
            />

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
        </>
      )}
    </div>
  );
}
