/**
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

