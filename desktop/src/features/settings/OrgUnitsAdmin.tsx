import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { ORG_UNIT_TYPES } from "../../lib/types";
import type { OrgUnit, OrgUnitInput } from "../../lib/types";

const EMPTY_INPUT: OrgUnitInput = {
  name: "",
  unit_type: "Department",
  parent_org_unit_id: null,
  manager_user_id: null,
  effective_from: null,
  effective_to: null,
};

/**
 * Enterprise Access Foundation, Phase 1 (spec §1.2): the Organization Unit
 * hierarchy - Divisions, Regions, Departments, Branches - that owning_org_unit_id
 * scopes every record to. Rendered as a flat, indented list ordered by the
 * backend's materialized path (a parent always sorts before its own
 * descendants), so no client-side tree-building pass is needed. The root
 * unit (no parent) can't be created, moved or deleted here - it's
 * provisioned once per workspace by the backend's own bootstrap.
 */
export function OrgUnitsAdmin({ onOpenHelp }: { onOpenHelp: (slug: string) => void }) {
  const [creatingUnderId, setCreatingUnderId] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [movingId, setMovingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const units = useQuery({ queryKey: ["orgUnits"], queryFn: () => api.listOrgUnits() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["orgUnits"] });
  }

  const editing = units.data?.find((u) => u.id === editingId) ?? null;
  const moving = units.data?.find((u) => u.id === movingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar" style={{ justifyContent: "space-between" }}>
        <h3 style={{ margin: 0 }}>Organization Units</h3>
        <button className="btn btn-secondary" onClick={() => onOpenHelp("organization-and-org-units")}>
          📖 Help
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        The hierarchy every record's Owning Organization Unit scopes against - Divisions, Regions, Departments,
        Branches. Every workspace has exactly one root unit; everything else nests under it.
      </p>

      {units.isLoading && <p>Loading...</p>}

      {units.data && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {units.data.map((u) => (
              <tr key={u.id}>
                <td style={{ paddingLeft: u.depth * 20 + 8 }}>
                  {u.depth > 0 && <span style={{ color: "var(--text-muted)" }}>└ </span>}
                  {u.name}
                </td>
                <td>{u.unit_type}</td>
                <td>
                  <span className={`badge${u.status === "Active" ? " badge-success" : ""}`}>{u.status}</span>
                </td>
                <td>
                  <div style={{ display: "flex", gap: 6 }}>
                    <button className="btn" onClick={() => setCreatingUnderId(u.id)}>
                      + Child
                    </button>
                    <button className="btn" onClick={() => setEditingId(u.id)}>
                      Edit
                    </button>
                    {u.parent_org_unit_id && (
                      <button className="btn" onClick={() => setMovingId(u.id)}>
                        Move
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {creatingUnderId && units.data && (
        <UnitForm
          parentId={creatingUnderId}
          onDone={() => {
            invalidate();
            setCreatingUnderId(null);
          }}
          onCancel={() => setCreatingUnderId(null)}
        />
      )}

      {editing && (
        <UnitEditForm
          unit={editing}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {moving && units.data && (
        <MoveUnitDialog
          unit={moving}
          allUnits={units.data}
          onDone={() => {
            invalidate();
            setMovingId(null);
          }}
          onCancel={() => setMovingId(null)}
        />
      )}
    </div>
  );
}

function UnitForm({ parentId, onDone, onCancel }: { parentId: string; onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<OrgUnitInput>({ ...EMPTY_INPUT, parent_org_unit_id: parentId });
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createOrgUnit(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this Organization Unit"),
  });

  return (
    <div className="card" style={{ marginTop: 16, background: "var(--surface-2, transparent)" }}>
      <h4 style={{ marginTop: 0 }}>New Organization Unit</h4>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate();
        }}
      >
        <div className="form-field">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={input.unit_type} onChange={(e) => setInput({ ...input, unit_type: e.target.value })}>
            {ORG_UNIT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function UnitEditForm({ unit, onDone, onCancel }: { unit: OrgUnit; onDone: () => void; onCancel: () => void }) {
  const [name, setName] = useState(unit.name);
  const [unitType, setUnitType] = useState(unit.unit_type);
  const [status, setStatus] = useState(unit.status);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.updateOrgUnit(unit.id, {
        name,
        unit_type: unitType,
        manager_user_id: unit.manager_user_id,
        status,
        effective_from: unit.effective_from,
        effective_to: unit.effective_to,
      }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this Organization Unit"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteOrgUnit(unit.id),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this Organization Unit"),
  });

  return (
    <div className="card" style={{ marginTop: 16, background: "var(--surface-2, transparent)" }}>
      <h4 style={{ marginTop: 0 }}>Edit '{unit.name}'</h4>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>Name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} required disabled={!unit.parent_org_unit_id} />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={unitType} onChange={(e) => setUnitType(e.target.value)} disabled={!unit.parent_org_unit_id}>
            {ORG_UNIT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        {unit.parent_org_unit_id && (
          <div className="form-field">
            <label>Status</label>
            <select value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="Active">Active</option>
              <option value="Inactive">Inactive</option>
            </select>
          </div>
        )}
        <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          {unit.parent_org_unit_id && (
            <button
              className="btn"
              type="button"
              onClick={() => {
                if (confirm(`Delete '${unit.name}'? This only works if it has no child units.`)) remove.mutate();
              }}
              disabled={remove.isPending}
            >
              Delete
            </button>
          )}
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
      {!unit.parent_org_unit_id && (
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>The root Organization Unit's name and type follow the Organization and can't be edited here.</p>
      )}
    </div>
  );
}

function MoveUnitDialog({
  unit,
  allUnits,
  onDone,
  onCancel,
}: {
  unit: OrgUnit;
  allUnits: OrgUnit[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [newParentId, setNewParentId] = useState("");
  const [error, setError] = useState<string | null>(null);

  const candidates = allUnits.filter((u) => u.id !== unit.id && !u.path.startsWith(unit.path) && u.id !== unit.parent_org_unit_id);

  const preview = useQuery({
    queryKey: ["orgUnitMovePreview", unit.id, newParentId],
    queryFn: () => api.previewMoveOrgUnit(unit.id, newParentId),
    enabled: !!newParentId,
  });

  const move = useMutation({
    mutationFn: () => api.moveOrgUnit(unit.id, newParentId),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not move this Organization Unit"),
  });

  return (
    <div className="card" style={{ marginTop: 16, background: "var(--surface-2, transparent)" }}>
      <h4 style={{ marginTop: 0 }}>Move '{unit.name}'</h4>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-field">
        <label>New parent</label>
        <select value={newParentId} onChange={(e) => setNewParentId(e.target.value)}>
          <option value="">Select a new parent...</option>
          {candidates.map((c) => (
            <option key={c.id} value={c.id}>
              {"— ".repeat(c.depth)}
              {c.name}
            </option>
          ))}
        </select>
      </div>

      {newParentId && preview.data && (
        <div
          style={{
            marginTop: 12,
            padding: 12,
            border: "1px solid var(--border, #ddd)",
            borderRadius: 6,
            background: "var(--surface-2, transparent)",
          }}
        >
          <p style={{ marginTop: 0 }}>
            <strong>Impact:</strong> {preview.data.descendant_unit_count} descendant unit(s) will move with it.
          </p>
          {preview.data.owned_record_counts.length > 0 ? (
            <ul style={{ marginBottom: 0 }}>
              {preview.data.owned_record_counts.map(([objectKey, count]) => (
                <li key={objectKey}>
                  {count} {objectKey} record(s) owned under this subtree
                </li>
              ))}
            </ul>
          ) : (
            <p style={{ marginBottom: 0, color: "var(--text-muted)" }}>No records are owned under this subtree.</p>
          )}
        </div>
      )}

      <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
        <button className="btn btn-primary" onClick={() => move.mutate()} disabled={!newParentId || move.isPending}>
          Confirm move
        </button>
        <button className="btn" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
