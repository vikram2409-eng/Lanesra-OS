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
  user,
  onLogout,
  customObjects,
  children,
}: {
  active: Section;
  onNavigate: (section: Section) => void;
  user: User;
  onLogout: () => void;
  customObjects: CustomObjectDefinition[];
  children: React.ReactNode;
}) {
  const isAdmin = user.roles.includes("Administrator");

  return (
    <div className="app-shell">
      <nav className="sidebar">
        <div className="sidebar-brand">Lanesra OS</div>
        {NAV_ITEMS.filter((item) => !item.adminOnly || isAdmin).map((item) => (
          <button
            key={item.section}
            className={`nav-item${active === item.section ? " active" : ""}`}
            onClick={() => onNavigate(item.section)}
          >
            {item.label}
          </button>
        ))}
        {customObjects.length > 0 && <div className="sidebar-divider" />}
        {customObjects.map((o) => (
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
          <div />
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
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
