import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { RECORD_SCOPES, MAX_ACTION_LEVELS, PROCESSING_BOUNDARIES } from "../../lib/types";
import type {
  AccessRole,
  AccessRoleGrant,
  AccessRoleGrantInput,
  AccessRoleInput,
  VoicePolicyBinding,
  VoicePolicyBindingInput,
} from "../../lib/types";
import { AccessInspector } from "./AccessInspector";
import { VoiceActivitySearch } from "../voice/VoiceActivitySearch";

/** Every built-in object_key the ownership/access engine knows about -
 * mirrors `ownership_service::BUILTIN_OWNED_OBJECT_KEYS` on the backend.
 * '*' is the wildcard default every role can optionally customize. */
const BUILTIN_OBJECT_KEYS = ["Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task"];

function emptyRoleInput(): AccessRoleInput {
  return { name: "", description: "" };
}

/**
 * Access Control v1 (spec: "Access Control v1" - Access Roles, capabilities,
 * Record Scopes, Access Inspector). Distinct from the legacy `roles`
 * (Administrator/Manager/Sales/Finance/ReadOnly) still used by the handful
 * of admin-only checks elsewhere - this is the new, real, per-object
 * capability + scope engine `ownership_service::require_assign_capability`
 * now enforces for the Assign capability specifically.
 */
export function AccessRolesAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [grantsOfId, setGrantsOfId] = useState<string | null>(null);
  const [membersOfId, setMembersOfId] = useState<string | null>(null);
  const [inspecting, setInspecting] = useState(false);
  const [governingVoice, setGoverningVoice] = useState(false);
  const [voiceActivity, setVoiceActivity] = useState(false);
  const queryClient = useQueryClient();

  const roles = useQuery({ queryKey: ["accessRoles"], queryFn: () => api.listAccessRoles() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["accessRoles"] });
  }

  const editing = roles.data?.find((r) => r.id === editingId) ?? null;
  const grantsOf = roles.data?.find((r) => r.id === grantsOfId) ?? null;
  const membersOf = roles.data?.find((r) => r.id === membersOfId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Access Roles</h3>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn" onClick={() => setInspecting((v) => !v)}>
            Access Inspector
          </button>
          <button
            className="btn"
            onClick={() => {
              setGoverningVoice((v) => !v);
              setVoiceActivity(false);
            }}
          >
            Voice Governance
          </button>
          <button
            className="btn"
            onClick={() => {
              setVoiceActivity((v) => !v);
              setGoverningVoice(false);
            }}
          >
            Voice Activity
          </button>
          <button
            className="btn btn-primary"
            onClick={() => {
              setCreating((v) => !v);
              setEditingId(null);
              setGrantsOfId(null);
              setMembersOfId(null);
            }}
          >
            + New role
          </button>
        </div>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        A named, per-object capability set (create/read/update/delete/assign) plus a Record Scope (Owner, Owner's
        Team, Owner's Organization Unit and below, or Organization-wide). Managing roles here still requires the
        Administrator role - a deliberate seam, so a misconfigured role can never lock out the people who'd fix it.
      </p>

      {inspecting && <AccessInspector onClose={() => setInspecting(false)} />}
      {governingVoice && <VoiceGovernancePanel roles={roles.data ?? []} onClose={() => setGoverningVoice(false)} />}
      {voiceActivity && <VoiceActivitySearch onClose={() => setVoiceActivity(false)} />}

      {creating && (
        <RoleForm
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <RoleEditForm
          role={editing}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {grantsOf && <GrantsPanel role={grantsOf} onClose={() => setGrantsOfId(null)} />}
      {membersOf && <MembersPanel role={membersOf} onClose={() => setMembersOfId(null)} />}

      {roles.isLoading && <p>Loading...</p>}
      {roles.data && roles.data.length > 0 && !creating && !editing && !grantsOf && !membersOf && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Description</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {roles.data.map((r) => (
              <tr key={r.id}>
                <td>
                  {r.name}
                  {r.is_system && (
                    <span className="badge" style={{ marginLeft: 8 }}>
                      System
                    </span>
                  )}
                </td>
                <td>{r.description}</td>
                <td>
                  <div style={{ display: "flex", gap: 6 }}>
                    <button className="btn" onClick={() => setGrantsOfId(r.id)}>
                      Grants
                    </button>
                    <button className="btn" onClick={() => setMembersOfId(r.id)}>
                      Members
                    </button>
                    {!r.is_system && (
                      <button className="btn" onClick={() => setEditingId(r.id)}>
                        Edit
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

function RoleForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<AccessRoleInput>(emptyRoleInput());
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createAccessRole(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this Access Role"),
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
        <div className="form-field full">
          <label>Description</label>
          <input value={input.description} onChange={(e) => setInput({ ...input, description: e.target.value })} />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create role
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function RoleEditForm({ role, onDone, onCancel }: { role: AccessRole; onDone: () => void; onCancel: () => void }) {
  const [name, setName] = useState(role.name);
  const [description, setDescription] = useState(role.description);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () => api.updateAccessRole(role.id, { name, description }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this Access Role"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteAccessRole(role.id),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this Access Role"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
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
        <div className="form-field full">
          <label>Description</label>
          <input value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap" }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              if (confirm(`Delete '${role.name}'? This only works if it has no assigned users.`)) remove.mutate();
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

function GrantsPanel({ role, onClose }: { role: AccessRole; onClose: () => void }) {
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const grants = useQuery({ queryKey: ["accessRoleGrants", role.id], queryFn: () => api.listAccessRoleGrants(role.id) });
  const customObjects = useQuery({ queryKey: ["customObjects"], queryFn: () => api.listCustomObjects(true) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["accessRoleGrants", role.id] });
  }

  const save = useMutation({
    mutationFn: (input: AccessRoleGrantInput) => api.upsertAccessRoleGrant(role.id, input),
    onSuccess: invalidate,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this grant"),
  });

  const objectKeys = ["*", ...BUILTIN_OBJECT_KEYS, ...(customObjects.data ?? []).map((o) => o.key)];
  const grantByKey = new Map((grants.data ?? []).map((g) => [g.object_key, g]));

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Grants for '{role.name}'</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        <code>*</code> is this role's default for any object without its own row below (including a custom object
        created later). A blank row uses that default until you save it with its own capabilities.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {(grants.isLoading || customObjects.isLoading) && <p>Loading...</p>}
      {grants.data && customObjects.data && (
        <div style={{ overflowX: "auto" }}>
          <table>
            <thead>
              <tr>
                <th>Object</th>
                <th>Create</th>
                <th>Read</th>
                <th>Update</th>
                <th>Delete</th>
                <th>Assign</th>
                <th>Record Scope</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {objectKeys.map((key) => (
                <GrantRow
                  key={key}
                  objectKey={key}
                  grant={grantByKey.get(key) ?? null}
                  onSave={(input) => save.mutate(input)}
                  saving={save.isPending}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function GrantRow({
  objectKey,
  grant,
  onSave,
  saving,
}: {
  objectKey: string;
  grant: AccessRoleGrant | null;
  onSave: (input: AccessRoleGrantInput) => void;
  saving: boolean;
}) {
  const [canCreate, setCanCreate] = useState(grant?.can_create ?? false);
  const [canRead, setCanRead] = useState(grant?.can_read ?? false);
  const [canUpdate, setCanUpdate] = useState(grant?.can_update ?? false);
  const [canDelete, setCanDelete] = useState(grant?.can_delete ?? false);
  const [canAssign, setCanAssign] = useState(grant?.can_assign ?? false);
  const [recordScope, setRecordScope] = useState(grant?.record_scope ?? "OWNER");

  return (
    <tr>
      <td>{objectKey === "*" ? <b>Default (*)</b> : objectKey}</td>
      <td>
        <input type="checkbox" checked={canCreate} onChange={(e) => setCanCreate(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canRead} onChange={(e) => setCanRead(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canUpdate} onChange={(e) => setCanUpdate(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canDelete} onChange={(e) => setCanDelete(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canAssign} onChange={(e) => setCanAssign(e.target.checked)} />
      </td>
      <td>
        <select value={recordScope} onChange={(e) => setRecordScope(e.target.value as typeof recordScope)}>
          {RECORD_SCOPES.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
      </td>
      <td>
        <button
          className="btn"
          disabled={saving}
          onClick={() =>
            onSave({
              object_key: objectKey,
              can_create: canCreate,
              can_read: canRead,
              can_update: canUpdate,
              can_delete: canDelete,
              can_assign: canAssign,
              record_scope: recordScope,
            })
          }
        >
          Save
        </button>
      </td>
    </tr>
  );
}

function MembersPanel({ role, onClose }: { role: AccessRole; onClose: () => void }) {
  const [userId, setUserId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const assigneeIds = useQuery({ queryKey: ["accessRoleAssignees", role.id], queryFn: () => api.listAccessRoleAssignees(role.id) });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["accessRoleAssignees", role.id] });
  }

  const assign = useMutation({
    mutationFn: () => api.assignAccessRoleToUser(userId, role.id),
    onSuccess: () => {
      invalidate();
      setUserId("");
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not assign this role"),
  });

  const remove = useMutation({
    mutationFn: (uid: string) => api.removeAccessRoleFromUser(uid, role.id),
    onSuccess: invalidate,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not remove this assignment"),
  });

  const assigneeIdSet = new Set(assigneeIds.data ?? []);
  const assignedUsers = (users.data ?? []).filter((u) => assigneeIdSet.has(u.id));
  const availableUsers = (users.data ?? []).filter((u) => !assigneeIdSet.has(u.id));

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Users with '{role.name}'</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      {error && <div className="error-banner">{error}</div>}

      <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
        <select value={userId} onChange={(e) => setUserId(e.target.value)} style={{ flex: 1 }}>
          <option value="">Assign to a user...</option>
          {availableUsers.map((u) => (
            <option key={u.id} value={u.id}>
              {u.display_name}
            </option>
          ))}
        </select>
        <button className="btn btn-primary" onClick={() => assign.mutate()} disabled={!userId || assign.isPending}>
          Assign
        </button>
      </div>

      {(assigneeIds.isLoading || users.isLoading) && <p>Loading...</p>}
      {assignedUsers.length === 0 && !assigneeIds.isLoading && <p className="empty-state">No users hold this role.</p>}
      {assignedUsers.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>User</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {assignedUsers.map((u) => (
              <tr key={u.id}>
                <td>{u.display_name}</td>
                <td>
                  <button className="btn" onClick={() => remove.mutate(u.id)} disabled={remove.isPending}>
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

/**
 * Voice Governance (Voice-First Mode PR 1): editing `voice_policy_bindings`
 * per Access Role, plus one virtual "Default (all users)" row for the
 * workspace-default binding (`access_role_id IS NULL`). This composes with -
 * never replaces - the capability grants above: a Voice action still needs
 * BOTH the matching Voice capability here AND the ordinary object capability
 * from the Grants panel, exactly like `voice_policy_service::voice_capability_allows`
 * is used alongside `access_service::require_capability` on the backend.
 */
function VoiceGovernancePanel({ roles, onClose }: { roles: AccessRole[]; onClose: () => void }) {
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const bindings = useQuery({ queryKey: ["voicePolicyBindings"], queryFn: () => api.listVoicePolicyBindings() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["voicePolicyBindings"] });
  }

  const save = useMutation({
    mutationFn: (input: VoicePolicyBindingInput) => api.upsertVoicePolicyBinding(input),
    onSuccess: invalidate,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this Voice policy"),
  });

  const bindingByRoleId = new Map((bindings.data ?? []).map((b) => [b.access_role_id, b]));

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Voice Governance</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Voice permissions can only narrow what a role can already do - they never grant access beyond that role's
        own capability grants above. "Default (all users)" is the workspace-wide fallback for any role without its
        own row below.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {bindings.isLoading && <p>Loading...</p>}
      {bindings.data && (
        <div style={{ overflowX: "auto" }}>
          <table>
            <thead>
              <tr>
                <th>Role</th>
                <th>Voice</th>
                <th>Search</th>
                <th>Create</th>
                <th>Update</th>
                <th>Act</th>
                <th>Bulk act</th>
                <th>External act</th>
                <th>AI Agents</th>
                <th>Max action level</th>
                <th>Processing boundary</th>
                <th>Unlock (min)</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <VoiceGovernanceRow
                label="Default (all users)"
                accessRoleId={null}
                binding={bindingByRoleId.get(null) ?? null}
                onSave={(input) => save.mutate(input)}
                saving={save.isPending}
              />
              {roles.map((r) => (
                <VoiceGovernanceRow
                  key={r.id}
                  label={r.name}
                  accessRoleId={r.id}
                  binding={bindingByRoleId.get(r.id) ?? null}
                  onSave={(input) => save.mutate(input)}
                  saving={save.isPending}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function VoiceGovernanceRow({
  label,
  accessRoleId,
  binding,
  onSave,
  saving,
}: {
  label: string;
  accessRoleId: string | null;
  binding: VoicePolicyBinding | null;
  onSave: (input: VoicePolicyBindingInput) => void;
  saving: boolean;
}) {
  const [canUseVoice, setCanUseVoice] = useState(binding?.can_use_voice ?? false);
  const [canSearch, setCanSearch] = useState(binding?.can_search ?? false);
  const [canCreate, setCanCreate] = useState(binding?.can_create ?? false);
  const [canUpdate, setCanUpdate] = useState(binding?.can_update ?? false);
  const [canAct, setCanAct] = useState(binding?.can_act ?? false);
  const [canBulkAct, setCanBulkAct] = useState(binding?.can_bulk_act ?? false);
  const [canExternalAct, setCanExternalAct] = useState(binding?.can_external_act ?? false);
  const [canUseAgents, setCanUseAgents] = useState(binding?.can_use_agents ?? false);
  const [maxActionLevel, setMaxActionLevel] = useState(binding?.max_action_level ?? "ask_only");
  const [processingBoundary, setProcessingBoundary] = useState(binding?.processing_boundary ?? "cloud");
  const [maxUnlockMinutes, setMaxUnlockMinutes] = useState(binding?.max_unlock_minutes ?? 15);

  return (
    <tr>
      <td>{accessRoleId === null ? <b>{label}</b> : label}</td>
      <td>
        <input type="checkbox" checked={canUseVoice} onChange={(e) => setCanUseVoice(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canSearch} onChange={(e) => setCanSearch(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canCreate} onChange={(e) => setCanCreate(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canUpdate} onChange={(e) => setCanUpdate(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canAct} onChange={(e) => setCanAct(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canBulkAct} onChange={(e) => setCanBulkAct(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canExternalAct} onChange={(e) => setCanExternalAct(e.target.checked)} />
      </td>
      <td>
        <input type="checkbox" checked={canUseAgents} onChange={(e) => setCanUseAgents(e.target.checked)} />
      </td>
      <td>
        <select value={maxActionLevel} onChange={(e) => setMaxActionLevel(e.target.value as typeof maxActionLevel)}>
          {MAX_ACTION_LEVELS.map((l) => (
            <option key={l} value={l}>
              {l}
            </option>
          ))}
        </select>
      </td>
      <td>
        <select
          value={processingBoundary}
          onChange={(e) => setProcessingBoundary(e.target.value as typeof processingBoundary)}
        >
          {PROCESSING_BOUNDARIES.map((b) => (
            <option key={b} value={b}>
              {b}
            </option>
          ))}
        </select>
      </td>
      <td>
        <input
          type="number"
          min={1}
          style={{ width: 64 }}
          value={maxUnlockMinutes}
          onChange={(e) => setMaxUnlockMinutes(Number(e.target.value))}
        />
      </td>
      <td>
        <button
          className="btn"
          disabled={saving}
          onClick={() =>
            onSave({
              access_role_id: accessRoleId,
              can_use_voice: canUseVoice,
              can_search: canSearch,
              can_create: canCreate,
              can_update: canUpdate,
              can_act: canAct,
              can_bulk_act: canBulkAct,
              can_external_act: canExternalAct,
              can_use_agents: canUseAgents,
              max_action_level: maxActionLevel,
              processing_boundary: processingBoundary,
              max_unlock_minutes: maxUnlockMinutes,
            })
          }
        >
          Save
        </button>
      </td>
    </tr>
  );
}
