import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { WORK_TEAM_TYPES } from "../../lib/types";
import type { WorkTeam, WorkTeamInput } from "../../lib/types";

function emptyInput(defaultOrgUnitId: string): WorkTeamInput {
  return {
    name: "",
    code: "",
    team_type: "Operational",
    primary_org_unit_id: defaultOrgUnitId,
    owner_user_id: null,
    can_own_records: true,
    effective_from: null,
    effective_to: null,
  };
}

/**
 * Enterprise Access Foundation, Phase 1 (spec §1.3): Work Teams - the
 * first-class, record-owning security principal, distinct from a group
 * that only ever groups principals for policies. No "reassign owned
 * records" action lives here - membership and ownership are deliberately
 * independent (see work_team_service.rs's own doc comment); ending a
 * membership never touches any record this team owns.
 */
export function WorkTeamsAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [membersOfId, setMembersOfId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const teams = useQuery({ queryKey: ["workTeams"], queryFn: () => api.listWorkTeams() });
  const orgUnits = useQuery({ queryKey: ["orgUnits"], queryFn: () => api.listOrgUnits() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["workTeams"] });
  }

  const editing = teams.data?.find((t) => t.id === editingId) ?? null;
  const membersOf = teams.data?.find((t) => t.id === membersOfId) ?? null;
  const rootOrgUnitId = orgUnits.data?.find((u) => !u.parent_org_unit_id)?.id ?? "";

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Work Teams</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
            setMembersOfId(null);
          }}
          disabled={!orgUnits.data}
        >
          + New team
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        A named group of users that can own records as a unit - a queue, a project team, a partner desk. Distinct
        from an Organization Unit (a place in the hierarchy) and from a future Access Group (which only groups
        principals for a policy and never owns anything itself).
      </p>

      {creating && orgUnits.data && (
        <TeamForm
          orgUnits={orgUnits.data}
          defaultOrgUnitId={rootOrgUnitId}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && orgUnits.data && (
        <TeamEditForm
          team={editing}
          orgUnits={orgUnits.data}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {membersOf && <TeamMembersPanel team={membersOf} onClose={() => setMembersOfId(null)} />}

      {teams.isLoading && <p>Loading...</p>}
      {teams.data && teams.data.length === 0 && <p className="empty-state">No Work Teams yet.</p>}
      {teams.data && teams.data.length > 0 && !creating && !editing && !membersOf && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Code</th>
              <th>Type</th>
              <th>Owns records</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {teams.data.map((t) => (
              <tr key={t.id}>
                <td>{t.name}</td>
                <td>
                  <code>{t.code}</code>
                </td>
                <td>{t.team_type}</td>
                <td>{t.can_own_records ? "Yes" : "No"}</td>
                <td>
                  <span className={`badge${t.status === "Active" ? " badge-success" : ""}`}>{t.status}</span>
                </td>
                <td>
                  <div style={{ display: "flex", gap: 6 }}>
                    <button className="btn" onClick={() => setMembersOfId(t.id)}>
                      Members
                    </button>
                    <button className="btn" onClick={() => setEditingId(t.id)}>
                      Edit
                    </button>
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

function TeamForm({
  orgUnits,
  defaultOrgUnitId,
  onDone,
  onCancel,
}: {
  orgUnits: { id: string; name: string; depth: number }[];
  defaultOrgUnitId: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<WorkTeamInput>(emptyInput(defaultOrgUnitId));
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createWorkTeam(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this Work Team"),
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
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Code</label>
          <input value={input.code} onChange={(e) => setInput({ ...input, code: e.target.value })} placeholder="EAST-SALES" required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={input.team_type} onChange={(e) => setInput({ ...input, team_type: e.target.value })}>
            {WORK_TEAM_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Primary Organization Unit</label>
          <select value={input.primary_org_unit_id} onChange={(e) => setInput({ ...input, primary_org_unit_id: e.target.value })}>
            {orgUnits.map((u) => (
              <option key={u.id} value={u.id}>
                {"— ".repeat(u.depth)}
                {u.name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>
            <input
              type="checkbox"
              checked={input.can_own_records}
              onChange={(e) => setInput({ ...input, can_own_records: e.target.checked })}
              style={{ marginRight: 6 }}
            />
            Can own records
          </label>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create team
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function TeamEditForm({
  team,
  orgUnits,
  onDone,
  onCancel,
}: {
  team: WorkTeam;
  orgUnits: { id: string; name: string; depth: number }[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState(team.name);
  const [teamType, setTeamType] = useState(team.team_type);
  const [primaryOrgUnitId, setPrimaryOrgUnitId] = useState(team.primary_org_unit_id);
  const [canOwnRecords, setCanOwnRecords] = useState(team.can_own_records);
  const [status, setStatus] = useState(team.status);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.updateWorkTeam(team.id, {
        name,
        team_type: teamType,
        primary_org_unit_id: primaryOrgUnitId,
        owner_user_id: team.owner_user_id,
        can_own_records: canOwnRecords,
        status,
        effective_from: team.effective_from,
        effective_to: team.effective_to,
      }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this Work Team"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteWorkTeam(team.id),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this Work Team"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Code: <code>{team.code}</code> (fixed)
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
          <label>Name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={teamType} onChange={(e) => setTeamType(e.target.value)}>
            {WORK_TEAM_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Primary Organization Unit</label>
          <select value={primaryOrgUnitId} onChange={(e) => setPrimaryOrgUnitId(e.target.value)}>
            {orgUnits.map((u) => (
              <option key={u.id} value={u.id}>
                {"— ".repeat(u.depth)}
                {u.name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>
            <input type="checkbox" checked={canOwnRecords} onChange={(e) => setCanOwnRecords(e.target.checked)} style={{ marginRight: 6 }} />
            Can own records
          </label>
        </div>
        <div className="form-field">
          <label>Status</label>
          <select value={status} onChange={(e) => setStatus(e.target.value)}>
            <option value="Active">Active</option>
            <option value="Inactive">Inactive</option>
          </select>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              if (confirm(`Delete '${team.name}'? This only works if it has no active members.`)) remove.mutate();
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

function TeamMembersPanel({ team, onClose }: { team: WorkTeam; onClose: () => void }) {
  const [userId, setUserId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const members = useQuery({ queryKey: ["teamMembers", team.id], queryFn: () => api.listTeamMembers(team.id) });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["teamMembers", team.id] });
  }

  const add = useMutation({
    mutationFn: () => api.addTeamMember(team.id, userId, null),
    onSuccess: () => {
      invalidate();
      setUserId("");
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not add this member"),
  });

  const end = useMutation({
    mutationFn: (membershipId: string) => api.endTeamMembership(membershipId),
    onSuccess: invalidate,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not remove this member"),
  });

  const memberUserIds = new Set((members.data ?? []).map((m) => m.user_id));
  const availableUsers = (users.data ?? []).filter((u) => !memberUserIds.has(u.id));

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Members of '{team.name}'</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      {error && <div className="error-banner">{error}</div>}

      <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <select value={userId} onChange={(e) => setUserId(e.target.value)} style={{ flex: 1 }}>
          <option value="">Add a user...</option>
          {availableUsers.map((u) => (
            <option key={u.id} value={u.id}>
              {u.display_name}
            </option>
          ))}
        </select>
        <button className="btn btn-primary" onClick={() => add.mutate()} disabled={!userId || add.isPending}>
          Add
        </button>
      </div>

      {members.isLoading && <p>Loading...</p>}
      {members.data && members.data.length === 0 && <p className="empty-state">No active members.</p>}
      {members.data && members.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>User</th>
              <th>Member since</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {members.data.map((m) => (
              <tr key={m.id}>
                <td>{users.data?.find((u) => u.id === m.user_id)?.display_name ?? m.user_id}</td>
                <td>{new Date(m.effective_from).toLocaleDateString()}</td>
                <td>
                  <button className="btn" onClick={() => end.mutate(m.id)} disabled={end.isPending}>
                    Remove
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
