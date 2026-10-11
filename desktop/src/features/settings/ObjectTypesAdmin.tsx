import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { ActionCapability, ActionEffect, ActionParameterSpec, ActionTypeDefinition, ActionTypeDefinitionInput, ObjectTypeSummary } from "../../lib/types";
import { CUSTOM_FIELD_TYPES } from "../../lib/types";

const ACTION_EFFECTS: ActionEffect[] = ["set_status", "set_fields", "reassign_owner"];
const ACTION_CAPABILITIES: ActionCapability[] = ["create", "read", "update", "delete", "assign"];

/**
 * Next-Gen program, Ontology Layer epic (issues #340-#345): the
 * Object Type Registry (#341) and Link Types (#342) - a single read-only
 * view over every Object Type (built-in and Custom Object alike) and the
 * Link Types it participates in. Not a second definition store: picking
 * an object type here just surfaces what Fields/Relationships/Business
 * Rules/Workflows already have for it, one click away via "Open in...".
 * The fuller Ontology Manager (Action Types, Agent Bindings) lands as a
 * later phase of this same epic (#343-#345) and will extend this screen
 * rather than replace it.
 */
export function ObjectTypesAdmin({
  onOpenTab,
  onOpenDependencyExplorerFor,
}: {
  onOpenTab?: (adminTab: string) => void;
  onOpenDependencyExplorerFor?: (objectType: string) => void;
}) {
  const [selected, setSelected] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const objectTypes = useQuery({ queryKey: ["ontologyObjectTypes"], queryFn: () => api.listObjectTypes() });
  const detail = useQuery({
    queryKey: ["ontologyObjectTypeDetail", selected],
    queryFn: () => api.getObjectTypeDetail(selected!),
    enabled: !!selected,
  });
  const actionTypes = useQuery({
    queryKey: ["actionTypes", selected, "all"],
    queryFn: () => api.listActionTypes(selected!, false),
    enabled: !!selected,
  });
  function invalidateActionTypes() {
    queryClient.invalidateQueries({ queryKey: ["actionTypes", selected] });
  }

  return (
    <div>
      <h3>Object Types</h3>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Every object in the system - built-in and Custom Object alike - as one typed registry: its identity (label, icon,
        color), its plain-English description where one is mapped in the Business Glossary, and the Link Types it
        participates in.
      </p>

      <div style={{ display: "flex", gap: 20, alignItems: "flex-start" }}>
        <div className="card" style={{ flex: "0 0 320px", padding: 0 }}>
          {objectTypes.isLoading && <p style={{ padding: 16 }}>Loading...</p>}
          <table className="data-table">
            <tbody>
              {(objectTypes.data ?? []).map((o: ObjectTypeSummary) => (
                <tr
                  key={o.key}
                  style={{ cursor: "pointer", background: selected === o.key ? "rgba(127, 127, 127, 0.06)" : undefined }}
                  onClick={() => setSelected(o.key)}
                >
                  <td style={{ width: 28, textAlign: "center" }}>
                    <span style={{ color: `var(--chart-${o.color_index})` }}>{o.icon}</span>
                  </td>
                  <td>
                    <strong>{o.label_plural}</strong>
                    {o.is_custom && <span className="badge" style={{ marginLeft: 8 }}>Custom</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="card" style={{ flex: 1, minWidth: 0 }}>
          {!selected && <p style={{ color: "var(--text-muted)" }}>Select an object type to see its details.</p>}
          {selected && detail.isLoading && <p>Loading...</p>}
          {selected && detail.data && (
            <>
              <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 4 }}>
                <span style={{ fontSize: 24, color: `var(--chart-${detail.data.object_type.color_index})` }}>{detail.data.object_type.icon}</span>
                <h4 style={{ margin: 0 }}>{detail.data.object_type.label_singular}</h4>
                {detail.data.object_type.is_custom && <span className="badge">Custom Object</span>}
              </div>
              <p style={{ color: "var(--text-muted)" }}>
                {detail.data.object_type.description ?? "No Business Glossary mapping yet for this object."}
              </p>

              {onOpenTab && (
                <div className="toolbar" style={{ marginBottom: 16 }}>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("fields")}>
                    Open in Fields
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("relationships")}>
                    Open in Relationships
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("rules")}>
                    Open in Business Rules
                  </button>
                  <button type="button" className="btn btn-secondary" onClick={() => onOpenTab("workflow")}>
                    Open in Workflow Automation
                  </button>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={() => (onOpenDependencyExplorerFor ? onOpenDependencyExplorerFor(selected) : onOpenTab("dependencyExplorer"))}
                  >
                    Open in Dependency Explorer
                  </button>
                </div>
              )}

              <h4>Link Types ({detail.data.link_types.length})</h4>
              {detail.data.link_types.length === 0 && <p style={{ color: "var(--text-muted)" }}>No relationships involve this object type yet.</p>}
              {detail.data.link_types.length > 0 && (
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Inverse name</th>
                      <th>Linked to</th>
                      <th>Type</th>
                    </tr>
                  </thead>
                  <tbody>
                    {detail.data.link_types.map((l) => (
                      <tr key={`${l.relationship_id}-${l.from_object_type}`}>
                        <td>{l.name}</td>
                        <td>{l.inverse_name}</td>
                        <td>{l.to_object_type ?? "Polymorphic (varies per record)"}</td>
                        <td>{l.relationship_type}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}

              <ActionTypesSection targetObjectType={selected!} actionTypes={actionTypes.data ?? []} onChanged={invalidateActionTypes} />
            </>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * Ontology Layer, issue #343: Action Types - a named, parameterized write
 * operation scoped to this object type, invokable identically by a
 * record page's generalized Quick Action button (`PageRenderer`'s
 * `QuickActionButton`) and, later, a governed AI agent (issue #344).
 * Deactivate only, never delete - a page layout's `quick_action`
 * component or an agent binding may still reference this id.
 */
function ActionTypesSection({
  targetObjectType,
  actionTypes,
  onChanged,
}: {
  targetObjectType: string;
  actionTypes: ActionTypeDefinition[];
  onChanged: () => void;
}) {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const editing = actionTypes.find((a) => a.id === editingId) ?? null;
  const deactivate = useMutation({ mutationFn: (id: string) => api.deactivateActionType(id), onSuccess: onChanged });

  return (
    <div style={{ marginTop: 24 }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Action Types ({actionTypes.length})</h4>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New Action Type
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        A named, governed write - "Mark Won," "Approve," "Reassign owner" - that always runs through this object's
        real write path (business rules and access checks apply exactly as they would to a manual edit).
      </p>

      {creating && (
        <ActionTypeForm
          targetObjectType={targetObjectType}
          onDone={() => {
            onChanged();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}
      {editing && (
        <ActionTypeForm
          targetObjectType={targetObjectType}
          actionType={editing}
          onDone={() => {
            onChanged();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {actionTypes.length === 0 && !creating && <p style={{ color: "var(--text-muted)" }}>No Action Types defined yet for this object.</p>}
      {actionTypes.length > 0 && (
        <table className="data-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Effect</th>
              <th>Required capability</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {actionTypes.map((a) => (
              <tr key={a.id}>
                <td>
                  <b>{a.name}</b>
                  {a.description && (
                    <>
                      <br />
                      <small className="muted">{a.description}</small>
                    </>
                  )}
                </td>
                <td>{a.effect}</td>
                <td>{a.required_capability}</td>
                <td>{a.is_active ? <span className="badge badge-success">Active</span> : <span className="badge">Inactive</span>}</td>
                <td>
                  <div className="actions">
                    <button
                      className="icon-btn"
                      onClick={() => {
                        setEditingId(a.id);
                        setCreating(false);
                      }}
                    >
                      Edit
                    </button>
                    {a.is_active && (
                      <button className="icon-btn" onClick={() => deactivate.mutate(a.id)}>
                        Deactivate
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function emptyParameter(): ActionParameterSpec {
  return { key: "", label: "", field_type: "text", required: false, default_value: null };
}

function ActionTypeForm({
  targetObjectType,
  actionType,
  onDone,
  onCancel,
}: {
  targetObjectType: string;
  actionType?: ActionTypeDefinition;
  onDone: () => void;
  onCancel: () => void;
}) {
  const customFields = useQuery({ queryKey: ["customFieldDefinitions", targetObjectType, "active"], queryFn: () => api.listCustomFieldDefinitions(targetObjectType, true) });

  const [name, setName] = useState(actionType?.name ?? "");
  const [description, setDescription] = useState(actionType?.description ?? "");
  const [requiredCapability, setRequiredCapability] = useState<ActionCapability>(actionType?.required_capability ?? "update");
  const [effect, setEffect] = useState<ActionEffect>(actionType?.effect ?? "set_status");
  const [parameters, setParameters] = useState<ActionParameterSpec[]>(actionType?.parameters ?? []);
  const initialConfig = actionType ? (JSON.parse(actionType.effect_config_json) as Record<string, unknown>) : {};
  const [targetStatus, setTargetStatus] = useState<string>((initialConfig.target_status as string) ?? "");
  const [ownerParam, setOwnerParam] = useState<string>((initialConfig.owner_param as string) ?? "");
  const [fieldSources, setFieldSources] = useState<Record<string, string>>((initialConfig.field_sources as Record<string, string>) ?? {});
  const [error, setError] = useState<string | null>(null);

  function buildInput(): ActionTypeDefinitionInput {
    const effect_config_json =
      effect === "set_status"
        ? JSON.stringify({ target_status: targetStatus })
        : effect === "reassign_owner"
          ? JSON.stringify({ owner_param: ownerParam })
          : JSON.stringify({ field_sources: Object.fromEntries(Object.entries(fieldSources).filter(([, v]) => v)) });
    return { name, description, target_object_type: targetObjectType, required_capability: requiredCapability, parameters, effect, effect_config_json };
  }

  const save = useMutation({
    mutationFn: () => (actionType ? api.updateActionType(actionType.id, buildInput()) : api.createActionType(buildInput())),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this Action Type"),
  });

  function updateParam(i: number, patch: Partial<ActionParameterSpec>) {
    setParameters(parameters.map((p, idx) => (idx === i ? { ...p, ...patch } : p)));
  }

  return (
    <div className="panel" style={{ marginBottom: 16 }}>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-grid">
        <div className="form-field">
          <label>Name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Mark Won" />
        </div>
        <div className="form-field">
          <label>Required capability</label>
          <select value={requiredCapability} onChange={(e) => setRequiredCapability(e.target.value as ActionCapability)}>
            {ACTION_CAPABILITIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Effect</label>
          <select value={effect} onChange={(e) => setEffect(e.target.value as ActionEffect)}>
            {ACTION_EFFECTS.map((e) => (
              <option key={e} value={e}>
                {e}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Description (shown on the button, and to an agent deciding whether to call it)</label>
          <textarea rows={2} value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
      </div>

      <div style={{ marginTop: 12 }}>
        <div className="toolbar">
          <strong style={{ fontSize: 13 }}>Parameters</strong>
          <button className="btn" onClick={() => setParameters([...parameters, emptyParameter()])}>
            + Add parameter
          </button>
        </div>
        {parameters.length === 0 && <p className="muted" style={{ fontSize: 12 }}>No parameters - this action runs with no input.</p>}
        {parameters.map((p, i) => (
          <div key={i} style={{ display: "flex", gap: 8, marginBottom: 6, flexWrap: "wrap", alignItems: "center" }}>
            <input placeholder="key" value={p.key} onChange={(e) => updateParam(i, { key: e.target.value })} style={{ width: 120 }} />
            <input placeholder="Label" value={p.label} onChange={(e) => updateParam(i, { label: e.target.value })} style={{ width: 140 }} />
            <select value={p.field_type} onChange={(e) => updateParam(i, { field_type: e.target.value })}>
              {CUSTOM_FIELD_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
            <label style={{ display: "flex", gap: 4, alignItems: "center", fontSize: 12 }}>
              <input type="checkbox" checked={p.required} onChange={(e) => updateParam(i, { required: e.target.checked })} />
              Required
            </label>
            <input
              placeholder="Default value"
              value={p.default_value ?? ""}
              onChange={(e) => updateParam(i, { default_value: e.target.value || null })}
              style={{ width: 120 }}
            />
            <button className="icon-btn" onClick={() => setParameters(parameters.filter((_, idx) => idx !== i))}>
              Remove
            </button>
          </div>
        ))}
      </div>

      <div style={{ marginTop: 12 }}>
        <strong style={{ fontSize: 13 }}>Effect configuration</strong>
        {effect === "set_status" && (
          <div className="form-field" style={{ maxWidth: 260 }}>
            <label>Sets status to</label>
            <input value={targetStatus} onChange={(e) => setTargetStatus(e.target.value)} placeholder="Won" />
          </div>
        )}
        {effect === "reassign_owner" && (
          <div className="form-field" style={{ maxWidth: 260 }}>
            <label>New owner comes from parameter</label>
            <select value={ownerParam} onChange={(e) => setOwnerParam(e.target.value)}>
              <option value="">Choose a parameter...</option>
              {parameters.map((p) => (
                <option key={p.key} value={p.key}>
                  {p.key}
                </option>
              ))}
            </select>
          </div>
        )}
        {effect === "set_fields" && (
          <div>
            <p className="muted" style={{ fontSize: 12 }}>Map each Custom Field this action writes to the parameter supplying its value.</p>
            {(customFields.data ?? []).map((f) => (
              <div key={f.id} style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 4 }}>
                <span style={{ width: 160 }}>{f.label}</span>
                <select
                  value={fieldSources[f.key] ?? ""}
                  onChange={(e) => setFieldSources({ ...fieldSources, [f.key]: e.target.value })}
                >
                  <option value="">Not written by this action</option>
                  {parameters.map((p) => (
                    <option key={p.key} value={p.key}>
                      {p.key}
                    </option>
                  ))}
                </select>
              </div>
            ))}
            {(customFields.data ?? []).length === 0 && <p className="muted" style={{ fontSize: 12 }}>No Custom Fields defined on this object yet.</p>}
          </div>
        )}
      </div>

      <div className="actions" style={{ marginTop: 12 }}>
        <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>
          {actionType ? "Save" : "Create"}
        </button>
        <button className="btn" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
