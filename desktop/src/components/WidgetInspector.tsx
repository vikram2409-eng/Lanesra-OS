import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  TASK_QUEUE_MODES,
  entityTypeLabel,
  type CustomFieldEntityType,
  type CustomReport,
  type DashboardWidget,
  type RecordListMode,
  type TaskQueueMode,
} from "../lib/types";
import { DUE_SOON_ENTITY_TYPES, TASK_QUEUE_MODE_LABELS, widgetLabel } from "../features/dashboard/widgetMeta";
import { kpiLabel } from "../features/dashboard/kpis";

/**
 * Runtime UX Modernization (issue #198): the right-side panel for the
 * dashboard-layout editor's canvas - edits the selected widget's config
 * in place (entity type/mode/limit/saved view/agent, depending on kind),
 * replacing the old badge-chip row's only affordance (remove + reorder).
 * Position/size are edited on the canvas itself (drag/resize), not here.
 *
 * A `kpi`/`chart` widget's data source isn't editable here - both were
 * already fully specified at creation time (the KPI key picked from the
 * fixed catalog, the report picked from Admin -> Reports) and re-pointing
 * either is indistinguishable from removing this widget and adding a new
 * one, so the Inspector just shows what's already configured.
 */
export function WidgetInspector({
  widget,
  reports,
  onConfigChange,
  onRemove,
  onClose,
}: {
  widget: DashboardWidget;
  reports: CustomReport[];
  onConfigChange: (config: Record<string, unknown>) => void;
  onRemove: () => void;
  onClose: () => void;
}) {
  return (
    <div className="card" style={{ width: 280, flexShrink: 0 }}>
      <div className="toolbar">
        <h3 style={{ margin: 0, fontSize: 15 }}>Widget</h3>
        <button className="link-button" onClick={onClose} title="Close">
          ×
        </button>
      </div>
      <p style={{ fontSize: 13, color: "var(--text-muted)", marginTop: 0 }}>{widgetLabel(widget, reports)}</p>

      <WidgetConfigForm widget={widget} onConfigChange={onConfigChange} />

      <button className="btn btn-danger" style={{ marginTop: 16, width: "100%" }} onClick={onRemove}>
        Remove widget
      </button>
    </div>
  );
}

function WidgetConfigForm({
  widget,
  onConfigChange,
}: {
  widget: DashboardWidget;
  onConfigChange: (config: Record<string, unknown>) => void;
}) {
  if (widget.kind === "kpi") {
    return <p style={{ fontSize: 13 }}>KPI tile: {kpiLabel(widget.config.kpi_key as string)}</p>;
  }

  if (widget.kind === "chart") {
    return <p style={{ fontSize: 13 }}>Runs a saved Custom Report - edit it from Admin → Reports.</p>;
  }

  if (widget.kind === "record_list" || widget.kind === "table") {
    const entityType = widget.config.entity_type as CustomFieldEntityType;
    const mode = widget.config.mode as RecordListMode;
    const limit = (widget.config.limit as number) ?? 5;
    const savedViewId = (widget.config.saved_view_id as string | undefined) ?? "";
    const dueSoonAvailable = DUE_SOON_ENTITY_TYPES.includes(entityType);
    const views = useQuery({ queryKey: ["savedViewsForWidget", entityType], queryFn: () => api.listSavedViews(entityType) });
    return (
      <div className="form-grid">
        <div className="form-field">
          <label>Object</label>
          <select
            value={entityType}
            onChange={(e) => {
              const next = e.target.value as CustomFieldEntityType;
              onConfigChange({
                ...widget.config,
                entity_type: next,
                mode: DUE_SOON_ENTITY_TYPES.includes(next) ? mode : "recent",
                saved_view_id: undefined,
              });
            }}
          >
            {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
              <option key={t} value={t}>
                {entityTypeLabel(t)}
              </option>
            ))}
          </select>
        </div>
        {dueSoonAvailable && (
          <div className="form-field">
            <label>Mode</label>
            <select value={mode} onChange={(e) => onConfigChange({ ...widget.config, mode: e.target.value })}>
              <option value="due_soon">Due soon</option>
              <option value="recent">Recently created</option>
            </select>
          </div>
        )}
        {views.data && views.data.length > 0 && (
          <div className="form-field">
            <label title="Narrows this widget's rows to one saved view's filters">Saved view (optional)</label>
            <select
              value={savedViewId}
              onChange={(e) => onConfigChange({ ...widget.config, saved_view_id: e.target.value || undefined })}
            >
              <option value="">All records</option>
              {views.data.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.name}
                </option>
              ))}
            </select>
          </div>
        )}
        <div className="form-field">
          <label>Row limit</label>
          <input
            type="number"
            min={1}
            max={20}
            value={limit}
            onChange={(e) => onConfigChange({ ...widget.config, limit: Number(e.target.value) || 5 })}
          />
        </div>
      </div>
    );
  }

  if (widget.kind === "saved_view") {
    const entityType = widget.config.entity_type as CustomFieldEntityType;
    const savedViewId = (widget.config.saved_view_id as string | undefined) ?? "";
    const limit = (widget.config.limit as number) ?? 5;
    const views = useQuery({ queryKey: ["savedViewsForWidget", entityType], queryFn: () => api.listSavedViews(entityType) });
    return (
      <div className="form-grid">
        <div className="form-field">
          <label>Object</label>
          <select
            value={entityType}
            onChange={(e) => onConfigChange({ ...widget.config, entity_type: e.target.value, saved_view_id: "" })}
          >
            {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
              <option key={t} value={t}>
                {entityTypeLabel(t)}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Saved view</label>
          <select value={savedViewId} onChange={(e) => onConfigChange({ ...widget.config, saved_view_id: e.target.value })}>
            <option value="">Choose...</option>
            {(views.data ?? []).map((v) => (
              <option key={v.id} value={v.id}>
                {v.name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Row limit</label>
          <input
            type="number"
            min={1}
            max={20}
            value={limit}
            onChange={(e) => onConfigChange({ ...widget.config, limit: Number(e.target.value) || 5 })}
          />
        </div>
      </div>
    );
  }

  if (widget.kind === "task_queue") {
    const mode = widget.config.mode as TaskQueueMode;
    const limit = (widget.config.limit as number) ?? 5;
    return (
      <div className="form-grid">
        <div className="form-field">
          <label>Filter</label>
          <select value={mode} onChange={(e) => onConfigChange({ ...widget.config, mode: e.target.value })}>
            {TASK_QUEUE_MODES.map((m) => (
              <option key={m} value={m}>
                {TASK_QUEUE_MODE_LABELS[m]}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Row limit</label>
          <input
            type="number"
            min={1}
            max={20}
            value={limit}
            onChange={(e) => onConfigChange({ ...widget.config, limit: Number(e.target.value) || 5 })}
          />
        </div>
      </div>
    );
  }

  if (widget.kind === "agent_insight") {
    const agentId = widget.config.agent_id as string;
    const agents = useQuery({ queryKey: ["aiAgentsForWidget"], queryFn: () => api.listAiAgents(true) });
    return (
      <div className="form-grid">
        <div className="form-field">
          <label>Agent</label>
          <select value={agentId} onChange={(e) => onConfigChange({ ...widget.config, agent_id: e.target.value })}>
            {(agents.data ?? []).map((a) => (
              <option key={a.id} value={a.id}>
                {a.name}
              </option>
            ))}
          </select>
        </div>
      </div>
    );
  }

  return null;
}
