import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CsvImportDialog, type ParsedCsvRow } from "../../components/CsvImportDialog";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { RelatedRecordsCard } from "../../components/RelatedRecordsCard";
import { TabListCard } from "../../components/TabListCard";
import type { Prefill, Section } from "../../components/AppShell";
import { field } from "../../lib/csv";
import { COMPANY_STATUSES, type Company, type CompanyInput, type CustomFieldValues } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string } | { mode: "detail"; id: string };

const COMPANY_EXPORT_COLUMNS = [
  { label: "Number", get: (c: Company) => c.customer_number },
  { label: "Name", get: (c: Company) => c.name },
  { label: "Status", get: (c: Company) => c.status },
  { label: "Tax number", get: (c: Company) => c.tax_number ?? "" },
  { label: "Billing address", get: (c: Company) => c.billing_address ?? "" },
  { label: "Shipping address", get: (c: Company) => c.shipping_address ?? "" },
  { label: "Tags", get: (c: Company) => c.tags ?? "" },
  { label: "Notes", get: (c: Company) => c.notes ?? "" },
];

const COMPANY_IMPORT_COLUMNS = [
  { label: "Name", required: true },
  { label: "Status" },
  { label: "Tax number" },
  { label: "Billing address" },
  { label: "Shipping address" },
  { label: "Tags" },
  { label: "Notes" },
];

function parseCompanyRow(record: Record<string, string>): ParsedCsvRow<CompanyInput> {
  const name = field(record, "Name");
  if (!name) return { preview: "(missing name)", error: "Name is required" };

  const statusRaw = field(record, "Status");
  const status = statusRaw ? COMPANY_STATUSES.find((s) => s.toLowerCase() === statusRaw.toLowerCase()) : "Prospect";
  if (!status) return { preview: name, error: `Unknown status "${statusRaw}"` };

  return {
    preview: name,
    input: {
      name,
      status,
      owner_user_id: null,
      tax_number: field(record, "Tax number") || null,
      billing_address: field(record, "Billing address") || null,
      shipping_address: field(record, "Shipping address") || null,
      tags: field(record, "Tags") || null,
      notes: field(record, "Notes") || null,
    },
  };
}

const emptyInput: CompanyInput = {
  name: "",
  status: "Prospect",
  owner_user_id: null,
  tax_number: null,
  billing_address: null,
  shipping_address: null,
  tags: null,
  notes: null,
};

export function Companies({ onNavigateTo }: { onNavigateTo?: (section: Section, prefill: Prefill) => void } = {}) {
  const [view, setView] = useState<View>({ mode: "list" });
  const [importing, setImporting] = useState(false);
  const queryClient = useQueryClient();
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["companies"] });
  }

  if (view.mode === "create" || view.mode === "edit") {
    return (
      <CompanyForm
        companyId={view.mode === "edit" ? view.id : undefined}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  if (view.mode === "detail") {
    return (
      <CompanyDetail
        id={view.id}
        onEdit={() => setView({ mode: "edit", id: view.id })}
        onBack={() => setView({ mode: "list" })}
        onNavigateTo={onNavigateTo}
      />
    );
  }

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Companies</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton rows={companies.data ?? []} columns={COMPANY_EXPORT_COLUMNS} filename="companies.csv" />
          <button className="btn" onClick={() => setImporting((v) => !v)}>
            Import CSV
          </button>
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New company
          </button>
        </div>
      </div>
      {importing && (
        <CsvImportDialog
          title="Import companies"
          columns={COMPANY_IMPORT_COLUMNS}
          parseRow={parseCompanyRow}
          createFn={(input) => api.createCompany(input)}
          onImported={invalidate}
          onClose={() => setImporting(false)}
        />
      )}
      {companies.isLoading && <p>Loading...</p>}
      {companies.data && companies.data.length === 0 && (
        <p className="empty-state">No companies yet. Create your first one.</p>
      )}
      {companies.data && companies.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Name</th>
              <th>Status</th>
              <th>Tax number</th>
            </tr>
          </thead>
          <tbody>
            {companies.data.map((c) => (
              <tr key={c.id} onClick={() => setView({ mode: "detail", id: c.id })} style={{ cursor: "pointer" }}>
                <td>{c.customer_number}</td>
                <td>{c.name}</td>
                <td>
                  <StatusBadge status={c.status} />
                </td>
                <td>{c.tax_number ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function CompanyForm({
  companyId,
  onDone,
  onCancel,
}: {
  companyId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["company", companyId],
    queryFn: () => api.getCompany(companyId as string),
    enabled: !!companyId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", companyId],
    queryFn: () => api.getCustomFieldValues(companyId as string),
    enabled: !!companyId,
  });
  const [input, setInput] = useState<CompanyInput>(emptyInput);
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [duplicateWarning, setDuplicateWarning] = useState<Company[] | null>(null);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== companyId) {
    setInput({
      name: existing.data.name,
      status: existing.data.status,
      owner_user_id: existing.data.owner_user_id,
      tax_number: existing.data.tax_number,
      billing_address: existing.data.billing_address,
      shipping_address: existing.data.shipping_address,
      tags: existing.data.tags,
      notes: existing.data.notes,
    });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(companyId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const company = companyId ? await api.updateCompany(companyId, input) : await api.createCompany(input);
      const ruleMessages = await api.setCustomFieldValues("Company", company.id, customValues);
      showRuleMessages(ruleMessages);
      return company;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the company"),
  });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (!duplicateWarning) {
      const duplicates = await api.checkCompanyDuplicates(input.name, companyId);
      if (duplicates.length > 0) {
        setDuplicateWarning(duplicates);
        return;
      }
    }
    save.mutate();
  }

  return (
    <div>
      <h2>{companyId ? "Edit company" : "New company"}</h2>
      {error && <div className="error-banner">{error}</div>}
      {duplicateWarning && (
        <div className="error-banner" style={{ borderColor: "var(--warning)", color: "var(--warning)" }}>
          A company with a similar name already exists ({duplicateWarning.map((d) => d.name).join(", ")}).
          Submit again to save anyway.
        </div>
      )}
      <form className="form-grid" onSubmit={handleSubmit}>
        <div className="form-field full">
          <label>Company name</label>
          <input
            value={input.name}
            onChange={(e) => {
              setDuplicateWarning(null);
              setInput({ ...input, name: e.target.value });
            }}
            required
          />
        </div>
        <div className="form-field">
          <label>Status</label>
          <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
            {COMPANY_STATUSES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Tax number</label>
          <input
            value={input.tax_number ?? ""}
            onChange={(e) => setInput({ ...input, tax_number: e.target.value || null })}
          />
        </div>
        <div className="form-field full">
          <label>Billing address</label>
          <input
            value={input.billing_address ?? ""}
            onChange={(e) => setInput({ ...input, billing_address: e.target.value || null })}
          />
        </div>
        <div className="form-field full">
          <label>Notes</label>
          <textarea
            value={input.notes ?? ""}
            onChange={(e) => setInput({ ...input, notes: e.target.value || null })}
          />
        </div>
        <CustomFieldsSection entityType="Company" status={input.status} values={customValues} onChange={setCustomValues} />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            {duplicateWarning ? "Save anyway" : "Save"}
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

type CompanyTab = "overview" | "contacts" | "opportunities" | "quotes" | "orders" | "invoices" | "contracts" | "tasks" | "activity";
const COMPANY_TABS: { tab: CompanyTab; label: string }[] = [
  { tab: "overview", label: "Overview" },
  { tab: "contacts", label: "Contacts" },
  { tab: "opportunities", label: "Opportunities" },
  { tab: "quotes", label: "Quotes" },
  { tab: "orders", label: "Orders" },
  { tab: "invoices", label: "Invoices" },
  { tab: "contracts", label: "Contracts" },
  { tab: "tasks", label: "Tasks" },
  { tab: "activity", label: "Activity" },
];

/**
 * Addendum Phase 5 (Customer 360, spec §5): a tabbed record view
 * replacing the old plain-grid CompanyDetail - Contracts and Tasks tabs
 * added (previously missing entirely), a clickable KPI strip that jumps
 * straight to the matching tab, "+ New" from each tab pre-filling the
 * relationship via `onNavigateTo` (see Prefill's doc comment in
 * AppShell.tsx), and a chronological Activity feed across everything
 * below.
 */
function CompanyDetail({
  id,
  onEdit,
  onBack,
  onNavigateTo,
}: {
  id: string;
  onEdit: () => void;
  onBack: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
}) {
  const [tab, setTab] = useState<CompanyTab>("overview");
  const company = useQuery({ queryKey: ["company", id], queryFn: () => api.getCompany(id) });
  const contacts = useQuery({ queryKey: ["contactsByCompany", id], queryFn: () => api.listContactsByCompany(id) });
  const opportunities = useQuery({
    queryKey: ["opportunitiesByCompany", id],
    queryFn: () => api.listOpportunitiesByCompany(id),
  });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const orders = useQuery({ queryKey: ["orders"], queryFn: () => api.listOrders() });
  const invoices = useQuery({ queryKey: ["invoices"], queryFn: () => api.listInvoices() });
  const contracts = useQuery({ queryKey: ["contractsByCompany", id], queryFn: () => api.listContractsByCompany(id) });
  const tasks = useQuery({ queryKey: ["tasksByRelated", "Company", id], queryFn: () => api.listTasksByRelated("Company", id) });

  if (!company.data) return <p>Loading...</p>;

  const relatedContacts = contacts.data ?? [];
  const relatedOpportunities = opportunities.data ?? [];
  const relatedQuotes = (quotes.data ?? []).filter((q) => q.company_id === id);
  const relatedOrders = (orders.data ?? []).filter((o) => o.company_id === id);
  const relatedInvoices = (invoices.data ?? []).filter((i) => i.company_id === id);
  const relatedContracts = contracts.data ?? [];
  const relatedTasks = tasks.data ?? [];

  const kpis: { tab: CompanyTab; label: string; count: number }[] = [
    { tab: "contacts", label: "Contacts", count: relatedContacts.length },
    { tab: "opportunities", label: "Opportunities", count: relatedOpportunities.length },
    { tab: "quotes", label: "Quotes", count: relatedQuotes.length },
    { tab: "orders", label: "Orders", count: relatedOrders.length },
    { tab: "invoices", label: "Invoices", count: relatedInvoices.length },
    { tab: "contracts", label: "Contracts", count: relatedContracts.length },
    { tab: "tasks", label: "Tasks", count: relatedTasks.length },
  ];

  const activity: { at: string; text: string }[] = [
    ...relatedContacts.map((c) => ({ at: c.created_at, text: `Contact ${c.first_name} ${c.last_name} added` })),
    ...relatedOpportunities.map((o) => ({ at: o.created_at, text: `Opportunity ${o.opportunity_number} "${o.name}" created (${o.stage})` })),
    ...relatedQuotes.map((q) => ({ at: q.created_at, text: `Quote ${q.quote_number} created (${q.status})` })),
    ...relatedOrders.map((o) => ({ at: o.created_at, text: `Order ${o.order_number} created (${o.status})` })),
    ...relatedInvoices.map((i) => ({ at: i.created_at, text: `Invoice ${i.invoice_number} created (${i.status})` })),
    ...relatedContracts.map((c) => ({ at: c.created_at, text: `Contract ${c.contract_number} "${c.title}" created (${c.status})` })),
    ...relatedTasks.map((t) => ({ at: t.created_at, text: `Task "${t.title}" created (${t.status})` })),
  ].sort((a, b) => b.at.localeCompare(a.at));

  const goNew = (section: Section) => onNavigateTo?.(section, { companyId: id });

  return (
    <div>
      <div className="toolbar">
        <div>
          <button className="btn" onClick={onBack}>
            ← Back
          </button>
        </div>
        <button className="btn" onClick={onEdit}>
          Edit
        </button>
      </div>
      <h2>
        {company.data.name} <StatusBadge status={company.data.status} />
      </h2>
      <p style={{ color: "var(--text-muted)" }}>{company.data.customer_number}</p>

      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 16 }}>
        {kpis.map((k) => (
          <button
            key={k.tab}
            className={`badge${tab === k.tab ? " badge-success" : ""}`}
            style={{ cursor: "pointer" }}
            onClick={() => setTab(k.tab)}
          >
            {k.label}: {k.count}
          </button>
        ))}
      </div>

      <div className="tab-row">
        {COMPANY_TABS.map((t) => (
          <button key={t.tab} className={`tab${tab === t.tab ? " active" : ""}`} onClick={() => setTab(t.tab)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === "overview" && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginTop: 16 }}>
          <div className="card">
            <h3 style={{ marginTop: 0 }}>Details</h3>
            <p><strong>Tax number:</strong> {company.data.tax_number ?? "—"}</p>
            <p><strong>Billing address:</strong> {company.data.billing_address ?? "—"}</p>
            <p><strong>Shipping address:</strong> {company.data.shipping_address ?? "—"}</p>
            <p><strong>Tags:</strong> {company.data.tags ?? "—"}</p>
            <p><strong>Notes:</strong> {company.data.notes ?? "—"}</p>
          </div>
          <RelatedRecordsCard entityType="Company" entityId={id} />
        </div>
      )}

      {tab === "contacts" && (
        <TabListCard
          title="Contacts"
          newLabel="+ New contact"
          onNew={() => goNew("contacts")}
          rows={relatedContacts}
          columns={["Number", "Name", "Email", "Status"]}
          render={(c) => [c.contact_number, `${c.first_name} ${c.last_name}${c.is_primary ? " (Primary)" : ""}`, c.email ?? "—", c.status]}
        />
      )}
      {tab === "opportunities" && (
        <TabListCard
          title="Opportunities"
          newLabel="+ New opportunity"
          onNew={() => goNew("opportunities")}
          rows={relatedOpportunities}
          columns={["Number", "Name", "Stage", "Value"]}
          render={(o) => [o.opportunity_number, o.name, o.stage, formatCents(o.value_cents, o.currency_code)]}
        />
      )}
      {tab === "quotes" && (
        <TabListCard
          title="Quotes"
          newLabel="+ New quote"
          onNew={() => goNew("quotes")}
          rows={relatedQuotes}
          columns={["Number", "Status", "Total"]}
          render={(q) => [q.quote_number, q.status, formatCents(q.total_cents, q.currency_code)]}
        />
      )}
      {tab === "orders" && (
        <TabListCard
          title="Orders"
          newLabel="+ New order"
          onNew={() => goNew("orders")}
          rows={relatedOrders}
          columns={["Number", "Status", "Total"]}
          render={(o) => [o.order_number, o.status, formatCents(o.total_cents, o.currency_code)]}
        />
      )}
      {tab === "invoices" && (
        <TabListCard
          title="Invoices"
          newLabel="+ New invoice"
          onNew={() => goNew("invoices")}
          rows={relatedInvoices}
          columns={["Number", "Status", "Balance"]}
          render={(i) => [i.invoice_number, i.status, formatCents(i.balance_cents, i.currency_code)]}
        />
      )}
      {tab === "contracts" && (
        <TabListCard
          title="Contracts"
          newLabel="+ New contract"
          onNew={() => goNew("contracts")}
          rows={relatedContracts}
          columns={["Number", "Title", "Status", "Renewal date"]}
          render={(c) => [c.contract_number, c.title, c.status, c.renewal_date ?? "—"]}
        />
      )}
      {tab === "tasks" && (
        <TabListCard
          title="Tasks"
          newLabel="+ New task"
          onNew={() => onNavigateTo?.("tasks", { companyId: id })}
          rows={relatedTasks}
          columns={["Number", "Title", "Status"]}
          render={(t) => [t.task_number, t.title, t.status]}
        />
      )}
      {tab === "activity" && (
        <div className="card" style={{ marginTop: 16 }}>
          <h3 style={{ marginTop: 0 }}>Activity</h3>
          {activity.length === 0 ? (
            <p className="empty-state">Nothing yet</p>
          ) : (
            <ul style={{ margin: 0, paddingLeft: 18, fontSize: 14 }}>
              {activity.map((a, idx) => (
                <li key={idx}>
                  {a.text} <span style={{ color: "var(--text-muted)" }}>— {a.at.slice(0, 10)}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
