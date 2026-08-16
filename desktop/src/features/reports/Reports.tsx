import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { formatCents } from "../../lib/money";
import { Bar } from "../../components/Bar";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  REPORT_AGGREGATES,
  entityTypeLabel,
  builtinTriggerFieldFor,
  type ArAgingBucket,
  type CustomFieldEntityType,
  type CustomReport,
  type CustomReportInput,
  type LostReasonBreakdown,
  type ReportAggregate,
  type ReportGroupBySource,
  type RevenueByMonth,
  type SalesByOwner,
  type WinRateByOwner,
} from "../../lib/types";

type ReportKey = "revenue" | "winRate" | "lostReasons" | "arAging" | "salesByOwner" | "custom";

const REPORTS: { key: ReportKey; label: string }[] = [
  { key: "revenue", label: "Revenue by month" },
  { key: "winRate", label: "Win rate by owner" },
  { key: "lostReasons", label: "Lost reasons" },
  { key: "arAging", label: "AR aging" },
  { key: "salesByOwner", label: "Sales by owner" },
  { key: "custom", label: "Custom reports" },
];

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

export function Reports({ isAdmin }: { isAdmin: boolean }) {
  const [active, setActive] = useState<ReportKey>("revenue");
  const [from, setFrom] = useState<string>("");
  const [to, setTo] = useState<string>(todayIso());
  const [asOfDate, setAsOfDate] = useState<string>(todayIso());

  const range = { from: from || null, to: to || null };

  return (
    <div>
      <h2>Reports</h2>
      <p style={{ color: "var(--text-muted)" }}>
        Beyond the dashboard's KPI tiles - revenue, pipeline outcomes, aging receivables, sales by owner, and
        admin-built custom reports.
      </p>

      <div className="tab-row">
        {REPORTS.map((r) => (
          <button key={r.key} className={`tab${active === r.key ? " active" : ""}`} onClick={() => setActive(r.key)}>
            {r.label}
          </button>
        ))}
      </div>

      {active !== "arAging" && active !== "custom" && (
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
      {active === "custom" && <CustomReportsPanel isAdmin={isAdmin} />}
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

function emptyCustomReportInput(): CustomReportInput {
  return {
    name: "",
    entity_type: CUSTOM_FIELD_ENTITY_TYPES[0],
    group_by_source: "builtin",
    group_by_field: builtinTriggerFieldFor(CUSTOM_FIELD_ENTITY_TYPES[0]),
    aggregate: "count",
    sum_field_key: null,
  };
}

function CustomReportsPanel({ isAdmin }: { isAdmin: boolean }) {
  const queryClient = useQueryClient();
  const [creating, setCreating] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const reports = useQuery({ queryKey: ["customReports"], queryFn: () => api.listCustomReports() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["customReports"] });
  }

  const deleteReport = useMutation({
    mutationFn: (id: string) => api.deleteCustomReport(id),
    onSuccess: () => {
      invalidate();
      setSelectedId(null);
    },
  });

  const selected = reports.data?.find((r) => r.id === selectedId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Custom reports</h3>
        {isAdmin && (
          <button className="btn btn-primary" onClick={() => setCreating((v) => !v)}>
            + New report
          </button>
        )}
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Pick an entity, a field to group by, and an aggregate - a small alternative to the fixed reports above for
        questions those don't answer.
      </p>

      {creating && isAdmin && (
        <CustomReportForm
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {reports.isLoading && <p>Loading...</p>}
      {reports.data && reports.data.length === 0 && !creating && (
        <p className="empty-state">No custom reports yet.</p>
      )}
      {reports.data && reports.data.length > 0 && (
        <table style={{ marginBottom: 16 }}>
          <thead>
            <tr>
              <th>Name</th>
              <th>Entity</th>
              <th>Group by</th>
              <th>Aggregate</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {reports.data.map((r) => (
              <tr key={r.id} onClick={() => setSelectedId(r.id)} style={{ cursor: "pointer" }}>
                <td>{r.name}</td>
                <td>{entityTypeLabel(r.entity_type)}</td>
                <td>{r.group_by_field}</td>
                <td>{r.aggregate === "sum" ? `Sum of ${r.sum_field_key}` : "Count"}</td>
                <td>
                  {isAdmin && (
                    <button
                      className="btn btn-danger"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (window.confirm(`Delete report "${r.name}"?`)) deleteReport.mutate(r.id);
                      }}
                    >
                      Delete
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {selected && <CustomReportRunner report={selected} />}
    </div>
  );
}

function CustomReportForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<CustomReportInput>(emptyCustomReportInput());
  const [error, setError] = useState<string | null>(null);

  const defs = useQuery({
    queryKey: ["customFieldDefinitions", input.entity_type, "all"],
    queryFn: () => api.listCustomFieldDefinitions(input.entity_type, true),
  });
  // ADM-CF-05: a field an admin flagged not reportable is excluded here -
  // the server would reject it as a group-by/sum target anyway.
  const activeDefs = (defs.data ?? []).filter((d) => d.is_reportable);
  const numericDefs = activeDefs.filter((d) => d.field_type === "number");

  const create = useMutation({
    mutationFn: () => api.createCustomReport(input),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this report"),
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
          <label>Report name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Entity</label>
          <select
            value={input.entity_type}
            onChange={(e) => {
              const entityType = e.target.value as CustomFieldEntityType;
              setInput({
                ...input,
                entity_type: entityType,
                group_by_source: "builtin",
                group_by_field: builtinTriggerFieldFor(entityType),
                sum_field_key: null,
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
        <div className="form-field">
          <label>Group by</label>
          <select
            value={input.group_by_source === "builtin" ? "__builtin__" : input.group_by_field}
            onChange={(e) => {
              if (e.target.value === "__builtin__") {
                setInput({
                  ...input,
                  group_by_source: "builtin",
                  group_by_field: builtinTriggerFieldFor(input.entity_type),
                });
              } else {
                const source: ReportGroupBySource = "custom";
                setInput({ ...input, group_by_source: source, group_by_field: e.target.value });
              }
            }}
          >
            <option value="__builtin__">
              {builtinTriggerFieldFor(input.entity_type) === "is_active" ? "Active" : "Status"}
            </option>
            {activeDefs.map((d) => (
              <option key={d.key} value={d.key}>
                {d.label}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Aggregate</label>
          <select
            value={input.aggregate}
            onChange={(e) => {
              const aggregate = e.target.value as ReportAggregate;
              setInput({ ...input, aggregate, sum_field_key: null });
            }}
          >
            {REPORT_AGGREGATES.map((a) => (
              <option key={a} value={a}>
                {a === "count" ? "Count of records" : "Sum of a numeric field"}
              </option>
            ))}
          </select>
        </div>
        {input.aggregate === "sum" && (
          <div className="form-field">
            <label>Sum field</label>
            <select
              value={input.sum_field_key ?? ""}
              onChange={(e) => setInput({ ...input, sum_field_key: e.target.value || null })}
              required
            >
              <option value="">— Select —</option>
              {numericDefs.map((d) => (
                <option key={d.key} value={d.key}>
                  {d.label}
                </option>
              ))}
            </select>
            {numericDefs.length === 0 && (
              <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
                No active numeric custom fields on {entityTypeLabel(input.entity_type)} yet.
              </p>
            )}
          </div>
        )}
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button
            className="btn btn-primary"
            type="submit"
            disabled={create.isPending || (input.aggregate === "sum" && !input.sum_field_key)}
          >
            Create report
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function CustomReportRunner({ report }: { report: CustomReport }) {
  const q = useQuery({ queryKey: ["runCustomReport", report.id], queryFn: () => api.runCustomReport(report.id) });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.value));

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>{report.name}</h3>
        <ExportCsvButton
          rows={rows}
          filename={`${report.name.toLowerCase().replace(/\s+/g, "-")}.csv`}
          columns={[
            { label: "Group", get: (r) => r.group },
            { label: "Value", get: (r) => String(r.value) },
          ]}
        />
      </div>
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
