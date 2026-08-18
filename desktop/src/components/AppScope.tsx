import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import type { AppDefinition } from "../lib/types";

/**
 * Per-app scoped automation: business rules, workflow definitions, and
 * dashboard layouts can each optionally be tagged with the App Builder
 * app they belong to (migration 0028's `app_id`) instead of always being
 * workspace-wide - see that migration's own doc comment for the full
 * rationale. Shared here so BusinessRulesAdmin, WorkflowAutomationAdmin,
 * and DashboardLayoutsAdmin all get the same "App" scope selector (in
 * the builder) and "App" filter (in the list) rather than three
 * hand-rolled copies.
 */
export function useApps() {
  return useQuery({ queryKey: ["apps"], queryFn: () => api.listApps() });
}

/** The builder's own "which app does this belong to" control - defaults
 * to "Workspace-wide" (app_id: null), matching every rule/workflow/
 * dashboard's behavior before this feature existed. */
export function AppScopeSelect({
  apps,
  value,
  onChange,
}: {
  apps: AppDefinition[];
  value: string | null;
  onChange: (appId: string | null) => void;
}) {
  return (
    <label style={{ display: "flex", flexDirection: "column", gap: 4, fontSize: 13 }}>
      App
      <select value={value ?? ""} onChange={(e) => onChange(e.target.value === "" ? null : e.target.value)}>
        <option value="">Workspace-wide</option>
        {apps.map((a) => (
          <option key={a.id} value={a.id}>
            {a.icon} {a.name}
          </option>
        ))}
      </select>
    </label>
  );
}

/** The list view's own "show me just this app's rules/workflows/
 * dashboards" filter - a pill row matching the entity-type tab row's own
 * style, single-select, defaulting to "All". Purely a client-side filter
 * (the list is already fetched in full); "Workspace-wide" is its own
 * pill since a null app_id is a real, common category, not "no filter". */
export function AppScopeFilter({
  apps,
  value,
  onChange,
}: {
  apps: AppDefinition[];
  value: "all" | "none" | string;
  onChange: (value: "all" | "none" | string) => void;
}) {
  if (apps.length === 0) return null;
  return (
    <div className="tab-row" style={{ marginBottom: 8 }}>
      <button className={`tab${value === "all" ? " active" : ""}`} onClick={() => onChange("all")}>
        All apps
      </button>
      <button className={`tab${value === "none" ? " active" : ""}`} onClick={() => onChange("none")}>
        Workspace-wide
      </button>
      {apps.map((a) => (
        <button key={a.id} className={`tab${value === a.id ? " active" : ""}`} onClick={() => onChange(a.id)}>
          {a.icon} {a.name}
        </button>
      ))}
    </div>
  );
}

/** Matches a record's `app_id` against an `AppScopeFilter` selection. */
export function matchesAppFilter(appId: string | null, filter: "all" | "none" | string): boolean {
  if (filter === "all") return true;
  if (filter === "none") return appId === null;
  return appId === filter;
}
