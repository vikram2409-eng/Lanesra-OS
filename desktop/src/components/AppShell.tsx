import type { User } from "../lib/types";

export type Section =
  | "dashboard"
  | "companies"
  | "contacts"
  | "products"
  | "opportunities"
  | "quotes"
  | "orders"
  | "invoices";

const NAV_ITEMS: { section: Section; label: string }[] = [
  { section: "dashboard", label: "Dashboard" },
  { section: "companies", label: "Companies" },
  { section: "contacts", label: "Contacts" },
  { section: "opportunities", label: "Sales Pipeline" },
  { section: "products", label: "Products" },
  { section: "quotes", label: "Quotes" },
  { section: "orders", label: "Orders" },
  { section: "invoices", label: "Invoices" },
];

export function AppShell({
  active,
  onNavigate,
  user,
  onLogout,
  children,
}: {
  active: Section;
  onNavigate: (section: Section) => void;
  user: User;
  onLogout: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="app-shell">
      <nav className="sidebar">
        <div className="sidebar-brand">Lanesra OS</div>
        {NAV_ITEMS.map((item) => (
          <button
            key={item.section}
            className={`nav-item${active === item.section ? " active" : ""}`}
            onClick={() => onNavigate(item.section)}
          >
            {item.label}
          </button>
        ))}
      </nav>
      <main className="main">
        <div className="topbar">
          <div />
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <span style={{ fontSize: 13, color: "var(--text-muted)" }}>
              {user.display_name} · {user.roles.join(", ")}
            </span>
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
