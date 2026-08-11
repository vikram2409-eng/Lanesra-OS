import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents, parseDecimalToCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { LineItemsEditor } from "../../components/LineItemsEditor";
import { PrintableDocument } from "../../components/PrintableDocument";
import { PrintOverlay } from "../../components/PrintOverlay";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { CustomFieldsCard } from "../../components/CustomFieldsCard";
import type { Prefill } from "../../components/AppShell";
import type { CustomFieldValues, Invoice, InvoiceInput, PaymentInput } from "../../lib/types";
import type { LineInput } from "../../lib/lineCalc";

type View = { mode: "list" } | { mode: "create" } | { mode: "detail"; id: string };

function invoiceExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (i: Invoice) => i.invoice_number },
    { label: "Company", get: (i: Invoice) => companyNameById.get(i.company_id) ?? "" },
    { label: "Status", get: (i: Invoice) => i.status },
    { label: "Issue date", get: (i: Invoice) => i.issue_date ?? "" },
    { label: "Due date", get: (i: Invoice) => i.due_date ?? "" },
    { label: "Currency", get: (i: Invoice) => i.currency_code },
    { label: "Total (cents)", get: (i: Invoice) => String(i.total_cents) },
    { label: "Paid (cents)", get: (i: Invoice) => String(i.amount_paid_cents) },
    { label: "Balance (cents)", get: (i: Invoice) => String(i.balance_cents) },
  ];
}

export function Invoices({
  prefill,
  onPrefillConsumed,
}: { prefill?: Prefill | null; onPrefillConsumed?: () => void } = {}) {
  const [view, setView] = useState<View>(() => (prefill?.companyId ? { mode: "create" } : { mode: "list" }));
  const queryClient = useQueryClient();
  const invoices = useQuery({ queryKey: ["invoices"], queryFn: () => api.listInvoices() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  useEffect(() => {
    if (prefill?.companyId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["invoices"] });
  }

  if (view.mode === "create") {
    return (
      <InvoiceForm
        companies={companies.data ?? []}
        initialCompanyId={prefill?.companyId}
        initialContactId={prefill?.contactId}
        onDone={(id) => {
          invalidate();
          setView({ mode: "detail", id });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  if (view.mode === "detail") {
    return <InvoiceDetail id={view.id} onBack={() => setView({ mode: "list" })} onChanged={invalidate} />;
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Invoices</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={invoices.data ?? []}
            columns={invoiceExportColumns(companyNameById)}
            filename="invoices.csv"
          />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New invoice
          </button>
        </div>
      </div>
      {invoices.isLoading && <p>Loading...</p>}
      {invoices.data && invoices.data.length === 0 && <p className="empty-state">No invoices yet.</p>}
      {invoices.data && invoices.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Company</th>
              <th>Status</th>
              <th>Total</th>
              <th>Balance</th>
              <th>Due</th>
            </tr>
          </thead>
          <tbody>
            {invoices.data.map((inv) => (
              <tr key={inv.id} onClick={() => setView({ mode: "detail", id: inv.id })} style={{ cursor: "pointer" }}>
                <td>{inv.invoice_number}</td>
                <td>{companyNameById.get(inv.company_id) ?? "—"}</td>
                <td>
                  <StatusBadge status={inv.status} />
                </td>
                <td>{formatCents(inv.total_cents, inv.currency_code)}</td>
                <td>{formatCents(inv.balance_cents, inv.currency_code)}</td>
                <td>{inv.due_date ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function InvoiceForm({
  companies,
  initialCompanyId,
  initialContactId,
  onDone,
  onCancel,
}: {
  companies: { id: string; name: string }[];
  initialCompanyId?: string;
  initialContactId?: string;
  onDone: (id: string) => void;
  onCancel: () => void;
}) {
  const [companyId, setCompanyId] = useState(initialCompanyId ?? companies[0]?.id ?? "");
  const [contactId, setContactId] = useState<string | null>(initialContactId ?? null);
  const [currencyCode, setCurrencyCode] = useState("USD");
  const [dueDate, setDueDate] = useState("");
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
  const products = useQuery({ queryKey: ["products"], queryFn: () => api.listProducts() });

  const save = useMutation({
    mutationFn: async () => {
      const input: InvoiceInput = {
        company_id: companyId,
        contact_id: contactId,
        currency_code: currencyCode,
        issue_date: null,
        due_date: dueDate || null,
        payment_terms: null,
        notes: notes || null,
        lines,
      };
      const result = await api.createInvoice(input);
      const ruleMessages = await api.setCustomFieldValues("Invoice", result.invoice.id, customValues);
      showRuleMessages(ruleMessages);
      return result;
    },
    onSuccess: (result) => onDone(result.invoice.id),
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create the invoice"),
  });

  return (
    <div>
      <h2>New invoice</h2>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Direct invoice entry, without an order (FR-INV-03). To convert an existing order instead, open it
        from the Orders screen and use Convert to invoice.
      </p>
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
            <label>Currency</label>
            <input value={currencyCode} onChange={(e) => setCurrencyCode(e.target.value.toUpperCase())} maxLength={3} />
          </div>
          <div className="form-field">
            <label>Due date</label>
            <input type="date" value={dueDate} onChange={(e) => setDueDate(e.target.value)} />
          </div>
          <div className="form-field full">
            <label>Notes</label>
            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} />
          </div>
          <CustomFieldsSection entityType="Invoice" status="Draft" values={customValues} onChange={setCustomValues} />
        </div>

        <LineItemsEditor lines={lines} onChange={setLines} products={products.data ?? []} currencyCode={currencyCode} />

        <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending || lines.length === 0}>
            Create invoice
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function InvoiceDetail({ id, onBack, onChanged }: { id: string; onBack: () => void; onChanged: () => void }) {
  const queryClient = useQueryClient();
  const invoice = useQuery({ queryKey: ["invoice", id], queryFn: () => api.getInvoice(id) });
  const [error, setError] = useState<string | null>(null);
  const [paymentAmount, setPaymentAmount] = useState("0.00");
  const [paymentMethod, setPaymentMethod] = useState("");
  const [printing, setPrinting] = useState(false);

  function refresh() {
    queryClient.invalidateQueries({ queryKey: ["invoice", id] });
    onChanged();
  }

  const issue = useMutation({
    mutationFn: () => api.issueInvoice(id),
    onSuccess: refresh,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not issue the invoice"),
  });

  const voidInvoice = useMutation({
    mutationFn: () => api.voidInvoice(id),
    onSuccess: refresh,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not void the invoice"),
  });

  const recordPayment = useMutation({
    mutationFn: (payment: PaymentInput) => api.recordInvoicePayment(id, payment),
    onSuccess: () => {
      refresh();
      setPaymentAmount("0.00");
      setPaymentMethod("");
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not record the payment"),
  });

  if (!invoice.data) return <p>Loading...</p>;
  const { invoice: inv, lines, payments } = invoice.data;

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
      </div>
      <h2>
        {inv.invoice_number} <StatusBadge status={inv.status} />
      </h2>
      {error && <div className="error-banner">{error}</div>}

      <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
        {inv.status === "Draft" && (
          <button className="btn btn-primary" onClick={() => issue.mutate()} disabled={issue.isPending}>
            Issue invoice
          </button>
        )}
        {inv.status !== "Void" && inv.status !== "Paid" && (
          <button className="btn btn-danger" onClick={() => voidInvoice.mutate()} disabled={voidInvoice.isPending}>
            Void invoice
          </button>
        )}
        <button className="btn" onClick={() => setPrinting(true)}>
          Print / Save as PDF
        </button>
      </div>

      {printing && (
        <PrintOverlay onClose={() => setPrinting(false)}>
          <PrintableDocument
            kind="Invoice"
            documentNumber={inv.invoice_number}
            status={inv.status}
            currencyCode={inv.currency_code}
            companyId={inv.company_id}
            contactId={inv.contact_id}
            dateFields={[
              { label: "Issue date", value: inv.issue_date },
              { label: "Due date", value: inv.due_date },
            ]}
            lines={lines}
            subtotalCents={inv.subtotal_cents}
            discountCents={inv.discount_cents}
            taxCents={inv.tax_cents}
            totalCents={inv.total_cents}
            extraTotals={[
              { label: "Paid", value: formatCents(inv.amount_paid_cents, inv.currency_code) },
              { label: "Balance due", value: formatCents(inv.balance_cents, inv.currency_code), bold: true },
            ]}
            notes={inv.notes}
          />
        </PrintOverlay>
      )}

      <div className="card" style={{ marginBottom: 16 }}>
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
                <td>{formatCents(line.unit_price_cents, inv.currency_code)}</td>
                <td>{formatCents(line.line_total_cents, inv.currency_code)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <div style={{ marginTop: 12, textAlign: "right" }}>
          <div>Subtotal: {formatCents(inv.subtotal_cents, inv.currency_code)}</div>
          <div>Discount: -{formatCents(inv.discount_cents, inv.currency_code)}</div>
          <div>Tax: {formatCents(inv.tax_cents, inv.currency_code)}</div>
          <div style={{ fontWeight: 700 }}>Total: {formatCents(inv.total_cents, inv.currency_code)}</div>
          <div>Paid: {formatCents(inv.amount_paid_cents, inv.currency_code)}</div>
          <div style={{ fontWeight: 700 }}>Balance: {formatCents(inv.balance_cents, inv.currency_code)}</div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Payments</h3>
        {payments.length === 0 ? (
          <p className="empty-state">No payments recorded yet</p>
        ) : (
          <table style={{ marginBottom: 16 }}>
            <thead>
              <tr>
                <th>Date</th>
                <th>Amount</th>
                <th>Method</th>
              </tr>
            </thead>
            <tbody>
              {payments.map((p) => (
                <tr key={p.id}>
                  <td>{p.paid_at}</td>
                  <td>{formatCents(p.amount_cents, inv.currency_code)}</td>
                  <td>{p.method ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {inv.balance_cents > 0 && inv.status !== "Void" && inv.status !== "Cancelled" && inv.status !== "Draft" && (
          <form
            style={{ display: "flex", gap: 8, alignItems: "flex-end" }}
            onSubmit={(e) => {
              e.preventDefault();
              recordPayment.mutate({
                amount_cents: parseDecimalToCents(paymentAmount),
                paid_at: new Date().toISOString().slice(0, 10),
                method: paymentMethod || null,
                reference: null,
              });
            }}
          >
            <div className="form-field">
              <label>Payment amount</label>
              <input
                type="number"
                step="0.01"
                value={paymentAmount}
                onChange={(e) => setPaymentAmount(e.target.value)}
              />
            </div>
            <div className="form-field">
              <label>Method</label>
              <input value={paymentMethod} onChange={(e) => setPaymentMethod(e.target.value)} />
            </div>
            <button className="btn btn-primary" type="submit" disabled={recordPayment.isPending}>
              Record payment
            </button>
          </form>
        )}
      </div>

      <div style={{ marginTop: 16 }}>
        <CustomFieldsCard entityType="Invoice" entityId={inv.id} status={inv.status} />
      </div>
    </div>
  );
}
