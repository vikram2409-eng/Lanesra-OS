import { customObjectSection, type Section } from "../components/AppShell";

// Object 360 (issue #352): `entity_type` strings here are always
// `entity_registry::CORE_ENTITY_TYPES`' own singular-capitalized keys
// ("Company", "Contact", ...) - the same convention every existing
// `RelatedRecordsCard`/`AuditTrail`/`OwnershipByline` call on these pages
// already uses, confirmed against `Companies.tsx`. English pluralization
// isn't regular enough to derive the matching `Section` value
// programmatically, so this is a small, explicit lookup rather than a
// `.toLowerCase() + "s"` guess.
const BUILTIN_ENTITY_TYPE_TO_SECTION: Record<string, Section> = {
  Company: "companies",
  Contact: "contacts",
  Opportunity: "opportunities",
  Quote: "quotes",
  Order: "orders",
  Invoice: "invoices",
  Contract: "contracts",
  Task: "tasks",
  Product: "products",
};

/** Any `entity_type` not in the built-in map above is a Custom Object's own key. */
export function sectionForEntityType(entityType: string): Section {
  return BUILTIN_ENTITY_TYPE_TO_SECTION[entityType] ?? customObjectSection(entityType);
}
