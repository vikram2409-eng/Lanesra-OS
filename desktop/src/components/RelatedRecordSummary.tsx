import type { Prefill, Section } from "./AppShell";
import type { Company, Contact } from "../lib/types";

interface RelatedList<T> {
  title: string;
  rows: T[];
  render: (row: T) => string;
  onOpen: (row: T) => void;
}

/**
 * Record-detail-page round: a small "who/what this record belongs to" strip
 * for Quote/Order/Invoice detail pages, which show line items and status
 * actions but - before this - never actually linked back to the Company/
 * Contact/source-document they belong to, or forward to whatever got
 * created from them (an Order from a Quote, an Invoice from an Order).
 * Company/Contact are always clickable (both have a detail page); `extra`
 * covers fields with no detail page of their own (e.g. an Opportunity name)
 * as plain text; `relatedLists` are the clickable ones - a source document
 * this one was converted from, and/or documents converted from this one
 * (e.g. an Order shows both its source Quote and any Invoices created from
 * it - either, neither, or both can be non-empty at once).
 */
export function RelatedRecordSummary({
  companyId,
  contactId,
  companies,
  contacts,
  onNavigateTo,
  extra,
  relatedLists,
}: {
  companyId: string;
  contactId?: string | null;
  companies?: Company[];
  contacts?: Contact[];
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
  extra?: ({ label: string; text: string } | null)[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  relatedLists?: (RelatedList<any> | null | undefined)[];
}) {
  const company = companies?.find((c) => c.id === companyId);
  const contact = contactId ? contacts?.find((c) => c.id === contactId) : undefined;
  const cleanExtra = (extra ?? []).filter((e): e is { label: string; text: string } => e !== null);
  const cleanLists = (relatedLists ?? []).filter(
    (l): l is RelatedList<unknown> => !!l && l.rows.length > 0,
  );

  return (
    <div className="card" style={{ marginBottom: 16, display: "flex", flexWrap: "wrap", gap: 24 }}>
      <div>
        <div style={{ fontSize: 12, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: 0.4 }}>
          Company
        </div>
        <button className="link-button" style={{ color: "var(--accent)", fontWeight: 600 }} onClick={() => onNavigateTo?.("companies", { openId: companyId })}>
          {company?.name ?? "—"}
        </button>
      </div>
      {contactId && (
        <div>
          <div style={{ fontSize: 12, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: 0.4 }}>
            Contact
          </div>
          <button className="link-button" style={{ color: "var(--accent)", fontWeight: 600 }} onClick={() => onNavigateTo?.("contacts", { openId: contactId })}>
            {contact ? `${contact.first_name} ${contact.last_name}` : "—"}
          </button>
        </div>
      )}
      {cleanExtra.map((e) => (
        <div key={e.label}>
          <div style={{ fontSize: 12, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: 0.4 }}>
            {e.label}
          </div>
          <div>{e.text}</div>
        </div>
      ))}
      {cleanLists.map((list) => (
        <div key={list.title}>
          <div style={{ fontSize: 12, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: 0.4 }}>
            {list.title}
          </div>
          {list.rows.map((row, idx) => (
            <div key={idx}>
              <button className="link-button" style={{ color: "var(--accent)", fontWeight: 600 }} onClick={() => list.onOpen(row)}>
                {list.render(row)}
              </button>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
