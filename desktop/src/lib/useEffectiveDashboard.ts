import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

/** The widgets the Dashboard should render for the signed-in user - the
 * published dashboard layout whose roles include one of theirs, or the
 * workspace's Default if none do (see
 * `dashboard_layout_service::resolve_effective_dashboard` in the Rust
 * core). `data?.widgets` is null when no dashboard layout has ever been
 * published, including the auto-provisioned Default (which starts
 * unpublished) - callers should fall back to the pre-this-feature
 * Dashboard (the fixed KPI-row-plus-two-panels layout driven by
 * `dashboard_kpi_prefs`) in that case, exactly as it behaved before this
 * feature existed. */
export function useEffectiveDashboard() {
  return useQuery({
    queryKey: ["effectiveDashboardLayout"],
    queryFn: () => api.effectiveDashboardLayout(),
  });
}
