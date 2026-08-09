import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import { formatCents } from "../lib/money";

export interface PrintableLine {
  id: string;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  line_total_cents: number;
}

export interface PrintableTotal {
  label: string;
  value: string;
  bold?: boolean;
}

export interface PrintableDocumentProps {
  kind: string; // "Quote" | "Sales Order" | "Invoice"
  documentNumber: string;
  status: string;
  currencyCode: string;
  companyId: string;
  contactId: string | null;
  dateFields: { label: string; value: string | null }[];
  lines: PrintableLine[];
  subtotalCents: number;
  discountCents: number;
  taxCents: number;
  totalCents: number;
  extraTotals?: PrintableTotal[];
  notes?: string | null;
}

/// A letterhead-style, print-only rendering of a quote/order/invoice -
/// shared by all three so they stay visually consistent. Rendered inside
/// PrintOverlay, which handles the on-screen preview chrome and the actual
/// window.print() call; this component only cares about laying out the
/// document itself.
export function PrintableDocument(props: PrintableDocumentProps) {
  const { kind, documentNumber, status, currencyCode, companyId, contactId, dateFields, lines } = props;
  const { subtotalCents, discountCents, taxCents, totalCents, extraTotals, notes } = props;

  const workspace = useQuery({ queryKey: ["workspaceStatus"], queryFn: () => api.workspaceStatus() });
  const company = useQuery({ queryKey: ["company", companyId], queryFn: () => api.getCompany(companyId) });
  const contact = useQuery({
    queryKey: ["contact", contactId],
    queryFn: () => api.getContact(contactId as string),
    enabled: !!contactId,
  });

  return (
    <div className="print-doc">
      <div className="print-doc-head">
        <div>
          <div className="print-doc-business">{workspace.data?.business_name ?? " "}</div>
          {workspace.data?.legal_name && <div className="print-doc-muted">{workspace.data.legal_name}</div>}
        </div>
        <div className="print-doc-heading">
          <h1>{kind}</h1>
          <div>{documentNumber}</div>
          <div className="print-doc-muted">{status}</div>
        </div>
      </div>

      <div className="print-doc-parties">
        <div>
          <div className="print-doc-label">Bill to</div>
          <div className="print-doc-strong">{company.data?.name ?? "—"}</div>
          {company.data?.billing_address && <div className="print-doc-muted">{company.data.billing_address}</div>}
          {contactId && contact.data && (
            <div className="print-doc-muted">
              {contact.data.first_name} {contact.data.last_name}
              {contact.data.email ? ` · ${contact.data.email}` : ""}
            </div>
          )}
        </div>
        <div className="print-doc-dates">
          {dateFields
            .filter((d) => d.value)
            .map((d) => (
              <div key={d.label}>
                <span className="print-doc-label">{d.label}: </span>
                {d.value}
              </div>
            ))}
        </div>
      </div>

      <table className="print-doc-table">
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
              <td>{formatCents(line.unit_price_cents, currencyCode)}</td>
              <td>{formatCents(line.line_total_cents, currencyCode)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="print-doc-totals">
        <div>
          <span>Subtotal</span>
          <span>{formatCents(subtotalCents, currencyCode)}</span>
        </div>
        <div>
          <span>Discount</span>
          <span>-{formatCents(discountCents, currencyCode)}</span>
        </div>
        <div>
          <span>Tax</span>
          <span>{formatCents(taxCents, currencyCode)}</span>
        </div>
        <div className="print-doc-total-line">
          <span>Total</span>
          <span>{formatCents(totalCents, currencyCode)}</span>
        </div>
        {extraTotals?.map((t) => (
          <div key={t.label} className={t.bold ? "print-doc-total-line" : undefined}>
            <span>{t.label}</span>
            <span>{t.value}</span>
          </div>
        ))}
      </div>

      {notes && (
        <div className="print-doc-notes">
          <div className="print-doc-label">Notes</div>
          <p>{notes}</p>
        </div>
      )}
    </div>
  );
}
