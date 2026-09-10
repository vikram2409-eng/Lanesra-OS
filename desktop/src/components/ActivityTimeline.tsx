import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import type { ActivityInput, User } from "../lib/types";

const CHANNELS: { key: string; label: string }[] = [
  { key: "email", label: "Email" },
  { key: "call", label: "Call" },
  { key: "message", label: "Message" },
];
const DIRECTIONS: { key: string; label: string }[] = [
  { key: "inbound", label: "Inbound" },
  { key: "outbound", label: "Outbound" },
];

function apiErrorMessage(err: unknown, fallback: string): string {
  return err instanceof ApiError ? err.message : fallback;
}

function actorName(userId: string | null | undefined, users: User[] | undefined): string {
  if (!userId) return "System";
  return users?.find((u) => u.id === userId)?.display_name ?? "Unknown user";
}

function nowForInput(): string {
  // yyyy-MM-ddThh:mm, what <input type="datetime-local"> needs - local
  // time, not UTC, so a person logging "the call I just had" gets now,
  // not an hour off in either direction.
  const d = new Date();
  d.setMinutes(d.getMinutes() - d.getTimezoneOffset());
  return d.toISOString().slice(0, 16);
}

function emptyInput(entityType: string, entityId: string): ActivityInput {
  return { entity_type: entityType, entity_id: entityId, channel: "email", direction: "inbound", subject: "", body: "", participants: "", occurred_at: nowForInput() };
}

/**
 * AI & Agentic Layer, Phase 3: the Unified Activity Timeline's card -
 * every email/call/message logged against this record, distinct from
 * `AuditTrail`'s "History" (what this workspace's own users changed on
 * the record, not what happened around it - see `core::services::
 * activity_service`'s own doc comment). Scoped to Company/Contact/
 * Opportunity, matching the backend. Every entry here is `source:
 * "manual"` today - a person recording an interaction they just had, a
 * complete feature on its own, not a placeholder. Automated ingestion
 * (an email Connection, call-transcript upload, Slack) is real,
 * separately-scoped future work - see the product backlog.
 */
export function ActivityTimeline({ entityType, entityId }: { entityType: string; entityId: string }) {
  const queryClient = useQueryClient();
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const activities = useQuery({
    queryKey: ["activities", entityType, entityId],
    queryFn: () => api.listActivities(entityType, entityId),
  });
  const [logging, setLogging] = useState(false);
  const [input, setInput] = useState<ActivityInput>(() => emptyInput(entityType, entityId));
  const [error, setError] = useState<string | null>(null);

  const log = useMutation({
    mutationFn: () => api.logActivity(input),
    onSuccess: () => {
      setLogging(false);
      setInput(emptyInput(entityType, entityId));
      setError(null);
      queryClient.invalidateQueries({ queryKey: ["activities", entityType, entityId] });
    },
    onError: (err) => setError(apiErrorMessage(err, "Could not log that interaction")),
  });

  return (
    <div className="card">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12 }}>
        <h3 style={{ marginTop: 0 }}>Interactions</h3>
        <button className="btn btn-secondary" style={{ flexShrink: 0 }} onClick={() => setLogging((v) => !v)}>
          {logging ? "Cancel" : "+ Log an interaction"}
        </button>
      </div>

      {logging && (
        <form
          className="form-grid"
          style={{ marginBottom: 16 }}
          onSubmit={(e) => {
            e.preventDefault();
            log.mutate();
          }}
        >
          {error && <div className="error-banner" style={{ gridColumn: "1 / -1" }}>{error}</div>}
          <div className="form-field">
            <label>Channel</label>
            <select value={input.channel} onChange={(e) => setInput({ ...input, channel: e.target.value })}>
              {CHANNELS.map((c) => (
                <option key={c.key} value={c.key}>
                  {c.label}
                </option>
              ))}
            </select>
          </div>
          {input.channel !== "call" && (
            <div className="form-field">
              <label>Direction</label>
              <select value={input.direction ?? ""} onChange={(e) => setInput({ ...input, direction: e.target.value || null })}>
                {DIRECTIONS.map((d) => (
                  <option key={d.key} value={d.key}>
                    {d.label}
                  </option>
                ))}
              </select>
            </div>
          )}
          <div className="form-field full">
            <label>Subject</label>
            <input value={input.subject ?? ""} onChange={(e) => setInput({ ...input, subject: e.target.value || null })} placeholder="Following up on pricing" />
          </div>
          <div className="form-field">
            <label>When</label>
            <input type="datetime-local" value={input.occurred_at} onChange={(e) => setInput({ ...input, occurred_at: e.target.value })} required />
          </div>
          <div className="form-field">
            <label>Participants</label>
            <input value={input.participants ?? ""} onChange={(e) => setInput({ ...input, participants: e.target.value || null })} placeholder="jane@acme.com" />
          </div>
          <div className="form-field full">
            <label>Notes</label>
            <textarea value={input.body} onChange={(e) => setInput({ ...input, body: e.target.value })} rows={3} required />
          </div>
          <div className="form-field full">
            <button className="btn btn-primary" type="submit" disabled={log.isPending}>
              {log.isPending ? "Logging..." : "Log interaction"}
            </button>
          </div>
        </form>
      )}

      {activities.isLoading && <p className="empty-state">Loading...</p>}
      {!activities.isLoading && (activities.data ?? []).length === 0 && <p className="empty-state">No interactions logged yet.</p>}
      {(activities.data ?? []).length > 0 && (
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {(activities.data ?? []).map((a) => (
            <li key={a.id} style={{ padding: "8px 0", borderBottom: "1px solid var(--border, #eee)", fontSize: 13 }}>
              <div style={{ display: "flex", gap: 8, alignItems: "baseline", flexWrap: "wrap" }}>
                <span className="badge">{a.channel}</span>
                {a.direction && <span style={{ color: "var(--text-muted)", fontSize: 11 }}>{a.direction}</span>}
                {a.subject && <strong>{a.subject}</strong>}
              </div>
              <div style={{ marginTop: 4 }}>{a.body}</div>
              <div style={{ color: "var(--text-muted)", fontSize: 11, marginTop: 4 }}>
                {a.participants && <>{a.participants} · </>}
                {new Date(a.occurred_at).toLocaleString()} · logged by {actorName(a.created_by, users.data)}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
