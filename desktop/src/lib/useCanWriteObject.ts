import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

/** App Builder: whether the signed-in user can create/update/archive
 * `entityType` records right now - UI convenience only, so a Viewer-only
 * app grant can hide/disable Create/Edit/Delete controls instead of only
 * surfacing a rejected-command error after the fact. This is never itself
 * the security boundary (see `app_service::can_write_object`'s own doc
 * comment) - the command layer's own `require_object_write_access` check
 * is what actually blocks the write, so a stale or wrongly-permissive
 * answer here is a UX inconvenience, not a security hole.
 *
 * Defaults to `true` while the query is loading (or on any error) so the
 * overwhelming common case - an entity type App Builder has never scoped
 * at all - never flashes a disabled control while this resolves. A brief
 * window where a Viewer sees an enabled button just means their first
 * click gets the same rejected-command error banner it always would have. */
export function useCanWriteObject(entityType: string): boolean {
  const q = useQuery({
    queryKey: ["canWriteObject", entityType],
    queryFn: () => api.canWriteObject(entityType),
  });
  return q.data ?? true;
}
