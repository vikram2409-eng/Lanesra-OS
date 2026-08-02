import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { api } from "../../lib/api";
import { formatCents } from "../../lib/money";
import type { Section } from "../../components/AppShell";

export function Dashboard({ onNavigate }: { onNavigate: (section: Section) => void }) {
  const queryClient = useQueryClient();
  const { data, isLoading, error } = useQuery({
    queryKey: ["dashboard"],
    queryFn: () => api.dashboardSummary(),
  });

  useEffect(() => {
    api.refreshOverdueInvoices().then(() => {
      queryClient.invalidateQueries({ queryKey: ["dashboard"] });
    });
    // Runs once when the dashboard first mounts.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (isLoading) return <p>Loading dashboard...</p>;
  if (error || !data) return <div className="error-banner">Could not load the dashboard</div>;

  return (
    <div>
      <h2>Dashboard</h2>
      <div className="kpi-row">
        <div className="kpi-tile" onClick={() => onNavigate("opportunities")}>
          <div className="value">{formatCents(data.open_pipeline_value_cents)}</div>
          <div className="label">Open pipeline ({data.open_pipeline_count})</div>
        </div>
        <div className="kpi-tile" onClick={() => onNavigate("opportunities")}>
          <div className="value">{formatCents(data.won_revenue_cents)}</div>
          <div className="label">Won revenue</div>
        </div>
        <div className="kpi-tile" onClick={() => onNavigate("invoices")}>
          <div className="value">{formatCents(data.outstanding_invoices_cents)}</div>
          <div className="label">Outstanding invoices</div>
        </div>
        <div className="kpi-tile" onClick={() => onNavigate("invoices")}>
          <div className="value">{formatCents(data.overdue_invoices_cents)}</div>
          <div className="label">Overdue ({data.overdue_invoices_count})</div>
        </div>
        <div className="kpi-tile" onClick={() => onNavigate("quotes")}>
          <div className="value">{data.quotes_awaiting_response}</div>
          <div className="label">Quotes awaiting response</div>
        </div>
      </div>

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
