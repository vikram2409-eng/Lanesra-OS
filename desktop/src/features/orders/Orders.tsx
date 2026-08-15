import { useEffect, useState } from "react";
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
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import { RelatedRecordSummary } from "../../components/RelatedRecordSummary";
import type { Prefill, Section } from "../../components/AppShell";
import { ORDER_STATUSES, type CustomFieldValues, type Order, type OrderInput } from "../../lib/types";
import type { LineInput } from "../../lib/lineCalc";
import { useCustomFieldFilters } from "../../lib/useCustomFieldFilters";

type View = { mode: "list" } | { mode: "create" } | { mode: "detail"; id: string };

function orderExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (o: Order) => o.order_number },
    { label: "Company", get: (o: Order) => companyNameById.get(o.company_id) ?? "" },
    { label: "Status", get: (o: Order) => o.status },
    { label: "Order date", get: (o: Order) => o.order_date ?? "" },
    { label: "From quote", get: (o: Order) => (o.source_quote_id ? "Yes" : "Direct") },
    { label: "Currency", get: (o: Order) => o.currency_code },
    { label: "Subtotal (cents)", get: (o: Order) => String(o.subtotal_cents) },
    { label: "Discount (cents)", get: (o: Order) => String(o.discount_cents) },
    { label: "Tax (cents)", get: (o: Order) => String(o.tax_cents) },
    { label: "Total (cents)", get: (o: Order) => String(o.total_cents) },
  ];
}

export function Orders({
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
  const orders = useQuery({ queryKey: ["orders"], queryFn: () => api.listOrders() });
  const fieldFilters = useCustomFieldFilters("Order");
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  useEffect(() => {
    if (prefill?.companyId || prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["orders"] });
  }

  if (view.mode === "create") {
    return (
      <OrderForm
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
    return (
      <OrderDetail
        id={view.id}
        onBack={() => setView({ mode: "list" })}
        onChanged={invalidate}
        onNavigateTo={onNavigateTo}
      />
    );
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Orders</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={orders.data ?? []}
            columns={orderExportColumns(companyNameById)}
            filename="orders.csv"
          />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New order
          </button>
        </div>
      </div>
      <CustomFieldFilterBar filters={fieldFilters} />
      {orders.isLoading && <p>Loading...</p>}
      {orders.data && orders.data.length === 0 && <p className="empty-state">No orders yet.</p>}
      {orders.data && orders.data.length > 0 && (() => {
        const rows = orders.data.filter((o) => fieldFilters.matches(o.id));
        return rows.length === 0 ? (
          <p className="empty-state">No orders match the current filters.</p>
        ) : (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Company</th>
              <th>Status</th>
              <th>From quote</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((o) => (
              <tr key={o.id} onClick={() => setView({ mode: "detail", id: o.id })} style={{ cursor: "pointer" }}>
                <td><span className="id-link">{o.order_number}</span></td>
                <td>{companyNameById.get(o.company_id) ?? "—"}</td>
                <td>
                  <StatusBadge status={o.status} />
                </td>
                <td>{o.source_quote_id ? "Yes" : "Direct"}</td>
                <td>{formatCents(o.total_cents, o.currency_code)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        );
      })()}
    </div>
  );
}

function OrderForm({
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
      const input: OrderInput = {
        company_id: companyId,
        contact_id: contactId,
        currency_code: currencyCode,
        order_date: null,
        notes: notes || null,
        lines,
      };
      const result = await api.createOrder(input);
      const ruleMessages = await api.setCustomFieldValues("Order", result.order.id, customValues);
      showRuleMessages(ruleMessages);
      return result;
    },
    onSuccess: (result) => onDone(result.order.id),
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create the order"),
  });

  return (
    <div>
      <h2>New order</h2>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Direct order entry, without a quote or opportunity (FR-ORD-03). To convert an existing quote instead,
        open it from the Quotes screen and use Convert to order.
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
          <div className="form-field full">
            <label>Notes</label>
            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} />
          </div>
          <CustomFieldsSection entityType="Order" status="Draft" values={customValues} onChange={setCustomValues} />
        </div>

        <LineItemsEditor lines={lines} onChange={setLines} products={products.data ?? []} currencyCode={currencyCode} />

        <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending || lines.length === 0}>
            Create order
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function OrderDetail({
  id,
  onBack,
  onChanged,
  onNavigateTo,
}: {
  id: string;
  onBack: () => void;
  onChanged: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
}) {
  const queryClient = useQueryClient();
  const order = useQuery({ queryKey: ["order", id], queryFn: () => api.getOrder(id) });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });
  const contacts = useQuery({ queryKey: ["contacts"], queryFn: () => api.listContacts() });
  const quotes = useQuery({ queryKey: ["quotes"], queryFn: () => api.listQuotes() });
  const invoices = useQuery({ queryKey: ["invoices"], queryFn: () => api.listInvoices() });
  const [error, setError] = useState<string | null>(null);
  const [printing, setPrinting] = useState(false);

  const setStatus = useMutation({
    mutationFn: (status: string) => api.setOrderStatus(id, status),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["order", id] });
      onChanged();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not update status"),
  });

  const convert = useMutation({
    mutationFn: () => api.convertOrderToInvoice(id),
    onSuccess: () => {
      onChanged();
      onBack();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not convert to an invoice"),
  });

  if (!order.data) return <p>Loading...</p>;
  const { order: o, lines } = order.data;

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
      </div>
      <h2>
        {o.order_number} <StatusBadge status={o.status} />
      </h2>
      {error && <div className="error-banner">{error}</div>}

      <RelatedRecordSummary
        companyId={o.company_id}
        contactId={o.contact_id}
        companies={companies.data}
        contacts={contacts.data}
        onNavigateTo={onNavigateTo}
        relatedLists={[
          o.source_quote_id
            ? {
                title: "Source quote",
                rows: [quotes.data?.find((q) => q.id === o.source_quote_id)].filter(
                  (q): q is NonNullable<typeof q> => !!q,
                ),
                render: (q) => q.quote_number,
                onOpen: (q) => onNavigateTo?.("quotes", { openId: q.id }),
              }
            : null,
          {
            title: "Invoices created from this order",
            rows: (invoices.data ?? []).filter((i) => i.source_order_id === id),
            render: (i) => `${i.invoice_number} · ${i.status}`,
            onOpen: (i) => onNavigateTo?.("invoices", { openId: i.id }),
          },
        ]}
      />

      <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap" }}>
        {ORDER_STATUSES.filter((s) => s !== o.status).map((s) => (
          <button key={s} className="btn" onClick={() => setStatus.mutate(s)} disabled={setStatus.isPending}>
            Mark {s}
          </button>
        ))}
        {o.status !== "Cancelled" && (
          <button className="btn btn-primary" onClick={() => convert.mutate()} disabled={convert.isPending}>
            Convert to invoice
          </button>
        )}
        <button className="btn" onClick={() => setPrinting(true)}>
          Print / Save as PDF
        </button>
      </div>

      {printing && (
        <PrintOverlay onClose={() => setPrinting(false)}>
          <PrintableDocument
            kind="Sales Order"
            documentNumber={o.order_number}
            status={o.status}
            currencyCode={o.currency_code}
            companyId={o.company_id}
            contactId={o.contact_id}
            dateFields={[{ label: "Order date", value: o.order_date }]}
            lines={lines}
            subtotalCents={o.subtotal_cents}
            discountCents={o.discount_cents}
            taxCents={o.tax_cents}
            totalCents={o.total_cents}
            notes={o.notes}
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
                <td>{formatCents(line.unit_price_cents, o.currency_code)}</td>
                <td>{formatCents(line.line_total_cents, o.currency_code)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <div style={{ marginTop: 12, textAlign: "right" }}>
          <div>Subtotal: {formatCents(o.subtotal_cents, o.currency_code)}</div>
          <div>Discount: -{formatCents(o.discount_cents, o.currency_code)}</div>
          <div>Tax: {formatCents(o.tax_cents, o.currency_code)}</div>
          <div style={{ fontWeight: 700 }}>Total: {formatCents(o.total_cents, o.currency_code)}</div>
        </div>
      </div>

      <div style={{ marginTop: 16 }}>
        <CustomFieldsCard entityType="Order" entityId={o.id} status={o.status} />
      </div>
    </div>
  );
}
