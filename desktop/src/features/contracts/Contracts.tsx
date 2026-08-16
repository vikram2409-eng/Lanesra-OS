import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { useCustomFieldElements } from "../../components/CustomFieldsSection";
import { LayoutFormFields } from "../../components/LayoutFormFields";
import { CustomFieldsCard } from "../../components/CustomFieldsCard";
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import { RelatedRecordSummary } from "../../components/RelatedRecordSummary";
import { TabListCard } from "../../components/TabListCard";
import type { Prefill, Section } from "../../components/AppShell";
import { CONTRACT_STATUSES, type Contract, type ContractInput, type CustomFieldValues } from "../../lib/types";
import { useCustomFieldFilters } from "../../lib/useCustomFieldFilters";
import { useCanWriteObject } from "../../lib/useCanWriteObject";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string } | { mode: "detail"; id: string };

function contractExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (c: Contract) => c.contract_number },
    { label: "Title", get: (c: Contract) => c.title },
    { label: "Company", get: (c: Contract) => companyNameById.get(c.company_id) ?? "" },
    { label: "Status", get: (c: Contract) => c.status },
    { label: "Value (cents)", get: (c: Contract) => String(c.value_cents) },
    { label: "Currency", get: (c: Contract) => c.currency_code },
    { label: "Start date", get: (c: Contract) => c.start_date ?? "" },
    { label: "End date", get: (c: Contract) => c.end_date ?? "" },
    { label: "Renewal date", get: (c: Contract) => c.renewal_date ?? "" },
    { label: "Notes", get: (c: Contract) => c.notes ?? "" },
  ];
}

function emptyInput(companyId: string, currency: string): ContractInput {
  return {
    company_id: companyId,
    contact_id: null,
    source_quote_id: null,
    title: "",
    type: null,
    value_cents: 0,
    currency_code: currency,
    owner_user_id: null,
    start_date: null,
    end_date: null,
    renewal_date: null,
    notice_period_days: 30,
    status: "Draft",
    notes: null,
  };
}

function isRenewingSoon(renewalDate: string | null): boolean {
  if (!renewalDate) return false;
  const days = (new Date(renewalDate).getTime() - Date.now()) / (1000 * 60 * 60 * 24);
  return days >= 0 && days <= 90;
}

export function Contracts({
  prefill,
  onPrefillConsumed,
  onNavigateTo,
}: {
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
} = {}) {
  const [view, setView] = useState<View>(() =>
    prefill?.openId ? { mode: "detail", id: prefill.openId } : prefill?.companyId ? { mode: "create" } : { mode: "list" }
  );
  const queryClient = useQueryClient();
  const contracts = useQuery({ queryKey: ["contracts"], queryFn: () => api.listContracts() });
  const fieldFilters = useCustomFieldFilters("Contract");
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const canWrite = useCanWriteObject("Contract");

  useEffect(() => {
    if (prefill?.companyId || prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["contracts"] });
  }

  if (view.mode === "create" || view.mode === "edit") {
    return (
      <ContractForm
        contractId={view.mode === "edit" ? view.id : undefined}
        companies={companies.data ?? []}
        initialCompanyId={view.mode === "create" ? prefill?.companyId : undefined}
        initialContactId={view.mode === "create" ? prefill?.contactId : undefined}
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
      <ContractDetail
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
        <h2 style={{ margin: 0 }}>Contracts</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={contracts.data ?? []}
            columns={contractExportColumns(companyNameById)}
            filename="contracts.csv"
          />
          <button
            className="btn btn-primary"
            onClick={() => setView({ mode: "create" })}
            disabled={!canWrite}
            title={canWrite ? undefined : "You have view-only access to Contracts through an app"}
          >
            + New contract
          </button>
        </div>
      </div>
      <CustomFieldFilterBar filters={fieldFilters} />
      {contracts.isLoading && <p>Loading...</p>}
      {contracts.data && contracts.data.length === 0 && <p className="empty-state">No contracts yet.</p>}
      {contracts.data && contracts.data.length > 0 && (() => {
        const rows = contracts.data.filter((c) => fieldFilters.matches(c.id));
        return rows.length === 0 ? (
          <p className="empty-state">No contracts match the current filters.</p>
        ) : (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Title</th>
              <th>Company</th>
              <th>Status</th>
              <th>Value</th>
              <th>Renewal date</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((c) => (
              <tr key={c.id} onClick={() => setView({ mode: "detail", id: c.id })} style={{ cursor: "pointer" }}>
                <td><span className="id-link">{c.contract_number}</span></td>
                <td>{c.title}</td>
                <td>{companyNameById.get(c.company_id) ?? "—"}</td>
                <td>
                  <StatusBadge status={c.status} />
                </td>
                <td>{formatCents(c.value_cents, c.currency_code)}</td>
                <td>
                  {c.renewal_date ?? "—"}
                  {isRenewingSoon(c.renewal_date) && <span className="badge badge-warning" style={{ marginLeft: 6 }}>Renewing soon</span>}
                </td>
                <td>
                  <button
                    className="btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      setView({ mode: "edit", id: c.id });
                    }}
                    disabled={!canWrite}
                    title={canWrite ? undefined : "You have view-only access to Contracts through an app"}
                  >
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        );
      })()}
    </div>
  );
}

function ContractForm({
  contractId,
  companies,
  initialCompanyId,
  initialContactId,
  onDone,
  onCancel,
}: {
  contractId?: string;
  companies: { id: string; name: string }[];
  initialCompanyId?: string;
  initialContactId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["contract", contractId],
    queryFn: () => api.getContract(contractId as string),
    enabled: !!contractId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", contractId],
    queryFn: () => api.getCustomFieldValues(contractId as string),
    enabled: !!contractId,
  });
  const [input, setInput] = useState<ContractInput>(() => {
    const base = emptyInput(initialCompanyId ?? companies[0]?.id ?? "", "USD");
    return initialContactId ? { ...base, contact_id: initialContactId } : base;
  });
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  const contacts = useQuery({
    queryKey: ["contactsByCompany", input.company_id],
    queryFn: () => api.listContactsByCompany(input.company_id),
    enabled: !!input.company_id,
  });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const companyQuotes = (quotes.data ?? []).filter((q) => q.company_id === input.company_id);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== contractId) {
    const {
      company_id,
      contact_id,
      source_quote_id,
      title,
      type,
      value_cents,
      currency_code,
      owner_user_id,
      start_date,
      end_date,
      renewal_date,
      notice_period_days,
      status,
      notes,
    } = existing.data;
    setInput({
      company_id,
      contact_id,
      source_quote_id,
      title,
      type,
      value_cents,
      currency_code,
      owner_user_id,
      start_date,
      end_date,
      renewal_date,
      notice_period_days,
      status,
      notes,
    });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(contractId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const contract = contractId ? await api.updateContract(contractId, input) : await api.createContract(input);
      const ruleMessages = await api.setCustomFieldValues("Contract", contract.id, customValues);
      showRuleMessages(ruleMessages);
      return contract;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the contract"),
  });

  const { order: customFieldOrder, elements: customFieldElements } = useCustomFieldElements({
    entityType: "Contract",
    status: input.status,
    values: customValues,
    onChange: setCustomValues,
  });

  return (
    <div>
      <h2>{contractId ? "Edit contract" : "New contract"}</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <LayoutFormFields
          entityType="Contract"
          order={[
            "title", "company_id", "contact_id", "source_quote_id", "type", "status", "value_cents",
            "notice_period_days", "start_date", "end_date", "renewal_date", "notes", ...customFieldOrder,
          ]}
          fields={{
            title: (
              <div className="form-field full" key="title">
                <label>Title</label>
                <input value={input.title} onChange={(e) => setInput({ ...input, title: e.target.value })} required />
              </div>
            ),
            company_id: (
              <div className="form-field" key="company_id">
                <label>Company</label>
                <select
                  value={input.company_id}
                  onChange={(e) => setInput({ ...input, company_id: e.target.value, contact_id: null, source_quote_id: null })}
                  required
                >
                  {companies.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
            ),
            contact_id: (
              <div className="form-field" key="contact_id">
                <label>Contact (optional)</label>
                <select value={input.contact_id ?? ""} onChange={(e) => setInput({ ...input, contact_id: e.target.value || null })}>
                  <option value="">— None —</option>
                  {(contacts.data ?? []).map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.first_name} {c.last_name}
                    </option>
                  ))}
                </select>
              </div>
            ),
            source_quote_id: (
              <div className="form-field" key="source_quote_id">
                <label>Source quote (optional)</label>
                <select
                  value={input.source_quote_id ?? ""}
                  onChange={(e) => setInput({ ...input, source_quote_id: e.target.value || null })}
                >
                  <option value="">— None —</option>
                  {companyQuotes.map((q) => (
                    <option key={q.id} value={q.id}>
                      {q.quote_number}
                    </option>
                  ))}
                </select>
              </div>
            ),
            type: (
              <div className="form-field" key="type">
                <label>Type</label>
                <input value={input.type ?? ""} onChange={(e) => setInput({ ...input, type: e.target.value || null })} />
              </div>
            ),
            status: (
              <div className="form-field" key="status">
                <label>Status</label>
                <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
                  {CONTRACT_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </div>
            ),
            value_cents: (
              <div className="form-field" key="value_cents">
                <label>Value</label>
                <input
                  type="number"
                  step="0.01"
                  value={(input.value_cents / 100).toFixed(2)}
                  onChange={(e) => setInput({ ...input, value_cents: Math.round(parseFloat(e.target.value || "0") * 100) })}
                />
              </div>
            ),
            notice_period_days: (
              <div className="form-field" key="notice_period_days">
                <label>Notice period (days)</label>
                <input
                  type="number"
                  value={input.notice_period_days ?? ""}
                  onChange={(e) => setInput({ ...input, notice_period_days: e.target.value ? Number(e.target.value) : null })}
                />
              </div>
            ),
            start_date: (
              <div className="form-field" key="start_date">
                <label>Start date</label>
                <input type="date" value={input.start_date ?? ""} onChange={(e) => setInput({ ...input, start_date: e.target.value || null })} />
              </div>
            ),
            end_date: (
              <div className="form-field" key="end_date">
                <label>End date</label>
                <input type="date" value={input.end_date ?? ""} onChange={(e) => setInput({ ...input, end_date: e.target.value || null })} />
              </div>
            ),
            renewal_date: (
              <div className="form-field" key="renewal_date">
                <label>Renewal date</label>
                <input
                  type="date"
                  value={input.renewal_date ?? ""}
                  onChange={(e) => setInput({ ...input, renewal_date: e.target.value || null })}
                />
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
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

/** Record-detail-page round: Contracts previously had no detail view -
 * list row click and menu went straight to Edit, and there was no link
 * back to the Company/Contact/source quote it belongs to. */
function ContractDetail({
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
  const contract = useQuery({ queryKey: ["contract", id], queryFn: () => api.getContract(id) });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const contacts = useQuery({ queryKey: ["contacts"], queryFn: () => api.listContacts() });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const tasks = useQuery({ queryKey: ["tasksByRelated", "Contract", id], queryFn: () => api.listTasksByRelated("Contract", id) });
  const canWrite = useCanWriteObject("Contract");

  if (!contract.data) return <p>Loading...</p>;
  const c = contract.data;

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
        <button
          className="btn"
          onClick={onEdit}
          disabled={!canWrite}
          title={canWrite ? undefined : "You have view-only access to Contracts through an app"}
        >
          Edit
        </button>
      </div>
      <h2>
        {c.title} <StatusBadge status={c.status} />
        {isRenewingSoon(c.renewal_date) && <span className="badge badge-warning" style={{ marginLeft: 6 }}>Renewing soon</span>}
      </h2>
      <p style={{ color: "var(--text-muted)" }}>{c.contract_number}</p>

      <RelatedRecordSummary
        companyId={c.company_id}
        contactId={c.contact_id}
        companies={companies.data}
        contacts={contacts.data}
        onNavigateTo={onNavigateTo}
        relatedLists={[
          c.source_quote_id
            ? {
                title: "Source quote",
                rows: [quotes.data?.find((q) => q.id === c.source_quote_id)].filter(
                  (q): q is NonNullable<typeof q> => !!q,
                ),
                render: (q) => q.quote_number,
                onOpen: (q) => onNavigateTo?.("quotes", { openId: q.id }),
              }
            : null,
        ]}
      />

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Details</h3>
        <p><strong>Type:</strong> {c.type ?? "—"}</p>
        <p><strong>Value:</strong> {formatCents(c.value_cents, c.currency_code)}</p>
        <p><strong>Start date:</strong> {c.start_date ?? "—"}</p>
        <p><strong>End date:</strong> {c.end_date ?? "—"}</p>
        <p><strong>Renewal date:</strong> {c.renewal_date ?? "—"}</p>
        <p><strong>Notice period:</strong> {c.notice_period_days !== null ? `${c.notice_period_days} days` : "—"}</p>
        <p><strong>Notes:</strong> {c.notes ?? "—"}</p>
      </div>

      {/* No "+ New task" here (unlike Company/Contact's Tasks tabs) - Tasks'
          create-form prefill only knows how to relate a new task to a
          Company or Contact today, and pointing it at either instead of
          this Contract would create a misleadingly-related task. */}
      <TabListCard
        title="Tasks"
        newLabel="+ New task"
        rows={tasks.data ?? []}
        columns={["Number", "Title", "Status"]}
        render={(t) => [t.task_number, t.title, t.status]}
      />

      <div style={{ marginTop: 16 }}>
        <CustomFieldsCard entityType="Contract" entityId={c.id} status={c.status} />
      </div>
    </div>
  );
}
