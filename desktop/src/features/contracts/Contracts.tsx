import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { CONTRACT_STATUSES, type Contract, type ContractInput, type CustomFieldValues } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

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

export function Contracts() {
  const [view, setView] = useState<View>({ mode: "list" });
  const queryClient = useQueryClient();
  const contracts = useQuery({ queryKey: ["contracts"], queryFn: () => api.listContracts() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["contracts"] });
  }

  if (view.mode !== "list") {
    return (
      <ContractForm
        contractId={view.mode === "edit" ? view.id : undefined}
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
        <h2 style={{ margin: 0 }}>Contracts</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={contracts.data ?? []}
            columns={contractExportColumns(companyNameById)}
            filename="contracts.csv"
          />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New contract
          </button>
        </div>
      </div>
      {contracts.isLoading && <p>Loading...</p>}
      {contracts.data && contracts.data.length === 0 && <p className="empty-state">No contracts yet.</p>}
      {contracts.data && contracts.data.length > 0 && (
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
            {contracts.data.map((c) => (
              <tr key={c.id}>
                <td>{c.contract_number}</td>
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

function ContractForm({
  contractId,
  companies,
  onDone,
  onCancel,
}: {
  contractId?: string;
  companies: { id: string; name: string }[];
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
  const [input, setInput] = useState<ContractInput>(emptyInput(companies[0]?.id ?? "", "USD"));
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
        <div className="form-field full">
          <label>Title</label>
          <input value={input.title} onChange={(e) => setInput({ ...input, title: e.target.value })} required />
        </div>
        <div className="form-field">
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
        <div className="form-field">
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
        <div className="form-field">
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
        <div className="form-field">
          <label>Type</label>
          <input value={input.type ?? ""} onChange={(e) => setInput({ ...input, type: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Status</label>
          <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
            {CONTRACT_STATUSES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Value</label>
          <input
            type="number"
            step="0.01"
            value={(input.value_cents / 100).toFixed(2)}
            onChange={(e) => setInput({ ...input, value_cents: Math.round(parseFloat(e.target.value || "0") * 100) })}
          />
        </div>
        <div className="form-field">
          <label>Notice period (days)</label>
          <input
            type="number"
            value={input.notice_period_days ?? ""}
            onChange={(e) => setInput({ ...input, notice_period_days: e.target.value ? Number(e.target.value) : null })}
          />
        </div>
        <div className="form-field">
          <label>Start date</label>
          <input type="date" value={input.start_date ?? ""} onChange={(e) => setInput({ ...input, start_date: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>End date</label>
          <input type="date" value={input.end_date ?? ""} onChange={(e) => setInput({ ...input, end_date: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Renewal date</label>
          <input
            type="date"
            value={input.renewal_date ?? ""}
            onChange={(e) => setInput({ ...input, renewal_date: e.target.value || null })}
          />
        </div>
        <div className="form-field full">
          <label>Notes</label>
          <textarea value={input.notes ?? ""} onChange={(e) => setInput({ ...input, notes: e.target.value || null })} />
        </div>
        <CustomFieldsSection entityType="Contract" status={input.status} values={customValues} onChange={setCustomValues} />
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
