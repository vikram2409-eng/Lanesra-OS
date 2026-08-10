import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  CUSTOM_FIELD_TYPES,
  entityTypeLabel,
  type CustomFieldDefinition,
  type CustomFieldDefinitionInput,
} from "../../lib/types";

function emptyInput(entityType: string): CustomFieldDefinitionInput {
  return { entity_type: entityType, label: "", field_type: "text", options: [], required: false, show_in_list: false, sort_order: 0 };
}

/** "Retail, Manufacturing, Services" <-> ["Retail", "Manufacturing", "Services"] */
function optionsToText(options: string[]): string {
  return options.join(", ");
}
function textToOptions(text: string): string[] {
  return text
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

export function CustomFieldsAdmin() {
  const [entityType, setEntityType] = useState<string>("Company");
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  // Active custom objects appear as extra tabs alongside the nine built-in
  // entity types - a custom object's fields go through this exact same
  // admin screen, not a separate one.
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];

  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType, "all"],
    queryFn: () => api.listCustomFieldDefinitions(entityType, false),
  });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["customFieldDefinitions"] });
  }

  const editing = defs.data?.find((d) => d.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Custom fields</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New field
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Extra fields your business needs that aren't built in - shown on that record's create/edit form. Type and
        key can't change once created; everything else can.
      </p>

      <div className="tab-row">
        {entityTabs.map((t) => (
          <button
            key={t.key}
            className={`tab${entityType === t.key ? " active" : ""}`}
            onClick={() => {
              setEntityType(t.key);
              setCreating(false);
              setEditingId(null);
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {creating && (
        <DefinitionForm
          entityType={entityType}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <DefinitionEditForm
          definition={editing}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {defs.isLoading && <p>Loading...</p>}
      {defs.data && defs.data.length === 0 && (
        <p className="empty-state">
          No custom fields defined for {entityTabs.find((t) => t.key === entityType)?.label ?? entityType} yet.
        </p>
      )}
      {defs.data && defs.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Label</th>
              <th>Type</th>
              <th>Required</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {defs.data.map((d) => (
              <tr key={d.id}>
                <td>{d.label}</td>
                <td>{d.field_type}</td>
                <td>{d.required ? "Yes" : "No"}</td>
                <td>
                  <span className={`badge${d.is_active ? " badge-success" : ""}`}>{d.is_active ? "Active" : "Inactive"}</span>
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(d.id)}>
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

function DefinitionForm({
  entityType,
  onDone,
  onCancel,
}: {
  entityType: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<CustomFieldDefinitionInput>(emptyInput(entityType));
  const [optionsText, setOptionsText] = useState("");
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createCustomFieldDefinition({ ...input, options: textToOptions(optionsText) }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this field"),
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
        <div className="form-field full">
          <label>Label</label>
          <input value={input.label} onChange={(e) => setInput({ ...input, label: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={input.field_type} onChange={(e) => setInput({ ...input, field_type: e.target.value as typeof input.field_type })}>
            {CUSTOM_FIELD_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={input.required} onChange={(e) => setInput({ ...input, required: e.target.checked })} />
            Required
          </label>
        </div>
        {input.field_type === "select" && (
          <div className="form-field full">
            <label>Options (comma-separated)</label>
            <input value={optionsText} onChange={(e) => setOptionsText(e.target.value)} placeholder="Retail, Manufacturing, Services" required />
          </div>
        )}
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Add field
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function DefinitionEditForm({
  definition,
  onDone,
  onCancel,
}: {
  definition: CustomFieldDefinition;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [label, setLabel] = useState(definition.label);
  const [optionsText, setOptionsText] = useState(optionsToText(definition.options));
  const [required, setRequired] = useState(definition.required);
  const [isActive, setIsActive] = useState(definition.is_active);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.updateCustomFieldDefinition(definition.id, {
        label,
        options: textToOptions(optionsText),
        required,
        show_in_list: definition.show_in_list,
        sort_order: definition.sort_order,
        is_active: isActive,
      }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this field"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Type: <code>{definition.field_type}</code> · Key: <code>{definition.key}</code> (fixed)
      </p>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field full">
          <label>Label</label>
          <input value={label} onChange={(e) => setLabel(e.target.value)} required />
        </div>
        {definition.field_type === "select" && (
          <div className="form-field full">
            <label>Options (comma-separated)</label>
            <input value={optionsText} onChange={(e) => setOptionsText(e.target.value)} required />
          </div>
        )}
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={required} onChange={(e) => setRequired(e.target.checked)} />
            Required
          </label>
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
            Active
          </label>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
