import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { Bar } from "../../components/Bar";
import { ListTable, type ListTableColumn } from "../../components/ListTable";
import type { Section } from "../../components/AppShell";
import { sectionFor } from "../../components/GlobalSearch";
import { useCurrentUser } from "../../lib/useCurrentUser";
import {
  entityTypeLabel,
  type CustomReport,
  type DashboardSummary,
  type DashboardWidget,
  type RecordListMode,
  type RecordListRow,
  type Task,
  type TaskQueueMode,
} from "../../lib/types";
import { KPI_DEFS } from "./kpis";
import { TASK_QUEUE_MODE_LABELS } from "./widgetMeta";

/**
 * Runtime UX Modernization (issue #198): the one renderer every widget kind
 * goes through, used identically by the live Dashboard (view mode) and the
 * dashboard-layout editor's canvas (edit mode) - "one shared card system"
 * rather than Dashboard.tsx special-casing each kind in its own markup the
 * way it did before this grid existed. `interactive=false` (the editor)
 * suppresses row-click navigation, since a click there selects the widget
 * for the Inspector instead of leaving the admin screen.
 */
export function DashboardWidgetCard({
  widget,
  summary,
  reports,
  interactive = true,
  onNavigate,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  summary: DashboardSummary | null;
  reports: CustomReport[];
  interactive?: boolean;
  onNavigate: (section: Section) => void;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  switch (widget.kind) {
    case "kpi":
      return <KpiWidgetCard widget={widget} summary={summary} interactive={interactive} onNavigate={onNavigate} />;
    case "chart":
      return <ChartWidgetCard widget={widget} reports={reports} />;
    case "record_list":
      return <RecordListWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "table":
      return <TableWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "saved_view":
      return <SavedViewWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "task_queue":
      return <TaskQueueWidgetCard widget={widget} interactive={interactive} onNavigate={onNavigate} />;
    case "agent_insight":
      return <AgentInsightWidgetCard widget={widget} />;
    default:
      return (
        <div className="card" style={{ height: "100%", margin: 0 }}>
          <p className="empty-state">Unknown widget "{widget.kind}"</p>
        </div>
      );
  }
}

function KpiWidgetCard({
  widget,
  summary,
  interactive,
  onNavigate,
}: {
  widget: DashboardWidget;
  summary: DashboardSummary | null;
  interactive: boolean;
  onNavigate: (section: Section) => void;
}) {
  const kpi = KPI_DEFS.find((k) => k.key === widget.config.kpi_key);
  if (!kpi) return null;
  return (
    <div
      className="kpi-tile"
      style={{ height: "100%", margin: 0, cursor: interactive ? "pointer" : "default" }}
      onClick={interactive ? () => onNavigate(kpi.section) : undefined}
    >
      <div className="value">{summary ? kpi.value(summary) : "—"}</div>
      <div className="label">{summary ? kpi.label(summary) : "Loading..."}</div>
    </div>
  );
}

/** Runs its report fresh (the same `run_custom_report` command the Reports
 * screen's own runner uses) and draws it with the same dependency-free
 * `Bar` the Reports screen uses, so a chart looks identical whether it's
 * viewed there or here. */
function ChartWidgetCard({ widget, reports }: { widget: DashboardWidget; reports: CustomReport[] }) {
  const report = reports.find((r) => r.id === widget.config.report_id);
  const q = useQuery({
    queryKey: ["runCustomReport", report?.id],
    queryFn: () => api.runCustomReport(report!.id),
    enabled: !!report,
  });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.value));

  if (!report) {
    return (
      <div className="card" style={{ height: "100%", margin: 0 }}>
        <p className="empty-state">(report deleted)</p>
      </div>
    );
  }

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>{report.name}</h3>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No data yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Group</th>
              <th></th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.group}>
                <td>{r.group}</td>
                <td>
                  <Bar value={r.value} max={max} />
                </td>
                <td>{report.aggregate === "sum" ? r.value.toLocaleString() : r.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function useDashboardRecordListRows(widget: DashboardWidget) {
  const entityType = widget.config.entity_type as string;
  const mode = widget.config.mode as RecordListMode;
  const limit = (widget.config.limit as number) ?? 5;
  const savedViewId = (widget.config.saved_view_id as string | undefined) ?? null;
  const q = useQuery({
    queryKey: ["dashboardRecordList", entityType, mode, limit, savedViewId],
    queryFn: () => api.runDashboardRecordList(entityType, mode, limit, savedViewId),
  });
  return { entityType, mode, rows: q.data ?? [], isLoading: q.isLoading };
}

/** A short list of records for `widget.config.entity_type`, run fresh via
 * `run_dashboard_record_list` (see `dashboard_widget_service` in core for
 * what "recent" vs "due_soon" mean). Clicking a row jumps straight to that
 * record, the same one-shot navigation an ID hyperlink or a Global search
 * result already uses. */
function RecordListWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const { entityType, mode, rows, isLoading } = useDashboardRecordListRows(widget);

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>
        {entityTypeLabel(entityType)} - {mode === "due_soon" ? "due soon" : "recent"}
      </h3>
      {isLoading && <p>Loading...</p>}
      {rows.length === 0 && !isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((r) => (
            <li
              key={r.entity_id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
            >
              <span>{r.title}</span>
              {r.subtitle && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{r.subtitle}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

const TABLE_WIDGET_COLUMNS: ListTableColumn<RecordListRow>[] = [
  { key: "title", label: "Name", getValue: (r) => r.title },
  { key: "subtitle", label: "Detail", getValue: (r) => r.subtitle },
];

/** Same data as a record-list widget, rendered as a full `ListTable` (the
 * same sortable/resizable grid a list screen uses) instead of a compact
 * bullet list - for a dashboard tile that wants to read more like a real
 * data grid. No new backend: identical `run_dashboard_record_list` call. */
function TableWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const { entityType, mode, rows, isLoading } = useDashboardRecordListRows(widget);

  return (
    <div className="card" style={{ height: "100%", margin: 0, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <h3 style={{ marginTop: 0, flexShrink: 0 }}>
        {entityTypeLabel(entityType)} - {mode === "due_soon" ? "due soon" : "recent"}
      </h3>
      {isLoading && <p>Loading...</p>}
      <div style={{ flex: 1, overflow: "auto" }}>
        <ListTable
          storageKey={`dashboardTable:${widget.id}`}
          columns={TABLE_WIDGET_COLUMNS}
          visibleKeys={null}
          onVisibleKeysChange={() => {}}
          groups={[{ label: "", rows }]}
          getRowId={(r) => r.entity_id}
          onRowClick={interactive ? (r) => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
          density="compact"
          mobileCollapse={false}
          emptyMessage="Nothing here yet."
        />
      </div>
    </div>
  );
}

/** Embeds one specific Saved View's exact filters/sort (see `useSavedViews`)
 * as a dashboard tile - the data resolution is identical to a record-list
 * widget narrowed by `saved_view_id`, just always scoped to that one view
 * rather than letting the widget's own `mode` pick "recent"/"due soon". */
function SavedViewWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const entityType = widget.config.entity_type as string;
  const limit = (widget.config.limit as number) ?? 5;
  const savedViewId = widget.config.saved_view_id as string;
  const views = useQuery({ queryKey: ["savedViewsForWidget", entityType], queryFn: () => api.listSavedViews(entityType) });
  const view = views.data?.find((v) => v.id === savedViewId);
  const q = useQuery({
    queryKey: ["dashboardRecordList", entityType, "recent", limit, savedViewId],
    queryFn: () => api.runDashboardRecordList(entityType, "recent", limit, savedViewId),
  });
  const rows = q.data ?? [];

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>{view ? view.name : `${entityTypeLabel(entityType)} saved view`}</h3>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((r) => (
            <li
              key={r.entity_id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
            >
              <span>{r.title}</span>
              {r.subtitle && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{r.subtitle}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** A queue of open Tasks filtered the same way Tasks.tsx's own tabs filter
 * them, plus "mine" (owner_user_id === the signed-in user, via
 * `useCurrentUser`) since a dashboard tile is personal in a way a shared
 * list screen tab isn't. No new backend - filters the same
 * `api.listTasks()` rows the Tasks screen already fetches. */
function TaskQueueWidgetCard({
  widget,
  interactive,
  onNavigate,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onNavigate: (section: Section) => void;
}) {
  const mode = widget.config.mode as TaskQueueMode;
  const limit = (widget.config.limit as number) ?? 5;
  const tasks = useQuery({ queryKey: ["tasks"], queryFn: () => api.listTasks() });
  const currentUser = useCurrentUser();
  const today = new Date().toISOString().slice(0, 10);

  const open = (tasks.data ?? []).filter((t) => t.status !== "Completed" && !t.archived_at);
  let rows: Task[];
  if (mode === "today") rows = open.filter((t) => t.due_date === today);
  else if (mode === "overdue") rows = open.filter((t) => !!t.due_date && t.due_date < today);
  else if (mode === "upcoming") rows = open.filter((t) => !!t.due_date && t.due_date > today);
  else rows = open.filter((t) => t.owner_user_id === currentUser.data?.id);
  rows = rows.slice(0, limit);

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>Tasks - {TASK_QUEUE_MODE_LABELS[mode] ?? mode}</h3>
      {tasks.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !tasks.isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((t) => (
            <li
              key={t.id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onNavigate("tasks") : undefined}
            >
              <span>{t.title}</span>
              {t.due_date && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{t.due_date}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** One active AI agent's today's token usage and recent run outcomes -
 * resolved from `api.getAiAgentTokenUsage`/`api.listAiAgentRuns`, both
 * already exposed to the frontend (see the AI Gateway's existing
 * per-(workspace, agent, user, day) accounting), so this adds no new Rust
 * surface either. */
function AgentInsightWidgetCard({ widget }: { widget: DashboardWidget }) {
  const agentId = widget.config.agent_id as string;
  const agents = useQuery({ queryKey: ["aiAgentsForWidget"], queryFn: () => api.listAiAgents(false) });
  const usage = useQuery({ queryKey: ["aiAgentTokenUsage", agentId], queryFn: () => api.getAiAgentTokenUsage(agentId) });
  const runs = useQuery({ queryKey: ["aiAgentRunsForWidget", agentId], queryFn: () => api.listAiAgentRuns("agent", agentId, 5) });
  const agent = agents.data?.find((a) => a.id === agentId);
  const recentRuns = runs.data ?? [];
  const succeeded = recentRuns.filter((r) => r.status === "succeeded").length;

  if (!agent) {
    return (
      <div className="card" style={{ height: "100%", margin: 0 }}>
        <p className="empty-state">(agent deleted)</p>
      </div>
    );
  }

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>
        {agent.name} <span className={`badge${agent.is_active ? " badge-success" : ""}`}>{agent.is_active ? "Active" : "Inactive"}</span>
      </h3>
      {usage.data && (
        <p style={{ fontSize: 13, color: "var(--text-muted)", margin: "4px 0" }}>
          Today: {usage.data.today_input_tokens + usage.data.today_output_tokens} tokens
          {usage.data.daily_token_budget ? ` of ${usage.data.daily_token_budget} budget` : ""}
        </p>
      )}
      {recentRuns.length > 0 ? (
        <p style={{ fontSize: 13, margin: "4px 0" }}>
          Last {recentRuns.length} runs: {succeeded} succeeded, {recentRuns.length - succeeded} other
        </p>
      ) : (
        <p className="empty-state">No runs yet.</p>
      )}
    </div>
  );
}
