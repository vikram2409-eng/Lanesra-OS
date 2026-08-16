import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import type { User } from "../lib/types";

/** Shared by both exports below - resolves a `created_by`/`updated_by`/
 * `AuditEvent.user_id` value to a display name, the same
 * `users.data?.find(...)` pattern every other owner/assignee lookup in
 * this codebase already uses. `null` (no authenticated actor on that
 * write - a system/scheduled job) reads as "System", never as an error. */
function actorName(userId: string | null | undefined, users: User[] | undefined): string {
  if (!userId) return "System";
  return users?.find((u) => u.id === userId)?.display_name ?? "Unknown user";
}

function useUsers() {
  return useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
}

/**
 * One-line "Created by X on ... · Last updated by Y on ..." byline for a
 * record's own created_at/created_by/updated_at/updated_by columns -
 * every entity in the schema carries these (see migration 0026's header
 * comment for the two that needed a schema change to get there). Omits
 * the "by <name>" clause instead of showing "by System" when there's no
 * authenticated actor, since most records were touched by an admin at
 * setup time or a background job, not literally "the system".
 */
export function AuditByline({
  createdAt,
  createdBy,
  updatedAt,
  updatedBy,
}: {
  createdAt: string;
  createdBy: string | null;
  updatedAt: string;
  updatedBy: string | null;
}) {
  const users = useUsers();

  const created = new Date(createdAt).toLocaleString();
  const updated = new Date(updatedAt).toLocaleString();
  const createdLine = createdBy ? `Created by ${actorName(createdBy, users.data)} on ${created}` : `Created on ${created}`;
  // updated_at is set to created_at on insert (every repo's INSERT writes
  // both), so a record that's never been edited again has nothing new to
  // say here - only show the second clause once the two diverge.
  const showUpdated = updatedAt !== createdAt;
  const updatedLine = updatedBy ? `updated by ${actorName(updatedBy, users.data)} on ${updated}` : `updated on ${updated}`;

  return (
    <p style={{ color: "var(--text-muted)", fontSize: 12, margin: "2px 0 0" }}>
      {createdLine}
      {showUpdated && <> · Last {updatedLine}</>}
    </p>
  );
}

/**
 * Full history card for one record - every create/update/archive
 * `audit_repo::record` has logged against it (see that module's own
 * comment for which services call it). Same card/list styling as
 * `RelatedRecordsCard`/`NotificationBell` so it drops into any detail
 * page's tab grid without looking like a bolted-on afterthought.
 */
export function AuditTrail({ entityType, entityId }: { entityType: string; entityId: string }) {
  const users = useUsers();
  const events = useQuery({
    queryKey: ["auditEvents", entityType, entityId],
    queryFn: () => api.listAuditEvents(entityType, entityId),
  });

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>History</h3>
      {events.isLoading && <p className="empty-state">Loading...</p>}
      {!events.isLoading && (events.data ?? []).length === 0 && <p className="empty-state">No history recorded yet.</p>}
      {(events.data ?? []).length > 0 && (
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {(events.data ?? []).map((e) => (
            <li key={e.id} style={{ padding: "6px 0", borderBottom: "1px solid var(--border, #eee)", fontSize: 13 }}>
              <div>{e.summary}</div>
              <div style={{ color: "var(--text-muted)", fontSize: 11, marginTop: 2 }}>
                {actorName(e.user_id, users.data)} · {new Date(e.occurred_at).toLocaleString()}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
