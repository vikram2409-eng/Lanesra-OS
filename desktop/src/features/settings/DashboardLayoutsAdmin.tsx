import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { ROLES, type DashboardLayout, type DashboardWidget, type DashboardWidgets } from "../../lib/types";
import { KPI_DEFS, kpiLabel } from "../dashboard/kpis";

function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `id-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

/**
 * Dashboard customization Phase 1: lets an Administrator build multiple
 * named dashboard layouts - each an ordered list of widgets - and assign
 * them by role, with a required Default fallback. Structurally the same
 * feature as Screen/App Builder (see `ScreenLayoutsAdmin`'s doc comment
 * for the shared draft/publish/role-resolution model this mirrors), just
 * at the workspace level: one dashboard per layout, not one per object,
 * so there's no entity-type tab row here.
 *
 * Phase 1 ships one widget kind - KPI tiles, the same catalog
 * `Dashboard.tsx`'s pre-existing (workspace-wide) KPI picker already
 * drew from (see `kpis.tsx`) - now placed per dashboard layout instead.
 * Chart and record-list widgets are follow-up phases, the same
 * incremental-capability rollout Screen/App Builder's own phases used.
 *
 * Every workspace always has at least one layout: the Default,
 * auto-created server-side (empty, unpublished) the first time this
 * screen (or resolve_effective_dashboard) looks at a workspace with none
 * yet - unpublished, it has zero effect on the live Dashboard, which
 * keeps rendering exactly as it did before this feature existed (driven
 * by the older workspace-wide `dashboard_kpi_prefs` selection) until an
 * admin actually builds and publishes a layout.
 */
export function DashboardLayoutsAdmin() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const layouts = useQuery({ queryKey: ["dashboardLayouts"], queryFn: () => api.listDashboardLayouts() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["dashboardLayouts"] });
    queryClient.invalidateQueries({ queryKey: ["effectiveDashboardLayout"] });
  }

  const list = layouts.data ?? [];
  const selected = list.find((l) => l.id === selectedId) ?? list.find((l) => l.is_default) ?? list[0] ?? null;
  const previewLayout = list.find((l) => l.id === previewId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Dashboards</h3>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Build named dashboard layouts - an ordered list of widgets - and assign them to roles. Anyone whose roles
        don't match a published layout sees the Default.
      </p>

      {layouts.isLoading && <p>Loading...</p>}

      {list.length > 0 && (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", margin: "12px 0" }}>
          {list.map((l) => (
            <button
              key={l.id}
              className={`tab${selected?.id === l.id ? " active" : ""}`}
              onClick={() => {
                setSelectedId(l.id);
                setCreating(false);
              }}
            >
              {l.name}
              {l.is_default ? " · Default" : ""}
            </button>
          ))}
          <button className="btn" onClick={() => setCreating((v) => !v)}>
            + New dashboard
          </button>
        </div>
      )}

      {creating && (
        <NewLayoutForm
          onDone={(created) => {
            invalidate();
            setCreating(false);
            setSelectedId(created.id);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {selected && !creating && (
        <LayoutEditor
          key={selected.id}
          layout={selected}
          layoutCount={list.length}
          onChanged={invalidate}
          onDeleted={() => {
            invalidate();
            setSelectedId(null);
          }}
          onPreview={() => setPreviewId(selected.id)}
        />
      )}

      {previewLayout && <LayoutPreviewModal layout={previewLayout} onClose={() => setPreviewId(null)} />}
    </div>
  );
}

function NewLayoutForm({ onDone, onCancel }: { onDone: (created: DashboardLayout) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createDashboardLayout({ name, initial_kpi_keys: KPI_DEFS.map((k) => k.key) }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this dashboard"),
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
          <label>Dashboard name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Sales dashboard" required autoFocus />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Create dashboard
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function LayoutEditor({
  layout,
  layoutCount,
  onChanged,
  onDeleted,
  onPreview,
}: {
  layout: DashboardLayout;
  layoutCount: number;
  onChanged: () => void;
  onDeleted: () => void;
  onPreview: () => void;
}) {
  const [name, setName] = useState(layout.name);
  const [roles, setRoles] = useState<string[]>(layout.roles);
  const [widgets, setWidgets] = useState<DashboardWidgets>(layout.draft);
  const [error, setError] = useState<string | null>(null);

  // Every structural edit (add/remove/reorder a widget) saves immediately
  // - same reasoning as ScreenLayoutsAdmin's identical choice.
  const update = useMutation({
    mutationFn: (next: { name: string; roles: string[]; draft: DashboardWidgets }) => api.updateDashboardLayout(layout.id, next),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this dashboard"),
  });

  function save(nextWidgets: DashboardWidgets, nextName = name, nextRoles = roles) {
    setWidgets(nextWidgets);
    update.mutate({ name: nextName, roles: nextRoles, draft: nextWidgets });
  }

  const makeDefault = useMutation({
    mutationFn: () => api.makeDashboardLayoutDefault(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not make this the default"),
  });

  const remove = useMutation({
    mutationFn: () => api.deleteDashboardLayout(layout.id),
    onSuccess: onDeleted,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not delete this dashboard"),
  });

  const publish = useMutation({
    mutationFn: () => api.publishDashboardLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not publish this dashboard"),
  });

  const unpublish = useMutation({
    mutationFn: () => api.unpublishDashboardLayout(layout.id),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not unpublish this dashboard"),
  });

  const revert = useMutation({
    mutationFn: () => api.revertDashboardLayoutDraft(layout.id),
    onSuccess: (updated) => {
      setWidgets(updated.draft);
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not revert this draft"),
  });

  function addKpi(key: string) {
    save({ widgets: [...widgets.widgets, { id: newId(), kind: "kpi", config: { kpi_key: key } }] });
  }

  function removeWidget(id: string) {
    save({ widgets: widgets.widgets.filter((w) => w.id !== id) });
  }

  function moveWidget(id: string, direction: -1 | 1) {
    const idx = widgets.widgets.findIndex((w) => w.id === id);
    const swapWith = idx + direction;
    if (idx < 0 || swapWith < 0 || swapWith >= widgets.widgets.length) return;
    const next = [...widgets.widgets];
    [next[idx], next[swapWith]] = [next[swapWith], next[idx]];
    save({ widgets: next });
  }

  const usedKpiKeys = new Set(widgets.widgets.filter((w) => w.kind === "kpi").map((w) => w.config.kpi_key as string));
  const availableKpis = KPI_DEFS.filter((k) => !usedKpiKeys.has(k.key));

  const hasPublished = layout.published !== null;
  const draftPublishedMatch = hasPublished && JSON.stringify(widgets) === JSON.stringify(layout.published);

  return (
    <div>
      {error && <div className="error-banner">{error}</div>}

      <div className="card" style={{ background: "var(--surface-2, transparent)", marginBottom: 16 }}>
        <div className="form-grid">
          <div className="form-field">
            <label>Dashboard name</label>
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={() => {
                if (name.trim() && name !== layout.name) save(widgets, name, roles);
              }}
            />
          </div>
          <div className="form-field">
            <label title="Whichever roles a signed-in user has, the first published dashboard that claims one of them wins - the Default is the fallback for everyone else.">
              Roles
            </label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
              {ROLES.map((role) => (
                <label key={role} style={{ display: "flex", gap: 6, alignItems: "center", fontSize: 13 }}>
                  <input
                    type="checkbox"
                    checked={roles.includes(role)}
                    onChange={(e) => {
                      const nextRoles = e.target.checked ? [...roles, role] : roles.filter((r) => r !== role);
                      setRoles(nextRoles);
                      save(widgets, name, nextRoles);
                    }}
                  />
                  {role}
                </label>
              ))}
            </div>
          </div>
          <div className="form-field full" style={{ flexDirection: "row", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            <span className={`badge${layout.is_default ? " badge-success" : ""}`}>
              {layout.is_default ? "Default dashboard" : "Not default"}
            </span>
            <span className={`badge${hasPublished ? " badge-success" : ""}`}>{hasPublished ? "Published" : "Never published"}</span>
            {hasPublished && !draftPublishedMatch && <span className="badge badge-warning">Unpublished changes</span>}
            <div style={{ flex: 1 }} />
            {!layout.is_default && (
              <button className="btn" onClick={() => makeDefault.mutate()} disabled={makeDefault.isPending}>
                Make default
              </button>
            )}
            <button className="btn" onClick={onPreview}>
              Preview
            </button>
            <button
              className="btn btn-danger"
              onClick={() => {
                if (confirm(`Delete dashboard '${layout.name}'?`)) remove.mutate();
              }}
              disabled={remove.isPending || layoutCount <= 1 || layout.is_default}
              title={
                layout.is_default
                  ? "The default dashboard can't be deleted"
                  : layoutCount <= 1
                    ? "A workspace needs at least one dashboard"
                    : undefined
              }
            >
              Delete dashboard
            </button>
          </div>
        </div>
      </div>

      <div className="card" style={{ background: "var(--surface-2, transparent)" }}>
        <div style={{ fontWeight: 600, marginBottom: 8 }}>Widgets</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 8, margin: "8px 0" }}>
          {widgets.widgets.length === 0 && <span className="empty-state">No widgets yet.</span>}
          {widgets.widgets.map((w, i) => (
            <span key={w.id} className="badge" style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
              {widgetLabel(w)}
              <button className="link-button" onClick={() => moveWidget(w.id, -1)} disabled={i === 0} title="Move earlier">
                ↑
              </button>
              <button
                className="link-button"
                onClick={() => moveWidget(w.id, 1)}
                disabled={i === widgets.widgets.length - 1}
                title="Move later"
              >
                ↓
              </button>
              <button className="link-button" onClick={() => removeWidget(w.id)} title="Remove from dashboard">
                ×
              </button>
            </span>
          ))}
        </div>
        {availableKpis.length > 0 && (
          <select
            value=""
            onChange={(e) => {
              if (e.target.value) addKpi(e.target.value);
            }}
          >
            <option value="">+ Add KPI tile...</option>
            {availableKpis.map((k) => (
              <option key={k.key} value={k.key}>
                {kpiLabel(k.key)}
              </option>
            ))}
          </select>
        )}
      </div>

      <div className="toolbar" style={{ marginTop: 16 }}>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" onClick={() => publish.mutate()} disabled={publish.isPending || draftPublishedMatch}>
            Publish
          </button>
          {hasPublished && (
            <button className="btn" onClick={() => unpublish.mutate()} disabled={unpublish.isPending}>
              Unpublish
            </button>
          )}
          {hasPublished && !draftPublishedMatch && (
            <button className="btn" onClick={() => revert.mutate()} disabled={revert.isPending}>
              Revert draft to published
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function widgetLabel(w: DashboardWidget): string {
  if (w.kind === "kpi") return kpiLabel(w.config.kpi_key as string);
  return w.kind;
}

function LayoutPreviewModal({ layout, onClose }: { layout: DashboardLayout; onClose: () => void }) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div className="card" style={{ width: 480, maxHeight: "80vh", overflowY: "auto" }} onClick={(e) => e.stopPropagation()}>
        <div className="toolbar">
          <h3 style={{ margin: 0 }}>Preview - {layout.name}</h3>
          <button className="btn" onClick={onClose}>
            Close
          </button>
        </div>
        <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 0 }}>
          Shows this dashboard's draft, as it will appear once published. Values are illustrative here.
        </p>
        {layout.draft.widgets.length === 0 && <span className="empty-state">No widgets on this dashboard yet.</span>}
        <div style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
          {layout.draft.widgets.map((w) => (
            <div key={w.id} className="kpi-tile" style={{ minWidth: 140 }}>
              <div className="value">—</div>
              <div className="label">{widgetLabel(w)}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
