import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CsvImportDialog, type ParsedCsvRow } from "../../components/CsvImportDialog";
import { field } from "../../lib/csv";
import { CONTACT_STATUSES, type Company, type Contact, type ContactInput } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

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

export function Contacts() {
  const [view, setView] = useState<View>({ mode: "list" });
  const [importing, setImporting] = useState(false);
  const queryClient = useQueryClient();
  const contacts = useQuery({ queryKey: ["contacts"], queryFn: () => api.listContacts() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["contacts"] });
  }

  if (view.mode !== "list") {
    return (
      <ContactForm
        contactId={view.mode === "edit" ? view.id : undefined}
        companies={companies.data ?? []}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
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
              <tr key={c.id}>
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
                  <button className="btn" onClick={() => setView({ mode: "edit", id: c.id })}>
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
  onDone,
  onCancel,
}: {
  contactId?: string;
  companies: { id: string; name: string }[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["contact", contactId],
    queryFn: () => api.getContact(contactId as string),
    enabled: !!contactId,
  });
  const [input, setInput] = useState<ContactInput>(emptyInput(companies[0]?.id ?? ""));
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [duplicateWarning, setDuplicateWarning] = useState<Contact[] | null>(null);

  if (existing.data && loadedFor !== contactId) {
    const { first_name, last_name, job_title, email, phone, mobile, is_primary, status, tags, notes, company_id } =
      existing.data;
    setInput({ company_id, first_name, last_name, job_title, email, phone, mobile, is_primary, status, tags, notes });
    setLoadedFor(contactId);
  }

  const save = useMutation({
    mutationFn: () => (contactId ? api.updateContact(contactId, input) : api.createContact(input)),
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
