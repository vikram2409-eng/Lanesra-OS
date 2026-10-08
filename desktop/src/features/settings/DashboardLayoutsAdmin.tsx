import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { AppScopeFilter, AppScopeSelect, matchesAppFilter, useApps } from "../../components/AppScope";
import { DashboardGrid } from "../../components/DashboardGrid";
import { WidgetInspector } from "../../components/WidgetInspector";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  ROLES,
  TASK_QUEUE_MODES,
  entityTypeLabel,
  type AppDefinition,
  type CustomFieldEntityType,
  type CustomReport,
  type DashboardLayout,
  type DashboardSummary,
  type DashboardWidgets,
  type RecordListMode,
  type TaskQueueMode,
  type WidgetLayout,
} from "../../lib/types";
import { DashboardWidgetCard } from "../dashboard/DashboardWidgetCard";
import { KPI_DEFS, kpiLabel } from "../dashboard/kpis";
import { DUE_SOON_ENTITY_TYPES, TASK_QUEUE_MODE_LABELS, widgetLabel } from "../dashboard/widgetMeta";

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
 * Phase 1 shipped one widget kind - KPI tiles, the same catalog
 * `Dashboard.tsx`'s pre-existing (workspace-wide) KPI picker already
 * drew from (see `kpis.tsx`) - now placed per dashboard layout instead.
 * Phase 2 adds chart widgets: pick an existing saved Custom Report (see
 * Admin -> Reports) and it's rendered as a bar chart on the dashboard,
 * reusing the same report engine rather than a second one just for
 * dashboards. Phase 3 adds record-list widgets: pick an entity type and
 * a mode (recently created, or - for Tasks and Invoices specifically -
 * soonest due) and it renders as a short list of records, same
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
  const [appFilter, setAppFilter] = useState<"all" | "none" | string>("all");
  const queryClient = useQueryClient();

  const apps = useApps();
  const appList = apps.data ?? [];

  const layouts = useQuery({ queryKey: ["dashboardLayouts"], queryFn: () => api.listDashboardLayouts() });
  // Chart widgets reference a saved Custom Report by id - fetched once
  // here and threaded down, rather than re-fetched per widget.
  const reports = useQuery({ queryKey: ["customReports"], queryFn: () => api.listCustomReports() });
  const reportList = reports.data ?? [];
  // KPI widgets on the edit canvas render real values (the same live
  // preview the grid gives every other widget kind), rather than a mock
  // "—" placeholder - one more cheap query, not a second data path.
  const summary = useQuery({ queryKey: ["dashboardSummaryForAdmin"], queryFn: () => api.dashboardSummary() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["dashboardLayouts"] });
    queryClient.invalidateQueries({ queryKey: ["effectiveDashboardLayout"] });
  }

  const list = layouts.data ?? [];
  const visibleList = list.filter((l) => matchesAppFilter(l.app_id, appFilter));
  const selected = visibleList.find((l) => l.id === selectedId) ?? visibleList.find((l) => l.is_default) ?? visibleList[0] ?? null;
  const newLayoutAppId = appFilter !== "all" && appFilter !== "none" ? appFilter : null;

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

      <AppScopeFilter apps={appList} value={appFilter} onChange={setAppFilter} />

      {visibleList.length > 0 && (
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", margin: "12px 0" }}>
          {visibleList.map((l) => (
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
      {visibleList.length === 0 && list.length > 0 && (
        <div style={{ display: "flex", gap: 8, alignItems: "center", margin: "12px 0" }}>
          <p className="empty-state" style={{ margin: 0 }}>No dashboards match this app filter.</p>
          <button className="btn" onClick={() => setCreating((v) => !v)}>
            + New dashboard
          </button>
        </div>
      )}

      {creating && (
        <NewLayoutForm
          apps={appList}
          defaultAppId={newLayoutAppId}
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
          reports={reportList}
          apps={appList}
          summary={summary.data ?? null}
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

function NewLayoutForm({
  apps,
  defaultAppId,
  onDone,
  onCancel,
}: {
  apps: AppDefinition[];
  defaultAppId: string | null;
  onDone: (created: DashboardLayout) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState("");
  const [appId, setAppId] = useState<string | null>(defaultAppId);
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createDashboardLayout({ name, initial_kpi_keys: KPI_DEFS.map((k) => k.key), app_id: appId }),
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
        <div className="form-field">
          <AppScopeSelect apps={apps} value={appId} onChange={setAppId} />
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
  reports,
  apps,
  summary,
  onChanged,
  onDeleted,
}: {
  layout: DashboardLayout;
  layoutCount: number;
  reports: CustomReport[];
  apps: AppDefinition[];
  summary: DashboardSummary | null;
  onChanged: () => void;
  onDeleted: () => void;
}) {
  const [name, setName] = useState(layout.name);
  const [roles, setRoles] = useState<string[]>(layout.roles);
  const [widgets, setWidgets] = useState<DashboardWidgets>(layout.draft);
  const [appId, setAppId] = useState<string | null>(layout.app_id);
  const [error, setError] = useState<string | null>(null);
  const [selectedWidgetId, setSelectedWidgetId] = useState<string | null>(null);

  // Every structural edit (add/remove/reorder a widget) saves immediately
  // - same reasoning as ScreenLayoutsAdmin's identical choice.
  const update = useMutation({
    mutationFn: (next: { name: string; roles: string[]; draft: DashboardWidgets; app_id: string | null }) => api.updateDashboardLayout(layout.id, next),
    onSuccess: onChanged,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this dashboard"),
  });

  function save(nextWidgets: DashboardWidgets, nextName = name, nextRoles = roles, nextAppId = appId) {
    setWidgets(nextWidgets);
    update.mutate({ name: nextName, roles: nextRoles, draft: nextWidgets, app_id: nextAppId });
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
    save({ widgets: [...widgets.widgets, { id: newId(), kind: "kpi", config: { kpi_key: key }, layout: null }] });
  }

  function addChart(reportId: string) {
    save({ widgets: [...widgets.widgets, { id: newId(), kind: "chart", config: { report_id: reportId }, layout: null }] });
  }

  function addRecordList(entityType: string, mode: RecordListMode) {
    save({
      widgets: [...widgets.widgets, { id: newId(), kind: "record_list", config: { entity_type: entityType, mode, limit: 5 }, layout: null }],
    });
  }

  function addTable(entityType: string, mode: RecordListMode) {
    save({
      widgets: [...widgets.widgets, { id: newId(), kind: "table", config: { entity_type: entityType, mode, limit: 8 }, layout: null }],
    });
  }

  function addTaskQueue(mode: TaskQueueMode) {
    save({ widgets: [...widgets.widgets, { id: newId(), kind: "task_queue", config: { mode, limit: 5 }, layout: null }] });
  }

  function addSavedView(entityType: string, savedViewId: string) {
    save({
      widgets: [
        ...widgets.widgets,
        { id: newId(), kind: "saved_view", config: { entity_type: entityType, saved_view_id: savedViewId, limit: 5 }, layout: null },
      ],
    });
  }

  function addAgentInsight(agentId: string) {
    save({ widgets: [...widgets.widgets, { id: newId(), kind: "agent_insight", config: { agent_id: agentId }, layout: null }] });
  }

  function removeWidget(id: string) {
    save({ widgets: widgets.widgets.filter((w) => w.id !== id) });
  }

  function updateWidgetLayout(id: string, layout: WidgetLayout) {
    save({ widgets: widgets.widgets.map((w) => (w.id === id ? { ...w, layout } : w)) });
  }

  function updateWidgetConfig(id: string, config: Record<string, unknown>) {
    save({ widgets: widgets.widgets.map((w) => (w.id === id ? { ...w, config } : w)) });
  }

  const usedKpiKeys = new Set(widgets.widgets.filter((w) => w.kind === "kpi").map((w) => w.config.kpi_key as string));
  const availableKpis = KPI_DEFS.filter((k) => !usedKpiKeys.has(k.key));

  const usedReportIds = new Set(widgets.widgets.filter((w) => w.kind === "chart").map((w) => w.config.report_id as string));
  const availableReports = reports.filter((r) => !usedReportIds.has(r.id));

  const hasPublished = layout.published !== null;
  const draftPublishedMatch = hasPublished && JSON.stringify(widgets) === JSON.stringify(layout.published);
  const selectedWidget = widgets.widgets.find((w) => w.id === selectedWidgetId) ?? null;

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
          <div className="form-field">
            <AppScopeSelect
              apps={apps}
              value={appId}
              onChange={(next) => {
                setAppId(next);
                save(widgets, name, roles, next);
              }}
            />
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
        <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 0 }}>
          Drag a widget's header to move it, its bottom-right corner to resize it, or click it to edit its data source.
        </p>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 16 }}>
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
          {reports.length === 0 ? (
            <span style={{ color: "var(--text-muted)", fontSize: 12, alignSelf: "center" }}>
              No custom reports yet - build one in Admin → Reports to add it as a chart here.
            </span>
          ) : (
            availableReports.length > 0 && (
              <select
                value=""
                onChange={(e) => {
                  if (e.target.value) addChart(e.target.value);
                }}
              >
                <option value="">+ Add chart...</option>
                {availableReports.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
              </select>
            )
          )}
          <AddRecordListWidget onAdd={addRecordList} />
          <AddTableWidget onAdd={addTable} />
          <AddTaskQueueWidget onAdd={addTaskQueue} />
          <AddSavedViewWidget onAdd={addSavedView} />
          <AddAgentInsightWidget onAdd={addAgentInsight} />
        </div>

        {widgets.widgets.length === 0 ? (
          <span className="empty-state">No widgets yet - add one above.</span>
        ) : (
          <div style={{ display: "flex", gap: 16, alignItems: "flex-start" }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <DashboardGrid
                widgets={widgets.widgets}
                editable
                selectedId={selectedWidgetId}
                onSelect={setSelectedWidgetId}
                onLayoutChange={updateWidgetLayout}
                renderHeader={(w) => <span style={{ fontSize: 12 }}>{widgetLabel(w, reports)}</span>}
                renderWidget={(w) => (
                  <DashboardWidgetCard
                    widget={w}
                    summary={summary}
                    reports={reports}
                    interactive={false}
                    onNavigate={() => {}}
                    onOpenRecord={() => {}}
                  />
                )}
              />
            </div>
            {selectedWidget && (
              <WidgetInspector
                widget={selectedWidget}
                reports={reports}
                onConfigChange={(config) => updateWidgetConfig(selectedWidget.id, config)}
                onRemove={() => {
                  removeWidget(selectedWidget.id);
                  setSelectedWidgetId(null);
                }}
                onClose={() => setSelectedWidgetId(null)}
              />
            )}
          </div>
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

/** A small inline picker for a record-list widget - two dropdowns
 * (entity type, mode) rather than one, so it's its own component instead
 * of a single `<select>` like the KPI/chart pickers above. */
function AddRecordListWidget({ onAdd }: { onAdd: (entityType: string, mode: RecordListMode) => void }) {
  const [entityType, setEntityType] = useState<CustomFieldEntityType>("Task");
  const dueSoonAvailable = DUE_SOON_ENTITY_TYPES.includes(entityType);
  const [mode, setMode] = useState<RecordListMode>("due_soon");

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <select
        value={entityType}
        onChange={(e) => {
          const next = e.target.value as CustomFieldEntityType;
          setEntityType(next);
          if (!DUE_SOON_ENTITY_TYPES.includes(next)) setMode("recent");
        }}
      >
        {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
          <option key={t} value={t}>
            {entityTypeLabel(t)}
          </option>
        ))}
      </select>
      {dueSoonAvailable && (
        <select value={mode} onChange={(e) => setMode(e.target.value as RecordListMode)}>
          <option value="due_soon">Due soon</option>
          <option value="recent">Recently created</option>
        </select>
      )}
      <button className="btn" onClick={() => onAdd(entityType, dueSoonAvailable ? mode : "recent")}>
        + Add record list
      </button>
    </span>
  );
}

/** Same entity-type/mode choice as `AddRecordListWidget`, but produces a
 * `"table"` widget instead of a `"record_list"` one - the dashboard grid
 * (issue #198) renders a table widget as a full interactive `ListTable`
 * (sortable/resizable columns) rather than the compact bullet list a
 * record-list widget gets, for a dashboard tile that needs to show more
 * than a title/subtitle per row. */
function AddTableWidget({ onAdd }: { onAdd: (entityType: string, mode: RecordListMode) => void }) {
  const [entityType, setEntityType] = useState<CustomFieldEntityType>("Task");
  const dueSoonAvailable = DUE_SOON_ENTITY_TYPES.includes(entityType);
  const [mode, setMode] = useState<RecordListMode>("due_soon");

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <select
        value={entityType}
        onChange={(e) => {
          const next = e.target.value as CustomFieldEntityType;
          setEntityType(next);
          if (!DUE_SOON_ENTITY_TYPES.includes(next)) setMode("recent");
        }}
      >
        {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
          <option key={t} value={t}>
            {entityTypeLabel(t)}
          </option>
        ))}
      </select>
      {dueSoonAvailable && (
        <select value={mode} onChange={(e) => setMode(e.target.value as RecordListMode)}>
          <option value="due_soon">Due soon</option>
          <option value="recent">Recently created</option>
        </select>
      )}
      <button className="btn" onClick={() => onAdd(entityType, dueSoonAvailable ? mode : "recent")}>
        + Add table
      </button>
    </span>
  );
}

/** A queue of open Tasks filtered the same way Tasks.tsx's own tabs filter
 * them (see that file's `Tab` type), plus "mine" for a dashboard tile
 * personalized to the signed-in user. No new backend - resolved client-side
 * from the same `api.listTasks()` rows the Tasks screen already fetches. */
function AddTaskQueueWidget({ onAdd }: { onAdd: (mode: TaskQueueMode) => void }) {
  const [mode, setMode] = useState<TaskQueueMode>("today");

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <select value={mode} onChange={(e) => setMode(e.target.value as TaskQueueMode)}>
        {TASK_QUEUE_MODES.map((m) => (
          <option key={m} value={m}>
            {TASK_QUEUE_MODE_LABELS[m]}
          </option>
        ))}
      </select>
      <button className="btn" onClick={() => onAdd(mode)}>
        + Add task queue
      </button>
    </span>
  );
}

/** Embeds one specific Saved View (its exact filters/sort/columns, see
 * `useSavedViews`) as a dashboard tile - unlike a table/record-list widget's
 * ad hoc entity-type+mode choice, this mirrors a view a user already
 * curated on a list screen. */
function AddSavedViewWidget({ onAdd }: { onAdd: (entityType: string, savedViewId: string) => void }) {
  const [entityType, setEntityType] = useState<CustomFieldEntityType>("Task");
  const [savedViewId, setSavedViewId] = useState("");
  const views = useQuery({
    queryKey: ["savedViewsForWidget", entityType],
    queryFn: () => api.listSavedViews(entityType),
  });

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <select
        value={entityType}
        onChange={(e) => {
          setEntityType(e.target.value as CustomFieldEntityType);
          setSavedViewId("");
        }}
      >
        {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
          <option key={t} value={t}>
            {entityTypeLabel(t)}
          </option>
        ))}
      </select>
      <select value={savedViewId} onChange={(e) => setSavedViewId(e.target.value)} disabled={!views.data || views.data.length === 0}>
        <option value="">
          {views.data && views.data.length > 0 ? "Choose a saved view..." : "No saved views for this object"}
        </option>
        {(views.data ?? []).map((v) => (
          <option key={v.id} value={v.id}>
            {v.name}
          </option>
        ))}
      </select>
      <button className="btn" disabled={!savedViewId} onClick={() => onAdd(entityType, savedViewId)}>
        + Add saved view
      </button>
    </span>
  );
}

/** A small card about one active AI agent (usage, recent run outcomes) -
 * resolved client-side from `api.getAiAgentTokenUsage`/`api.listAiAgentRuns`,
 * both already exposed to the frontend, so this adds no new Rust surface. */
function AddAgentInsightWidget({ onAdd }: { onAdd: (agentId: string) => void }) {
  const [agentId, setAgentId] = useState("");
  const agents = useQuery({ queryKey: ["aiAgentsForWidget"], queryFn: () => api.listAiAgents(true) });

  return (
    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
      <select value={agentId} onChange={(e) => setAgentId(e.target.value)} disabled={!agents.data || agents.data.length === 0}>
        <option value="">{agents.data && agents.data.length > 0 ? "Choose an agent..." : "No active agents yet"}</option>
        {(agents.data ?? []).map((a) => (
          <option key={a.id} value={a.id}>
            {a.name}
          </option>
        ))}
      </select>
      <button className="btn" disabled={!agentId} onClick={() => onAdd(agentId)}>
        + Add agent insight
      </button>
    </span>
  );
}

