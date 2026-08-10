import type { Section } from "../../components/AppShell";
import { formatCents } from "../../lib/money";
import type { DashboardSummary } from "../../lib/types";

export interface KpiDef {
  key: string;
  section: Section;
  value: (d: DashboardSummary) => string;
  label: (d: DashboardSummary) => string;
}

/**
 * The full catalog of Dashboard KPI tiles, in their default order. An
 * Administrator can choose a subset and reorder it (FR-KPI); Workspace's
 * `dashboard_kpi_prefs` stores the chosen key order as JSON, and this
 * array is the single source of truth both Dashboard.tsx and the picker
 * admin screen key off of, so a new KPI only needs to be added here once.
 */
export const KPI_DEFS: KpiDef[] = [
  {
    key: "open_pipeline",
    section: "opportunities",
    value: (d) => formatCents(d.open_pipeline_value_cents),
    label: (d) => `Open pipeline (${d.open_pipeline_count})`,
  },
  {
    key: "won_revenue",
    section: "opportunities",
    value: (d) => formatCents(d.won_revenue_cents),
    label: () => "Won revenue",
  },
  {
    key: "outstanding_invoices",
    section: "invoices",
    value: (d) => formatCents(d.outstanding_invoices_cents),
    label: () => "Outstanding invoices",
  },
  {
    key: "overdue_invoices",
    section: "invoices",
    value: (d) => formatCents(d.overdue_invoices_cents),
    label: (d) => `Overdue (${d.overdue_invoices_count})`,
  },
  {
    key: "quotes_awaiting_response",
    section: "quotes",
    value: (d) => String(d.quotes_awaiting_response),
    label: () => "Quotes awaiting response",
  },
  {
    key: "contracts_renewing",
    section: "contracts",
    value: (d) => String(d.contracts_renewing_90_days),
    label: (d) => `Renewing 90 days (${d.contracts_renewing_30_days} in 30 / ${d.contracts_renewing_60_days} in 60)`,
  },
  {
    key: "open_tasks",
    section: "tasks",
    value: (d) => String(d.open_tasks),
    label: (d) => `Open tasks (${d.overdue_tasks} overdue)`,
  },
];

export function kpiLabel(key: string): string {
  const labels: Record<string, string> = {
    open_pipeline: "Open pipeline value",
    won_revenue: "Won revenue",
    outstanding_invoices: "Outstanding invoices",
    overdue_invoices: "Overdue invoices",
    quotes_awaiting_response: "Quotes awaiting response",
    contracts_renewing: "Contracts renewing soon",
    open_tasks: "Open tasks",
  };
  return labels[key] ?? key;
}

/** Parses Workspace.dashboard_kpi_prefs (JSON array or null) into an
 * ordered list of KpiDefs - unknown keys are dropped defensively (e.g. a
 * backup restored from a build that had a KPI this one no longer does). */
export function resolveVisibleKpis(prefsJson: string | null): KpiDef[] {
  if (!prefsJson) return KPI_DEFS;
  try {
    const keys: unknown = JSON.parse(prefsJson);
    if (!Array.isArray(keys) || keys.length === 0) return KPI_DEFS;
    const byKey = new Map(KPI_DEFS.map((k) => [k.key, k]));
    const resolved = keys.map((k) => byKey.get(String(k))).filter((k): k is KpiDef => !!k);
    return resolved.length > 0 ? resolved : KPI_DEFS;
  } catch {
    return KPI_DEFS;
  }
}
