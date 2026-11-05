import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { SecuritySettingsUpdate } from "../../lib/types";

/**
 * WP-0106: Team Workspace first-run and sign-in hardening.
 *
 * Admins set the sign-in lockout thresholds, the web-session lifetime and
 * idle timeout, and the public base URL, and can see the active sessions and
 * sign them all out. The same core services back this on the desktop app
 * (where the throttling and settings apply) and the Team Workspace server
 * (which also has the web sessions listed here).
 */
export function SecurityAdmin() {
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const settings = useQuery({
    queryKey: ["security-settings"],
    queryFn: () => api.getSecuritySettings(),
  });
  const sessions = useQuery({
    queryKey: ["active-sessions"],
    queryFn: () => api.listActiveSessions(),
  });

  const [form, setForm] = useState<SecuritySettingsUpdate>({
    lockout_max_failures: 10,
    lockout_minutes: 15,
    session_lifetime_hours: 336,
    session_idle_minutes: 0,
    public_base_url: null,
  });

  function startEdit() {
    if (!settings.data) return;
    setForm({
      lockout_max_failures: settings.data.lockout_max_failures,
      lockout_minutes: settings.data.lockout_minutes,
      session_lifetime_hours: settings.data.session_lifetime_hours,
      session_idle_minutes: settings.data.session_idle_minutes,
      public_base_url: settings.data.public_base_url,
    });
    setError(null);
    setEditing(true);
  }

  const save = useMutation({
    mutationFn: () => api.updateSecuritySettings(form),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["security-settings"] });
      setEditing(false);
    },
    onError: (err) =>
      setError(err instanceof ApiError ? err.message : "Could not save the security settings"),
  });

  const signOutAll = useMutation({
    mutationFn: () => api.signOutAllSessions(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["active-sessions"] }),
    onError: (err) =>
      setError(err instanceof ApiError ? err.message : "Could not sign out the sessions"),
  });

  const num = (v: string, min: number) => {
    const n = Number.parseInt(v, 10);
    return Number.isNaN(n) ? min : n;
  };

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Security &amp; sign-in</h3>
        {!editing && (
          <button className="btn btn-primary" onClick={startEdit} disabled={!settings.data}>
            Edit
          </button>
        )}
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Sign-in lockout, web-session lifetime and idle timeout, and the public URL this server is
        reached at. Repeated failed sign-ins are delayed and then locked out; each failure, lockout
        and unlock is in the audit log.
      </p>

      {settings.isLoading && <p>Loading...</p>}
      {error && <div className="error-banner">{error}</div>}

      {settings.data && !editing && (
        <table>
          <tbody>
            <tr>
              <th style={{ textAlign: "left", width: 240 }}>Lockout after failed sign-ins</th>
              <td>
                {settings.data.lockout_max_failures} failures, {settings.data.lockout_minutes} minute
                lockout
              </td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Session lifetime</th>
              <td>{settings.data.session_lifetime_hours} hours</td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Idle timeout</th>
              <td>
                {settings.data.session_idle_minutes === 0
                  ? "Off"
                  : `${settings.data.session_idle_minutes} minutes`}
              </td>
            </tr>
            <tr>
              <th style={{ textAlign: "left" }}>Public base URL</th>
              <td>
                {settings.data.public_base_url ? (
                  <code>{settings.data.public_base_url}</code>
                ) : (
                  <span style={{ color: "var(--text-muted)" }}>Not set</span>
                )}
              </td>
            </tr>
          </tbody>
        </table>
      )}

      {settings.data && editing && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            save.mutate();
          }}
        >
          <div className="form-row">
            <label>Max failed sign-ins before lockout</label>
            <input
              aria-label="Max failed sign-ins before lockout"
              type="number"
              min={1}
              value={form.lockout_max_failures}
              onChange={(e) =>
                setForm({ ...form, lockout_max_failures: num(e.target.value, 1) })
              }
            />
          </div>
          <div className="form-row">
            <label>Lockout duration (minutes)</label>
            <input
              aria-label="Lockout duration (minutes)"
              type="number"
              min={1}
              value={form.lockout_minutes}
              onChange={(e) => setForm({ ...form, lockout_minutes: num(e.target.value, 1) })}
            />
          </div>
          <div className="form-row">
            <label>Session lifetime (hours)</label>
            <input
              aria-label="Session lifetime (hours)"
              type="number"
              min={1}
              value={form.session_lifetime_hours}
              onChange={(e) =>
                setForm({ ...form, session_lifetime_hours: num(e.target.value, 1) })
              }
            />
          </div>
          <div className="form-row">
            <label>Idle timeout (minutes, 0 = off)</label>
            <input
              aria-label="Idle timeout (minutes)"
              type="number"
              min={0}
              value={form.session_idle_minutes}
              onChange={(e) =>
                setForm({ ...form, session_idle_minutes: num(e.target.value, 0) })
              }
            />
          </div>
          <div className="form-row">
            <label>Public base URL (https://&hellip;)</label>
            <input
              aria-label="Public base URL"
              type="text"
              placeholder="https://workspace.example.com"
              value={form.public_base_url ?? ""}
              onChange={(e) =>
                setForm({ ...form, public_base_url: e.target.value.trim() === "" ? null : e.target.value })
              }
            />
          </div>
          <div className="toolbar" style={{ marginTop: 12 }}>
            <button type="submit" className="btn btn-primary" disabled={save.isPending}>
              {save.isPending ? "Saving..." : "Save"}
            </button>
            <button type="button" className="btn" onClick={() => setEditing(false)}>
              Cancel
            </button>
          </div>
        </form>
      )}

      <div className="toolbar" style={{ marginTop: 24 }}>
        <h3 style={{ margin: 0 }}>Active sessions</h3>
        <button
          className="btn"
          disabled={signOutAll.isPending || (sessions.data?.length ?? 0) === 0}
          onClick={() => {
            if (window.confirm("Sign out every session, including yours?")) signOutAll.mutate();
          }}
        >
          Sign out all sessions
        </button>
      </div>
      {sessions.isLoading && <p>Loading...</p>}
      {sessions.data && sessions.data.length === 0 && (
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No active web sessions.</p>
      )}
      {sessions.data && sessions.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th style={{ textAlign: "left" }}>User</th>
              <th style={{ textAlign: "left" }}>Last seen</th>
              <th style={{ textAlign: "left" }}>Expires</th>
            </tr>
          </thead>
          <tbody>
            {sessions.data.map((s) => (
              <tr key={s.id}>
                <td>
                  {s.display_name} <span style={{ color: "var(--text-muted)" }}>({s.username})</span>
                  {s.current && <span className="badge" style={{ marginLeft: 8 }}>This device</span>}
                </td>
                <td>{new Date(s.last_seen_at).toLocaleString()}</td>
                <td>{new Date(s.expires_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
