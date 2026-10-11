import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import type { RelatedRecord } from "../lib/types";
import { VisualBuilderCanvas, type CanvasEdgeView } from "./visualBuilder/VisualBuilderCanvas";

const ROOT_SIZE = 26;
const NODE_SIZE = 15;
const RING_RADIUS = 170;
const MAX_DOTS_PER_GROUP = 20;

type RootRef = { entityType: string; entityId: string; label: string };
type InspectTarget = RootRef | { kind: "orgUnit"; id: string; label: string };

/**
 * Object 360 (issue #352): a record-level counterpart to the Ontology
 * Layer's schema-level Dependency Explorer (#339) - same radial dot-graph,
 * same type-color legend (doubling as a toggle-filter, same as the
 * Dependency Explorer's) built from whichever object types actually
 * appear among this record's neighbors - not a fixed list, since any
 * given record only ever touches a handful of types - same floating
 * inspector card and `.ontology-*` CSS classes, but centered on one real
 * record instead of an object type, with one dot per actual related
 * record (not per type).
 * Given one record, shows every record it's directly linked to across
 * every relationship it participates in (`relationship_service::
 * related_records_for`, already aggregates all of them in one call), plus
 * its Business Unit, the active Business Rules/Workflows that apply to
 * records of its type, the AI Agents scoped to that type, and its own
 * real audit history - open to any user who can already view this record
 * (same access model `list_audit_events`/`list_activities` already use).
 *
 * `onOpenRecord` is optional - a few record pages (Opportunities today)
 * don't yet have a working cross-object navigation callback wired from
 * App.tsx, so on those this screen still fully works for browsing/
 * re-centering, just without the "jump straight to that page" shortcut.
 */
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
  const [hiddenTypes, setHiddenTypes] = useState<Set<string>>(() => new Set());

  function toggleLegendType(key: string) {
    setHiddenTypes((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
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

  const groups = new Map<string, RelatedRecord[]>();
  (related.data ?? []).forEach((r) => {
    const list = groups.get(r.relationship_key) ?? [];
    list.push(r);
    groups.set(r.relationship_key, list);
  });
  const groupEntries = Array.from(groups.entries());

  type LaidOutNode = { key: string; x: number; y: number; size: number; colorIndex: number; target: RootRef | { kind: "orgUnit"; id: string; label: string } };
  const totalArcs = groupEntries.length + (orgUnit ? 1 : 0);
  const cx = RING_RADIUS + 60;
  const cy = RING_RADIUS + 60;
  // The inspector card is pinned top-right and can grow tall (up to 11
  // groups); dots are confined to the left semicircle (see below), but
  // the card is 280px wide - still wide enough to reach well past the
  // circle's center on a canvas sized to exactly fit the circle. A
  // dedicated reserved lane on the right, clear of every dot regardless
  // of the card's height, is simpler and more robust than fine-tuning
  // angles against a specific card width.
  const INSPECTOR_LANE_WIDTH = 340;
  const canvasWidth = cx + RING_RADIUS + 60 + INSPECTOR_LANE_WIDTH;
  const canvasHeight = cy * 2;
  const nodes: LaidOutNode[] = [];
  const edges: CanvasEdgeView[] = [];
  const colorIndexFor = (et: string) => objectTypes.data?.find((o) => o.key === et)?.color_index ?? 1;
  if (totalArcs > 0) {
    // The fixed inspector card (top:16, right:16) can grow tall - up to
    // 11 groups (related records, Business Unit, Rules, Workflows,
    // Agents, Activity) - so a quadrant-sized gap isn't enough; dots are
    // confined to the left semicircle (90deg-270deg, i.e. x <= center)
    // so none can ever land under it, whatever its height ends up being.
    const START_DEG = 90;
    const SPAN_DEG = 180;
    const arcSize = ((SPAN_DEG * Math.PI) / 180) / totalArcs;
    const startAngle = (START_DEG * Math.PI) / 180;
    let arcIndex = 0;
    groupEntries.forEach(([key, members]) => {
      const start = startAngle + arcIndex * arcSize;
      const shown = members.slice(0, MAX_DOTS_PER_GROUP);
      shown.forEach((m, i) => {
        const angle = start + (shown.length === 1 ? arcSize / 2 : (arcSize * i) / Math.max(1, shown.length - 1));
        const x = cx + RING_RADIUS * Math.cos(angle);
        const y = cy + RING_RADIUS * Math.sin(angle);
        nodes.push({ key: `${key}-${i}`, x, y, size: NODE_SIZE, colorIndex: colorIndexFor(m.entity_type), target: { entityType: m.entity_type, entityId: m.entity_id, label: m.display_name } });
        edges.push({ key: `edge-${key}-${i}`, fromX: x, fromY: y, toX: cx, toY: cy, label: null });
      });
      arcIndex++;
    });
    if (orgUnit) {
      const angle = startAngle + arcIndex * arcSize + arcSize / 2;
      const x = cx + RING_RADIUS * Math.cos(angle);
      const y = cy + RING_RADIUS * Math.sin(angle);
      nodes.push({ key: "org-unit", x, y, size: NODE_SIZE, colorIndex: 6, target: { kind: "orgUnit", id: orgUnit.id, label: orgUnit.name } });
      edges.push({ key: "edge-org-unit", fromX: x, fromY: y, toX: cx, toY: cy, label: null });
    }
  }
  const rootNode: LaidOutNode = { key: "root", x: cx, y: cy, size: ROOT_SIZE, colorIndex: colorIndexFor(root.entityType), target: root };
  const canvasNodes = [rootNode, ...nodes];

  // Legend keys (and the toggle-filter it doubles as) are built from
  // whichever types actually appear among this record's neighbors, not a
  // fixed list - unlike the Dependency Explorer's schema-wide legend, any
  // one record only ever touches a handful of the platform's object
  // types. "orgUnit" is a synthetic key for the Business Unit dot, which
  // has no entry in `objectTypes`.
  function legendKeyFor(n: LaidOutNode): string {
    return "kind" in n.target ? "orgUnit" : n.target.entityType;
  }
  function legendLabelFor(key: string): string {
    return key === "orgUnit" ? "Business Unit" : objectTypes.data?.find((o) => o.key === key)?.label_singular ?? key;
  }
  function legendColorFor(key: string): number {
    return key === "orgUnit" ? 6 : colorIndexFor(key);
  }
  const legendKeys = Array.from(new Set(nodes.map(legendKeyFor)));

  function isSameTarget(a: InspectTarget, b: InspectTarget) {
    if ("kind" in a || "kind" in b) return "kind" in a && "kind" in b && a.id === b.id;
    return a.entityType === b.entityType && a.entityId === b.entityId;
  }
  const inspectingIsRoot = !("kind" in inspecting) && inspecting.entityType === root.entityType && inspecting.entityId === root.entityId;

  function recenterOn(target: RootRef) {
    setRoot(target);
    setInspecting(target);
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" style={{ width: "min(1100px, 95vw)", maxHeight: "92vh" }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
          <h3 style={{ margin: 0 }}>Object 360: {root.label}</h3>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
        </div>
        <p style={{ color: "var(--text-muted)", fontSize: 13, maxWidth: 720 }}>
          Every record directly linked to {root.label}, its Business Unit, the Business Rules/Workflows that apply to records like
          it, and which AI Agents are scoped to act on it.
        </p>

        {legendKeys.length > 0 && (
          <div className="ontology-legend">
            {legendKeys.map((key) => (
              <button
                key={key}
                type="button"
                className={`ontology-legend-item${hiddenTypes.has(key) ? " ontology-legend-item-off" : ""}`}
                onClick={() => toggleLegendType(key)}
                title={hiddenTypes.has(key) ? `Show ${legendLabelFor(key)}` : `Hide ${legendLabelFor(key)}`}
              >
                <span style={{ display: "inline-block", width: 10, height: 10, borderRadius: "50%", background: `var(--chart-${legendColorFor(key)})`, flexShrink: 0 }} />
                {legendLabelFor(key)}
              </button>
            ))}
          </div>
        )}

        <div className="graph-canvas-wrap ontology-canvas-wrap" style={{ position: "relative", height: canvasHeight + 40 }}>
          <VisualBuilderCanvas<LaidOutNode>
            nodes={canvasNodes}
            edges={edges}
            zoom={1}
            canvasWidth={canvasWidth}
            canvasHeight={canvasHeight}
            editable={false}
            renderNode={(n) => {
              const isRoot = n.key === "root";
              const isInspecting = !isRoot && isSameTarget(inspecting, n.target);
              const isDimmed = !isRoot && hiddenTypes.has(legendKeyFor(n));
              return (
                <div
                  key={n.key}
                  className={`ontology-node-dot${isRoot ? " ontology-node-root" : ""}${isInspecting ? " ontology-node-inspecting" : ""}${isDimmed ? " ontology-node-dimmed" : ""}`}
                  style={{ left: n.x - n.size / 2, top: n.y - n.size / 2, width: n.size, height: n.size, background: `var(--chart-${n.colorIndex})` }}
                  onClick={() => !isRoot && setInspecting(n.target)}
                  title={n.target.label}
                />
              );
            }}
          />

            <div className="ontology-inspector-card">
              {"kind" in inspecting ? (
                <>
                  <div className="ontology-inspector-head">
                    <span style={{ display: "inline-block", width: 14, height: 14, borderRadius: "50%", background: "var(--chart-6)", flexShrink: 0 }} />
                    <div>
                      <strong>{inspecting.label}</strong>
                      <div className="ontology-inspector-subtitle">Business Unit</div>
                    </div>
                  </div>
                  <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Business Units aren't individually re-centerable here - see Admin → Organization Units for its own detail.</p>
                </>
              ) : (
                <>
                  <div className="ontology-inspector-head">
                    <span style={{ display: "inline-block", width: 14, height: 14, borderRadius: "50%", background: `var(--chart-${colorIndexFor(inspecting.entityType)})`, flexShrink: 0 }} />
                    <div>
                      <strong>{inspecting.label}</strong>
                      <div className="ontology-inspector-subtitle">{objectTypes.data?.find((o) => o.key === inspecting.entityType)?.label_singular ?? inspecting.entityType}</div>
                    </div>
                  </div>

                  <div className="ontology-inspector-actions">
                    {onOpenRecord && (
                      <button className="btn btn-primary" onClick={() => onOpenRecord(inspecting.entityType, inspecting.entityId)}>
                        Open record
                      </button>
                    )}
                    <button className="btn btn-secondary" onClick={() => recenterOn(inspecting as RootRef)} disabled={inspectingIsRoot}>
                      Show what's around it
                    </button>
                  </div>

                  {inspectingIsRoot && (
                    <div className="ontology-inspector-groups">
                      {groupEntries.map(([key, members]) => (
                        <div key={key} className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">{members[0]?.label ?? key}</div>
                          {members.slice(0, MAX_DOTS_PER_GROUP).map((m) => (
                            <button
                              key={m.instance_id}
                              type="button"
                              className="ontology-inspector-bullet"
                              onClick={() => setInspecting({ entityType: m.entity_type, entityId: m.entity_id, label: m.display_name })}
                            >
                              <span style={{ display: "inline-block", width: 10, height: 10, borderRadius: "50%", background: `var(--chart-${colorIndexFor(m.entity_type)})`, flexShrink: 0 }} />
                              {m.display_name}
                            </button>
                          ))}
                          {members.length > MAX_DOTS_PER_GROUP && (
                            <div style={{ fontSize: 12, color: "var(--text-muted)" }}>+{members.length - MAX_DOTS_PER_GROUP} more</div>
                          )}
                        </div>
                      ))}
                      {groupEntries.length === 0 && <p style={{ fontSize: 12, color: "var(--text-muted)" }}>No related records yet.</p>}

                      {orgUnit && (
                        <div className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">Business Unit</div>
                          <div style={{ fontSize: 13 }}>{orgUnit.name}</div>
                        </div>
                      )}

                      {(rules.data ?? []).length > 0 && (
                        <div className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">Business Rules that apply</div>
                          {(rules.data ?? []).map((r) => (
                            <div key={r.id} style={{ fontSize: 13 }}>
                              {r.name}
                            </div>
                          ))}
                        </div>
                      )}

                      {activeWorkflows.length > 0 && (
                        <div className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">Workflows that apply</div>
                          {activeWorkflows.map((w) => (
                            <div key={w.id} style={{ fontSize: 13 }}>
                              {w.name}
                            </div>
                          ))}
                        </div>
                      )}

                      {(scopedAgents.data ?? []).length > 0 && (
                        <div className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">AI Agents scoped to this object type</div>
                          {(scopedAgents.data ?? []).map((a) => (
                            <div key={a.id} style={{ fontSize: 13 }}>
                              {a.icon} {a.name}
                            </div>
                          ))}
                        </div>
                      )}

                      {(auditEvents.data ?? []).length > 0 && (
                        <div className="ontology-inspector-group">
                          <div className="ontology-inspector-group-label">Activity</div>
                          {(auditEvents.data ?? []).slice(0, 10).map((e) => (
                            <div key={e.id} style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 4 }}>
                              {e.summary}
                              <br />
                              {new Date(e.occurred_at).toLocaleString()}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  )}
                </>
              )}
            </div>
        </div>
      </div>
    </div>
  );
}

