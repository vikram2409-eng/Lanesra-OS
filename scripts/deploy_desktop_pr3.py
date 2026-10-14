#!/usr/bin/env python3
"""Deploy Desktop Object 360 + Tests (PR 3)."""

import os

REPO = "."

def write_file(path, content):
    full = os.path.join(REPO, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w", encoding="utf-8").write(content)
    print(f"Wrote {path}")

# File: desktop/src/components/graph/Object360Graph.tsx
write_file("desktop/src/components/graph/Object360Graph.tsx", """/**
 * PR 3 — Desktop Object 360: record-level graph on the shared engine.
 *
 * Thin wrapper over `GraphCanvas` for record neighborhoods. The root is
 * a real record; neighbors are its related records (one pill per actual
 * record), its Business Unit, the active Business Rules / Workflows that
 * apply to its type, and the AI Agents scoped to its type — mirroring the
 * demo's Object 360 graph (pill nodes, hover spotlight, dashed-ellipse
 * cluster boundaries, legend that doubles as graph-layer toggles).
 *
 * Record entity types (Company, Contact, …) are mapped onto the 13
 * system node types for pill coloring via `entityTypeToSystemType`
 * (fixed table for known CRM types + deterministic hash fallback), so
 * the shared `GraphCanvas`/`ClusterBoundary`/`useHoverDimming` need no
 * changes.
 */
import { memo, useMemo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, type GraphEdge, type GraphModel, type GraphNode } from "./graphTypes";
import { GraphCanvas } from "./GraphCanvas";
import { useHoverDimming } from "./useHoverDimming";
import type { SystemNodeType } from "../../lib/types";
import type { RelatedRecord } from "../../lib/types";

/** Record being inspected / re-centered (mirrors Object360Modal's RootRef). */
export interface Object360RecordRef {
  entityType: string;
  entityId: string;
  label: string;
}

export type Object360InspectTarget = Object360RecordRef | { kind: "orgUnit"; id: string; label: string };

export interface Object360GraphData {
  related: RelatedRecord[];
  businessRules: { id: string; name: string }[];
  workflows: { id: string; name: string }[];
  scopedAgents: { id: string; name: string }[];
  orgUnit: { id: string; name: string } | null;
}

/** Legend keys used by the layer toggles + hover sync. */
export type Object360LegendKey = string; // `entity:<EntityType>` | "rule" | "workflow" | "agent" | "orgUnit"

const KNOWN_TYPE_MAP: Record<string, SystemNodeType> = {
  Company: "custom_object",
  Contact: "ai_agent",
  Opportunity: "relationship",
  Quote: "metric_definition",
  Order: "business_glossary_term",
  Invoice: "execution_graph",
  Contract: "business_rule",
  Task: "custom_field",
  Product: "action_type_definition",
  BusinessUnit: "page_layout",
};

const HASH_POOL: SystemNodeType[] = [
  "custom_object",
  "ai_agent",
  "relationship",
  "metric_definition",
  "business_glossary_term",
  "execution_graph",
  "business_rule",
  "custom_field",
  "action_type_definition",
  "page_layout",
  "screen_layout",
  "business_glossary_term",
  "metric_definition",
];

/**
 * Maps a record entity type to a system node type for pill coloring.
 * Known CRM types use the fixed table; anything else (custom objects)
 * falls back to a deterministic hash so colors stay stable.
 */
export function entityTypeToSystemType(entityType: string): SystemNodeType {
  const known = KNOWN_TYPE_MAP[entityType];
  if (known) return known;
  let h = 0;
  for (let i = 0; i < entityType.length; i++) {
    h = (h * 31 + entityType.charCodeAt(i)) >>> 0;
  }
  return HASH_POOL[h % HASH_POOL.length];
}

/** Color used for a record entity type's pills + legend chip. */
export function entityTypeColor(entityType: string): string {
  return SYSTEM_NODE_TYPE_COLORS[entityTypeToSystemType(entityType)] ?? "#64748B";
}

export const LEGEND_RULE = "rule";
export const LEGEND_WORKFLOW = "workflow";
export const LEGEND_AGENT = "agent";
export const LEGEND_ORG_UNIT = "orgUnit";

function entityLegendKey(entityType: string): Object360LegendKey {
  return `entity:${entityType}`;
}

/** Stable node key for an inspect target (used for `inspectingKey`). */
export function recordNodeKey(target: Object360InspectTarget): string {
  return "kind" in target ? `orgunit:${target.id}` : `record:${target.entityType}:${target.entityId}`;
}

interface BuiltModel {
  model: GraphModel;
  /** node key → inspect target (records + org unit). */
  targetByKey: Map<string, Object360InspectTarget>;
  /** node key → legend key. */
  legendKeyByNode: Map<string, Object360LegendKey>;
  legend: { key: Object360LegendKey; label: string; color: string; count: number }[];
}

function humanize(s: string): string {
  return s
    .replace(/[_-]+/g, " ")
    .replace(/\\s+/g, " ")
    .trim()
    .replace(/\\b\\w/g, (c) => c.toUpperCase());
}

export function buildRecordGraphModel(
  root: Object360RecordRef,
  data: Object360GraphData,
  entityTypeLabel: (entityType: string) => string,
): BuiltModel {
  const nodes: GraphNode[] = [];
  const edges: GraphEdge[] = [];
  const targetByKey = new Map<string, Object360InspectTarget>();
  const legendKeyByNode = new Map<string, Object360LegendKey>();
  const legendCounts = new Map<Object360LegendKey, number>();
  const legendMeta = new Map<Object360LegendKey, { label: string; color: string }>();

  const bump = (legendKey: Object360LegendKey, label: string, color: string) => {
    legendCounts.set(legendKey, (legendCounts.get(legendKey) ?? 0) + 1);
    if (!legendMeta.has(legendKey)) legendMeta.set(legendKey, { label, color });
  };

  const rootKey = recordNodeKey(root);
  const rootType = entityTypeToSystemType(root.entityType);
  nodes.push({ key: rootKey, id: `${root.entityType}:${root.entityId}`, type: rootType, label: root.label, isRoot: true });
  targetByKey.set(rootKey, root);
  const rootLegendKey = entityLegendKey(root.entityType);
  legendKeyByNode.set(rootKey, rootLegendKey);
  bump(rootLegendKey, entityTypeLabel(root.entityType), entityTypeColor(root.entityType));

  const addNeighbor = (
    node: GraphNode,
    target: Object360InspectTarget | null,
    legendKey: Object360LegendKey,
    legendLabel: string,
    legendColor: string,
    edgeLabel: string,
  ) => {
    nodes.push(node);
    if (target) targetByKey.set(node.key, target);
    legendKeyByNode.set(node.key, legendKey);
    bump(legendKey, legendLabel, legendColor);
    edges.push({ fromKey: node.key, toKey: rootKey, edgeType: "related_to", label: edgeLabel });
  };

  data.related.forEach((m) => {
    const key = `record:${m.entity_type}:${m.entity_id}`;
    const type = entityTypeToSystemType(m.entity_type);
    addNeighbor(
      { key, id: `${m.entity_type}:${m.entity_id}`, type, label: m.display_name || m.entity_id },
      { entityType: m.entity_type, entityId: m.entity_id, label: m.display_name || m.entity_id },
      entityLegendKey(m.entity_type),
      entityTypeLabel(m.entity_type),
      entityTypeColor(m.entity_type),
      humanize(m.label || m.relationship_key || "related"),
    );
  });

  data.businessRules.forEach((r) => {
    addNeighbor(
      { key: `rule:${r.id}`, id: r.id, type: "business_rule", label: r.name },
      null,
      LEGEND_RULE,
      "Business Rules",
      SYSTEM_NODE_TYPE_COLORS.business_rule,
      "applies to",
    );
  });

  data.workflows.forEach((w) => {
    addNeighbor(
      { key: `workflow:${w.id}`, id: w.id, type: "workflow", label: w.name },
      null,
      LEGEND_WORKFLOW,
      "Workflows",
      SYSTEM_NODE_TYPE_COLORS.workflow,
      "applies to",
    );
  });

  data.scopedAgents.forEach((a) => {
    addNeighbor(
      { key: `agent:${a.id}`, id: a.id, type: "ai_agent", label: a.name },
      null,
      LEGEND_AGENT,
      "AI Agents",
      SYSTEM_NODE_TYPE_COLORS.ai_agent,
      "scoped to",
    );
  });

  if (data.orgUnit) {
    const key = `orgunit:${data.orgUnit.id}`;
    addNeighbor(
      { key, id: data.orgUnit.id, type: "page_layout", label: data.orgUnit.name },
      { kind: "orgUnit", id: data.orgUnit.id, label: data.orgUnit.name },
      LEGEND_ORG_UNIT,
      "Business Unit",
      SYSTEM_NODE_TYPE_COLORS.page_layout,
      "belongs to",
    );
  }

  const legend = Array.from(legendCounts.entries()).map(([key, count]) => ({
    key,
    label: legendMeta.get(key)!.label,
    color: legendMeta.get(key)!.color,
    count,
  }));

  return { model: { nodes, edges }, targetByKey, legendKeyByNode, legend };
}

export interface Object360GraphProps {
  root: Object360RecordRef;
  data: Object360GraphData;
  entityTypeLabel: (entityType: string) => string;
  /** Currently inspected node (outline ring). */
  inspecting: Object360InspectTarget;
  /** Hidden legend keys (layer toggles). */
  hiddenLegendKeys: Set<string>;
  onToggleLegendKey: (key: string) => void;
  onInspect: (target: Object360InspectTarget) => void;
  /** Double-click / "show what's around it" re-centers the graph. */
  onRecenter: (target: Object360RecordRef) => void;
  className?: string;
}

export const Object360Graph = memo(function Object360Graph(props: Object360GraphProps) {
  const { root, data, entityTypeLabel, inspecting, hiddenLegendKeys, onToggleLegendKey, onInspect, onRecenter, className } =
    props;

  const built = useMemo(() => buildRecordGraphModel(root, data, entityTypeLabel), [root, data, entityTypeLabel]);

  // Filter hidden legend keys out of the model so layout/render never see them.
  const visibleModel = useMemo<GraphModel>(() => {
    const keep = new Set<string>();
    built.model.nodes.forEach((n) => {
      const lk = built.legendKeyByNode.get(n.key);
      if (!lk || !hiddenLegendKeys.has(lk)) keep.add(n.key);
    });
    return {
      nodes: built.model.nodes.filter((n) => keep.has(n.key)),
      edges: built.model.edges.filter((e) => keep.has(e.fromKey) && keep.has(e.toKey)),
    };
  }, [built, hiddenLegendKeys]);

  const hover = useHoverDimming(visibleModel);
  const rootKey = recordNodeKey(root);
  const inspectingKey = recordNodeKey(inspecting);

  // Legend sync: legend keys present in the hover spotlight's neighborhood.
  const connectedLegendKeys = useMemo(() => {
    const s = new Set<Object360LegendKey>();
    hover.connectedSet.forEach((k) => {
      const lk = built.legendKeyByNode.get(k);
      if (lk) s.add(lk);
    });
    return s;
  }, [hover.connectedSet, built.legendKeyByNode]);

  const handleNodeClick = (node: GraphNode) => {
    const target = built.targetByKey.get(node.key);
    if (target) onInspect(target);
  };

  const handleNodeDoubleClick = (node: GraphNode) => {
    const target = built.targetByKey.get(node.key);
    if (target && !("kind" in target)) onRecenter(target);
  };

  return (
    <div className={cn("object360-graph", className)} style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Graph layers: legend chips double as toggle-filters (demo parity). */}
      {built.legend.length > 0 && (
        <div className="ontology-legend" role="group" aria-label="Graph layers">
          {built.legend.map((l) => {
            const hidden = hiddenLegendKeys.has(l.key);
            const highlighted = hover.dimming && connectedLegendKeys.has(l.key);
            const dimmed = hover.dimming && !connectedLegendKeys.has(l.key);
            return (
              <button
                key={l.key}
                type="button"
                className={cn(
                  "ontology-legend-item",
                  hidden && "ontology-legend-item-off",
                  highlighted && "legend-highlight",
                  dimmed && "legend-dim",
                )}
                data-legend-toggle={l.key}
                onClick={() => onToggleLegendKey(l.key)}
                title={`${hidden ? "Show" : "Hide"} ${l.label}`}
              >
                <span
                  style={{ display: "inline-block", width: 10, height: 10, borderRadius: "50%", background: l.color, flexShrink: 0 }}
                />
                {l.label}
                <span className="ontology-legend-count">{l.count}</span>
              </button>
            );
          })}
        </div>
      )}

      <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
        <GraphCanvas
          model={visibleModel}
          rootKey={rootKey}
          hiddenTypes={new Set()}
          inspectingKey={inspectingKey}
          onNodeClick={handleNodeClick}
          onNodeDoubleClick={handleNodeDoubleClick}
          hover={hover}
        />
      </div>
    </div>
  );
});
""")

# File: desktop/src/components/graph/GraphInspector.tsx
write_file("desktop/src/components/graph/GraphInspector.tsx", """/**
 * PR 3 — Shared graph inspector panel.
 *
 * Presentational inspector used by the graph surfaces (Object 360 today;
 * the Ontology Graph admin inspector can adopt it when behavior matches).
 * Shows a type-badge header, action buttons, and grouped relationship
 * sections — the same visual contract as the demo's Object 360 inspector.
 */
import { memo, type ReactNode } from "react";

import { cn } from "../ui/cn";
import { Button } from "../ui/button";

export interface InspectorItem {
  key: string;
  label: string;
  /** Colored dot shown before the label. */
  dotColor?: string;
  /** When present the item renders as a button (inspector bullet). */
  onClick?: () => void;
  /** Trailing content (e.g. an arrow). */
  right?: ReactNode;
}

export interface InspectorGroup {
  key: string;
  label: string;
  items: InspectorItem[];
}

export interface InspectorAction {
  label: string;
  onClick: () => void;
  variant?: "default" | "secondary" | "outline" | "ghost";
  disabled?: boolean;
}

export interface GraphInspectorProps {
  title: string;
  subtitle?: string;
  /** Badge dot color (node type color). */
  badgeColor?: string;
  actions?: InspectorAction[];
  groups: InspectorGroup[];
  /** Message when there are no groups. */
  emptyMessage?: string;
  footer?: ReactNode;
  className?: string;
  /** Extra props spread onto the header (e.g. drag handlers). */
  headerProps?: React.HTMLAttributes<HTMLDivElement>;
}

function Dot({ color }: { color: string }) {
  return (
    <span
      aria-hidden="true"
      style={{ display: "inline-block", width: 10, height: 10, borderRadius: "50%", background: color, flexShrink: 0 }}
    />
  );
}

export const GraphInspector = memo(function GraphInspector(props: GraphInspectorProps) {
  const { title, subtitle, badgeColor, actions, groups, emptyMessage, footer, className, headerProps } = props;

  return (
    <div className={cn("graph-inspector", className)}>
      <div className="ontology-inspector-head" {...headerProps}>
        {badgeColor && <Dot color={badgeColor} />}
        <div>
          <strong>{title}</strong>
          {subtitle && <div className="ontology-inspector-subtitle">{subtitle}</div>}
        </div>
      </div>

      {actions && actions.length > 0 && (
        <div className="ontology-inspector-actions">
          {actions.map((a) => (
            <Button
              key={a.label}
              variant={a.variant ?? "secondary"}
              size="sm"
              onClick={a.onClick}
              disabled={a.disabled}
            >
              {a.label}
            </Button>
          ))}
        </div>
      )}

      <div className="ontology-inspector-groups">
        {groups.map((g) => (
          <div key={g.key} className="ontology-inspector-group">
            <div className="ontology-inspector-group-label">{g.label}</div>
            {g.items.map((it) =>
              it.onClick ? (
                <button key={it.key} type="button" className="ontology-inspector-bullet" onClick={it.onClick}>
                  {it.dotColor && <Dot color={it.dotColor} />}
                  <span>{it.label}</span>
                  {it.right}
                </button>
              ) : (
                <div key={it.key} className="ontology-inspector-row" style={{ fontSize: 13 }}>
                  {it.dotColor && <Dot color={it.dotColor} />}
                  <span>{it.label}</span>
                  {it.right}
                </div>
              ),
            )}
          </div>
        ))}
        {groups.length === 0 && emptyMessage && (
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>{emptyMessage}</p>
        )}
      </div>

      {footer}
    </div>
  );
});
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
export { Object360Graph, type Object360GraphProps, type Object360GraphData, type Object360RecordRef, type Object360InspectTarget, entityTypeToSystemType, entityTypeColor, recordNodeKey } from "./Object360Graph";
export { GraphInspector, type GraphInspectorProps, type InspectorGroup, type InspectorItem, type InspectorAction } from "./GraphInspector";
""")

# File: desktop/src/components/Object360Modal.tsx
write_file("desktop/src/components/Object360Modal.tsx", """/**
 * Object 360 (PR 3): record-level graph rebuilt on the shared pill-node
 * engine (`graph/Object360Graph`), mirroring the demo's Object 360.
 *
 * Keeps the modal shell, title/breadcrumb, all data queries (related
 * records, business rules, workflows, scoped agents, ownership/org units,
 * audit events), the `inspecting` state, `onOpenRecord`, and the draggable
 * inspector card. The old radial dot-graph is replaced by
 * `<Object360Graph>`; pill clicks refresh the inspector.
 */
import { useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import type { RelatedRecord } from "../lib/types";
import {
  Object360Graph,
  entityTypeColor,
  type Object360GraphData,
  type Object360InspectTarget,
  type Object360RecordRef,
} from "./graph/Object360Graph";
import { GraphInspector, type InspectorGroup } from "./graph/GraphInspector";

const MAX_BULLETS_PER_GROUP = 20;

type RootRef = Object360RecordRef;
type InspectTarget = Object360InspectTarget;

export function Object360Modal({
  entityType,
  entityId,
  label,
  onOpenRecord,
  onClose,
}: {
  entityType: string;
  entityId: string;
  label: string;
  onOpenRecord?: (entityType: string, entityId: string) => void;
  onClose: () => void;
}) {
  const [root, setRoot] = useState<RootRef>({ entityType, entityId, label });
  const [inspecting, setInspecting] = useState<InspectTarget>({ entityType, entityId, label });
  const [hiddenLegendKeys, setHiddenLegendKeys] = useState<Set<string>>(() => new Set());
  const [cardOffset, setCardOffset] = useState<{ left: number; top: number } | null>(null);
  const dragStateRef = useRef<{ pointerId: number; startX: number; startY: number; origLeft: number; origTop: number } | null>(null);

  function toggleLegendKey(key: string) {
    setHiddenLegendKeys((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

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

  const objectTypes = useQuery({ queryKey: ["ontologyObjectTypes"], queryFn: () => api.listObjectTypes() });
  // Unions the generic, admin-defined relationship system with the
  // built-in CRM's own FK-based relationships (company_id on contacts/
  // opportunities/..., Task.related_type/related_id, ...) - the latter
  // predates and completely bypasses the former, so a fresh workspace's
  // own Company/Contact/etc. records would otherwise show zero related
  // records here. See `native_related_records_for`'s own doc comment.
  const related = useQuery({
    queryKey: ["relatedRecords", root.entityType, root.entityId],
    queryFn: async () => {
      const [generic, native] = await Promise.all([api.listRelatedRecords(root.entityType, root.entityId), api.listNativeRelatedRecords(root.entityType, root.entityId)]);
      return [...generic, ...native];
    },
  });
  const rules = useQuery({ queryKey: ["businessRulesForEntity", root.entityType], queryFn: () => api.listBusinessRules(root.entityType, true) });
  const workflows = useQuery({ queryKey: ["workflowsForEntity", root.entityType], queryFn: () => api.listWorkflowRules(root.entityType) });
  const scopedAgents = useQuery({ queryKey: ["agentsScopedToObjectType", root.entityType], queryFn: () => api.listAiAgentsScopedToObjectType(root.entityType, true) });
  const ownership = useQuery({ queryKey: ["recordOwner", root.entityType, root.entityId], queryFn: () => api.getRecordOwner(root.entityType, root.entityId) });
  const orgUnits = useQuery({ queryKey: ["orgUnits"], queryFn: () => api.listOrgUnits() });
  const auditEvents = useQuery({ queryKey: ["auditEventsForEntity", root.entityType, root.entityId], queryFn: () => api.listAuditEvents(root.entityType, root.entityId) });

  const activeWorkflows = (workflows.data ?? []).filter((w) => w.is_active);
  const orgUnit = ownership.data?.owning_org_unit_id ? orgUnits.data?.find((o) => o.id === ownership.data!.owning_org_unit_id) : null;

  function entityTypeLabel(et: string): string {
    return objectTypes.data?.find((o) => o.key === et)?.label_singular ?? et;
  }

  const graphData = useMemo<Object360GraphData>(
    () => ({
      related: related.data ?? [],
      businessRules: (rules.data ?? []).map((r) => ({ id: r.id, name: r.name })),
      workflows: activeWorkflows.map((w) => ({ id: w.id, name: w.name })),
      scopedAgents: (scopedAgents.data ?? []).map((a) => ({ id: a.id, name: a.name })),
      orgUnit: orgUnit ? { id: orgUnit.id, name: orgUnit.name } : null,
    }),
    [related.data, rules.data, activeWorkflows, scopedAgents.data, orgUnit],
  );

  const groups = useMemo(() => {
    const map = new Map<string, RelatedRecord[]>();
    (related.data ?? []).forEach((r) => {
      const list = map.get(r.relationship_key) ?? [];
      list.push(r);
      map.set(r.relationship_key, list);
    });
    return Array.from(map.entries());
  }, [related.data]);

  const inspectingIsRoot = !("kind" in inspecting) && inspecting.entityType === root.entityType && inspecting.entityId === root.entityId;

  function recenterOn(target: RootRef) {
    setRoot(target);
    setInspecting(target);
  }

  const dragHeadProps = {
    style: { cursor: "grab", touchAction: "none" as const },
    onPointerDown: startCardDrag,
    onPointerMove: onCardDragMove,
    onPointerUp: endCardDrag,
    onPointerCancel: endCardDrag,
  };

  // Inspector content for the currently inspected target.
  const inspector = (() => {
    if ("kind" in inspecting) {
      return (
        <GraphInspector
          title={inspecting.label}
          subtitle="Business Unit"
          badgeColor={entityTypeColor("BusinessUnit")}
          groups={[]}
          headerProps={dragHeadProps}
          footer={
            <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
              Business Units aren't individually re-centerable here — see Admin → Organization Units for its own detail.
            </p>
          }
        />
      );
    }

    const inspectorGroups: InspectorGroup[] = [];
    if (inspectingIsRoot) {
      groups.forEach(([key, members]) => {
        inspectorGroups.push({
          key: `rel-${key}`,
          label: members[0]?.label ?? key,
          items: members.slice(0, MAX_BULLETS_PER_GROUP).map((m) => ({
            key: m.instance_id,
            label: m.display_name,
            dotColor: entityTypeColor(m.entity_type),
            onClick: () => setInspecting({ entityType: m.entity_type, entityId: m.entity_id, label: m.display_name }),
            right: <span aria-hidden="true" style={{ marginLeft: "auto", color: "var(--text-muted)" }}>→</span>,
          })),
        });
      });
      if (orgUnit) {
        inspectorGroups.push({
          key: "org-unit",
          label: "Business Unit",
          items: [{ key: orgUnit.id, label: orgUnit.name, dotColor: entityTypeColor("BusinessUnit") }],
        });
      }
      if ((rules.data ?? []).length > 0) {
        inspectorGroups.push({
          key: "rules",
          label: "Business Rules that apply",
          items: (rules.data ?? []).map((r) => ({ key: r.id, label: r.name, dotColor: entityTypeColor("business_rule") })),
        });
      }
      if (activeWorkflows.length > 0) {
        inspectorGroups.push({
          key: "workflows",
          label: "Workflows that apply",
          items: activeWorkflows.map((w) => ({ key: w.id, label: w.name, dotColor: entityTypeColor("workflow") })),
        });
      }
      if ((scopedAgents.data ?? []).length > 0) {
        inspectorGroups.push({
          key: "agents",
          label: "AI Agents scoped to this object type",
          items: (scopedAgents.data ?? []).map((a) => ({ key: a.id, label: a.name, dotColor: entityTypeColor("ai_agent") })),
        });
      }
      if ((auditEvents.data ?? []).length > 0) {
        inspectorGroups.push({
          key: "activity",
          label: "Activity",
          items: (auditEvents.data ?? []).slice(0, 10).map((e) => ({
            key: e.id,
            label: e.summary,
            right: (
              <span style={{ display: "block", fontSize: 11, color: "var(--text-muted)" }}>
                {new Date(e.occurred_at).toLocaleString()}
              </span>
            ),
          })),
        });
      }
    }

    return (
      <GraphInspector
        title={inspecting.label}
        subtitle={entityTypeLabel(inspecting.entityType)}
        badgeColor={entityTypeColor(inspecting.entityType)}
        headerProps={dragHeadProps}
        actions={[
          ...(onOpenRecord
            ? [
                {
                  label: "Open record",
                  variant: "default" as const,
                  onClick: () => onOpenRecord(inspecting.entityType, inspecting.entityId),
                },
              ]
            : []),
          {
            label: "Show what's around it",
            variant: "secondary" as const,
            disabled: inspectingIsRoot,
            onClick: () => recenterOn(inspecting as RootRef),
          },
        ]}
        groups={inspectorGroups}
        emptyMessage={inspectingIsRoot ? "No related records yet." : undefined}
      />
    );
  })();

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" style={{ width: "min(1100px, 95vw)", maxHeight: "92vh" }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 4 }}>
          <h3 style={{ margin: 0 }}>Object 360 · {root.label}</h3>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
        </div>
        <div className="breadcrumb" style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 8 }}>
          {entityTypeLabel(root.entityType)} <span aria-hidden="true">›</span> {root.label} <span aria-hidden="true">›</span> Relationship neighborhood
        </div>
        <p style={{ color: "var(--text-muted)", fontSize: 13, maxWidth: 720 }}>
          Every record directly linked to {root.label}, its Business Unit, the Business Rules/Workflows that apply to records like
          it, and which AI Agents are scoped to act on it.
        </p>

        <div className="graph-canvas-wrap ontology-canvas-wrap" style={{ position: "relative", height: 560 }}>
          <Object360Graph
            root={root}
            data={graphData}
            entityTypeLabel={entityTypeLabel}
            inspecting={inspecting}
            hiddenLegendKeys={hiddenLegendKeys}
            onToggleLegendKey={toggleLegendKey}
            onInspect={setInspecting}
            onRecenter={recenterOn}
          />

          <div className="ontology-inspector-card" style={cardOffset ? { left: cardOffset.left, top: cardOffset.top, right: "auto" } : undefined}>
            {inspector}
          </div>
        </div>
      </div>
    </div>
  );
}

""")

# File: desktop/vite.config.ts
write_file("desktop/vite.config.ts", """import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri expects a fixed port and will fail if it is already in use.
const port = 1420;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },
});
""")

# File: desktop/src/test-setup.ts
write_file("desktop/src/test-setup.ts", """/**
 * PR 3 — vitest setup.
 *
 * - jest-dom matchers
 * - ResizeObserver mock (GraphCanvas measures its container; jsdom lacks it)
 * - Pointer capture mocks (canvas drag-pan; jsdom lacks them)
 */
import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// jsdom + explicit imports (no vitest globals): wire up RTL auto-cleanup.
afterEach(() => {
  cleanup();
});

class ResizeObserverMock {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

(globalThis as unknown as { ResizeObserver: unknown }).ResizeObserver = ResizeObserverMock;

if (typeof Element !== "undefined") {
  const proto = Element.prototype as unknown as Record<string, unknown>;
  if (typeof proto.setPointerCapture !== "function") {
    proto.setPointerCapture = () => {};
  }
  if (typeof proto.releasePointerCapture !== "function") {
    proto.releasePointerCapture = () => {};
  }
  if (typeof proto.hasPointerCapture !== "function") {
    proto.hasPointerCapture = () => false;
  }
}
""")

# File: desktop/src/__tests__/graph.test.tsx
write_file("desktop/src/__tests__/graph.test.tsx", """/**
 * PR 3 — GraphCanvas regression tests.
 *
 * Verifies the shared pill-node renderer: one pill per node, root pill
 * styling, dashed-ellipse cluster boundaries per type, edges with
 * data-edge-from/to, edge labels, inspecting ring, and hidden-type
 * filtering.
 */
import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphCanvas } from "../components/graph/GraphCanvas";
import { fixtureModel } from "./fixtures";

function renderCanvas(extra: Partial<Parameters<typeof GraphCanvas>[0]> = {}) {
  return render(
    <GraphCanvas
      model={fixtureModel}
      rootKey="custom_object:companies"
      hiddenTypes={new Set()}
      {...extra}
    />,
  );
}

describe("GraphCanvas", () => {
  it("renders one pill per node with data-node-key", () => {
    const { container } = renderCanvas();
    const pills = container.querySelectorAll(".og-pill");
    expect(pills).toHaveLength(fixtureModel.nodes.length);
    const keys = Array.from(pills).map((p) => p.getAttribute("data-node-key"));
    fixtureModel.nodes.forEach((n) => expect(keys).toContain(n.key));
  });

  it("renders the root pill with the root class", () => {
    const { container } = renderCanvas();
    const root = container.querySelector('[data-node-key="custom_object:companies"]');
    expect(root).toHaveClass("og-pill-root");
  });

  it("renders dashed-ellipse cluster boundaries per type", () => {
    const { container } = renderCanvas();
    // custom_object, custom_field, workflow, ai_agent, metric_definition
    expect(container.querySelectorAll("ellipse").length).toBe(5);
  });

  it("renders edges with data-edge-from/to and the access edge dashed", () => {
    const { container } = renderCanvas();
    const lines = container.querySelectorAll("line[data-edge-from]");
    expect(lines).toHaveLength(fixtureModel.edges.length);
    const access = container.querySelector('line[data-edge-from="ai_agent:a1"]');
    expect(access?.getAttribute("stroke-dasharray")).toBeTruthy();
  });

  it("renders an edge label per edge", () => {
    const { container } = renderCanvas();
    expect(container.querySelectorAll(".ontology-edge-label")).toHaveLength(fixtureModel.edges.length);
  });

  it("marks the inspected pill with the inspecting ring", () => {
    const { container } = renderCanvas({ inspectingKey: "custom_field:name" });
    const pill = container.querySelector('[data-node-key="custom_field:name"]');
    expect(pill).toHaveClass("og-pill-inspecting");
  });

  it("hides nodes of hidden types", () => {
    const { container } = renderCanvas({ hiddenTypes: new Set(["workflow"]) });
    expect(container.querySelector('[data-node-key="workflow:wf1"]')).toBeNull();
    // other pills remain
    expect(container.querySelector('[data-node-key="ai_agent:a1"]')).not.toBeNull();
  });

  it("fires onNodeClick with the clicked node", () => {
    const onNodeClick = vi.fn();
    const { container } = renderCanvas({ onNodeClick });
    const pill = container.querySelector('[data-node-key="ai_agent:a1"]')!;
    fireEvent.click(pill);
    expect(onNodeClick).toHaveBeenCalledTimes(1);
    expect(onNodeClick.mock.calls[0][0]).toMatchObject({ key: "ai_agent:a1", label: "Research Analyst" });
  });
});
""")

# File: desktop/src/__tests__/hover.test.tsx
write_file("desktop/src/__tests__/hover.test.tsx", """/**
 * PR 3 — Hover spotlight regression tests.
 *
 * Verifies the demo-parity spotlight: hovering a pill dims everything
 * outside its 1-hop neighborhood (`graph-hover-dim` on the canvas,
 * `graph-hover-on` on connected pills), mouse-leave clears it, hovering
 * a cluster highlights the whole type, and the connected type set (the
 * legend-sync contract) is computed correctly.
 */
import { act, fireEvent, render, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphCanvas } from "../components/graph/GraphCanvas";
import { GraphLegend } from "../components/graph/GraphLegend";
import { useHoverDimming } from "../components/graph/useHoverDimming";
import { fixtureModel, ROOT_KEY as ROOT } from "./fixtures";

describe("useHoverDimming", () => {
  it("starts with no spotlight", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    expect(result.current.dimming).toBe(false);
    expect(result.current.connectedSet.size).toBe(0);
    expect(result.current.hoveredKey).toBeNull();
  });

  it("computes the 1-hop connected set on pill enter", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.dimming).toBe(true);
    // root + its 4 neighbors; the unconnected metric node stays out.
    expect(result.current.connectedSet.has(ROOT)).toBe(true);
    expect(result.current.connectedSet.has("custom_field:name")).toBe(true);
    expect(result.current.connectedSet.has("ai_agent:a1")).toBe(true);
    expect(result.current.connectedSet.has("metric_definition:m1")).toBe(false);
  });

  it("exposes connected types for legend sync", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.connectedTypes.has("custom_object")).toBe(true);
    expect(result.current.connectedTypes.has("custom_field")).toBe(true);
    expect(result.current.connectedTypes.has("workflow")).toBe(true);
    expect(result.current.connectedTypes.has("ai_agent")).toBe(true);
    expect(result.current.connectedTypes.has("metric_definition")).toBe(false);
  });

  it("clears the spotlight on leave", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onPillEnter(ROOT));
    expect(result.current.dimming).toBe(true);
    act(() => result.current.onLeave());
    expect(result.current.dimming).toBe(false);
    expect(result.current.connectedSet.size).toBe(0);
  });

  it("highlights the whole type on cluster enter", () => {
    const { result } = renderHook(() => useHoverDimming(fixtureModel));
    act(() => result.current.onClusterEnter("custom_field"));
    expect(result.current.dimming).toBe(true);
    expect(result.current.connectedSet.has("custom_field:name")).toBe(true);
    expect(result.current.connectedSet.has("custom_field:revenue")).toBe(true);
    expect(result.current.connectedSet.has(ROOT)).toBe(false);
  });
});

describe("GraphCanvas hover spotlight", () => {
  it("dims the canvas and highlights the neighborhood on pill hover", () => {
    const { container } = render(
      <GraphCanvas model={fixtureModel} rootKey={ROOT} hiddenTypes={new Set()} />,
    );
    const canvas = container.querySelector(".ontology-graph-canvas")!;
    expect(canvas).not.toHaveClass("graph-hover-dim");

    const rootPill = container.querySelector(`[data-node-key="${ROOT}"]`)!;
    fireEvent.mouseEnter(rootPill);

    expect(canvas).toHaveClass("graph-hover-dim");
    expect(rootPill).toHaveClass("graph-hover-on");
    expect(container.querySelector('[data-node-key="custom_field:name"]')).toHaveClass("graph-hover-on");
    // Unconnected metric pill is dimmed (no hover-on class).
    expect(container.querySelector('[data-node-key="metric_definition:m1"]')).not.toHaveClass("graph-hover-on");

    fireEvent.mouseLeave(rootPill);
    expect(canvas).not.toHaveClass("graph-hover-dim");
  });
});

describe("GraphLegend sync", () => {
  it("highlights neighborhood types and dims the rest while spotlight is active", () => {
    const highlightedTypes = new Set(["custom_object", "custom_field", "workflow", "ai_agent"] as const);
    const { container } = render(
      <GraphLegend
        model={fixtureModel}
        hiddenTypes={new Set()}
        onToggleType={() => {}}
        highlightedTypes={highlightedTypes as unknown as Set<import("../lib/types").SystemNodeType>}
        spotlightActive
      />,
    );
    // Neighborhood types get the highlight ring…
    const chips = Array.from(container.querySelectorAll(".ontology-legend-item"));
    const highlighted = chips.filter((c) => c.classList.contains("legend-highlight"));
    expect(highlighted.length).toBe(4);
    // …and the out-of-neighborhood metric type is dimmed.
    const metricChip = chips.find((c) => c.getAttribute("data-legend-toggle") === "metric_definition");
    expect(metricChip).toHaveClass("legend-dim");
    expect(metricChip).not.toHaveClass("legend-highlight");
  });

  it("applies no sync classes when the spotlight is inactive", () => {
    const { container } = render(
      <GraphLegend model={fixtureModel} hiddenTypes={new Set()} onToggleType={() => {}} />,
    );
    expect(container.querySelectorAll(".legend-highlight,.legend-dim")).toHaveLength(0);
  });

  it("toggles a type off on chip click", () => {
    const onToggleType = vi.fn();
    const { container } = render(
      <GraphLegend model={fixtureModel} hiddenTypes={new Set()} onToggleType={onToggleType} />,
    );
    const chip = container.querySelector('[data-legend-toggle="workflow"]')!;
    fireEvent.click(chip);
    expect(onToggleType).toHaveBeenCalledWith("workflow");
  });
});
""")

# File: desktop/src/__tests__/toolbar.test.tsx
write_file("desktop/src/__tests__/toolbar.test.tsx", """/**
 * PR 3 — GraphToolbar regression tests.
 *
 * Verifies the compact 8-group toolbar: all groups render, zoom/fit/
 * reset callbacks fire, the zoom readout tracks state, depth changes
 * propagate, impact/lineage/fields toggles fire, find-path mode toggles,
 * and search input/selects behave.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GraphToolbar, type GraphToolbarProps } from "../components/graph/GraphToolbar";

function baseProps(overrides: Partial<GraphToolbarProps> = {}): GraphToolbarProps {
  return {
    nodeType: "custom_object",
    onNodeTypeChange: vi.fn(),
    componentId: "companies",
    onComponentChange: vi.fn(),
    componentOptions: [{ id: "companies", label: "Companies" }],
    searchQuery: "",
    onSearchQueryChange: vi.fn(),
    searchResults: [],
    onSearchSelect: vi.fn(),
    pathMode: false,
    onTogglePathMode: vi.fn(),
    zoom: 1,
    onZoomIn: vi.fn(),
    onZoomOut: vi.fn(),
    onFit: vi.fn(),
    onReset: vi.fn(),
    depth: "1",
    onDepthChange: vi.fn(),
    showImpact: false,
    onToggleImpact: vi.fn(),
    showLineage: false,
    onToggleLineage: vi.fn(),
    showFields: false,
    onToggleFields: vi.fn(),
    ...overrides,
  };
}

describe("GraphToolbar", () => {
  it("renders all 8 tool groups", () => {
    const { container } = render(<GraphToolbar {...baseProps()} />);
    expect(container.querySelectorAll(".og-toolbar-card .tool-group")).toHaveLength(8);
  });

  it("fires zoom callbacks and shows the zoom readout", () => {
    const props = baseProps({ zoom: 1.5 });
    const { container } = render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Zoom in" }));
    expect(props.onZoomIn).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Zoom out" }));
    expect(props.onZoomOut).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Fit" }));
    expect(props.onFit).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(props.onReset).toHaveBeenCalledTimes(1);
    expect(container.querySelector("[data-zoom-readout]")?.textContent).toBe("150%");
  });

  it("propagates depth changes", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.change(screen.getByLabelText("Depth"), { target: { value: "3" } });
    expect(props.onDepthChange).toHaveBeenCalledWith("3");
  });

  it("toggles find-path mode", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Find path" }));
    expect(props.onTogglePathMode).toHaveBeenCalledTimes(1);
  });

  it("toggles impact / lineage chips", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    const impact = screen.getByRole("button", { name: "Impact" });
    expect(impact).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(impact);
    expect(props.onToggleImpact).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Lineage" }));
    expect(props.onToggleLineage).toHaveBeenCalledTimes(1);
  });

  it("toggles the fields checkbox", () => {
    const props = baseProps();
    const { container } = render(<GraphToolbar {...props} />);
    const checkbox = container.querySelector('input[type="checkbox"]') as HTMLInputElement;
    expect(checkbox).not.toBeNull();
    fireEvent.click(checkbox);
    expect(props.onToggleFields).toHaveBeenCalledTimes(1);
  });

  it("forwards search input", () => {
    const props = baseProps();
    render(<GraphToolbar {...props} />);
    fireEvent.change(screen.getByLabelText("Search components"), { target: { value: "comp" } });
    expect(props.onSearchQueryChange).toHaveBeenCalledWith("comp");
  });

  it("fires simulate and copy-link when provided", () => {
    const props = baseProps({ onSimulate: vi.fn(), onCopyLink: vi.fn() });
    render(<GraphToolbar {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Simulate change" }));
    expect(props.onSimulate).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Copy link" }));
    expect(props.onCopyLink).toHaveBeenCalledTimes(1);
  });

  it("shows the path banner with clear action when a path is set", () => {
    const props = baseProps({ pathFromLabel: "A", pathToLabel: "B", onClearPath: vi.fn() });
    render(<GraphToolbar {...props} />);
    expect(screen.getByText(/A → B/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Clear path" }));
    expect(props.onClearPath).toHaveBeenCalledTimes(1);
  });
});
""")

# File: desktop/src/__tests__/object360.test.tsx
write_file("desktop/src/__tests__/object360.test.tsx", """/**
 * PR 3 — Object360Modal regression tests.
 *
 * Verifies the demo-verified behavior on the shared engine: the modal
 * opens with the record graph, legend layer chips render, clicking a
 * related pill refreshes the inspector, "Show what's around it"
 * re-centers the graph, "Open record" fires, and Close works.
 */
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Object360Modal } from "../components/Object360Modal";
import { api } from "../lib/api";

vi.mock("../lib/api", () => ({
  api: {
    listObjectTypes: vi.fn(),
    listRelatedRecords: vi.fn(),
    listNativeRelatedRecords: vi.fn(),
    listBusinessRules: vi.fn(),
    listWorkflowRules: vi.fn(),
    listAiAgentsScopedToObjectType: vi.fn(),
    getRecordOwner: vi.fn(),
    listOrgUnits: vi.fn(),
    listAuditEvents: vi.fn(),
  },
}));

const objectTypes = [
  { key: "Company", label_singular: "Company", color_index: 1 },
  { key: "Contact", label_singular: "Contact", color_index: 2 },
  { key: "Opportunity", label_singular: "Opportunity", color_index: 3 },
];

const relatedRecords = [
  {
    instance_id: "rel-1",
    relationship_definition_id: "rd1",
    relationship_key: "contacts",
    label: "contacts",
    entity_type: "Contact",
    entity_id: "c2",
    display_name: "Ava Martin",
    status: "active",
    archived: false,
  },
  {
    instance_id: "rel-2",
    relationship_definition_id: "rd2",
    relationship_key: "opportunities",
    label: "opportunities",
    entity_type: "Opportunity",
    entity_id: "o9",
    display_name: "Platform Modernization",
    status: "open",
    archived: false,
  },
];

function mockApi() {
  vi.mocked(api.listObjectTypes).mockResolvedValue(objectTypes as never);
  vi.mocked(api.listRelatedRecords).mockResolvedValue(relatedRecords as never);
  vi.mocked(api.listNativeRelatedRecords).mockResolvedValue([] as never);
  vi.mocked(api.listBusinessRules).mockResolvedValue([{ id: "r1", name: "Credit limit check" }] as never);
  vi.mocked(api.listWorkflowRules).mockResolvedValue([
    { id: "w1", name: "Renewal workflow", is_active: true },
    { id: "w2", name: "Retired workflow", is_active: false },
  ] as never);
  vi.mocked(api.listAiAgentsScopedToObjectType).mockResolvedValue([{ id: "a1", name: "Research Analyst", icon: "🔍" }] as never);
  vi.mocked(api.getRecordOwner).mockResolvedValue({ owning_org_unit_id: "ou1" } as never);
  vi.mocked(api.listOrgUnits).mockResolvedValue([{ id: "ou1", name: "Northstar" }] as never);
  vi.mocked(api.listAuditEvents).mockResolvedValue([
    { id: "e1", summary: "Record created", occurred_at: "2026-01-02T10:00:00Z" },
  ] as never);
}

function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () => new QueryClient({ defaultOptions: { queries: { retry: false, staleTime: Infinity } } }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function inspectorTitle(container: HTMLElement): string | null {
  return container.querySelector(".ontology-inspector-head strong")?.textContent ?? null;
}

describe("Object360Modal", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockApi();
  });

  function open(overrides: Partial<Parameters<typeof Object360Modal>[0]> = {}) {
    const onClose = vi.fn();
    const utils = render(
      <Providers>
        <Object360Modal entityType="Company" entityId="c1" label="BrightPath Logistics" onClose={onClose} {...overrides} />
      </Providers>,
    );
    return { onClose, ...utils };
  }

  it("opens with the record title, breadcrumb, and graph", async () => {
    const { container } = open();
    expect(await screen.findByText("Object 360 · BrightPath Logistics")).toBeInTheDocument();
    // Breadcrumb: Company › BrightPath Logistics › Relationship neighborhood
    expect(screen.getByText(/Relationship neighborhood/)).toBeInTheDocument();
    // Root pill + related pills render on the shared engine.
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Company:c1"]')).not.toBeNull();
    });
    expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    expect(container.querySelector('[data-node-key="record:Opportunity:o9"]')).not.toBeNull();
  });

  it("renders legend layer chips for every neighbor kind", async () => {
    open();
    // Wait for the related records to load before reading the legend.
    expect((await screen.findAllByText("Ava Martin")).length).toBeGreaterThan(0);
    const legend = await screen.findByRole("group", { name: "Graph layers" });
    const text = legend.textContent ?? "";
    for (const chip of ["Company", "Contact", "Opportunity", "Business Rules", "Workflows", "AI Agents", "Business Unit"]) {
      expect(text).toContain(chip);
    }
  });

  it("clicking a related pill refreshes the inspector", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    // Inspector starts on the root record.
    expect(inspectorTitle(container)).toBe("BrightPath Logistics");

    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);

    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    // Subtitle reflects the inspected record's type.
    const head = container.querySelector(".ontology-inspector-head")!;
    expect(within(head as HTMLElement).getByText("Contact")).toBeInTheDocument();
    // The inspected pill gets the ring.
    expect(container.querySelector('[data-node-key="record:Contact:c2"]')).toHaveClass("og-pill-inspecting");
  });

  it("shows related-record groups, rules, workflows, agents, and activity for the root", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    // Wait for the related records to load into the inspector.
    expect((await screen.findAllByText("Ava Martin")).length).toBeGreaterThan(0);
    const groups = container.querySelector(".ontology-inspector-groups")!;
    const text = groups.textContent ?? "";
    expect(text).toContain("Ava Martin");
    expect(text).toContain("Business Unit");
    expect(text).toContain("Northstar");
    expect(text).toContain("Credit limit check");
    expect(text).toContain("Renewal workflow");
    expect(text).toContain("Research Analyst");
    expect(text).toContain("Record created");
    // Inactive workflows are excluded.
    expect(text).not.toContain("Retired workflow");
  });

  it("'Show what's around it' re-centers the graph on the inspected record", async () => {
    const { container } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);
    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    fireEvent.click(screen.getByRole("button", { name: "Show what's around it" }));
    expect(await screen.findByText("Object 360 · Ava Martin")).toBeInTheDocument();
  });

  it("fires onOpenRecord for the inspected record", async () => {
    const onOpenRecord = vi.fn();
    const { container } = open({ onOpenRecord });
    await screen.findByText("Object 360 · BrightPath Logistics");
    await waitFor(() => {
      expect(container.querySelector('[data-node-key="record:Contact:c2"]')).not.toBeNull();
    });
    fireEvent.click(container.querySelector('[data-node-key="record:Contact:c2"]')!);
    await waitFor(() => {
      expect(inspectorTitle(container)).toBe("Ava Martin");
    });
    fireEvent.click(screen.getByRole("button", { name: "Open record" }));
    expect(onOpenRecord).toHaveBeenCalledWith("Contact", "c2");
  });

  it("closes via the Close button and overlay click", async () => {
    const { container, onClose } = open();
    await screen.findByText("Object 360 · BrightPath Logistics");
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(container.querySelector(".modal-overlay")!);
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
""")

# File: desktop/src/__tests__/ui.test.tsx
write_file("desktop/src/__tests__/ui.test.tsx", """/**
 * PR 3 — UI primitives smoke tests.
 *
 * Covers the PR 1 component library basics that the graph surfaces
 * depend on: Button variants/click, Card slots, and Dialog open/
 * Escape/overlay-close behavior.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { Button } from "../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "../components/ui/dialog";

describe("Button", () => {
  it("renders children and fires onClick", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Click me</Button>);
    const btn = screen.getByRole("button", { name: "Click me" });
    expect(btn).toHaveClass("ui-btn-default");
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("applies variant and size classes", () => {
    render(
      <Button variant="destructive" size="sm">
        Delete
      </Button>,
    );
    const btn = screen.getByRole("button", { name: "Delete" });
    expect(btn).toHaveClass("ui-btn-destructive");
    expect(btn).toHaveClass("ui-btn-size-sm");
  });

  it("forwards the disabled state", () => {
    render(<Button disabled>Off</Button>);
    expect(screen.getByRole("button", { name: "Off" })).toBeDisabled();
  });
});

describe("Card", () => {
  it("renders title, description, and content slots", () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Card title</CardTitle>
          <CardDescription>Card description</CardDescription>
        </CardHeader>
        <CardContent>Card body</CardContent>
      </Card>,
    );
    expect(screen.getByText("Card title")).toBeInTheDocument();
    expect(screen.getByText("Card description")).toBeInTheDocument();
    expect(screen.getByText("Card body")).toBeInTheDocument();
  });
});

describe("Dialog", () => {
  function Harness({ onOpenChange }: { onOpenChange: (open: boolean) => void }) {
    const [open, setOpen] = useState(true);
    return (
      <Dialog
        open={open}
        onOpenChange={(o) => {
          setOpen(o);
          onOpenChange(o);
        }}
      >
        <DialogContent>
          <DialogTitle>Dialog title</DialogTitle>
          <DialogDescription>Dialog description</DialogDescription>
          <button type="button">Inner button</button>
        </DialogContent>
      </Dialog>
    );
  }

  it("renders content when open and closes on Escape", () => {
    const onOpenChange = vi.fn();
    render(<Harness onOpenChange={onOpenChange} />);
    expect(screen.getByText("Dialog title")).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("closes on overlay click", () => {
    const onOpenChange = vi.fn();
    render(<Harness onOpenChange={onOpenChange} />);
    // Dialog portals to document.body, so query the document.
    const overlay = document.querySelector(".ui-dialog-overlay");
    expect(overlay).not.toBeNull();
    fireEvent.click(overlay!);
    expect(onOpenChange).toHaveBeenCalledWith(false);
    document.body.innerHTML = "";
  });

  it("renders nothing when closed", () => {
    render(
      <Dialog open={false} onOpenChange={() => {}}>
        <DialogContent>
          <DialogTitle>Hidden</DialogTitle>
        </DialogContent>
      </Dialog>,
    );
    expect(screen.queryByText("Hidden")).toBeNull();
  });
});
""")

# File: desktop/src/__tests__/fixtures.ts
write_file("desktop/src/__tests__/fixtures.ts", """/**
 * PR 3 — Shared graph fixtures for the regression suite.
 */
import type { GraphModel } from "../components/graph/graphTypes";

export const fixtureModel: GraphModel = {
  nodes: [
    { key: "custom_object:companies", id: "companies", type: "custom_object", label: "Companies", isRoot: true },
    { key: "custom_field:name", id: "name", type: "custom_field", label: "Name" },
    { key: "custom_field:revenue", id: "revenue", type: "custom_field", label: "Annual Revenue" },
    { key: "workflow:wf1", id: "wf1", type: "workflow", label: "Renewal workflow" },
    { key: "ai_agent:a1", id: "a1", type: "ai_agent", label: "Research Analyst" },
    // Deliberately unconnected: used by hover tests to assert dimming.
    { key: "metric_definition:m1", id: "m1", type: "metric_definition", label: "Pipeline coverage" },
  ],
  edges: [
    { fromKey: "custom_object:companies", toKey: "custom_field:name", edgeType: "has_field", label: "Has field" },
    { fromKey: "custom_object:companies", toKey: "custom_field:revenue", edgeType: "has_field", label: "Has field" },
    { fromKey: "custom_object:companies", toKey: "workflow:wf1", edgeType: "triggers", label: "Triggers" },
    { fromKey: "ai_agent:a1", toKey: "custom_object:companies", edgeType: "can_read", label: "Can read", access: true },
  ],
};

export const ROOT_KEY = "custom_object:companies";
""")

print("Done")
