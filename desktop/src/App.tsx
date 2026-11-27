import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { AppShell, type Prefill, type Section } from "./components/AppShell";
import { SessionLock } from "./components/SessionLock";
import { TaskReminderNotifier } from "./components/TaskReminderNotifier";
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
import { AdminPanel, type AdminTab } from "./features/settings/Settings";
import { AssistantPage } from "./features/assistant/AssistantPage";
import { LaneBlueprintPage } from "./features/lanes/LaneBlueprint";
import { LaneRunnerPage } from "./features/lanes/LaneRunner";
import { LaneBuilderPage } from "./features/builder/LaneBuilderPage";
import { NewLaneStart, type NewLaneStartChoice } from "./features/builder/NewLaneStart";
import { blankLaneDefinition, draftedLaneDefinition } from "./features/builder/builderUtils";
import { CustomObjectRecords } from "./features/customObjects/CustomObjectRecords";
import { Account } from "./features/account/Account";
import { VoiceContextProvider } from "./features/voice/VoiceContext";
import { api } from "./lib/api";
import { applyThemeTokens } from "./lib/applyTheme";
import type { LaneDefinition, User, Workspace } from "./lib/types";

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
        api
          .getPublishedTheme()
          .then((theme) => theme && applyThemeTokens(theme.tokens))
          .catch(() => {});
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
  // App Builder: which accessible app (if any) is scoping the sidebar and
  // Dashboard right now - see AppShell's own doc comment on activeAppId.
  // Same query AppShell itself runs (React Query dedupes to one fetch);
  // run here too since Dashboard needs the active app's dashboard_id,
  // which AppShell has no reason to expose back up.
  const accessibleApps = useQuery({ queryKey: ["accessibleApps"], queryFn: () => api.listAccessibleApps() });
  const [activeAppId, setActiveAppId] = useState<string | null>(null);
  const activeAppDashboardId = accessibleApps.data?.find((a) => a.app.id === activeAppId)?.app.dashboard_id ?? null;
  function switchApp(appId: string | null) {
    setActiveAppId(appId);
    // A section the just-left app exposed may not exist in the new scope
    // (or "All") - land back on Dashboard rather than risk stranding on a
    // now-hidden nav item, same reasoning as a prefill's one-shot reset.
    setSection("dashboard");
  }
  const [prefill, setPrefill] = useState<Prefill | null>(null);
  // WP-0707: the lane runner is a full screen opened from a record or a
  // list ("Start lane run"); closing it returns to the section underneath.
  const [laneRun, setLaneRun] = useState<{ runId: string; recordLabel: string } | null>(null);
  // WP-0718: the lane builder is the same kind of full screen, opened
  // from the blueprint page ("Open in builder" with a lane key, "New
  // lane" with null). It is not a `Section` — the Section union lives
  // in AppShell.tsx, outside this WP's ownership — so it rides on this
  // overlay state exactly like the runner does.
  // WP-0722: a new lane can also carry a starting definition — a blank
  // one with the object already picked (object-first entry), or the
  // Assistant's draft (guided start, "describe it"). LaneBuilderPage
  // already renders an `initialDefinition` when handed one.
  const [builderLane, setBuilderLane] = useState<{
    laneKey: string | null;
    initialDefinition?: LaneDefinition;
  } | null>(null);
  // WP-0722: the guided start dialog. `objectKey` is pre-filled when the
  // start comes from an object's page; null when it comes from the Lanes
  // section. The dialog is the same overlay kind as the builder above.
  const [newLaneStart, setNewLaneStart] = useState<{ objectKey?: string | null } | null>(null);
  const [newLaneBusy, setNewLaneBusy] = useState(false);
  const [newLaneError, setNewLaneError] = useState<string | null>(null);

  /// WP-0722: land the guided start's choice in the builder. Blank opens
  /// a blank definition on the chosen object — so the Object card never
  /// appears and publish/dry-run are available without touching it.
  /// Describe asks the Assistant to draft (WP-0715, always a draft) and
  /// opens the builder on the drafted lane; a draft the builder cannot
  /// read, or a failed draft call, keeps the dialog open with the reason.
  async function handleNewLaneCreate(choice: NewLaneStartChoice) {
    if (choice.start === "describe") {
      setNewLaneBusy(true);
      setNewLaneError(null);
      try {
        const drafted = await api.draftLaneFromDescription(choice.description, choice.objectKey);
        const parsed = draftedLaneDefinition(drafted);
        if (!parsed) throw new Error("The Assistant's draft could not be read — try again.");
        const definition =
          choice.name === "Untitled lane"
            ? parsed.definition
            : { ...parsed.definition, name: choice.name };
        setNewLaneStart(null);
        setBuilderLane({ laneKey: parsed.laneKey, initialDefinition: definition });
      } catch (err) {
        setNewLaneError(err instanceof Error ? err.message : "Drafting the lane failed — try again.");
      } finally {
        setNewLaneBusy(false);
      }
      return;
    }
    setNewLaneStart(null);
    setBuilderLane({
      laneKey: null,
      initialDefinition: {
        ...blankLaneDefinition(),
        name: choice.name,
        object_key: choice.objectKey,
      },
    });
  }
  function navigateTo(target: Section, pre: Prefill) {
    setPrefill(pre);
    setSection(target);
  }
  // Admin Control Center Modernization (issue #197): the Command
  // Palette's "Admin" results navigate here from outside AdminPanel's own
  // tree - `key` always changes so AdminPanel's effect fires even when
  // re-picking the same tab.
  const [pendingAdminTab, setPendingAdminTab] = useState<{ tab: AdminTab; key: number } | null>(null);
  function openAdminTab(tab: AdminTab) {
    setSection("admin");
    setPendingAdminTab({ tab, key: Date.now() });
  }
  function clearPrefill() {
    setPrefill(null);
  }
  const activeCustomObject = section.startsWith("custom:")
    ? customObjects.data?.find((o) => o.key === section.slice("custom:".length))
    : undefined;

  // ADM-WF-11: date_reached/due_overdue/scheduled workflows only run while
  // Lanesra is open (no OS-level background scheduler in Personal
  // Workspace) - once on load so anything missed while closed fires
  // promptly, then on a 5-minute interval for the rest of the session.
  // AI & Agentic Layer, Phase 6b: the same interval also drains
  // whatever's queued for the AI Agent Foundry (a due schedule Trigger,
  // a run_ai_agent workflow action) and, incidentally, Integration Hub's
  // own long-unwired call_connector_action queue - see
  // drain_pending_async_work's own doc comment. Kept as a second call,
  // not folded into runScheduledWorkflows, since that one stays
  // deliberately sync (scheduled workflow triggers never make a network
  // call) while this one is genuinely async.
  useEffect(() => {
    api.runScheduledWorkflows().catch(() => {});
    api.drainPendingAsyncWork().catch(() => {});
    const interval = setInterval(() => {
      api.runScheduledWorkflows().catch(() => {});
      api.drainPendingAsyncWork().catch(() => {});
    }, 5 * 60_000);
    return () => clearInterval(interval);
  }, []);

  return (
    <SessionLock user={user}>
      <TaskReminderNotifier currentUserId={user.id} />
      <VoiceContextProvider>
      <AppShell
        active={section}
        onNavigate={setSection}
        onOpenSearchResult={(target, id) => navigateTo(target, { openId: id })}
        onOpenAdminTab={(tab) => openAdminTab(tab as AdminTab)}
        user={user}
        onLogout={onLogout}
        customObjects={customObjects.data ?? []}
        activeAppId={activeAppId}
        onSwitchApp={switchApp}
      >
        {newLaneStart ? (
          <NewLaneStart
            initialObjectKey={newLaneStart.objectKey}
            objects={customObjects.data ?? []}
            busy={newLaneBusy}
            error={newLaneError}
            onCancel={() => {
              setNewLaneStart(null);
              setNewLaneError(null);
            }}
            onCreate={handleNewLaneCreate}
          />
        ) : laneRun ? (
          <LaneRunnerPage runId={laneRun.runId} recordLabel={laneRun.recordLabel} onClose={() => setLaneRun(null)} />
        ) : builderLane ? (
          <LaneBuilderPage
            laneKey={builderLane.laneKey}
            initialDefinition={builderLane.initialDefinition}
            onClose={() => setBuilderLane(null)}
          />
        ) : (
          <>
        {section === "dashboard" && (
          <Dashboard
            onNavigate={setSection}
            onOpenRecord={(target, id) => navigateTo(target, { openId: id })}
            appDashboardId={activeAppId ? activeAppDashboardId : null}
          />
        )}
        {section === "companies" && (
          <Companies
            prefill={prefill}
            onPrefillConsumed={clearPrefill}
            onNavigateTo={navigateTo}
            onOpenAdminTab={(tab) => openAdminTab(tab as AdminTab)}
          />
        )}
        {section === "contacts" && (
          <Contacts prefill={prefill} onPrefillConsumed={clearPrefill} onNavigateTo={navigateTo} />
        )}
        {section === "products" && (
          <Products
            prefill={prefill}
            onPrefillConsumed={clearPrefill}
            onNavigateTo={navigateTo}
            onOpenAdminTab={(tab) => openAdminTab(tab as AdminTab)}
          />
        )}
        {section === "opportunities" && <Opportunities prefill={prefill} onPrefillConsumed={clearPrefill} />}
        {section === "quotes" && (
          <Quotes prefill={prefill} onPrefillConsumed={clearPrefill} onNavigateTo={navigateTo} />
        )}
        {section === "orders" && (
          <Orders prefill={prefill} onPrefillConsumed={clearPrefill} onNavigateTo={navigateTo} />
        )}
        {section === "invoices" && (
          <Invoices prefill={prefill} onPrefillConsumed={clearPrefill} onNavigateTo={navigateTo} />
        )}
        {section === "contracts" && (
          <Contracts prefill={prefill} onPrefillConsumed={clearPrefill} onNavigateTo={navigateTo} />
        )}
        {section === "tasks" && (
          <Tasks
            currentUserId={user.id}
            prefill={prefill}
            onPrefillConsumed={clearPrefill}
            onNavigateTo={navigateTo}
          />
        )}
        {section === "reports" && <Reports isAdmin={user.roles.includes("Administrator")} />}
        {section === "assistant" && (
          <AssistantPage user={user} onOpenAdminTab={(tab) => openAdminTab(tab as AdminTab)} />
        )}
        {section === "lanes" && (
          <LaneBlueprintPage
            onOpenInBuilder={(laneKey) => setBuilderLane({ laneKey })}
            onNewLane={() => {
              setNewLaneError(null);
              setNewLaneStart({});
            }}
            onOpenRun={(runId, recordLabel) => setLaneRun({ runId, recordLabel })}
          />
        )}
        {section === "admin" && <AdminPanel openAdminTab={pendingAdminTab} />}
        {section === "account" && <Account user={user} />}
        {activeCustomObject && (
          <CustomObjectRecords
            definition={activeCustomObject}
            prefill={prefill}
            onPrefillConsumed={clearPrefill}
            onOpenLaneRun={(runId, recordLabel) => setLaneRun({ runId, recordLabel })}
            onNewLane={(objectKey) => {
              setNewLaneError(null);
              setNewLaneStart({ objectKey });
            }}
          />
        )}
          </>
        )}
      </AppShell>
      </VoiceContextProvider>
    </SessionLock>
  );
}
