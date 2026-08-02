import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { StatusBadge } from "../../components/StatusBadge";
import { CONTACT_STATUSES, type Contact, type ContactInput } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

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
        <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
          + New contact
        </button>
      </div>
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
