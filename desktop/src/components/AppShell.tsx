import { useQuery } from "@tanstack/react-query";

import { GlobalSearch } from "./GlobalSearch";
import { NotificationBell } from "./NotificationBell";
import { api } from "../lib/api";
import type { CustomObjectDefinition, User } from "../lib/types";

// Custom-object sections are dynamic (admin-defined at runtime), so their
// key is a template literal - `custom:<objectKey>` - rather than one more
// fixed union member. App.tsx parses this prefix to know when to render
// CustomObjectRecords instead of a hardcoded screen.
export type Section =
  | "dashboard"
  | "companies"
  | "contacts"
  | "products"
  | "opportunities"
  | "quotes"
  | "orders"
  | "invoices"
  | "contracts"
  | "tasks"
  | "reports"
  | "admin"
  | "account"
  | `custom:${string}`;

export const customObjectSection = (key: string): Section => `custom:${key}`;

// Core entities each have their own fixed nav section; anything else is a
// custom object, addressed by its `key` via customObjectSection. Lives
// here (rather than in GlobalSearch, its original home) so the App
// Switcher below can resolve an AppDefinition.object_keys entry to a nav
// Section without GlobalSearch.tsx importing back from this module -
// GlobalSearch already imports Section/customObjectSection from here, and
// a two-way import would be a circular dependency. GlobalSearch and
// Dashboard both now import `sectionFor` from here instead of defining/
// re-exporting their own copy.
const CORE_ENTITY_SECTION: Record<string, Section> = {
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

export function sectionFor(entityType: string): Section {
  return CORE_ENTITY_SECTION[entityType] ?? customObjectSection(entityType);
}

// Nav sections an App Builder app can scope down to - every entity-type
// section above, i.e. everything a Custom Object or a built-in object
// could resolve to via sectionFor. Structural sections (dashboard, admin,
// reports, account) are never filtered by an active app - see the
// App Switcher's own doc comment below for why.
const SCOPABLE_SECTIONS = new Set<Section>(Object.values(CORE_ENTITY_SECTION));

/** Addendum Phase 5 (Customer 360 / Contact 360): a "+ New X" button on a
 * detail view's related-record tab navigates to that record type's own
 * section and pre-fills its create form with the relationship it was
 * launched from, instead of starting from a blank form the user has to
 * re-select the company/contact on. One-shot: the target section reads
 * it once on mount (it fully unmounts/remounts on every section change,
 * so "on mount" reliably means "just navigated here") and immediately
 * clears it via `onPrefillConsumed`, so a later plain sidebar click into
 * the same section starts from a blank list as normal.
 *
 * Record-detail-page round: `openId` reuses this exact one-shot mechanism
 * for the opposite direction - a related-record link on a detail page
 * (e.g. a Quote's "Company" link, an Order's "Source quote" link) jumps to
 * that record's own section and opens its detail view directly instead of
 * landing on a plain list the user has to search. The target section reads
 * it once on mount to seed its initial view as `{mode:"detail", id:
 * openId}`, then clears it the same way companyId/contactId already do. */
export interface Prefill {
  companyId?: string;
  contactId?: string;
  openId?: string;
}

const NAV_ITEMS: { section: Section; label: string; adminOnly?: boolean }[] = [
  { section: "dashboard", label: "Dashboard" },
  { section: "companies", label: "Companies" },
  { section: "contacts", label: "Contacts" },
  { section: "opportunities", label: "Sales Pipeline" },
  { section: "products", label: "Products" },
  { section: "quotes", label: "Quotes" },
  { section: "orders", label: "Orders" },
  { section: "invoices", label: "Invoices" },
  { section: "contracts", label: "Contracts" },
  { section: "tasks", label: "Tasks" },
  { section: "reports", label: "Reports" },
  // Users lives inside the Admin panel now, alongside branding, custom
  // fields, business rules, workflow automation, numbering and Dashboard
  // KPIs - one nav item for every administrator-facing capability.
  { section: "admin", label: "Admin", adminOnly: true },
];

export function AppShell({
  active,
  onNavigate,
  onOpenSearchResult,
  user,
  onLogout,
  customObjects,
  activeAppId,
  onSwitchApp,
  children,
}: {
  active: Section;
  onNavigate: (section: Section) => void;
  /** Global search "jump to a record" - reuses the same one-shot openId
   * prefill mechanism every list screen's ID-hyperlinks already use. */
  onOpenSearchResult: (section: Section, id: string) => void;
  user: User;
  onLogout: () => void;
  customObjects: CustomObjectDefinition[];
  /** App Builder: which accessible app (if any) is currently selected in
   * the switcher below - `null` is "All", the pre-App-Builder sidebar with
   * every section visible. Lifted to App.tsx (not local state here)
   * because Dashboard also needs to know it, to render that app's own
   * dashboard instead of the role-resolved default. */
  activeAppId: string | null;
  onSwitchApp: (appId: string | null) => void;
  children: React.ReactNode;
}) {
  const isAdmin = user.roles.includes("Administrator");
  // Every published app the signed-in user has a grant on (or all of them,
  // if they're an Administrator) - see app_service::list_accessible. Fetched
  // here rather than threaded down as a prop, the same self-contained-query
  // pattern GlobalSearch/NotificationBell already use; App.tsx runs the same
  // query (React Query dedupes it) to resolve the active app's dashboard_id.
  const apps = useQuery({ queryKey: ["accessibleApps"], queryFn: () => api.listAccessibleApps() });
  const accessibleApps = apps.data ?? [];
  const activeApp = accessibleApps.find((a) => a.app.id === activeAppId)?.app ?? null;

  // With an app selected, only its own object_keys' sections (and any
  // structural section - Dashboard/Admin/Reports/Account) show in the
  // sidebar. `null` (no app selected, "All") shows everything, exactly the
  // pre-App-Builder sidebar.
  const allowedSections = activeApp ? new Set(activeApp.object_keys.map(sectionFor)) : null;
  function sectionVisible(section: Section): boolean {
    if (!allowedSections || !SCOPABLE_SECTIONS.has(section)) return true;
    return allowedSections.has(section);
  }

  return (
    <div className="app-shell">
      <nav className="sidebar">
        <div className="sidebar-brand">Lanesra OS</div>
        {accessibleApps.length > 0 && (
          <select
            className="app-switcher"
            value={activeAppId ?? ""}
            onChange={(e) => onSwitchApp(e.target.value || null)}
            aria-label="Switch app"
            style={{ width: "100%", marginBottom: 8 }}
          >
            <option value="">All</option>
            {accessibleApps.map((a) => (
              <option key={a.app.id} value={a.app.id}>
                {a.app.icon} {a.app.name}
              </option>
            ))}
          </select>
        )}
        {NAV_ITEMS.filter((item) => (!item.adminOnly || isAdmin) && sectionVisible(item.section)).map((item) => (
          <button
            key={item.section}
            className={`nav-item${active === item.section ? " active" : ""}`}
            onClick={() => onNavigate(item.section)}
          >
            {item.label}
          </button>
        ))}
        {customObjects.filter((o) => sectionVisible(customObjectSection(o.key))).length > 0 && (
          <div className="sidebar-divider" />
        )}
        {customObjects
          .filter((o) => sectionVisible(customObjectSection(o.key)))
          .map((o) => (
            <button
              key={o.key}
              className={`nav-item${active === customObjectSection(o.key) ? " active" : ""}`}
              onClick={() => onNavigate(customObjectSection(o.key))}
            >
              {o.icon} {o.plural_label}
            </button>
          ))}
      </nav>
      <main className="main">
        <div className="topbar">
          <GlobalSearch customObjects={customObjects} onOpenResult={onOpenSearchResult} />
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <NotificationBell />
            <button
              className="link-button"
              style={{ fontSize: 13, color: "var(--text-muted)" }}
              onClick={() => onNavigate("account")}
            >
              {user.display_name} · {user.roles.join(", ")}
            </button>
            <button className="btn" onClick={onLogout}>
              Sign out
            </button>
          </div>
        </div>
        {children}
      </main>
    </div>
  );
}
