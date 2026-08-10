import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { CustomObjectDefinition, CustomObjectDefinitionInput } from "../../lib/types";

const EMPTY_INPUT: CustomObjectDefinitionInput = {
  singular_label: "",
  plural_label: "",
  icon: "◆",
  prefix: "",
  digits: 6,
};

const ICON_CHOICES = ["◆", "🏭", "📦", "🚗", "🏢", "🔧", "📋", "🗂️", "💼", "🏗️"];

/**
 * Admin extensibility (spec §20.2): lets an Administrator define a whole
 * new business object at runtime - "Vendors", "Assets", "Projects" -
 * without a code change. Once defined, an object shows up as its own
 * sidebar section and as an extra tab in the Custom fields / Business
 * rules admin screens, exactly like any of the nine built-in entities.
 */
export function CustomObjectsAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const objects = useQuery({ queryKey: ["customObjects", "all"], queryFn: () => api.listCustomObjects(false) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["customObjects"] });
  }

  const editing = objects.data?.find((o) => o.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Custom objects</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New object
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Add a whole new business object - Vendors, Assets, Projects - without a code change. Once created it gets its
        own place in the sidebar and can carry custom fields, business rules and reports, exactly like Companies or
        Contacts.
      </p>

      {creating && (
        <ObjectForm
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <ObjectEditForm
          object={editing}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {objects.isLoading && <p>Loading...</p>}
      {objects.data && objects.data.length === 0 && <p className="empty-state">No custom objects yet.</p>}
      {objects.data && objects.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th></th>
              <th>Name</th>
              <th>Key</th>
              <th>Numbering</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {objects.data.map((o) => (
              <tr key={o.id}>
                <td>{o.icon}</td>
                <td>{o.plural_label}</td>
                <td>
                  <code>{o.key}</code>
                </td>
                <td>
                  <code>
                    {o.prefix}-{"0".repeat(o.digits)}
                  </code>
                </td>
                <td>
                  <span className={`badge${o.is_active ? " badge-success" : ""}`}>{o.is_active ? "Active" : "Inactive"}</span>
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(o.id)}>
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

function ObjectForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<CustomObjectDefinitionInput>(EMPTY_INPUT);
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createCustomObject(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this object"),
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
          <label>Singular name</label>
          <input
            value={input.singular_label}
            onChange={(e) => setInput({ ...input, singular_label: e.target.value })}
            placeholder="Vendor"
            required
          />
        </div>
        <div className="form-field">
          <label>Plural name</label>
          <input
            value={input.plural_label}
            onChange={(e) => setInput({ ...input, plural_label: e.target.value })}
            placeholder="Vendors"
            required
          />
        </div>
        <div className="form-field">
          <label>Icon</label>
          <select value={input.icon} onChange={(e) => setInput({ ...input, icon: e.target.value })}>
            {ICON_CHOICES.map((icon) => (
              <option key={icon} value={icon}>
                {icon}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Record-number prefix</label>
          <input
            value={input.prefix}
            onChange={(e) => setInput({ ...input, prefix: e.target.value })}
            placeholder="VEN"
            maxLength={20}
            required
          />
        </div>
        <div className="form-field">
          <label>Digit width</label>
          <input
            type="number"
            min={1}
            max={10}
            value={input.digits}
            onChange={(e) => setInput({ ...input, digits: Math.min(10, Math.max(1, Number(e.target.value) || 1)) })}
          />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create object
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function ObjectEditForm({
  object,
  onDone,
  onCancel,
}: {
  object: CustomObjectDefinition;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [singular, setSingular] = useState(object.singular_label);
  const [plural, setPlural] = useState(object.plural_label);
  const [icon, setIcon] = useState(object.icon);
  const [prefix, setPrefix] = useState(object.prefix);
  const [digits, setDigits] = useState(object.digits);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.updateCustomObject(object.id, {
        singular_label: singular,
        plural_label: plural,
        icon,
        prefix,
        digits,
        is_active: object.is_active,
      }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this object"),
  });

  const toggleActive = useMutation({
    mutationFn: () =>
      object.is_active
        ? api.deactivateCustomObject(object.id)
        : api.updateCustomObject(object.id, { singular_label: singular, plural_label: plural, icon, prefix, digits, is_active: true }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not change this object's status"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteCustomObject(object.id),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this object"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Key: <code>{object.key}</code> (fixed)
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
          <label>Singular name</label>
          <input value={singular} onChange={(e) => setSingular(e.target.value)} required />
        </div>
        <div className="form-field">
          <label>Plural name</label>
          <input value={plural} onChange={(e) => setPlural(e.target.value)} required />
        </div>
        <div className="form-field">
          <label>Icon</label>
          <select value={icon} onChange={(e) => setIcon(e.target.value)}>
            {ICON_CHOICES.map((i) => (
              <option key={i} value={i}>
                {i}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Record-number prefix</label>
          <input value={prefix} onChange={(e) => setPrefix(e.target.value)} maxLength={20} required />
        </div>
        <div className="form-field">
          <label>Digit width</label>
          <input
            type="number"
            min={1}
            max={10}
            value={digits}
            onChange={(e) => setDigits(Math.min(10, Math.max(1, Number(e.target.value) || 1)))}
          />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={() => toggleActive.mutate()} disabled={toggleActive.isPending}>
            {object.is_active ? "Deactivate" : "Reactivate"}
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              if (confirm(`Delete '${object.plural_label}'? This only works if it has no records.`)) remove.mutate();
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
