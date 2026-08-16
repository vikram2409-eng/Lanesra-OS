import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { useCustomFieldElements } from "../../components/CustomFieldsSection";
import { LayoutFormFields } from "../../components/LayoutFormFields";
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import { AuditByline, AuditTrail } from "../../components/AuditTrail";
import type { Prefill } from "../../components/AppShell";
import {
  CUSTOM_RECORD_STATUSES,
  type CustomFieldValues,
  type CustomObjectDefinition,
  type CustomRecordInput,
} from "../../lib/types";
import { useCustomFieldFilters } from "../../lib/useCustomFieldFilters";
import { useCanWriteObject } from "../../lib/useCanWriteObject";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

/**
 * Generic list/create/edit screen for records of one admin-defined custom
 * object (spec §20.2). Mounted once per active custom object from App.tsx,
 * the same way Products/Contracts/Tasks etc. are - the only difference is
 * `objectKey`/`definition` come from data instead of being hardcoded, so
 * one component serves every custom object rather than one per type.
 *
 * `prefill.openId` (global search "jump to a record") reuses the same
 * one-shot mechanism every other list screen already has - there's no
 * separate read-only "detail" mode here, so it opens straight into Edit,
 * which already shows every field plus its related-records lists (see
 * `LayoutFormFields`'s `entityId`/`relatedKeys` props - Screen/App
 * Builder Phase 3).
 */
export function CustomObjectRecords({
  definition,
  prefill,
  onPrefillConsumed,
}: {
  definition: CustomObjectDefinition;
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
}) {
  const [view, setView] = useState<View>(() => (prefill?.openId ? { mode: "edit", id: prefill.openId } : { mode: "list" }));
  useEffect(() => {
    if (prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  const queryClient = useQueryClient();
  const records = useQuery({
    queryKey: ["customRecords", definition.key],
    queryFn: () => api.listCustomRecords(definition.key),
  });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const fieldFilters = useCustomFieldFilters(definition.key);
  const canWrite = useCanWriteObject(definition.key);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["customRecords", definition.key] });
  }

  const ownerName = (id: string | null): string =>
    id ? users.data?.find((u) => u.id === id)?.display_name ?? "—" : "—";

  if (view.mode !== "list") {
    return (
      <RecordForm
        definition={definition}
        recordId={view.mode === "edit" ? view.id : undefined}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>
          {definition.icon} {definition.plural_label}
        </h2>
        <button
          className="btn btn-primary"
          onClick={() => setView({ mode: "create" })}
          disabled={!canWrite}
          title={canWrite ? undefined : `You have view-only access to ${definition.plural_label} through an app`}
        >
          + New {definition.singular_label.toLowerCase()}
        </button>
      </div>
      <CustomFieldFilterBar filters={fieldFilters} />
      {records.isLoading && <p>Loading...</p>}
      {records.data && records.data.length === 0 && (
        <p className="empty-state">No {definition.plural_label.toLowerCase()} yet.</p>
      )}
      {records.data && records.data.length > 0 && (() => {
        const rows = records.data.filter((r) => fieldFilters.matches(r.id));
        return rows.length === 0 ? (
          <p className="empty-state">No {definition.plural_label.toLowerCase()} match the current filters.</p>
        ) : (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Name</th>
              <th>Status</th>
              <th>Owner</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>{r.display_number}</td>
                <td>{r.primary_name}</td>
                <td>
                  <span className={`badge${r.status === "Active" ? " badge-success" : ""}`}>{r.status}</span>
                </td>
                <td>{ownerName(r.owner_user_id)}</td>
                <td>
                  <button
                    className="btn"
                    onClick={() => setView({ mode: "edit", id: r.id })}
                    disabled={!canWrite}
                    title={canWrite ? undefined : `You have view-only access to ${definition.plural_label} through an app`}
                  >
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        );
      })()}
    </div>
  );
}

function RecordForm({
  definition,
  recordId,
  onDone,
  onCancel,
}: {
  definition: CustomObjectDefinition;
  recordId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const existing = useQuery({
    queryKey: ["customRecord", recordId],
    queryFn: () => api.getCustomRecord(recordId as string),
    enabled: !!recordId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", recordId],
    queryFn: () => api.getCustomFieldValues(recordId as string),
    enabled: !!recordId,
  });

  const [input, setInput] = useState<CustomRecordInput>({
    object_key: definition.key,
    primary_name: "",
    status: "Active",
    owner_user_id: null,
    notes: null,
  });
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== recordId) {
    const { primary_name, status, owner_user_id, notes } = existing.data;
    setInput({ object_key: definition.key, primary_name, status, owner_user_id, notes });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(recordId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const record = recordId
        ? await api.updateCustomRecord(recordId, {
            primary_name: input.primary_name,
            status: input.status,
            owner_user_id: input.owner_user_id,
            notes: input.notes,
          })
        : await api.createCustomRecord(input);
      const ruleMessages = await api.setCustomFieldValues(definition.key, record.id, customValues);
      showRuleMessages(ruleMessages);
      return record;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : `Could not save this ${definition.singular_label.toLowerCase()}`),
  });

  const { order: customFieldOrder, elements: customFieldElements } = useCustomFieldElements({
    entityType: definition.key,
    status: input.status,
    values: customValues,
    onChange: setCustomValues,
  });

  // Screen/App Builder Phase 3: every relationship this object can show a
  // related-list for - same set RelatedRecordsCard itself would show
  // unfiltered, so LayoutFormFields knows what's still "unplaced" if a
  // layout only claims some of them for a tab.
  const relationshipDefs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });
  const relatedKeys = (relationshipDefs.data ?? [])
    .filter((d) => d.show_related_list && (d.source_entity_type === definition.key || d.target_entity_type === definition.key))
    .map((d) => d.key);

  return (
    <div>
      <h2>
        {recordId ? `Edit ${definition.singular_label.toLowerCase()}` : `New ${definition.singular_label.toLowerCase()}`}
      </h2>
      {existing.data && (
        <AuditByline
          createdAt={existing.data.created_at}
          createdBy={existing.data.created_by}
          updatedAt={existing.data.updated_at}
          updatedBy={existing.data.updated_by}
        />
      )}
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <LayoutFormFields
          entityType={definition.key}
          order={["primary_name", "status", "owner_user_id", "notes", ...customFieldOrder]}
          entityId={recordId}
          relatedKeys={relatedKeys}
          fields={{
            primary_name: (
              <div className="form-field full" key="primary_name">
                <label>Name</label>
                <input value={input.primary_name} onChange={(e) => setInput({ ...input, primary_name: e.target.value })} required />
              </div>
            ),
            status: (
              <div className="form-field" key="status">
                <label>Status</label>
                <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
                  {CUSTOM_RECORD_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </div>
            ),
            owner_user_id: (
              <div className="form-field" key="owner_user_id">
                <label>Owner</label>
                <select
                  value={input.owner_user_id ?? ""}
                  onChange={(e) => setInput({ ...input, owner_user_id: e.target.value || null })}
                >
                  <option value="">— Unassigned —</option>
                  {(users.data ?? []).map((u) => (
                    <option key={u.id} value={u.id}>
                      {u.display_name}
                    </option>
                  ))}
                </select>
              </div>
            ),
            notes: (
              <div className="form-field full" key="notes">
                <label>Notes</label>
                <textarea value={input.notes ?? ""} onChange={(e) => setInput({ ...input, notes: e.target.value || null })} />
              </div>
            ),
            ...customFieldElements,
          }}
        />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
      {recordId && <AuditTrail entityType={definition.key} entityId={recordId} />}
    </div>
  );
}
