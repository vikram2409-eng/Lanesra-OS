import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { LineItemsEditor } from "../../components/LineItemsEditor";
import { PrintableDocument } from "../../components/PrintableDocument";
import { PrintOverlay } from "../../components/PrintOverlay";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { CustomFieldsCard } from "../../components/CustomFieldsCard";
import { QUOTE_STATUSES, type CustomFieldValues, type Quote, type QuoteInput } from "../../lib/types";
import type { LineInput } from "../../lib/lineCalc";

type View = { mode: "list" } | { mode: "create" } | { mode: "detail"; id: string };

function quoteExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (q: Quote) => q.quote_number },
    { label: "Company", get: (q: Quote) => companyNameById.get(q.company_id) ?? "" },
    { label: "Status", get: (q: Quote) => q.status },
    { label: "Issue date", get: (q: Quote) => q.issue_date ?? "" },
    { label: "Expiry date", get: (q: Quote) => q.expiry_date ?? "" },
    { label: "Currency", get: (q: Quote) => q.currency_code },
    { label: "Subtotal (cents)", get: (q: Quote) => String(q.subtotal_cents) },
    { label: "Discount (cents)", get: (q: Quote) => String(q.discount_cents) },
    { label: "Tax (cents)", get: (q: Quote) => String(q.tax_cents) },
    { label: "Total (cents)", get: (q: Quote) => String(q.total_cents) },
  ];
}

export function Quotes() {
  const [view, setView] = useState<View>({ mode: "list" });
  const queryClient = useQueryClient();
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["quotes"] });
  }

  if (view.mode === "create") {
    return (
      <QuoteForm
        companies={companies.data ?? []}
        onDone={(id) => {
          invalidate();
          setView({ mode: "detail", id });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  if (view.mode === "detail") {
    return <QuoteDetail id={view.id} onBack={() => setView({ mode: "list" })} onChanged={invalidate} />;
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Quotes</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={quotes.data ?? []}
            columns={quoteExportColumns(companyNameById)}
            filename="quotes.csv"
          />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New quote
          </button>
        </div>
      </div>
      {quotes.isLoading && <p>Loading...</p>}
      {quotes.data && quotes.data.length === 0 && <p className="empty-state">No quotes yet.</p>}
      {quotes.data && quotes.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Company</th>
              <th>Status</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {quotes.data.map((q) => (
              <tr key={q.id} onClick={() => setView({ mode: "detail", id: q.id })} style={{ cursor: "pointer" }}>
                <td>{q.quote_number}</td>
                <td>{companyNameById.get(q.company_id) ?? "—"}</td>
                <td>
                  <StatusBadge status={q.status} />
                </td>
                <td>{formatCents(q.total_cents, q.currency_code)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function QuoteForm({
  companies,
  onDone,
  onCancel,
}: {
  companies: { id: string; name: string; currency_code?: string }[];
  onDone: (id: string) => void;
  onCancel: () => void;
}) {
  const [companyId, setCompanyId] = useState(companies[0]?.id ?? "");
  const [contactId, setContactId] = useState<string | null>(null);
  const [opportunityId, setOpportunityId] = useState<string | null>(null);
  const [currencyCode, setCurrencyCode] = useState("USD");
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<LineInput[]>([
    { product_id: null, description: "", quantity_milli: 1000, unit_price_cents: 0, discount_bp: 0, tax_rate_bp: 0 },
  ]);
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [error, setError] = useState<string | null>(null);

  const contacts = useQuery({
    queryKey: ["contactsByCompany", companyId],
    queryFn: () => api.listContactsByCompany(companyId),
    enabled: !!companyId,
  });
  const opportunities = useQuery({
    queryKey: ["opportunitiesByCompany", companyId],
    queryFn: () => api.listOpportunitiesByCompany(companyId),
    enabled: !!companyId,
  });
  const products = useQuery({ queryKey: ["products"], queryFn: () => api.listProducts() });

  const save = useMutation({
    mutationFn: async () => {
      const input: QuoteInput = {
        company_id: companyId,
        contact_id: contactId,
        opportunity_id: opportunityId,
        currency_code: currencyCode,
        issue_date: null,
        expiry_date: null,
        notes: notes || null,
        terms: null,
        lines,
      };
      const result = await api.createQuote(input);
      const ruleMessages = await api.setCustomFieldValues("Quote", result.quote.id, customValues);
      showRuleMessages(ruleMessages);
      return result;
    },
    onSuccess: (result) => onDone(result.quote.id),
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create the quote"),
  });

  return (
    <div>
      <h2>New quote</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-grid" style={{ marginBottom: 16 }}>
          <div className="form-field">
            <label>Company</label>
            <select
              value={companyId}
              onChange={(e) => {
                setCompanyId(e.target.value);
                setContactId(null);
                setOpportunityId(null);
              }}
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
            <select value={contactId ?? ""} onChange={(e) => setContactId(e.target.value || null)}>
              <option value="">— None —</option>
              {(contacts.data ?? []).map((c) => (
                <option key={c.id} value={c.id}>
                  {c.first_name} {c.last_name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field">
            <label>Opportunity (optional)</label>
            <select value={opportunityId ?? ""} onChange={(e) => setOpportunityId(e.target.value || null)}>
              <option value="">— None —</option>
              {(opportunities.data ?? []).map((o) => (
                <option key={o.id} value={o.id}>
                  {o.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-field">
            <label>Currency</label>
            <input value={currencyCode} onChange={(e) => setCurrencyCode(e.target.value.toUpperCase())} maxLength={3} />
          </div>
          <div className="form-field full">
            <label>Notes</label>
            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} />
          </div>
          <CustomFieldsSection entityType="Quote" status="Draft" values={customValues} onChange={setCustomValues} />
        </div>

        <LineItemsEditor lines={lines} onChange={setLines} products={products.data ?? []} currencyCode={currencyCode} />

        <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending || lines.length === 0}>
            Create quote
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function QuoteDetail({ id, onBack, onChanged }: { id: string; onBack: () => void; onChanged: () => void }) {
  const queryClient = useQueryClient();
  const quote = useQuery({ queryKey: ["quote", id], queryFn: () => api.getQuote(id) });
  const [error, setError] = useState<string | null>(null);
  const [printing, setPrinting] = useState(false);

  const setStatus = useMutation({
    mutationFn: (status: string) => api.setQuoteStatus(id, status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["quote", id] });
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not update status"),
  });

  const convert = useMutation({
    mutationFn: () => api.convertQuoteToOrder(id),
    onSuccess: () => {
      onChanged();
      onBack();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not convert to an order"),
  });

  if (!quote.data) return <p>Loading...</p>;
  const { quote: q, lines } = quote.data;

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
      </div>
      <h2>
        {q.quote_number} <StatusBadge status={q.status} />
      </h2>
      {error && <div className="error-banner">{error}</div>}

      <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
        {QUOTE_STATUSES.filter((s) => s !== q.status).map((s) => (
          <button key={s} className="btn" onClick={() => setStatus.mutate(s)} disabled={setStatus.isPending}>
            Mark {s}
          </button>
        ))}
        {q.status === "Accepted" && (
          <button className="btn btn-primary" onClick={() => convert.mutate()} disabled={convert.isPending}>
            Convert to order
          </button>
        )}
        <button className="btn" onClick={() => setPrinting(true)}>
          Print / Save as PDF
        </button>
      </div>

      {printing && (
        <PrintOverlay onClose={() => setPrinting(false)}>
          <PrintableDocument
            kind="Quote"
            documentNumber={q.quote_number}
            status={q.status}
            currencyCode={q.currency_code}
            companyId={q.company_id}
            contactId={q.contact_id}
            dateFields={[
              { label: "Issue date", value: q.issue_date },
              { label: "Valid until", value: q.expiry_date },
            ]}
            lines={lines}
            subtotalCents={q.subtotal_cents}
            discountCents={q.discount_cents}
            taxCents={q.tax_cents}
            totalCents={q.total_cents}
            notes={q.notes}
          />
        </PrintOverlay>
      )}

      <div className="card">
        <table>
          <thead>
            <tr>
              <th>Description</th>
              <th>Qty</th>
              <th>Unit price</th>
              <th>Line total</th>
            </tr>
          </thead>
          <tbody>
            {lines.map((line) => (
              <tr key={line.id}>
                <td>{line.description}</td>
                <td>{(line.quantity_milli / 1000).toString()}</td>
                <td>{formatCents(line.unit_price_cents, q.currency_code)}</td>
                <td>{formatCents(line.line_total_cents, q.currency_code)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <div style={{ marginTop: 12, textAlign: "right" }}>
          <div>Subtotal: {formatCents(q.subtotal_cents, q.currency_code)}</div>
          <div>Discount: -{formatCents(q.discount_cents, q.currency_code)}</div>
          <div>Tax: {formatCents(q.tax_cents, q.currency_code)}</div>
          <div style={{ fontWeight: 700 }}>Total: {formatCents(q.total_cents, q.currency_code)}</div>
        </div>
      </div>

      <div style={{ marginTop: 16 }}>
        <CustomFieldsCard entityType="Quote" entityId={q.id} status={q.status} />
      </div>
    </div>
  );
}
