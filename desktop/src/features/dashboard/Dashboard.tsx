import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { api } from "../../lib/api";
import { formatCents } from "../../lib/money";
import { Bar } from "../../components/Bar";
import type { Section } from "../../components/AppShell";
import type { CustomReport } from "../../lib/types";
import { useEffectiveDashboard } from "../../lib/useEffectiveDashboard";
import { KPI_DEFS, resolveVisibleKpis, type KpiDef } from "./kpis";

export function Dashboard({ onNavigate }: { onNavigate: (section: Section) => void }) {
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
  // `dashboard_kpi_prefs` selection below, unchanged.
  const effectiveDashboard = useEffectiveDashboard();
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

  const layoutWidgets = effectiveDashboard.data?.widgets?.widgets ?? null;
  const kpiByKey = new Map(KPI_DEFS.map((k) => [k.key, k]));
  const visibleKpis: KpiDef[] = layoutWidgets
    ? layoutWidgets
        .filter((w) => w.kind === "kpi")
        .map((w) => kpiByKey.get(w.config.kpi_key as string))
        .filter((k): k is KpiDef => !!k)
    : resolveVisibleKpis(workspace.data?.dashboard_kpi_prefs ?? null);

  // A chart widget's report_id may point at a report that's since been
  // deleted - that widget is simply skipped, not an error (see
  // DashboardWidget's own doc comment on this "opaque key can go stale"
  // choice, same as an unresolved KPI key above).
  const reportById = new Map((reports.data ?? []).map((r) => [r.id, r]));
  const chartReports: CustomReport[] = layoutWidgets
    ? layoutWidgets
        .filter((w) => w.kind === "chart")
        .map((w) => reportById.get(w.config.report_id as string))
        .filter((r): r is CustomReport => !!r)
    : [];

  return (
    <div>
      <h2>Dashboard</h2>
      <div className="kpi-row">
        {visibleKpis.map((kpi) => (
          <div className="kpi-tile" key={kpi.key} onClick={() => onNavigate(kpi.section)}>
            <div className="value">{kpi.value(data)}</div>
            <div className="label">{kpi.label(data)}</div>
          </div>
        ))}
      </div>

      {chartReports.length > 0 && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
          {chartReports.map((report) => (
            <DashboardChartCard key={report.id} report={report} />
          ))}
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

/** One chart widget on the live Dashboard - runs its report fresh (same
 * `run_custom_report` command the Reports screen's own runner uses) and
 * draws it with the same dependency-free `Bar` the Reports screen uses,
 * so a chart looks identical whether it's viewed there or here. */
function DashboardChartCard({ report }: { report: CustomReport }) {
  const q = useQuery({ queryKey: ["runCustomReport", report.id], queryFn: () => api.runCustomReport(report.id) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.value));

  return (
    <div className="card">
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
