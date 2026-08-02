import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { formatCents } from "../../lib/money";
import { StatusBadge } from "../../components/StatusBadge";
import { LineItemsEditor } from "../../components/LineItemsEditor";
import { ORDER_STATUSES, type OrderInput } from "../../lib/types";
import type { LineInput } from "../../lib/lineCalc";

type View = { mode: "list" } | { mode: "create" } | { mode: "detail"; id: string };

export function Orders() {
  const [view, setView] = useState<View>({ mode: "list" });
  const queryClient = useQueryClient();
  const orders = useQuery({ queryKey: ["orders"], queryFn: () => api.listOrders() });
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["orders"] });
  }

  if (view.mode === "create") {
    return (
      <OrderForm
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
    return <OrderDetail id={view.id} onBack={() => setView({ mode: "list" })} onChanged={invalidate} />;
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Orders</h2>
        <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
          + New order
        </button>
      </div>
      {orders.isLoading && <p>Loading...</p>}
      {orders.data && orders.data.length === 0 && <p className="empty-state">No orders yet.</p>}
      {orders.data && orders.data.length > 0 && (
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
            {orders.data.map((o) => (
              <tr key={o.id} onClick={() => setView({ mode: "detail", id: o.id })} style={{ cursor: "pointer" }}>
                <td>{o.order_number}</td>
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
      )}
    </div>
  );
}

function OrderForm({
  companies,
  onDone,
  onCancel,
}: {
  companies: { id: string; name: string }[];
  onDone: (id: string) => void;
  onCancel: () => void;
}) {
  const [companyId, setCompanyId] = useState(companies[0]?.id ?? "");
  const [contactId, setContactId] = useState<string | null>(null);
  const [currencyCode, setCurrencyCode] = useState("USD");
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<LineInput[]>([
    { product_id: null, description: "", quantity_milli: 1000, unit_price_cents: 0, discount_bp: 0, tax_rate_bp: 0 },
  ]);
  const [error, setError] = useState<string | null>(null);

  const contacts = useQuery({
    queryKey: ["contactsByCompany", companyId],
    queryFn: () => api.listContactsByCompany(companyId),
    enabled: !!companyId,
  });
  const products = useQuery({ queryKey: ["products"], queryFn: () => api.listProducts() });

  const save = useMutation({
    mutationFn: () => {
      const input: OrderInput = {
        company_id: companyId,
        contact_id: contactId,
        currency_code: currencyCode,
        order_date: null,
        notes: notes || null,
        lines,
      };
      return api.createOrder(input);
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

function OrderDetail({ id, onBack, onChanged }: { id: string; onBack: () => void; onChanged: () => void }) {
  const queryClient = useQueryClient();
  const order = useQuery({ queryKey: ["order", id], queryFn: () => api.getOrder(id) });
  const [error, setError] = useState<string | null>(null);

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
      </div>

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
    </div>
  );
}
