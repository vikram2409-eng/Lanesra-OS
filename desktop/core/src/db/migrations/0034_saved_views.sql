-- Saved Views & Bulk Actions (product backlog): global search and
-- per-field list filtering shipped without a way to save a filter/sort/
-- column/grouping combination as a named view - this is that.
--
-- Deliberately a persisted *preference* blob, not a new query engine: the
-- filters column stores exactly the shape the existing custom-field list
-- filtering already produces client-side (a flat {custom_field_key:
-- value} map - see lib/useCustomFieldFilters.ts on the frontend), so a
-- saved view is "remember what I already had set," not a broader
-- capability the rest of the list UI doesn't have. Sort/columns/group are
-- likewise just an ordered list of field keys the frontend already knows
-- how to render for that object_key - nothing here needs to understand
-- field types or evaluate anything server-side.
--
-- object_key matches api_object_service's own vocabulary (a built-in
-- entity name like "Company", or a Custom Object's key) - saved views
-- apply to anything a list screen can show, a broader surface than Bulk
-- Actions' write-capable subset (see bulk_action_service's own doc
-- comment for why those two scopes differ).
CREATE TABLE saved_views (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    object_key TEXT NOT NULL,
    name TEXT NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    -- 'private' (only the owner sees/uses it) or 'shared' (every user in
    -- the workspace can use it, but only the owner or an admin can edit
    -- or delete it - enforced in saved_view_service, not by a DB
    -- constraint, matching how every other admin-authored resource in
    -- this codebase draws that line).
    visibility TEXT NOT NULL DEFAULT 'private',
    filters TEXT NOT NULL DEFAULT '{}',
    sort_field TEXT,
    sort_direction TEXT NOT NULL DEFAULT 'asc',
    -- JSON array of field keys, in display order; NULL means "whatever
    -- this list screen's own default columns are" rather than an empty
    -- table.
    columns TEXT,
    group_by_field TEXT,
    -- At most one row per (workspace_id, object_key) may have this set -
    -- enforced in saved_view_service::set_default (unset the prior
    -- default, then set the new one), not a partial unique index, so the
    -- same "read the invariant from the service, not the schema"
    -- convention applies as everywhere else needing an app-level rule a
    -- plain column can't express alone.
    is_object_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_saved_views_workspace_object ON saved_views(workspace_id, object_key);
CREATE INDEX idx_saved_views_owner ON saved_views(owner_user_id);
