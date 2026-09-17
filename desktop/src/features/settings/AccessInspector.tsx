import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { CAPABILITIES } from "../../lib/types";
import type { AccessInspectorResult, Capability } from "../../lib/types";

const BUILTIN_OBJECT_KEYS = ["Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task"];

/**
 * Access Control v1's Access Inspector (spec: "a real, traceable answer to
 * 'why can/can't this user see this record' - not guesswork"). Calls the
 * exact same `access_service::explain_access` evaluator that enforcement
 * uses, then renders its per-role breakdown and final decision.
 */
export function AccessInspector({ onClose }: { onClose: () => void }) {
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const customObjects = useQuery({ queryKey: ["customObjects"], queryFn: () => api.listCustomObjects(true) });

  const [actorUserId, setActorUserId] = useState("");
  const [objectKey, setObjectKey] = useState(BUILTIN_OBJECT_KEYS[0]);
  const [capability, setCapability] = useState<Capability>("read");
  const [recordId, setRecordId] = useState("");
  const [result, setResult] = useState<AccessInspectorResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const objectKeys = [...BUILTIN_OBJECT_KEYS, ...(customObjects.data ?? []).map((o) => o.key)];

  const check = useMutation({
    mutationFn: () => api.inspectAccess(objectKey, capability, recordId.trim() || null, actorUserId || null),
    onSuccess: setResult,
    onError: (err) => {
      setResult(null);
      setError(err instanceof ApiError ? err.message : "Could not run the Access Inspector");
    },
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Access Inspector</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Pick a user, an object and capability, and optionally a specific record id - this runs the same evaluation
        that actually enforces access, so the answer here is exactly what would happen for real.
      </p>
      {error && <div className="error-banner">{error}</div>}

      <div className="form-grid">
        <div className="form-field">
          <label>User</label>
          <select value={actorUserId} onChange={(e) => setActorUserId(e.target.value)}>
            <option value="">Myself</option>
            {(users.data ?? []).map((u) => (
              <option key={u.id} value={u.id}>
                {u.display_name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Object</label>
          <select value={objectKey} onChange={(e) => setObjectKey(e.target.value)}>
            {objectKeys.map((k) => (
              <option key={k} value={k}>
                {k}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Capability</label>
          <select value={capability} onChange={(e) => setCapability(e.target.value as Capability)}>
            {CAPABILITIES.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Record id (optional)</label>
          <input value={recordId} onChange={(e) => setRecordId(e.target.value)} placeholder="Leave blank to check the capability generally" />
        </div>
        <div className="form-field full">
          <button className="btn btn-primary" onClick={() => check.mutate()} disabled={check.isPending}>
            Check access
          </button>
        </div>
      </div>

      {result && (
        <div style={{ marginTop: 12 }}>
          <p>
            <span className={`badge${result.decision.allowed ? " badge-success" : ""}`}>
              {result.decision.allowed ? "Allowed" : "Denied"}
            </span>{" "}
            {result.decision.reason}
          </p>
          {result.record_summary && (
            <p style={{ color: "var(--text-muted)", fontSize: 13 }}>Record: {result.record_summary}</p>
          )}
          <table>
            <thead>
              <tr>
                <th>Access Role</th>
                <th>Matched grant</th>
                <th>Grants this capability?</th>
                <th>Scope</th>
              </tr>
            </thead>
            <tbody>
              {result.roles_checked.map((c, i) => (
                <tr key={i}>
                  <td>{c.role_name}</td>
                  <td>{c.matched_object_key ?? "—"}</td>
                  <td>{c.capability_granted ? "Yes" : "No"}</td>
                  <td>{c.scope ?? "—"}</td>
                </tr>
              ))}
              {result.roles_checked.length === 0 && (
                <tr>
                  <td colSpan={4} className="empty-state">
                    This user holds no Access Roles.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
