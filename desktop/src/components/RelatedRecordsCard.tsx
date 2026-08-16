import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import type { RelatedRecord, RelationshipDefinition } from "../lib/types";

type PickableRecord = { id: string; label: string };

/** Maps any entity_type - built-in or custom object key - to a flat
 * id/label list suitable for a link picker `<select>`. Built-in types each
 * have their own dedicated list endpoint; anything else is assumed to be
 * an active custom object key, resolved through the one generic
 * custom-records endpoint. */
async function listRecordsOfType(entityType: string): Promise<PickableRecord[]> {
  switch (entityType) {
    case "Company":
      return (await api.listCompanies()).map((r) => ({ id: r.id, label: r.name }));
    case "Contact":
      return (await api.listContacts()).map((r) => ({ id: r.id, label: `${r.first_name} ${r.last_name}`.trim() }));
    case "Opportunity":
      return (await api.listOpportunities()).map((r) => ({ id: r.id, label: r.name }));
    case "Quote":
      return (await api.listQuotes()).map((r) => ({ id: r.id, label: r.quote_number }));
    case "Order":
      return (await api.listOrders()).map((r) => ({ id: r.id, label: r.order_number }));
    case "Invoice":
      return (await api.listInvoices()).map((r) => ({ id: r.id, label: r.invoice_number }));
    case "Contract":
      return (await api.listContracts()).map((r) => ({ id: r.id, label: r.title }));
    case "Task":
      return (await api.listTasks()).map((r) => ({ id: r.id, label: r.title }));
    case "Product":
      return (await api.listProducts()).map((r) => ({ id: r.id, label: r.name }));
    default:
      return (await api.listCustomRecords(entityType)).map((r) => ({ id: r.id, label: r.primary_name }));
  }
}

/**
 * Admin extensibility Phase B (spec §21): renders every related record for
 * one record, across every active relationship it participates in from
 * either direction - custom relationships automatically show up here with
 * no per-screen wiring, the same "compose for free" property custom
 * fields/business rules already have on a custom object.
 *
 * `only` (Screen/App Builder Phase 3): restricts which relationships
 * render, by `RelationshipDefinition.key` - how a Screen layout places
 * different related lists on different tabs (`LayoutFormFields` renders
 * one `RelatedRecordsCard` per tab that claims any keys, each with its
 * own `only`). Omitted (every caller outside the layout system) shows
 * everything applicable, the pre-Phase-3 behavior.
 */
export function RelatedRecordsCard({ entityType, entityId, only }: { entityType: string; entityId: string; only?: string[] }) {
  const queryClient = useQueryClient();
  const [linkingKey, setLinkingKey] = useState<string | null>(null);

  const related = useQuery({
    queryKey: ["relatedRecords", entityType, entityId],
    queryFn: () => api.listRelatedRecords(entityType, entityId),
  });
  const defs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });

  const applicableDefs = (defs.data ?? []).filter(
    (d) => d.show_related_list && (d.source_entity_type === entityType || d.target_entity_type === entityType) && (!only || only.includes(d.key)),
  );

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["relatedRecords", entityType, entityId] });
  }

  const unlink = useMutation({
    mutationFn: (instanceId: string) => api.unlinkRecords(instanceId),
    onSuccess: invalidate,
  });

  if (applicableDefs.length === 0) return null;

  const grouped = new Map<string, { label: string; rows: RelatedRecord[] }>();
  for (const d of applicableDefs) {
    grouped.set(d.key, { label: d.source_entity_type === entityType ? d.forward_label : d.reverse_label, rows: [] });
  }
  for (const r of related.data ?? []) {
    const group = grouped.get(r.relationship_key);
    if (group) group.rows.push(r);
  }

  const linkingDef = linkingKey ? applicableDefs.find((d) => d.key === linkingKey) ?? null : null;

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Related records</h3>
      {[...grouped.entries()].map(([key, group]) => (
        <div key={key} style={{ marginBottom: 14 }}>
          <div className="toolbar" style={{ marginBottom: 4 }}>
            <strong>{group.label}</strong>
            <button className="btn" onClick={() => setLinkingKey(key)}>
              + Link
            </button>
          </div>
          {group.rows.length === 0 && <p className="empty-state">None linked.</p>}
          {group.rows.length > 0 && (
            <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
              {group.rows.map((r) => (
                <li
                  key={r.instance_id}
                  style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "4px 0", borderBottom: "1px solid var(--border, #eee)" }}
                >
                  <span>
                    {r.display_name}
                    {r.archived ? " (archived)" : ""} — <span className="badge">{r.status}</span>
                  </span>
                  <button className="btn" onClick={() => unlink.mutate(r.instance_id)} disabled={unlink.isPending}>
                    Unlink
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}

      {linkingDef && (
        <LinkPicker
          definition={linkingDef}
          entityType={entityType}
          entityId={entityId}
          onDone={() => {
            invalidate();
            setLinkingKey(null);
          }}
          onCancel={() => setLinkingKey(null)}
        />
      )}
    </div>
  );
}

function LinkPicker({
  definition,
  entityType,
  entityId,
  onDone,
  onCancel,
}: {
  definition: RelationshipDefinition;
  entityType: string;
  entityId: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const isSource = definition.source_entity_type === entityType;
  const otherType = isSource ? definition.target_entity_type : definition.source_entity_type;
  const options = useQuery({ queryKey: ["pickableRecords", otherType], queryFn: () => listRecordsOfType(otherType) });
  const [selected, setSelected] = useState("");
  const [error, setError] = useState<string | null>(null);

  const link = useMutation({
    mutationFn: () =>
      isSource
        ? api.linkRecords(definition.id, entityType, entityId, otherType, selected)
        : api.linkRecords(definition.id, otherType, selected, entityType, entityId),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not link these records"),
  });

  return (
    <div className="card" style={{ background: "var(--surface-2, transparent)" }}>
      {error && <div className="error-banner">{error}</div>}
      <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <select value={selected} onChange={(e) => setSelected(e.target.value)}>
          <option value="">{options.isLoading ? "Loading..." : "Select a record..."}</option>
          {(options.data ?? []).map((o) => (
            <option key={o.id} value={o.id}>
              {o.label}
            </option>
          ))}
        </select>
        <button className="btn btn-primary" disabled={!selected || link.isPending} onClick={() => link.mutate()}>
          Link
        </button>
        <button className="btn" type="button" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
