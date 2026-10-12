import { api } from "./api";

export type PickableRecord = { id: string; label: string };

/** Maps any entity_type - built-in or custom object key - to a flat
 * id/label list suitable for a link/lookup picker. Built-in types each
 * have their own dedicated list endpoint; anything else is assumed to be
 * an active custom object key, resolved through the one generic
 * custom-records endpoint. Shared by `RelatedRecordsCard`'s link picker
 * and `LookupFieldPicker`, which both need the same "every pickable
 * record of this type" resolution. */
export async function listRecordsOfType(entityType: string): Promise<PickableRecord[]> {
  switch (entityType) {
    case "Company":
      return (await api.listCompanies()).map((r) => ({ id: r.id, label: r.name }));
    case "Contact":
      return (await api.listContacts()).map((r) => ({ id: r.id, label: `${r.first_name} ${r.last_name}`.trim() }));
    case "Opportunity":
      return (await api.listOpportunities()).map((r) => ({ id: r.id, label: r.name }));
    case "Quote":
      return (await api.listQuotes()).map((r) => ({ id: r.id, label: r.quote_number }));
    case "Order":
      return (await api.listOrders()).map((r) => ({ id: r.id, label: r.order_number }));
    case "Invoice":
      return (await api.listInvoices()).map((r) => ({ id: r.id, label: r.invoice_number }));
    case "Contract":
      return (await api.listContracts()).map((r) => ({ id: r.id, label: r.title }));
    case "Task":
      return (await api.listTasks()).map((r) => ({ id: r.id, label: r.title }));
    case "Product":
      return (await api.listProducts()).map((r) => ({ id: r.id, label: r.name }));
    default:
      return (await api.listCustomRecords(entityType)).map((r) => ({ id: r.id, label: r.primary_name }));
  }
}
