import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";

/**
 * Enterprise Access Foundation, Phase 1 (spec §1.1): "Organization" is a
 * view over this workspace plus the org-specific fields the Access
 * Foundation migration added (code, status, root Organization Unit) - not
 * a separate record with its own create/delete lifecycle. Name/legal
 * name/currency/locale/timezone already have their own editor (Business
 * profile, above in the Workspace category) and aren't duplicated here.
 */
export function OrganizationAdmin() {
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const org = useQuery({ queryKey: ["organization"], queryFn: () => api.getOrganization() });
  const [code, setCode] = useState("");
  const [status, setStatus] = useState("Active");

  function startEdit() {
    if (!org.data) return;
    setCode(org.data.code);
    setStatus(org.data.status);
    setEditing(true);
    setError(null);
  }

  const save = useMutation({
    mutationFn: () => api.updateOrganization({ code, status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["organization"] });
      setEditing(false);
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the Organization"),
  });

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Organization</h3>
        {!editing && (
          <button className="btn btn-primary" onClick={startEdit} disabled={!org.data}>
            Edit
          </button>
        )}
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        The single security tenant this workspace belongs to - every Organization Unit, Work Team and owned record
        rolls up under it. This is the top of the access hierarchy the rest of Access Foundation builds on.
      </p>

      {org.isLoading && <p>Loading...</p>}
      {error && <div className="error-banner">{error}</div>}

      {org.data && !editing && (
        <table>
          <tbody>
            <tr>
              <th style={{ textAlign: "left", width: 160 }}>Name</th>
              <td>{org.data.name}</td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Code</th>
              <td>
                <code>{org.data.code}</code>
              </td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Status</th>
              <td>
                <span className={`badge${org.data.status === "Active" ? " badge-success" : ""}`}>{org.data.status}</span>
              </td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Currency</th>
              <td>{org.data.default_currency}</td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Locale</th>
              <td>{org.data.locale}</td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Timezone</th>
              <td>{org.data.timezone}</td>
            </tr>
          </tbody>
        </table>
      )}

      {org.data && editing && (
        <form
          className="form-grid"
          onSubmit={(e) => {
            e.preventDefault();
            save.mutate();
          }}
        >
          <div className="form-field">
            <label>Code</label>
            <input value={code} onChange={(e) => setCode(e.target.value)} required />
          </div>
          <div className="form-field">
            <label>Status</label>
            <select value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="Active">Active</option>
              <option value="Inactive">Inactive</option>
            </select>
          </div>
          <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
            <button className="btn btn-primary" type="submit" disabled={save.isPending}>
              Save
            </button>
            <button className="btn" type="button" onClick={() => setEditing(false)}>
              Cancel
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
