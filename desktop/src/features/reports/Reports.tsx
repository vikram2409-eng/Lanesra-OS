import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { formatCents } from "../../lib/money";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import type {
  ArAgingBucket,
  LostReasonBreakdown,
  RevenueByMonth,
  SalesByOwner,
  WinRateByOwner,
} from "../../lib/types";

type ReportKey = "revenue" | "winRate" | "lostReasons" | "arAging" | "salesByOwner";

const REPORTS: { key: ReportKey; label: string }[] = [
  { key: "revenue", label: "Revenue by month" },
  { key: "winRate", label: "Win rate by owner" },
  { key: "lostReasons", label: "Lost reasons" },
  { key: "arAging", label: "AR aging" },
  { key: "salesByOwner", label: "Sales by owner" },
];

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

/** A dependency-free horizontal bar, sized relative to the row's own max value. */
function Bar({ value, max }: { value: number; max: number }) {
  const pct = max > 0 ? Math.max(2, Math.round((value / max) * 100)) : 0;
  return (
    <div style={{ background: "var(--surface-2, rgba(127,127,127,0.15))", borderRadius: 3, height: 8, width: 120 }}>
      <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)", borderRadius: 3 }} />
    </div>
  );
}

export function Reports() {
  const [active, setActive] = useState<ReportKey>("revenue");
  const [from, setFrom] = useState<string>("");
  const [to, setTo] = useState<string>(todayIso());
  const [asOfDate, setAsOfDate] = useState<string>(todayIso());

  const range = { from: from || null, to: to || null };

  return (
    <div>
      <h2>Reports</h2>
      <p style={{ color: "var(--text-muted)" }}>
        Beyond the dashboard's KPI tiles - revenue, pipeline outcomes, aging receivables and sales by owner.
      </p>

      <div className="tab-row">
        {REPORTS.map((r) => (
          <button key={r.key} className={`tab${active === r.key ? " active" : ""}`} onClick={() => setActive(r.key)}>
            {r.label}
          </button>
        ))}
      </div>

      {active !== "arAging" && (
        <div style={{ display: "flex", gap: 12, alignItems: "flex-end", marginBottom: 16 }}>
          <div className="form-field">
            <label>From</label>
            <input type="date" value={from} onChange={(e) => setFrom(e.target.value)} />
          </div>
          <div className="form-field">
            <label>To</label>
            <input type="date" value={to} onChange={(e) => setTo(e.target.value)} />
          </div>
        </div>
      )}
      {active === "arAging" && (
        <div style={{ display: "flex", gap: 12, alignItems: "flex-end", marginBottom: 16 }}>
          <div className="form-field">
            <label>As of</label>
            <input type="date" value={asOfDate} onChange={(e) => setAsOfDate(e.target.value)} />
          </div>
        </div>
      )}

      {active === "revenue" && <RevenueByMonthReport range={range} />}
      {active === "winRate" && <WinRateByOwnerReport range={range} />}
      {active === "lostReasons" && <LostReasonsReport range={range} />}
      {active === "arAging" && <ArAgingReport asOfDate={asOfDate} />}
      {active === "salesByOwner" && <SalesByOwnerReport range={range} />}
    </div>
  );
}

function RevenueByMonthReport({ range }: { range: { from: string | null; to: string | null } }) {
  const q = useQuery({ queryKey: ["reportRevenueByMonth", range], queryFn: () => api.reportRevenueByMonth(range) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.total_cents));

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Revenue by month</h3>
        <ExportCsvButton
          rows={rows}
          filename="revenue-by-month.csv"
          columns={[
            { label: "Month", get: (r: RevenueByMonth) => r.month },
            { label: "Invoices", get: (r: RevenueByMonth) => String(r.invoice_count) },
            { label: "Revenue (cents)", get: (r: RevenueByMonth) => String(r.total_cents) },
          ]}
        />
      </div>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No issued invoices in this range.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Month</th>
              <th>Invoices</th>
              <th></th>
              <th>Revenue</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.month}>
                <td>{r.month}</td>
                <td>{r.invoice_count}</td>
                <td>
                  <Bar value={r.total_cents} max={max} />
                </td>
                <td>{formatCents(r.total_cents, "USD")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function WinRateByOwnerReport({ range }: { range: { from: string | null; to: string | null } }) {
  const q = useQuery({ queryKey: ["reportWinRateByOwner", range], queryFn: () => api.reportWinRateByOwner(range) });
  const rows = q.data ?? [];

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Win rate by owner</h3>
        <ExportCsvButton
          rows={rows}
          filename="win-rate-by-owner.csv"
          columns={[
            { label: "Owner", get: (r: WinRateByOwner) => r.owner_name },
            { label: "Won", get: (r: WinRateByOwner) => String(r.won_count) },
            { label: "Lost", get: (r: WinRateByOwner) => String(r.lost_count) },
            {
              label: "Win rate",
              get: (r: WinRateByOwner) =>
                r.won_count + r.lost_count > 0
                  ? `${Math.round((r.won_count / (r.won_count + r.lost_count)) * 100)}%`
                  : "—",
            },
            { label: "Won value (cents)", get: (r: WinRateByOwner) => String(r.won_value_cents) },
          ]}
        />
      </div>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No won or lost opportunities in this range.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Owner</th>
              <th>Won</th>
              <th>Lost</th>
              <th>Win rate</th>
              <th>Won value</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const total = r.won_count + r.lost_count;
              const winRate = total > 0 ? Math.round((r.won_count / total) * 100) : null;
              return (
                <tr key={r.owner_user_id ?? "unassigned"}>
                  <td>{r.owner_name}</td>
                  <td>{r.won_count}</td>
                  <td>{r.lost_count}</td>
                  <td>{winRate === null ? "—" : `${winRate}%`}</td>
                  <td>{formatCents(r.won_value_cents, "USD")}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

function LostReasonsReport({ range }: { range: { from: string | null; to: string | null } }) {
  const q = useQuery({ queryKey: ["reportLostReasons", range], queryFn: () => api.reportLostReasons(range) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.count));

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Lost reasons</h3>
        <ExportCsvButton
          rows={rows}
          filename="lost-reasons.csv"
          columns={[
            { label: "Reason", get: (r: LostReasonBreakdown) => r.reason },
            { label: "Count", get: (r: LostReasonBreakdown) => String(r.count) },
            { label: "Value (cents)", get: (r: LostReasonBreakdown) => String(r.value_cents) },
          ]}
        />
      </div>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No lost opportunities in this range.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Reason</th>
              <th>Count</th>
              <th></th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.reason}>
                <td>{r.reason}</td>
                <td>{r.count}</td>
                <td>
                  <Bar value={r.count} max={max} />
                </td>
                <td>{formatCents(r.value_cents, "USD")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ArAgingReport({ asOfDate }: { asOfDate: string }) {
  const q = useQuery({ queryKey: ["reportArAging", asOfDate], queryFn: () => api.reportArAging(asOfDate || null) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.balance_cents));

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>AR aging</h3>
        <ExportCsvButton
          rows={rows}
          filename="ar-aging.csv"
          columns={[
            { label: "Bucket", get: (r: ArAgingBucket) => r.bucket },
            { label: "Invoices", get: (r: ArAgingBucket) => String(r.invoice_count) },
            { label: "Balance (cents)", get: (r: ArAgingBucket) => String(r.balance_cents) },
          ]}
        />
      </div>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No outstanding balances.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Bucket</th>
              <th>Invoices</th>
              <th></th>
              <th>Balance</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.bucket}>
                <td>{r.bucket}</td>
                <td>{r.invoice_count}</td>
                <td>
                  <Bar value={r.balance_cents} max={max} />
                </td>
                <td>{formatCents(r.balance_cents, "USD")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function SalesByOwnerReport({ range }: { range: { from: string | null; to: string | null } }) {
  const q = useQuery({ queryKey: ["reportSalesByOwner", range], queryFn: () => api.reportSalesByOwner(range) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.total_cents));

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Sales by owner</h3>
        <ExportCsvButton
          rows={rows}
          filename="sales-by-owner.csv"
          columns={[
            { label: "Owner", get: (r: SalesByOwner) => r.owner_name },
            { label: "Invoices", get: (r: SalesByOwner) => String(r.invoice_count) },
            { label: "Revenue (cents)", get: (r: SalesByOwner) => String(r.total_cents) },
          ]}
        />
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: -4 }}>
        Attributed via each invoice's Company owner - invoices have no owner of their own.
      </p>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No issued invoices in this range.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Owner</th>
              <th>Invoices</th>
              <th></th>
              <th>Revenue</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.owner_user_id ?? "unassigned"}>
                <td>{r.owner_name}</td>
                <td>{r.invoice_count}</td>
                <td>
                  <Bar value={r.total_cents} max={max} />
                </td>
                <td>{formatCents(r.total_cents, "USD")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
