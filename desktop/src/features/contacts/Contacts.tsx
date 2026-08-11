import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CsvImportDialog, type ParsedCsvRow } from "../../components/CsvImportDialog";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { RelatedRecordsCard } from "../../components/RelatedRecordsCard";
import { TabListCard } from "../../components/TabListCard";
import { field } from "../../lib/csv";
import type { Prefill, Section } from "../../components/AppShell";
import { formatCents } from "../../lib/money";
import { CONTACT_STATUSES, type Company, type Contact, type ContactInput, type CustomFieldValues } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string } | { mode: "detail"; id: string };

function contactExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (c: Contact) => c.contact_number },
    { label: "First name", get: (c: Contact) => c.first_name },
    { label: "Last name", get: (c: Contact) => c.last_name },
    { label: "Company", get: (c: Contact) => companyNameById.get(c.company_id) ?? "" },
    { label: "Job title", get: (c: Contact) => c.job_title ?? "" },
    { label: "Email", get: (c: Contact) => c.email ?? "" },
    { label: "Phone", get: (c: Contact) => c.phone ?? "" },
    { label: "Mobile", get: (c: Contact) => c.mobile ?? "" },
    { label: "Primary", get: (c: Contact) => (c.is_primary ? "Yes" : "No") },
    { label: "Status", get: (c: Contact) => c.status },
    { label: "Tags", get: (c: Contact) => c.tags ?? "" },
    { label: "Notes", get: (c: Contact) => c.notes ?? "" },
  ];
}

const CONTACT_IMPORT_COLUMNS = [
  { label: "First name", required: true },
  { label: "Last name", required: true },
  { label: "Company", required: true },
  { label: "Job title" },
  { label: "Email" },
  { label: "Phone" },
  { label: "Mobile" },
  { label: "Primary (Yes/No)" },
  { label: "Status" },
  { label: "Tags" },
  { label: "Notes" },
];

function parseContactRow(record: Record<string, string>, companies: Company[]): ParsedCsvRow<ContactInput> {
  const firstName = field(record, "First name");
  const lastName = field(record, "Last name");
  const companyName = field(record, "Company");
  const preview = `${firstName} ${lastName}`.trim() || "(unnamed)";

  if (!firstName || !lastName) return { preview, error: "First name and last name are required" };
  if (!companyName) return { preview, error: "Company is required" };

  const company = companies.find((c) => c.name.toLowerCase() === companyName.toLowerCase());
  if (!company) return { preview, error: `No company named "${companyName}" - create it first` };

  const statusRaw = field(record, "Status");
  const status = statusRaw
    ? CONTACT_STATUSES.find((s) => s.toLowerCase() === statusRaw.toLowerCase())
    : "Active";
  if (!status) return { preview, error: `Unknown status "${statusRaw}"` };

  return {
    preview,
    input: {
      company_id: company.id,
      first_name: firstName,
      last_name: lastName,
      job_title: field(record, "Job title") || null,
      email: field(record, "Email") || null,
      phone: field(record, "Phone") || null,
      mobile: field(record, "Mobile") || null,
      is_primary: field(record, "Primary (Yes/No)", "Primary").toLowerCase() === "yes",
      status,
      tags: field(record, "Tags") || null,
      notes: field(record, "Notes") || null,
    },
  };
}

function emptyInput(companyId: string): ContactInput {
  return {
    company_id: companyId,
    first_name: "",
    last_name: "",
    job_title: null,
    email: null,
    phone: null,
    mobile: null,
    is_primary: false,
    status: "Active",
    tags: null,
    notes: null,
  };
}

export function Contacts({
  prefill,
  onPrefillConsumed,
  onNavigateTo,
}: {
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
} = {}) {
  const [view, setView] = useState<View>(() => (prefill?.companyId ? { mode: "create" } : { mode: "list" }));
  const [importing, setImporting] = useState(false);
  const queryClient = useQueryClient();
  const contacts = useQuery({ queryKey: ["contacts"], queryFn: () => api.listContacts() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  useEffect(() => {
    if (prefill?.companyId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["contacts"] });
  }

  if (view.mode === "create" || view.mode === "edit") {
    return (
      <ContactForm
        contactId={view.mode === "edit" ? view.id : undefined}
        companies={companies.data ?? []}
        initialCompanyId={view.mode === "create" ? prefill?.companyId : undefined}
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
      <ContactDetail
        id={view.id}
        onEdit={() => setView({ mode: "edit", id: view.id })}
        onBack={() => setView({ mode: "list" })}
        onNavigateTo={onNavigateTo}
      />
    );
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Contacts</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={contacts.data ?? []}
            columns={contactExportColumns(companyNameById)}
            filename="contacts.csv"
          />
          <button className="btn" onClick={() => setImporting((v) => !v)}>
            Import CSV
          </button>
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New contact
          </button>
        </div>
      </div>
      {importing && (
        <CsvImportDialog
          title="Import contacts"
          columns={CONTACT_IMPORT_COLUMNS}
          parseRow={(record) => parseContactRow(record, companies.data ?? [])}
          createFn={(input) => api.createContact(input)}
          onImported={invalidate}
          onClose={() => setImporting(false)}
        />
      )}
      {contacts.isLoading && <p>Loading...</p>}
      {contacts.data && contacts.data.length === 0 && (
        <p className="empty-state">No contacts yet. Create your first one.</p>
      )}
      {contacts.data && contacts.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Name</th>
              <th>Company</th>
              <th>Email</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {contacts.data.map((c) => (
              <tr key={c.id} onClick={() => setView({ mode: "detail", id: c.id })} style={{ cursor: "pointer" }}>
                <td>{c.contact_number}</td>
                <td>
                  {c.first_name} {c.last_name} {c.is_primary && <span className="badge">Primary</span>}
                </td>
                <td>{companyNameById.get(c.company_id) ?? "—"}</td>
                <td>{c.email ?? "—"}</td>
                <td>
                  <StatusBadge status={c.status} />
                </td>
                <td>
                  <button
                    className="btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      setView({ mode: "edit", id: c.id });
                    }}
                  >
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function ContactForm({
  contactId,
  companies,
  initialCompanyId,
  onDone,
  onCancel,
}: {
  contactId?: string;
  companies: { id: string; name: string }[];
  initialCompanyId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["contact", contactId],
    queryFn: () => api.getContact(contactId as string),
    enabled: !!contactId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", contactId],
    queryFn: () => api.getCustomFieldValues(contactId as string),
    enabled: !!contactId,
  });
  const [input, setInput] = useState<ContactInput>(emptyInput(initialCompanyId ?? companies[0]?.id ?? ""));
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [duplicateWarning, setDuplicateWarning] = useState<Contact[] | null>(null);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== contactId) {
    const { first_name, last_name, job_title, email, phone, mobile, is_primary, status, tags, notes, company_id } =
      existing.data;
    setInput({ company_id, first_name, last_name, job_title, email, phone, mobile, is_primary, status, tags, notes });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(contactId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const contact = contactId ? await api.updateContact(contactId, input) : await api.createContact(input);
      const ruleMessages = await api.setCustomFieldValues("Contact", contact.id, customValues);
      showRuleMessages(ruleMessages);
      return contact;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the contact"),
  });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (!duplicateWarning && input.email) {
      const duplicates = await api.checkContactDuplicates(input.company_id, input.email, contactId);
      if (duplicates.length > 0) {
        setDuplicateWarning(duplicates);
        return;
      }
    }
    save.mutate();
  }

  return (
    <div>
      <h2>{contactId ? "Edit contact" : "New contact"}</h2>
      {error && <div className="error-banner">{error}</div>}
      {duplicateWarning && (
        <div className="error-banner" style={{ borderColor: "var(--warning)", color: "var(--warning)" }}>
          A contact with this email already exists at this company. Submit again to save anyway.
        </div>
      )}
      <form className="form-grid" onSubmit={handleSubmit}>
        <div className="form-field full">
          <label>Company</label>
          <select
            value={input.company_id}
            onChange={(e) => setInput({ ...input, company_id: e.target.value })}
            required
          >
            {companies.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>First name</label>
          <input value={input.first_name} onChange={(e) => setInput({ ...input, first_name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Last name</label>
          <input value={input.last_name} onChange={(e) => setInput({ ...input, last_name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Job title</label>
          <input value={input.job_title ?? ""} onChange={(e) => setInput({ ...input, job_title: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Email</label>
          <input
            type="email"
            value={input.email ?? ""}
            onChange={(e) => {
              setDuplicateWarning(null);
              setInput({ ...input, email: e.target.value || null });
            }}
          />
        </div>
        <div className="form-field">
          <label>Status</label>
          <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
            {CONTACT_STATUSES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={input.is_primary}
              onChange={(e) => setInput({ ...input, is_primary: e.target.checked })}
            />
            Primary contact
          </label>
        </div>
        <CustomFieldsSection entityType="Contact" status={input.status} values={customValues} onChange={setCustomValues} />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending || !input.company_id}>
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

type ContactTab = "overview" | "opportunities" | "quotes" | "orders" | "invoices" | "tasks" | "activity";
const CONTACT_TABS: { tab: ContactTab; label: string }[] = [
  { tab: "overview", label: "Overview" },
  { tab: "opportunities", label: "Opportunities" },
  { tab: "quotes", label: "Quotes" },
  { tab: "orders", label: "Orders" },
  { tab: "invoices", label: "Invoices" },
  { tab: "tasks", label: "Tasks" },
  { tab: "activity", label: "Activity" },
];

/**
 * Addendum Phase 5 (Contact 360, spec §5): a tabbed record view replacing
 * the plain list-only Contacts screen (there was no detail view at all
 * before this) - related opportunities/quotes/orders/invoices/tasks for
 * this contact, a clickable KPI strip that jumps straight to the matching
 * tab, "+ New" from each tab pre-filling the relationship via
 * `onNavigateTo` (see Prefill's doc comment in AppShell.tsx), and a
 * chronological Activity feed across everything below.
 */
function ContactDetail({
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
  const [tab, setTab] = useState<ContactTab>("overview");
  const contact = useQuery({ queryKey: ["contact", id], queryFn: () => api.getContact(id) });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const opportunities = useQuery({ queryKey: ["opportunities"], queryFn: () => api.listOpportunities() });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const orders = useQuery({ queryKey: ["orders"], queryFn: () => api.listOrders() });
  const invoices = useQuery({ queryKey: ["invoices"], queryFn: () => api.listInvoices() });
  const tasks = useQuery({ queryKey: ["tasksByRelated", "Contact", id], queryFn: () => api.listTasksByRelated("Contact", id) });

  if (!contact.data) return <p>Loading...</p>;
  const c = contact.data;
  const companyName = companies.data?.find((co) => co.id === c.company_id)?.name;

  const relatedOpportunities = (opportunities.data ?? []).filter((o) => o.primary_contact_id === id);
  const relatedQuotes = (quotes.data ?? []).filter((q) => q.contact_id === id);
  const relatedOrders = (orders.data ?? []).filter((o) => o.contact_id === id);
  const relatedInvoices = (invoices.data ?? []).filter((i) => i.contact_id === id);
  const relatedTasks = tasks.data ?? [];

  const goNew = (section: Section) => onNavigateTo?.(section, { companyId: c.company_id, contactId: c.id });

  const kpis: { tab: ContactTab; label: string; count: number }[] = [
    { tab: "opportunities", label: "Opportunities", count: relatedOpportunities.length },
    { tab: "quotes", label: "Quotes", count: relatedQuotes.length },
    { tab: "orders", label: "Orders", count: relatedOrders.length },
    { tab: "invoices", label: "Invoices", count: relatedInvoices.length },
    { tab: "tasks", label: "Tasks", count: relatedTasks.length },
  ];

  const activity: { at: string; text: string }[] = [
    ...relatedOpportunities.map((o) => ({ at: o.created_at, text: `Opportunity ${o.opportunity_number} "${o.name}" created (${o.stage})` })),
    ...relatedQuotes.map((q) => ({ at: q.created_at, text: `Quote ${q.quote_number} created (${q.status})` })),
    ...relatedOrders.map((o) => ({ at: o.created_at, text: `Order ${o.order_number} created (${o.status})` })),
    ...relatedInvoices.map((i) => ({ at: i.created_at, text: `Invoice ${i.invoice_number} created (${i.status})` })),
    ...relatedTasks.map((t) => ({ at: t.created_at, text: `Task "${t.title}" created (${t.status})` })),
  ].sort((a, b) => b.at.localeCompare(a.at));

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
        <button className="btn" onClick={onEdit}>
          Edit
        </button>
      </div>
      <h2>
        {c.first_name} {c.last_name} <StatusBadge status={c.status} />
      </h2>
      <p style={{ color: "var(--text-muted)" }}>
        {c.contact_number}
        {companyName ? ` · ${companyName}` : ""}
        {c.job_title ? ` · ${c.job_title}` : ""}
      </p>

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
        {CONTACT_TABS.map((t) => (
          <button key={t.tab} className={`tab${tab === t.tab ? " active" : ""}`} onClick={() => setTab(t.tab)}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === "overview" && (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginTop: 16 }}>
          <div className="card">
            <h3 style={{ marginTop: 0 }}>Details</h3>
            <p><strong>Email:</strong> {c.email ?? "—"}</p>
            <p><strong>Phone:</strong> {c.phone ?? "—"}</p>
            <p><strong>Mobile:</strong> {c.mobile ?? "—"}</p>
            <p><strong>Tags:</strong> {c.tags ?? "—"}</p>
            <p><strong>Notes:</strong> {c.notes ?? "—"}</p>
          </div>
          <RelatedRecordsCard entityType="Contact" entityId={id} />
        </div>
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
      {tab === "tasks" && (
        <TabListCard
          title="Tasks"
          newLabel="+ New task"
          onNew={() => onNavigateTo?.("tasks", { contactId: c.id })}
          rows={relatedTasks}
          columns={["Number", "Title", "Status"]}
          render={(t) => [t.task_number, t.title, t.status]}
        />
      )}
      {tab === "activity" && (
        <div className="card">
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

