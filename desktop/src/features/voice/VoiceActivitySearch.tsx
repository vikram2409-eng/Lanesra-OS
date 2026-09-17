import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";

/**
 * Admin-only "Voice Activity" search screen (Voice-First Mode PR 1) - the
 * workspace-wide counterpart to each user's own "My Voice Activity" panel
 * on their Account page. Backed by `voice_audit_service::search_voice_activity`,
 * which is Administrator-gated the same way every other admin-only audit
 * read in this codebase gates itself.
 */
export function VoiceActivitySearch({ onClose }: { onClose: () => void }) {
  const activity = useQuery({ queryKey: ["voiceActivitySearch"], queryFn: () => api.searchVoiceActivity(100) });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Voice Activity</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Every voice command across the workspace, most recent first. Raw audio is never stored - only the
        transcript, resolved intent/object, and the resulting plan's status and risk.
      </p>
      {activity.isLoading && <p>Loading...</p>}
      {activity.data && activity.data.length === 0 && <p className="empty-state">No voice commands yet.</p>}
      {activity.data && activity.data.length > 0 && (
        <div style={{ overflowX: "auto" }}>
          <table>
            <thead>
              <tr>
                <th>When</th>
                <th>User</th>
                <th>Said</th>
                <th>Intent</th>
                <th>Object</th>
                <th>Status</th>
                <th>Risk</th>
              </tr>
            </thead>
            <tbody>
              {activity.data.map((entry) => (
                <tr key={entry.command_id}>
                  <td>{new Date(entry.created_at).toLocaleString()}</td>
                  <td>{entry.user_id}</td>
                  <td>{entry.transcript}</td>
                  <td>{entry.intent ?? "—"}</td>
                  <td>{entry.object_key ?? "—"}</td>
                  <td>{entry.plan_status ?? "—"}</td>
                  <td>{entry.risk ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
