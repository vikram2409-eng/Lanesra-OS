import { useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
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
