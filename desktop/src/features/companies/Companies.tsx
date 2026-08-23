import { Fragment, useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents, centsToInputValue, parseDecimalToCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CsvImportDialog, type ParsedCsvRow } from "../../components/CsvImportDialog";
import { useCustomFieldElements } from "../../components/CustomFieldsSection";
import { LayoutFormFields } from "../../components/LayoutFormFields";
import { LayoutDetailFields } from "../../components/LayoutDetailFields";
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import { RelatedRecordsCard } from "../../components/RelatedRecordsCard";
import { AuditByline, AuditTrail } from "../../components/AuditTrail";
import { TabListCard } from "../../components/TabListCard";
import { SavedViewBar } from "../../components/SavedViewBar";
import { BulkActionBar, type BulkAction } from "../../components/BulkActionBar";
import { GroupHeaderRow } from "../../components/GroupHeaderRow";
import type { Prefill, Section } from "../../components/AppShell";
import { field } from "../../lib/csv";
import { useSavedViews } from "../../lib/useSavedViews";
import { useBulkSelection } from "../../lib/useBulkSelection";
import { useCanWriteObject } from "../../lib/useCanWriteObject";
import { COMPANY_STATUSES, PREFERRED_CONTACT_METHODS, type Company, type CompanyInput, type CustomFieldValues } from "../../lib/types";

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
  { label: "Phone", get: (c: Company) => c.phone ?? "" },
  { label: "Email", get: (c: Company) => c.email ?? "" },
  { label: "Website", get: (c: Company) => c.website ?? "" },
  { label: "Annual revenue", get: (c: Company) => (c.annual_revenue_cents === null ? "" : (c.annual_revenue_cents / 100).toFixed(2)) },
  { label: "Employees", get: (c: Company) => (c.employee_count === null ? "" : String(c.employee_count)) },
  { label: "Preferred contact method", get: (c: Company) => c.preferred_contact_method ?? "" },
];

const COMPANY_IMPORT_COLUMNS = [
  { label: "Name", required: true },
  { label: "Status" },
  { label: "Tax number" },
  { label: "Billing address" },
  { label: "Shipping address" },
  { label: "Tags" },
  { label: "Notes" },
  { label: "Phone" },
  { label: "Email" },
  { label: "Website" },
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
      phone: field(record, "Phone") || null,
      email: field(record, "Email") || null,
      website: field(record, "Website") || null,
      annual_revenue_cents: null,
      employee_count: null,
      preferred_contact_method: null,
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
  phone: null,
  email: null,
  website: null,
  annual_revenue_cents: null,
  employee_count: null,
  preferred_contact_method: null,
};

export function Companies({
  prefill,
  onPrefillConsumed,
  onNavigateTo,
}: {
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
} = {}) {
  const [view, setView] = useState<View>(() => (prefill?.openId ? { mode: "detail", id: prefill.openId } : { mode: "list" }));
  const [importing, setImporting] = useState(false);
  const queryClient = useQueryClient();
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const views = useSavedViews("Company");
  const fieldFilters = views.filters;
  const canWrite = useCanWriteObject("Company");
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  const filteredRows = (companies.data ?? []).filter((c) => fieldFilters.matches(c.id));
  const selection = useBulkSelection(filteredRows, (c) => c.id);

  function companyFieldValue(row: Company, key: string): string {
    switch (key) {
      case "name":
        return row.name;
      case "status":
        return row.status;
      case "tax_number":
        return row.tax_number ?? "";
      default:
        return fieldFilters.values[row.id]?.[key] ?? "";
    }
  }

  const bulkActions: BulkAction[] = [
    {
      key: "status",
      label: "Change status",
      valueOptions: COMPANY_STATUSES.map((s) => ({ key: s, label: s })),
      run: (ids, value) => api.bulkChangeStatus("Company", ids, value),
    },
    {
      key: "owner",
      label: "Reassign owner",
      valueOptions: (users.data ?? []).map((u) => ({ key: u.id, label: u.display_name })),
      run: (ids, value) => api.bulkReassignOwner("Company", ids, value),
    },
    {
      key: "tag",
      label: "Add tag",
      valuePlaceholder: "Tag name",
      run: (ids, value) => api.bulkUpdateTags("Company", ids, [value], true),
    },
    {
      key: "archive",
      label: "Archive",
      confirmMessage: "Archive {n} selected companies?",
      run: (ids) => api.bulkArchive("Company", ids),
    },
  ];

  useEffect(() => {
    if (prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
          <button
            className="btn btn-primary"
            onClick={() => setView({ mode: "create" })}
            disabled={!canWrite}
            title={canWrite ? undefined : "You have view-only access to Companies through an app"}
          >
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
      <SavedViewBar
        views={views}
        fields={[
          { key: "name", label: "Name" },
          { key: "status", label: "Status" },
          { key: "tax_number", label: "Tax number" },
        ]}
      />
      <CustomFieldFilterBar filters={fieldFilters} />
      <BulkActionBar selection={selection} actions={bulkActions} onDone={invalidate} />
      {companies.isLoading && <p>Loading...</p>}
      {companies.data && companies.data.length === 0 && (
        <p className="empty-state">No companies yet. Create your first one.</p>
      )}
      {companies.data && companies.data.length > 0 && (() => {
        const groups = views.transform(filteredRows, companyFieldValue);
        return filteredRows.length === 0 ? (
          <p className="empty-state">No companies match the current filters.</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th style={{ width: 28 }}>
                  <input type="checkbox" checked={selection.allSelected} ref={(el) => el && (el.indeterminate = selection.someSelected)} onChange={selection.toggleAll} />
                </th>
                <th>Number</th>
                <th>Name</th>
                <th>Status</th>
                <th>Tax number</th>
              </tr>
            </thead>
            <tbody>
              {groups.map((group) => (
                <Fragment key={group.label || "_"}>
                  {views.groupByField && <GroupHeaderRow label={group.label} colSpan={5} />}
                  {group.rows.map((c) => (
                    <tr key={c.id} style={{ cursor: "pointer" }}>
                      <td onClick={(e) => e.stopPropagation()}>
                        <input type="checkbox" checked={selection.isSelected(c.id)} onChange={() => selection.toggle(c.id)} />
                      </td>
                      <td onClick={() => setView({ mode: "detail", id: c.id })}>
                        <span className="id-link">{c.customer_number}</span>
                      </td>
                      <td onClick={() => setView({ mode: "detail", id: c.id })}>{c.name}</td>
                      <td onClick={() => setView({ mode: "detail", id: c.id })}>
                        <StatusBadge status={c.status} />
                      </td>
                      <td onClick={() => setView({ mode: "detail", id: c.id })}>{c.tax_number ?? "—"}</td>
                    </tr>
                  ))}
                </Fragment>
              ))}
            </tbody>
          </table>
        );
      })()}
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
      phone: existing.data.phone,
      email: existing.data.email,
      website: existing.data.website,
      annual_revenue_cents: existing.data.annual_revenue_cents,
      employee_count: existing.data.employee_count,
      preferred_contact_method: existing.data.preferred_contact_method,
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

  const { order: customFieldOrder, elements: customFieldElements } = useCustomFieldElements({
    entityType: "Company",
    status: input.status,
    values: customValues,
    onChange: setCustomValues,
  });

  // Screen/App Builder Phase 3: places related-records lists into the
  // create/edit form itself, tab-scoped per the effective layout - a new
  // capability for Companies, which previously only showed related
  // records on the (separate) detail page below.
  const relationshipDefs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });
  const relatedKeys = (relationshipDefs.data ?? [])
    .filter((d) => d.show_related_list && (d.source_entity_type === "Company" || d.target_entity_type === "Company"))
    .map((d) => d.key);

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
        <LayoutFormFields
          entityType="Company"
          order={[
            "name", "status", "tax_number", "phone", "email", "website",
            "annual_revenue_cents", "employee_count", "preferred_contact_method",
            "billing_address", "notes", ...customFieldOrder,
          ]}
          entityId={companyId}
          relatedKeys={relatedKeys}
          fields={{
            name: (
              <div className="form-field full" key="name">
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
            ),
            status: (
              <div className="form-field" key="status">
                <label>Status</label>
                <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
                  {COMPANY_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </div>
            ),
            tax_number: (
              <div className="form-field" key="tax_number">
                <label>Tax number</label>
                <input value={input.tax_number ?? ""} onChange={(e) => setInput({ ...input, tax_number: e.target.value || null })} />
              </div>
            ),
            phone: (
              <div className="form-field" key="phone">
                <label>Phone</label>
                <input value={input.phone ?? ""} onChange={(e) => setInput({ ...input, phone: e.target.value || null })} />
              </div>
            ),
            email: (
              <div className="form-field" key="email">
                <label>Email</label>
                <input type="email" value={input.email ?? ""} onChange={(e) => setInput({ ...input, email: e.target.value || null })} />
              </div>
            ),
            website: (
              <div className="form-field" key="website">
                <label>Website</label>
                <input value={input.website ?? ""} onChange={(e) => setInput({ ...input, website: e.target.value || null })} />
              </div>
            ),
            annual_revenue_cents: (
              <div className="form-field" key="annual_revenue_cents">
                <label>Annual revenue</label>
                <input
                  type="number"
                  min="0"
                  step="0.01"
                  value={input.annual_revenue_cents === null ? "" : centsToInputValue(input.annual_revenue_cents)}
                  onChange={(e) =>
                    setInput({ ...input, annual_revenue_cents: e.target.value === "" ? null : parseDecimalToCents(e.target.value) })
                  }
                />
              </div>
            ),
            employee_count: (
              <div className="form-field" key="employee_count">
                <label>Employees</label>
                <input
                  type="number"
                  min="0"
                  step="1"
                  value={input.employee_count ?? ""}
                  onChange={(e) => setInput({ ...input, employee_count: e.target.value === "" ? null : Number(e.target.value) })}
                />
              </div>
            ),
            preferred_contact_method: (
              <div className="form-field" key="preferred_contact_method">
                <label>Preferred contact method</label>
                <select
                  value={input.preferred_contact_method ?? ""}
                  onChange={(e) => setInput({ ...input, preferred_contact_method: e.target.value || null })}
                >
                  <option value="">— Unspecified —</option>
                  {PREFERRED_CONTACT_METHODS.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </div>
            ),
            billing_address: (
              <div className="form-field full" key="billing_address">
                <label>Billing address</label>
                <input value={input.billing_address ?? ""} onChange={(e) => setInput({ ...input, billing_address: e.target.value || null })} />
              </div>
            ),
            notes: (
              <div className="form-field full" key="notes">
                <label>Notes</label>
                <textarea value={input.notes ?? ""} onChange={(e) => setInput({ ...input, notes: e.target.value || null })} />
              </div>
            ),
            ...customFieldElements,
          }}
        />
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
  const canWrite = useCanWriteObject("Company");
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
        <button className="btn" onClick={onEdit} disabled={!canWrite} title={canWrite ? undefined : "You have view-only access to Companies through an app"}>
          Edit
        </button>
      </div>
      <h2>
        {company.data.name} <StatusBadge status={company.data.status} />
      </h2>
      <p style={{ color: "var(--text-muted)" }}>{company.data.customer_number}</p>
      <AuditByline
        createdAt={company.data.created_at}
        createdBy={company.data.created_by}
        updatedAt={company.data.updated_at}
        updatedBy={company.data.updated_by}
      />

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
            <div className="form-grid">
              <LayoutDetailFields
                entityType="Company"
                order={[
                  "phone", "email", "website", "annual_revenue_cents", "employee_count",
                  "preferred_contact_method", "tax_number", "billing_address", "shipping_address", "tags", "notes",
                ]}
                fields={{
                  phone: (
                    <div className="form-field" key="phone">
                      <label>Phone</label>
                      <div>{company.data.phone ?? "—"}</div>
                    </div>
                  ),
                  email: (
                    <div className="form-field" key="email">
                      <label>Email</label>
                      <div>{company.data.email ?? "—"}</div>
                    </div>
                  ),
                  website: (
                    <div className="form-field" key="website">
                      <label>Website</label>
                      <div>{company.data.website ?? "—"}</div>
                    </div>
                  ),
                  annual_revenue_cents: (
                    <div className="form-field" key="annual_revenue_cents">
                      <label>Annual revenue</label>
                      <div>{company.data.annual_revenue_cents === null ? "—" : formatCents(company.data.annual_revenue_cents)}</div>
                    </div>
                  ),
                  employee_count: (
                    <div className="form-field" key="employee_count">
                      <label>Employees</label>
                      <div>{company.data.employee_count ?? "—"}</div>
                    </div>
                  ),
                  preferred_contact_method: (
                    <div className="form-field" key="preferred_contact_method">
                      <label>Preferred contact method</label>
                      <div>{company.data.preferred_contact_method ?? "—"}</div>
                    </div>
                  ),
                  tax_number: (
                    <div className="form-field" key="tax_number">
                      <label>Tax number</label>
                      <div>{company.data.tax_number ?? "—"}</div>
                    </div>
                  ),
                  billing_address: (
                    <div className="form-field full" key="billing_address">
                      <label>Billing address</label>
                      <div>{company.data.billing_address ?? "—"}</div>
                    </div>
                  ),
                  shipping_address: (
                    <div className="form-field full" key="shipping_address">
                      <label>Shipping address</label>
                      <div>{company.data.shipping_address ?? "—"}</div>
                    </div>
                  ),
                  tags: (
                    <div className="form-field full" key="tags">
                      <label>Tags</label>
                      <div>{company.data.tags ?? "—"}</div>
                    </div>
                  ),
                  notes: (
                    <div className="form-field full" key="notes">
                      <label>Notes</label>
                      <div>{company.data.notes ?? "—"}</div>
                    </div>
                  ),
                }}
              />
            </div>
          </div>
          <RelatedRecordsCard entityType="Company" entityId={id} />
          <AuditTrail entityType="Company" entityId={id} />
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
