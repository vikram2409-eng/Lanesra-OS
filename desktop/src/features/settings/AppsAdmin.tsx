import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  APP_PERMISSION_LEVELS,
  CUSTOM_FIELD_ENTITY_TYPES,
  ROLES,
  entityTypeLabel,
  type AppDefinition,
  type AppDefinitionUpdate,
  type AppPermission,
  type AppPermissionLevel,
  type CustomObjectDefinition,
  type DashboardLayout,
  type User,
} from "../../lib/types";

const APP_ICON_CHOICES = ["⬡", "🏠", "👥", "💼", "🔧", "📦", "🏗️", "📋", "🚗", "🏭"];

/**
 * App Builder Phase 1: group a set of already-existing objects, their
 * screens and a dashboard into one named, publishable application -
 * Property Management, Recruitment, whatever an organization actually
 * runs on. Every primitive an app assembles (Custom Objects, Screen/App
 * Builder layouts, Dashboards) already ships and works elsewhere in
 * Admin; this screen is purely the packaging layer: which objects belong
 * together, which dashboard represents this app, and who can see it.
 *
 * Access is a genuinely new model, not the same role-checkbox pattern
 * Screen layouts and Dashboards use: a grant here is either to a role or
 * to one specific person, and with zero grants an app is invisible to
 * everyone but Administrators - see `AppPermissionsPanel` below and the
 * Rust core's `app_service` doc comment for what "Editor" does and
 * doesn't enforce yet (today: it's a stored intent the frontend can read,
 * not yet a server-side gate on every command).
 */
export function AppsAdmin() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const queryClient = useQueryClient();

  const apps = useQuery({ queryKey: ["apps"], queryFn: () => api.listApps() });
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const dashboards = useQuery({ queryKey: ["dashboardLayouts"], queryFn: () => api.listDashboardLayouts() });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["apps"] });
    // AppShell's App Switcher runs its own `list_accessible_apps` query
    // (keyed "accessibleApps") so it can be self-contained rather than
    // threaded down as a prop - it needs invalidating here too, or a
    // just-created/published/edited/deleted app won't show up (or update)
    // in the sidebar switcher until something else happens to refetch it.
    queryClient.invalidateQueries({ queryKey: ["accessibleApps"] });
  }

  const list = apps.data ?? [];
  const selected = list.find((a) => a.id === selectedId) ?? list[0] ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Apps</h3>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Group a set of objects, their screens and a dashboard into one named, publishable application, with its own
        icon and access grants. Every primitive an app assembles already exists elsewhere in Admin - this is the
        packaging layer on top.
      </p>

      {apps.isLoading && <p>Loading...</p>}

      {list.length > 0 && (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", margin: "12px 0" }}>
          {list.map((a) => (
            <button
              key={a.id}
              className={`tab${selected?.id === a.id ? " active" : ""}`}
              onClick={() => {
                setSelectedId(a.id);
                setCreating(false);
              }}
            >
              {a.icon} {a.name}
              {!a.is_published && " · Draft"}
            </button>
          ))}
          <button className="btn" onClick={() => setCreating((v) => !v)}>
            + New app
          </button>
        </div>
      )}

      {list.length === 0 && !apps.isLoading && !creating && (
        <div style={{ margin: "12px 0" }}>
          <p className="empty-state">No apps yet.</p>
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            + New app
          </button>
        </div>
      )}

      {creating && (
        <NewAppForm
          onDone={(created) => {
            invalidate();
            setCreating(false);
            setSelectedId(created.id);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {selected && !creating && (
        <AppEditor
          key={selected.id}
          app={selected}
          appCount={list.length}
          customObjects={customObjects.data ?? []}
          dashboards={dashboards.data ?? []}
          users={users.data ?? []}
          onChanged={invalidate}
          onDeleted={() => {
            invalidate();
            setSelectedId(null);
          }}
        />
      )}
    </div>
  );
}

function NewAppForm({ onDone, onCancel }: { onDone: (created: AppDefinition) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [icon, setIcon] = useState(APP_ICON_CHOICES[0]);
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createApp({ name, icon, description: null }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this app"),
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
        <div className="form-field full">
          <label>App name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Property Management" required autoFocus />
        </div>
        <div className="form-field">
          <label>Icon</label>
          <select value={icon} onChange={(e) => setIcon(e.target.value)}>
            {APP_ICON_CHOICES.map((i) => (
              <option key={i} value={i}>
                {i}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create app
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function AppEditor({
  app,
  appCount,
  customObjects,
  dashboards,
  users,
  onChanged,
  onDeleted,
}: {
  app: AppDefinition;
  appCount: number;
  customObjects: CustomObjectDefinition[];
  dashboards: DashboardLayout[];
  users: User[];
  onChanged: () => void;
  onDeleted: () => void;
}) {
  const [name, setName] = useState(app.name);
  const [icon, setIcon] = useState(app.icon);
  const [description, setDescription] = useState(app.description ?? "");
  const [objectKeys, setObjectKeys] = useState<string[]>(app.object_keys);
  const [dashboardId, setDashboardId] = useState<string | null>(app.dashboard_id);
  const [error, setError] = useState<string | null>(null);

  const update = useMutation({
    mutationFn: (next: AppDefinitionUpdate) => api.updateApp(app.id, next),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this app"),
  });

  function save(patch: Partial<AppDefinitionUpdate>) {
    update.mutate({
      name,
      icon,
      description: description.trim() ? description.trim() : null,
      object_keys: objectKeys,
      dashboard_id: dashboardId,
      ...patch,
    });
  }

  const publish = useMutation({
    mutationFn: () => api.publishApp(app.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not publish this app"),
  });

  const unpublish = useMutation({
    mutationFn: () => api.unpublishApp(app.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not unpublish this app"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteApp(app.id),
    onSuccess: onDeleted,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this app"),
  });

  function toggleObject(key: string) {
    const next = objectKeys.includes(key) ? objectKeys.filter((k) => k !== key) : [...objectKeys, key];
    setObjectKeys(next);
    save({ object_keys: next });
  }

  const objectChoices = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((k) => ({ key: k as string, label: entityTypeLabel(k) })),
    ...customObjects.map((o) => ({ key: o.key, label: o.plural_label })),
  ];

  return (
    <div>
      {error && <div className="error-banner">{error}</div>}

      <div className="card" style={{ background: "var(--surface-2, transparent)", marginBottom: 16 }}>
        <div className="form-grid">
          <div className="form-field">
            <label>App name</label>
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={() => {
                if (name.trim() && name !== app.name) save({ name: name.trim() });
              }}
            />
          </div>
          <div className="form-field">
            <label>Icon</label>
            <select
              value={icon}
              onChange={(e) => {
                setIcon(e.target.value);
                save({ icon: e.target.value });
              }}
            >
              {APP_ICON_CHOICES.map((i) => (
                <option key={i} value={i}>
                  {i}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field full">
            <label>Description (optional)</label>
            <input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              onBlur={() => save({ description: description.trim() ? description.trim() : null })}
              placeholder="What this app is for"
            />
          </div>
          <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            <span className={`badge${app.is_published ? " badge-success" : ""}`}>{app.is_published ? "Published" : "Draft"}</span>
            <div style={{ flex: 1 }} />
            {app.is_published ? (
              <button className="btn" onClick={() => unpublish.mutate()} disabled={unpublish.isPending}>
                Unpublish
              </button>
            ) : (
              <button className="btn btn-primary" onClick={() => publish.mutate()} disabled={publish.isPending}>
                Publish
              </button>
            )}
            <button
              className="btn btn-danger"
              onClick={() => {
                if (confirm(`Delete the "${app.name}" app? This can't be undone.`)) remove.mutate();
              }}
              disabled={remove.isPending}
              title={appCount <= 1 ? undefined : undefined}
            >
              Delete app
            </button>
          </div>
        </div>
      </div>

      <div className="card" style={{ background: "var(--surface-2, transparent)", marginBottom: 16 }}>
        <div style={{ fontWeight: 600, marginBottom: 8 }}>Objects in this app</div>
        <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 0 }}>
          {objectKeys.length === 0
            ? "Pick at least one object before publishing."
            : `${objectKeys.length} object${objectKeys.length === 1 ? "" : "s"} selected.`}
        </p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
          {objectChoices.map((c) => (
            <label key={c.key} style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 13 }}>
              <input type="checkbox" checked={objectKeys.includes(c.key)} onChange={() => toggleObject(c.key)} />
              {c.label}
            </label>
          ))}
        </div>
        <div className="form-field" style={{ marginTop: 14, maxWidth: 320 }}>
          <label>Dashboard for this app (optional)</label>
          <select
            value={dashboardId ?? ""}
            onChange={(e) => {
              const v = e.target.value || null;
              setDashboardId(v);
              save({ dashboard_id: v });
            }}
          >
            <option value="">No dashboard</option>
            {dashboards.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
                {d.is_default ? " · Default" : ""}
              </option>
            ))}
          </select>
        </div>
      </div>

      <AppPermissionsPanel appId={app.id} users={users} />
    </div>
  );
}

function principalLabel(p: AppPermission, users: User[]): string {
  if (p.principal_type === "role") return p.principal_id;
  return users.find((u) => u.id === p.principal_id)?.display_name ?? "(user removed)";
}

function levelLabel(level: string): string {
  return level === "editor" ? "Editor" : "Viewer";
}

function AppPermissionsPanel({ appId, users }: { appId: string; users: User[] }) {
  const queryClient = useQueryClient();
  const permissions = useQuery({ queryKey: ["appPermissions", appId], queryFn: () => api.listAppPermissions(appId) });
  const [error, setError] = useState<string | null>(null);
  const [roleToGrant, setRoleToGrant] = useState("");
  const [roleLevel, setRoleLevel] = useState<AppPermissionLevel>("viewer");
  const [userToGrant, setUserToGrant] = useState("");
  const [userLevel, setUserLevel] = useState<AppPermissionLevel>("viewer");

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["appPermissions", appId] });
  }

  const grant = useMutation({
    mutationFn: (input: { principal_type: "role" | "user"; principal_id: string; level: AppPermissionLevel }) =>
      api.grantAppPermission(appId, input),
    onSuccess: () => {
      invalidate();
      setRoleToGrant("");
      setUserToGrant("");
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not grant access"),
  });

  const revoke = useMutation({
    mutationFn: (id: string) => api.revokeAppPermission(id),
    onSuccess: invalidate,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not remove this grant"),
  });

  const list = permissions.data ?? [];
  const grantedRoles = new Set(list.filter((p) => p.principal_type === "role").map((p) => p.principal_id));
  const grantedUserIds = new Set(list.filter((p) => p.principal_type === "user").map((p) => p.principal_id));
  const availableRoles = ROLES.filter((r) => !grantedRoles.has(r));
  const availableUsers = users.filter((u) => !grantedUserIds.has(u.id));

  return (
    <div className="card" style={{ background: "var(--surface-2, transparent)" }}>
      <div style={{ fontWeight: 600, marginBottom: 8 }}>Access</div>
      <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 0 }}>
        Administrators always see every published app. Everyone else needs a grant here - to a role, or to one
        specific person - before this app appears for them at all.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {permissions.isLoading && <p>Loading...</p>}
      {list.length === 0 && !permissions.isLoading && (
        <p className="empty-state">No grants yet - this app is invisible to everyone but Administrators.</p>
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: 6, margin: "8px 0" }}>
        {list.map((p) => (
          <span
            key={p.id}
            className="badge"
            style={{ display: "inline-flex", alignItems: "center", gap: 8, justifyContent: "space-between", width: "fit-content" }}
          >
            {p.principal_type === "role" ? "Role" : "Person"}: {principalLabel(p, users)} — {levelLabel(p.level)}
            <button className="link-button" onClick={() => revoke.mutate(p.id)} title="Remove this grant">
              ×
            </button>
          </span>
        ))}
      </div>
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginTop: 8 }}>
        {availableRoles.length > 0 && (
          <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <select value={roleToGrant} onChange={(e) => setRoleToGrant(e.target.value)}>
              <option value="">Choose a role...</option>
              {availableRoles.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
            <select value={roleLevel} onChange={(e) => setRoleLevel(e.target.value as AppPermissionLevel)}>
              {APP_PERMISSION_LEVELS.map((l) => (
                <option key={l} value={l}>
                  {levelLabel(l)}
                </option>
              ))}
            </select>
            <button
              className="btn btn-secondary"
              onClick={() => roleToGrant && grant.mutate({ principal_type: "role", principal_id: roleToGrant, level: roleLevel })}
              disabled={grant.isPending || !roleToGrant}
            >
              + Grant role
            </button>
          </div>
        )}
        {availableUsers.length > 0 && (
          <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <select value={userToGrant} onChange={(e) => setUserToGrant(e.target.value)}>
              <option value="">Choose a person...</option>
              {availableUsers.map((u) => (
                <option key={u.id} value={u.id}>
                  {u.display_name}
                </option>
              ))}
            </select>
            <select value={userLevel} onChange={(e) => setUserLevel(e.target.value as AppPermissionLevel)}>
              {APP_PERMISSION_LEVELS.map((l) => (
                <option key={l} value={l}>
                  {levelLabel(l)}
                </option>
              ))}
            </select>
            <button
              className="btn btn-secondary"
              onClick={() => userToGrant && grant.mutate({ principal_type: "user", principal_id: userToGrant, level: userLevel })}
              disabled={grant.isPending || !userToGrant}
            >
              + Grant person
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
