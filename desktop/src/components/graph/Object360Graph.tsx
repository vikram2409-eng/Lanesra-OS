/**
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
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (c) => c.toUpperCase());
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
