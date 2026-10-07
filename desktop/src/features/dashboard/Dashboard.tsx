import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { api } from "../../lib/api";
import { formatCents } from "../../lib/money";
import { DashboardGrid } from "../../components/DashboardGrid";
import type { Section } from "../../components/AppShell";
import type { DashboardWidget } from "../../lib/types";
import { useEffectiveDashboard } from "../../lib/useEffectiveDashboard";
import { DashboardWidgetCard } from "./DashboardWidgetCard";
import { KPI_DEFS, resolveVisibleKpis } from "./kpis";

export function Dashboard({
  onNavigate,
  onOpenRecord,
  appDashboardId,
}: {
  onNavigate: (section: Section) => void;
  /** A record-list widget row jumps straight to that record, reusing the
   * same one-shot openId navigation Global search's own results already
   * use (see AppShell's Prefill doc comment). */
  onOpenRecord: (section: Section, id: string) => void;
  /** App Builder: when the sidebar's App Switcher has an app selected and
   * that app names a dashboard, render that dashboard's *published*
   * widgets instead of the role-resolved default below - the same
   * "content only ever comes from `published`, never `draft`" rule
   * `useEffectiveDashboard` already follows. `null`/`undefined` (no app
   * selected, or the selected app doesn't name one) is the pre-App-Builder
   * behavior, unchanged. */
  appDashboardId?: string | null;
}) {
  const queryClient = useQueryClient();
  const { data, isLoading, error } = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => api.dashboardSummary(),
  });
  const workspace = useQuery({ queryKey: ["workspaceStatus"], queryFn: () => api.workspaceStatus() });
  // Dashboard customization Phase 1: a published dashboard layout (see
  // useEffectiveDashboard's own doc comment) overrides which KPI tiles
  // show and in what order - `null` (the common case until an admin
  // builds one) falls back to the pre-this-feature workspace-wide
  // `dashboard_kpi_prefs` selection below, unchanged. Skipped entirely
  // once an app names its own dashboard (below) - that layout wins.
  const effectiveDashboard = useEffectiveDashboard();
  // App Builder: the full layout list, only fetched when an app-scoped
  // dashboard is actually in play - `list_dashboard_layouts` (unlike
  // `effective_dashboard_layout`) returns every layout so this can look
  // up one specific id instead of the role-resolved one.
  const appLayouts = useQuery({
    queryKey: ["dashboardLayouts"],
    queryFn: () => api.listDashboardLayouts(),
    enabled: !!appDashboardId,
  });
  // Phase 2: chart widgets reference a saved Custom Report by id -
  // fetched once here so every chart widget below can just look its
  // report up instead of each re-fetching the whole list.
  const reports = useQuery({ queryKey: ["customReports"], queryFn: () => api.listCustomReports() });

  useEffect(() => {
    api.refreshOverdueInvoices().then(() => {
      queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    });
    // Runs once when the dashboard first mounts.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (isLoading) return <p>Loading dashboard...</p>;
  if (error || !data) return <div className="error-banner">Could not load the dashboard</div>;

  const appLayout = appDashboardId ? appLayouts.data?.find((l) => l.id === appDashboardId) : undefined;
  const layoutWidgets = appDashboardId
    ? appLayout?.published?.widgets ?? null
    : effectiveDashboard.data?.widgets?.widgets ?? null;

  // Every widget kind renders through one shared `DashboardGrid` +
  // `DashboardWidgetCard` (issue #198) - a kpi/chart widget whose kpi_key/
  // report_id has since gone stale is simply skipped here rather than
  // shown broken, the same "opaque key can go stale, skip not error"
  // choice `DashboardWidget`'s own doc comment already makes; every other
  // kind degrades gracefully inside its own card instead (e.g. "(agent
  // deleted)"), since a reference there isn't necessarily stale (a
  // deactivated agent is still a real agent). The no-layout-published-yet
  // fallback keeps the older workspace-wide `dashboard_kpi_prefs` KPI
  // selection, routed through the same renderer via synthetic widgets so
  // there's still only one rendering path.
  const kpiByKey = new Map(KPI_DEFS.map((k) => [k.key, k]));
  const reportById = new Map((reports.data ?? []).map((r) => [r.id, r]));
  const gridWidgets: DashboardWidget[] = layoutWidgets
    ? layoutWidgets.filter((w) => {
        if (w.kind === "kpi") return kpiByKey.has(w.config.kpi_key as string);
        if (w.kind === "chart") return reportById.has(w.config.report_id as string);
        return true;
      })
    : resolveVisibleKpis(workspace.data?.dashboard_kpi_prefs ?? null).map((k) => ({
        id: k.key,
        kind: "kpi",
        config: { kpi_key: k.key },
        layout: null,
      }));

  return (
    <div>
      <h2>Dashboard</h2>

      {gridWidgets.length > 0 && (
        <div style={{ marginBottom: 16 }}>
          <DashboardGrid
            widgets={gridWidgets}
            renderWidget={(w) => (
              <DashboardWidgetCard widget={w} summary={data} reports={reports.data ?? []} onNavigate={onNavigate} onOpenRecord={onOpenRecord} />
            )}
          />
        </div>
      )}

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
        <div className="card">
          <h3 style={{ marginTop: 0 }}>Pipeline by stage</h3>
          {data.pipeline_by_stage.length === 0 ? (
            <p className="empty-state">No open opportunities yet</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Stage</th>
                  <th>Count</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                {data.pipeline_by_stage.map((s) => (
                  <tr key={s.stage}>
                    <td>{s.stage}</td>
                    <td>{s.count}</td>
                    <td>{formatCents(s.value_cents)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
        <div className="card">
          <h3 style={{ marginTop: 0 }}>Recent activity</h3>
          {data.recent_activity.length === 0 ? (
            <p className="empty-state">No activity yet</p>
          ) : (
            <ul style={{ margin: 0, paddingLeft: 18, fontSize: 14 }}>
              {data.recent_activity.map((a, idx) => (
                <li key={idx} style={{ marginBottom: 6 }}>
                  <span style={{ color: "var(--text-muted)" }}>
                    {new Date(a.occurred_at).toLocaleString()}
                  </span>{" "}
                  — {a.summary}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

