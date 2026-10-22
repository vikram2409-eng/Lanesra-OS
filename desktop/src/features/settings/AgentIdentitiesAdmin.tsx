import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AgentIdentity, AgentIdentityInput, IssuedAgentCredential } from "../../lib/types";

export const AGENT_IDENTITIES_FLAG = "agents.identities_ui";
export const AGENT_IDENTITIES_QUERY_KEY = ["agentIdentities"] as const;

function message(err: unknown, fallback: string): string {
  return err instanceof ApiError ? err.message : fallback;
}

/**
 * Admin → AI → Agent Identities (WP-0109, ADR-0016 §1). An Agent Identity
 * is who an unattended run (a schedule, a webhook, a workflow action) acts
 * as. It never signs in and never borrows a person's rights: it holds only
 * the Administrator role and Access Roles given here. Shown only while the
 * `agents.identities_ui` flag is on; the rules apply either way.
 */
export function AgentIdentitiesAdmin() {
  const queryClient = useQueryClient();
  const identities = useQuery({ queryKey: AGENT_IDENTITIES_QUERY_KEY, queryFn: () => api.listAgentIdentities() });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const roles = useQuery({ queryKey: ["accessRoles"], queryFn: () => api.listAccessRoles() });
  const [editing, setEditing] = useState<AgentIdentity | "new" | null>(null);
  const [credentialsFor, setCredentialsFor] = useState<AgentIdentity | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: AGENT_IDENTITIES_QUERY_KEY });
  }

  const toggle = useMutation({
    mutationFn: (i: AgentIdentity) => api.setAgentIdentityActive(i.id, i.status !== "active"),
    onSuccess: invalidate,
    onError: (err) => setError(message(err, "Could not change this identity")),
  });
  const remove = useMutation({
    mutationFn: (id: string) => api.deleteAgentIdentity(id),
    onSuccess: invalidate,
    onError: (err) => setError(message(err, "Could not delete this identity")),
  });

  const userName = (id: string) => users.data?.find((u) => u.id === id)?.display_name ?? id;
  const roleNames = (ids: string[]) =>
    ids.map((id) => roles.data?.find((r) => r.id === id)?.name ?? id).join(", ") || "None";

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Agent Identities</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        An Agent Identity is who a scheduled, webhook or workflow run acts as. It never signs in and never borrows a
        person's rights, so give it only what its agent needs. Pick one on each trigger. Every change is recorded in
        the audit log.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {identities.isLoading && <p>Loading...</p>}
      {identities.error && <p className="error">{message(identities.error, "Could not load Agent Identities")}</p>}
      {identities.data && identities.data.length === 0 && <p>No Agent Identities yet.</p>}
      {identities.data && identities.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Owner</th>
              <th>Rights</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {identities.data.map((i) => (
              <tr key={i.id}>
                <td>
                  <div>{i.name}</div>
                  {i.description && <div style={{ color: "var(--text-muted)", fontSize: 12 }}>{i.description}</div>}
                </td>
                <td>{userName(i.owner_user_id)}</td>
                <td>
                  {i.is_administrator && <span className="badge">Administrator</span>} {roleNames(i.access_role_ids)}
                </td>
                <td>
                  <span className={`badge${i.status === "active" ? " badge-success" : ""}`}>
                    {i.status === "active" ? "Active" : "Disabled"}
                  </span>
                </td>
                <td style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                  <button className="btn btn-secondary" onClick={() => setEditing(i)}>
                    Edit
                  </button>
                  <button className="btn btn-secondary" onClick={() => setCredentialsFor(i)}>
                    Credentials
                  </button>
                  <button className="btn btn-secondary" disabled={toggle.isPending} onClick={() => toggle.mutate(i)}>
                    {i.status === "active" ? "Disable" : "Enable"}
                  </button>
                  <button
                    className="btn btn-secondary"
                    disabled={remove.isPending}
                    onClick={() => {
                      if (window.confirm(`Delete Agent Identity '${i.name}'? Triggers using it stop running until you pick another.`)) {
                        remove.mutate(i.id);
                      }
                    }}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {editing === null ? (
        <button className="btn" style={{ marginTop: 8 }} onClick={() => setEditing("new")}>
          + New Agent Identity
        </button>
      ) : (
        <IdentityForm
          identity={editing === "new" ? null : editing}
          users={(users.data ?? []).filter((u) => u.is_active)}
          roles={roles.data ?? []}
          onDone={() => {
            setEditing(null);
            invalidate();
          }}
          onCancel={() => setEditing(null)}
        />
      )}
      {credentialsFor && <CredentialsPanel identity={credentialsFor} onClose={() => setCredentialsFor(null)} />}
    </div>
  );
}

function IdentityForm({
  identity,
  users,
  roles,
  onDone,
  onCancel,
}: {
  identity: AgentIdentity | null;
  users: { id: string; display_name: string }[];
  roles: { id: string; name: string }[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<AgentIdentityInput>({
    name: identity?.name ?? "",
    description: identity?.description ?? null,
    owner_user_id: identity?.owner_user_id ?? users[0]?.id ?? "",
    is_administrator: identity?.is_administrator ?? false,
    access_role_ids: identity?.access_role_ids ?? [],
  });
  const save = useMutation({
    mutationFn: () => (identity ? api.updateAgentIdentity(identity.id, input) : api.createAgentIdentity(input)),
    onSuccess: onDone,
  });

  function toggleRole(id: string, on: boolean) {
    setInput((prev) => ({
      ...prev,
      access_role_ids: on ? [...prev.access_role_ids, id] : prev.access_role_ids.filter((r) => r !== id),
    }));
  }

  return (
    <div className="card" style={{ marginTop: 8 }}>
      <h4 style={{ marginTop: 0 }}>{identity ? `Edit ${identity.name}` : "New Agent Identity"}</h4>
      {save.error && <div className="error-banner">{message(save.error, "Could not save this identity")}</div>}
      <label>
        Name
        <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} />
      </label>
      <label>
        Description
        <input
          value={input.description ?? ""}
          onChange={(e) => setInput({ ...input, description: e.target.value || null })}
        />
      </label>
      <label>
        Accountable owner
        <select value={input.owner_user_id} onChange={(e) => setInput({ ...input, owner_user_id: e.target.value })}>
          {users.map((u) => (
            <option key={u.id} value={u.id}>
              {u.display_name}
            </option>
          ))}
        </select>
      </label>
      <label style={{ display: "flex", gap: 6, alignItems: "center" }}>
        <input
          type="checkbox"
          checked={input.is_administrator}
          onChange={(e) => setInput({ ...input, is_administrator: e.target.checked })}
        />
        Administrator (needed only for agents that use admin tools)
      </label>
      <fieldset>
        <legend>Access Roles</legend>
        {roles.length === 0 && <p style={{ color: "var(--text-muted)" }}>No Access Roles defined.</p>}
        {roles.map((r) => (
          <label key={r.id} style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={input.access_role_ids.includes(r.id)}
              onChange={(e) => toggleRole(r.id, e.target.checked)}
            />
            {r.name}
          </label>
        ))}
      </fieldset>
      <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
        <button className="btn" disabled={save.isPending || !input.name.trim() || !input.owner_user_id} onClick={() => save.mutate()}>
          Save
        </button>
        <button className="btn btn-secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

function CredentialsPanel({ identity, onClose }: { identity: AgentIdentity; onClose: () => void }) {
  const queryClient = useQueryClient();
  const key = ["agentCredentials", identity.id];
  const credentials = useQuery({ queryKey: key, queryFn: () => api.listAgentCredentials(identity.id) });
  const [label, setLabel] = useState("");
  const [issued, setIssued] = useState<IssuedAgentCredential | null>(null);

  function shown(result: IssuedAgentCredential) {
    setIssued(result);
    setLabel("");
    queryClient.invalidateQueries({ queryKey: key });
  }

  const issue = useMutation({ mutationFn: () => api.issueAgentCredential(identity.id, label.trim(), null), onSuccess: shown });
  const rotate = useMutation({ mutationFn: (id: string) => api.rotateAgentCredential(id, null), onSuccess: shown });
  const revoke = useMutation({
    mutationFn: (id: string) => api.revokeAgentCredential(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: key }),
  });
  const failure = issue.error ?? rotate.error ?? revoke.error;

  return (
    <div className="card" style={{ marginTop: 8 }}>
      <h4 style={{ marginTop: 0 }}>Credentials for {identity.name}</h4>
      {failure && <div className="error-banner">{message(failure, "Could not change this credential")}</div>}
      {issued && (
        <p className="callout callout-warning" style={{ fontSize: 13 }}>
          Copy this token now; it is not shown again: <code>{issued.token}</code>
        </p>
      )}
      {(credentials.data ?? []).map((c) => (
        <div key={c.id} style={{ display: "flex", gap: 8, alignItems: "center", margin: "4px 0", flexWrap: "wrap" }}>
          <span>{c.label}</span>
          <span style={{ color: "var(--text-muted)" }}>
            {c.revoked_at ? "revoked" : `expires ${new Date(c.expires_at).toLocaleString()}`}
          </span>
          {!c.revoked_at && (
            <>
              <button className="btn btn-secondary" disabled={rotate.isPending} onClick={() => rotate.mutate(c.id)}>
                Rotate
              </button>
              <button className="btn btn-secondary" disabled={revoke.isPending} onClick={() => revoke.mutate(c.id)}>
                Revoke
              </button>
            </>
          )}
        </div>
      ))}
      <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
        <input aria-label="Credential label" placeholder="Label" value={label} onChange={(e) => setLabel(e.target.value)} />
        <button className="btn btn-secondary" disabled={issue.isPending || !label.trim()} onClick={() => issue.mutate()}>
          + Issue credential
        </button>
        <button className="btn btn-secondary" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}
