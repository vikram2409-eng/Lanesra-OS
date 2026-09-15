import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import { CUSTOM_FIELD_ENTITY_TYPES, entityTypeLabel, type RelatedRecord, type RelationshipDefinition } from "../lib/types";

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

  // A polymorphic-target definition's stored target_entity_type is an
  // unused '' placeholder (see RelationshipDefinition.target_is_polymorphic)
  // so it can never equal a real entityType - it's included unconditionally
  // instead, mirroring `list_definitions_for_entity`'s own "can't say which
  // types are eligible" reasoning server-side. `related.data` (the actual
  // linked records) still determines what renders inside each group.
  const applicableDefs = (defs.data ?? []).filter(
    (d) => d.show_related_list && (d.source_entity_type === entityType || d.target_entity_type === entityType || d.target_is_polymorphic) && (!only || only.includes(d.key)),
  );

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["relatedRecords", entityType, entityId] });
  }

  const unlink = useMutation({
    mutationFn: (instanceId: string) => api.unlinkRecords(instanceId),
    onSuccess: invalidate,
  });

  if (applicableDefs.length === 0) return null;

  // A self-referential definition (source and target object are the same
  // type) needs two distinct groups - "as source" and "as target" - not
  // one, since `source_entity_type === entityType` is trivially always
  // true here and would otherwise collapse both directions' rows into one
  // group permanently mislabeled with `forward_label`. Every other
  // definition still gets exactly one group, keyed by `d.key` alone,
  // unchanged from before.
  type Group = { label: string; rows: RelatedRecord[]; def: RelationshipDefinition; direction: "forward" | "reverse" };
  const grouped = new Map<string, Group>();
  for (const d of applicableDefs) {
    if (d.source_entity_type === d.target_entity_type) {
      grouped.set(`${d.key}:forward`, { label: d.forward_label, rows: [], def: d, direction: "forward" });
      grouped.set(`${d.key}:reverse`, { label: d.reverse_label, rows: [], def: d, direction: "reverse" });
    } else {
      const direction = d.source_entity_type === entityType ? "forward" : "reverse";
      grouped.set(d.key, { label: direction === "forward" ? d.forward_label : d.reverse_label, rows: [], def: d, direction });
    }
  }
  for (const r of related.data ?? []) {
    const def = applicableDefs.find((d) => d.key === r.relationship_key);
    if (!def) continue;
    // `related_records_for` already stamps each row's `label` with
    // `forward_label`/`reverse_label` per which side it came from - reuse
    // that instead of re-deriving direction, since for a self-referential
    // definition entity-type comparison alone can't tell the two apart.
    const groupKey = def.source_entity_type === def.target_entity_type ? `${r.relationship_key}:${r.label === def.forward_label ? "forward" : "reverse"}` : r.relationship_key;
    const group = grouped.get(groupKey);
    if (group) group.rows.push(r);
  }

  const linkingGroup = linkingKey ? grouped.get(linkingKey) ?? null : null;

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

      {linkingGroup && (
        <LinkPicker
          definition={linkingGroup.def}
          direction={linkingGroup.direction}
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
  direction,
  entityType,
  entityId,
  onDone,
  onCancel,
}: {
  definition: RelationshipDefinition;
  /** Which side the current record plays. For every non-self-referential
   * definition this is implied by entity type alone (`isSource` below);
   * for a self-referential one (source and target object are the same
   * type) it can't be, since the current record's type always matches
   * both sides - the caller passes the direction the clicked group
   * actually represents instead. */
  direction?: "forward" | "reverse";
  entityType: string;
  entityId: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const selfReferential = definition.source_entity_type === definition.target_entity_type;
  const isSource = selfReferential ? direction !== "reverse" : definition.source_entity_type === entityType;
  // Linking *from* the variable side (entityType is some polymorphic
  // target, isSource false) needs no special handling - `otherType`
  // resolves to the fixed `source_entity_type` same as any other
  // relationship. Only linking *from* the fixed source side needs a type
  // choice first, since `target_entity_type` on the definition is the
  // unused '' placeholder rather than one real type.
  const polymorphicChoice = isSource && definition.target_is_polymorphic;
  const [chosenType, setChosenType] = useState("");
  const otherType = polymorphicChoice ? chosenType : isSource ? definition.target_entity_type : definition.source_entity_type;

  const customObjects = useQuery({
    queryKey: ["customObjects", "active"],
    queryFn: () => api.listCustomObjects(true),
    enabled: polymorphicChoice,
  });
  const targetTypeChoices: string[] = polymorphicChoice ? [...CUSTOM_FIELD_ENTITY_TYPES, ...(customObjects.data?.map((o) => o.key) ?? [])] : [];
  const targetTypeLabel = (t: string) => customObjects.data?.find((o) => o.key === t)?.plural_label ?? entityTypeLabel(t);

  const options = useQuery({
    queryKey: ["pickableRecords", otherType],
    queryFn: () => listRecordsOfType(otherType),
    enabled: !!otherType,
    // A self-referential relationship's picker would otherwise list this
    // very record among its own candidate links - `link()` rejects
    // source_id === target_id server-side, but excluding it here is both
    // a clearer UI and one fewer round trip to find that out.
    select: (records) => (otherType === entityType ? records.filter((r) => r.id !== entityId) : records),
  });
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
        {polymorphicChoice && (
          <select
            value={chosenType}
            onChange={(e) => {
              setChosenType(e.target.value);
              setSelected("");
            }}
          >
            <option value="">Select a record type...</option>
            {targetTypeChoices.map((t) => (
              <option key={t} value={t}>
                {targetTypeLabel(t)}
              </option>
            ))}
          </select>
        )}
        <select value={selected} onChange={(e) => setSelected(e.target.value)} disabled={polymorphicChoice && !chosenType}>
          <option value="">{options.isLoading && otherType ? "Loading..." : "Select a record..."}</option>
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
