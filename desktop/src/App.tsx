import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { AppShell, type Section } from "./components/AppShell";
import { FirstRun } from "./features/firstRun/FirstRun";
import { Login } from "./features/auth/Login";
import { Dashboard } from "./features/dashboard/Dashboard";
import { Companies } from "./features/companies/Companies";
import { Contacts } from "./features/contacts/Contacts";
import { Products } from "./features/products/Products";
import { Opportunities } from "./features/opportunities/Opportunities";
import { Quotes } from "./features/quotes/Quotes";
import { Orders } from "./features/orders/Orders";
import { Invoices } from "./features/invoices/Invoices";
import { Contracts } from "./features/contracts/Contracts";
import { Tasks } from "./features/tasks/Tasks";
import { Reports } from "./features/reports/Reports";
import { AdminPanel } from "./features/settings/Settings";
import { CustomObjectRecords } from "./features/customObjects/CustomObjectRecords";
import { Account } from "./features/account/Account";
import { api } from "./lib/api";
import type { User, Workspace } from "./lib/types";

type BootState =
  | { phase: "loading" }
  | { phase: "first-run" }
  | { phase: "login"; workspace: Workspace }
  | { phase: "ready"; workspace: Workspace; user: User };

export function App() {
  const [boot, setBoot] = useState<BootState>({ phase: "loading" });
  const [section, setSection] = useState<Section>("dashboard");

  useEffect(() => {
    (async () => {
      const workspace = await api.workspaceStatus();
      if (!workspace) {
        setBoot({ phase: "first-run" });
        return;
      }
      const user = await api.currentUser();
      if (user) {
        setBoot({ phase: "ready", workspace, user });
      } else {
        setBoot({ phase: "login", workspace });
      }
    })();
  }, []);

  if (boot.phase === "loading") {
    return (
      <div className="centered-screen">
        <p>Loading Lanesra OS...</p>
      </div>
    );
  }

  if (boot.phase === "first-run") {
    return (
      <FirstRun
        onComplete={(workspace, user) => setBoot({ phase: "ready", workspace, user })}
      />
    );
  }

  if (boot.phase === "login") {
    return (
      <Login
        businessName={boot.workspace.business_name}
        onLogin={(user) => setBoot({ phase: "ready", workspace: boot.workspace, user })}
      />
    );
  }

  async function handleLogout() {
    await api.logout();
    setBoot({ phase: "login", workspace: (boot as { workspace: Workspace }).workspace });
  }

  return <Ready section={section} setSection={setSection} user={boot.user} onLogout={handleLogout} />;
}

/** Split out so its hooks (the active-custom-objects query) only run once
 * a workspace and a signed-in user actually exist. */
function Ready({
  section,
  setSection,
  user,
  onLogout,
}: {
  section: Section;
  setSection: (s: Section) => void;
  user: User;
  onLogout: () => void;
}) {
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const activeCustomObject = section.startsWith("custom:")
    ? customObjects.data?.find((o) => o.key === section.slice("custom:".length))
    : undefined;

  // ADM-WF-11: date_reached/due_overdue/scheduled workflows only run while
  // Lanesra is open (no OS-level background scheduler in Personal
  // Workspace) - once on load so anything missed while closed fires
  // promptly, then on a 5-minute interval for the rest of the session.
  useEffect(() => {
    api.runScheduledWorkflows().catch(() => {});
    const interval = setInterval(() => {
      api.runScheduledWorkflows().catch(() => {});
    }, 5 * 60_000);
    return () => clearInterval(interval);
  }, []);

  return (
    <AppShell active={section} onNavigate={setSection} user={user} onLogout={onLogout} customObjects={customObjects.data ?? []}>
      {section === "dashboard" && <Dashboard onNavigate={setSection} />}
      {section === "companies" && <Companies />}
      {section === "contacts" && <Contacts />}
      {section === "products" && <Products />}
      {section === "opportunities" && <Opportunities />}
      {section === "quotes" && <Quotes />}
      {section === "orders" && <Orders />}
      {section === "invoices" && <Invoices />}
      {section === "contracts" && <Contracts />}
      {section === "tasks" && <Tasks currentUserId={user.id} />}
      {section === "reports" && <Reports isAdmin={user.roles.includes("Administrator")} />}
      {section === "admin" && <AdminPanel />}
      {section === "account" && <Account user={user} />}
      {activeCustomObject && <CustomObjectRecords definition={activeCustomObject} />}
    </AppShell>
  );
}
