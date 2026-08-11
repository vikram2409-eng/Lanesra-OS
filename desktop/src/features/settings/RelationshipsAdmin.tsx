import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  DELETE_BEHAVIORS,
  entityTypeLabel,
  RELATIONSHIP_TYPES,
  type DeleteBehavior,
  type RelationshipDefinition,
  type RelationshipDefinitionInput,
  type RelationshipType,
} from "../../lib/types";

const RELATIONSHIP_TYPE_LABELS: Record<RelationshipType, string> = {
  many_to_one: "Many-to-one (many source records, one target each)",
  one_to_one: "One-to-one",
  many_to_many: "Many-to-many",
};

const DELETE_BEHAVIOR_LABELS: Record<DeleteBehavior, string> = {
  restrict: "Restrict - block archiving a linked record",
  archive: "Archive - drop the link, keep both records",
};

function emptyInput(entityTypes: string[]): RelationshipDefinitionInput {
  return {
    source_entity_type: entityTypes[0] ?? "Company",
    target_entity_type: entityTypes[1] ?? entityTypes[0] ?? "Contact",
    relationship_type: "many_to_one",
    forward_label: "",
    reverse_label: "",
    is_required: false,
    show_related_list: true,
    delete_behavior: "restrict",
    sort_order: 0,
  };
}

/**
 * Admin extensibility Phase B (spec §20.3/§21): lets an Administrator wire
 * up a relationship between any two object types - built-in or custom -
 * without a code change. Once created, both sides automatically show a
 * related list on their record detail pages (see RelatedRecordsCard),
 * exactly like Company already shows its Contacts today.
 */
export function RelationshipsAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTypes: string[] = [...CUSTOM_FIELD_ENTITY_TYPES, ...(customObjects.data?.map((o) => o.key) ?? [])];
  const labelFor = (t: string) => customObjects.data?.find((o) => o.key === t)?.plural_label ?? entityTypeLabel(t);

  const defs = useQuery({ queryKey: ["relationshipDefinitions", "all"], queryFn: () => api.listRelationshipDefinitions(false) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["relationshipDefinitions"] });
  }

  const editing = defs.data?.find((d) => d.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Relationships</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New relationship
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Connect any two object types - built-in or custom. Once created, both sides automatically show a related list
        on their record detail pages.
      </p>

      {creating && (
        <RelationshipForm
          entityTypes={entityTypes}
          labelFor={labelFor}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <RelationshipEditForm
          definition={editing}
          labelFor={labelFor}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {defs.isLoading && <p>Loading...</p>}
      {defs.data && defs.data.length === 0 && <p className="empty-state">No relationships defined yet.</p>}
      {defs.data && defs.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Connects</th>
              <th>Type</th>
              <th>Labels</th>
              <th>On delete</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {defs.data.map((d) => (
              <tr key={d.id}>
                <td>
                  {labelFor(d.source_entity_type)} → {labelFor(d.target_entity_type)}
                </td>
                <td>{RELATIONSHIP_TYPE_LABELS[d.relationship_type]}</td>
                <td>
                  <span title="Forward label, shown on the source record">{d.forward_label}</span>
                  {" / "}
                  <span title="Reverse label, shown on the target record">{d.reverse_label}</span>
                </td>
                <td>{DELETE_BEHAVIOR_LABELS[d.delete_behavior]}</td>
                <td>
                  <span className={`badge${d.is_active ? " badge-success" : ""}`}>{d.is_active ? "Active" : "Inactive"}</span>
                  {d.is_protected && <span className="badge" style={{ marginLeft: 4 }}>System</span>}
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(d.id)} disabled={d.is_protected}>
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function EntityTypeSelect({
  value,
  onChange,
  entityTypes,
  labelFor,
}: {
  value: string;
  onChange: (v: string) => void;
  entityTypes: string[];
  labelFor: (t: string) => string;
}) {
  return (
    <select value={value} onChange={(e) => onChange(e.target.value)}>
      {entityTypes.map((t) => (
        <option key={t} value={t}>
          {labelFor(t)}
        </option>
      ))}
    </select>
  );
}

function RelationshipForm({
  entityTypes,
  labelFor,
  onDone,
  onCancel,
}: {
  entityTypes: string[];
  labelFor: (t: string) => string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<RelationshipDefinitionInput>(emptyInput(entityTypes));
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createRelationshipDefinition(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this relationship"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate();
        }}
      >
        <div className="form-field">
          <label>Source (the "many"/owning side)</label>
          <EntityTypeSelect value={input.source_entity_type} onChange={(v) => setInput({ ...input, source_entity_type: v })} entityTypes={entityTypes} labelFor={labelFor} />
        </div>
        <div className="form-field">
          <label>Target</label>
          <EntityTypeSelect value={input.target_entity_type} onChange={(v) => setInput({ ...input, target_entity_type: v })} entityTypes={entityTypes} labelFor={labelFor} />
        </div>
        <div className="form-field">
          <label>Relationship type</label>
          <select value={input.relationship_type} onChange={(e) => setInput({ ...input, relationship_type: e.target.value as RelationshipType })}>
            {RELATIONSHIP_TYPES.map((t) => (
              <option key={t} value={t}>
                {RELATIONSHIP_TYPE_LABELS[t]}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>On delete</label>
          <select value={input.delete_behavior} onChange={(e) => setInput({ ...input, delete_behavior: e.target.value as DeleteBehavior })}>
            {DELETE_BEHAVIORS.map((b) => (
              <option key={b} value={b}>
                {DELETE_BEHAVIOR_LABELS[b]}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Forward label (shown on the {labelFor(input.source_entity_type)} record)</label>
          <input value={input.forward_label} onChange={(e) => setInput({ ...input, forward_label: e.target.value })} placeholder={labelFor(input.target_entity_type)} required />
        </div>
        <div className="form-field">
          <label>Reverse label (shown on the {labelFor(input.target_entity_type)} record)</label>
          <input value={input.reverse_label} onChange={(e) => setInput({ ...input, reverse_label: e.target.value })} placeholder={labelFor(input.source_entity_type)} required />
        </div>
        <div className="form-field">
          <label>
            <input type="checkbox" checked={input.show_related_list} onChange={(e) => setInput({ ...input, show_related_list: e.target.checked })} />
            {" "}Show as a related list on both records
          </label>
        </div>
        <div className="form-field">
          <label>
            <input type="checkbox" checked={input.is_required} onChange={(e) => setInput({ ...input, is_required: e.target.checked })} />
            {" "}Source record should have a target linked
          </label>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create relationship
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function RelationshipEditForm({
  definition,
  labelFor,
  onDone,
  onCancel,
}: {
  definition: RelationshipDefinition;
  labelFor: (t: string) => string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [forwardLabel, setForwardLabel] = useState(definition.forward_label);
  const [reverseLabel, setReverseLabel] = useState(definition.reverse_label);
  const [required, setRequired] = useState(definition.is_required);
  const [showRelatedList, setShowRelatedList] = useState(definition.show_related_list);
  const [deleteBehavior, setDeleteBehavior] = useState<DeleteBehavior>(definition.delete_behavior);
  const [error, setError] = useState<string | null>(null);

  function payload(isActive: boolean) {
    return {
      forward_label: forwardLabel,
      reverse_label: reverseLabel,
      is_required: required,
      show_related_list: showRelatedList,
      delete_behavior: deleteBehavior,
      sort_order: definition.sort_order,
      is_active: isActive,
    };
  }

  const save = useMutation({
    mutationFn: () => api.updateRelationshipDefinition(definition.id, payload(definition.is_active)),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this relationship"),
  });

  const toggleActive = useMutation({
    mutationFn: () => api.updateRelationshipDefinition(definition.id, payload(!definition.is_active)),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not change this relationship's status"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteRelationshipDefinition(definition.id),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this relationship"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        {labelFor(definition.source_entity_type)} → {labelFor(definition.target_entity_type)} ({RELATIONSHIP_TYPE_LABELS[definition.relationship_type]}, fixed)
      </p>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>Forward label</label>
          <input value={forwardLabel} onChange={(e) => setForwardLabel(e.target.value)} required />
        </div>
        <div className="form-field">
          <label>Reverse label</label>
          <input value={reverseLabel} onChange={(e) => setReverseLabel(e.target.value)} required />
        </div>
        <div className="form-field">
          <label>On delete</label>
          <select value={deleteBehavior} onChange={(e) => setDeleteBehavior(e.target.value as DeleteBehavior)}>
            {DELETE_BEHAVIORS.map((b) => (
              <option key={b} value={b}>
                {DELETE_BEHAVIOR_LABELS[b]}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>
            <input type="checkbox" checked={showRelatedList} onChange={(e) => setShowRelatedList(e.target.checked)} />
            {" "}Show as a related list
          </label>
        </div>
        <div className="form-field">
          <label>
            <input type="checkbox" checked={required} onChange={(e) => setRequired(e.target.checked)} />
            {" "}Source record should have a target linked
          </label>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={() => toggleActive.mutate()} disabled={toggleActive.isPending}>
            {definition.is_active ? "Deactivate" : "Reactivate"}
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              if (confirm("Delete this relationship? This only works if no records are linked through it.")) remove.mutate();
            }}
            disabled={remove.isPending}
          >
            Delete
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
