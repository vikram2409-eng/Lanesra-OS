import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

export const FEATURE_FLAGS_QUERY_KEY = ["featureFlags"] as const;

/** The resolved feature flags for this workspace (WP-0010, ADR-0012). */
export function useFeatureFlags() {
  return useQuery({
    queryKey: FEATURE_FLAGS_QUERY_KEY,
    queryFn: () => api.listFeatureFlags(),
    staleTime: 60_000,
  });
}

/**
 * Whether a feature is on. While the list is loading, or if it fails to
 * load, the answer is `false`: an unfinished feature must never flash into
 * view. Render a flagged entry point only when this returns true; the
 * server enforces the same flag on the feature's own commands.
 */
export function useFeature(key: string): boolean {
  const { data } = useFeatureFlags();
  return data?.flags.some((f) => f.key === key && f.enabled) ?? false;
}

/** Whether the Admin → Feature flags screen has anything to show. */
export function useShowFeatureFlagsAdmin(): boolean {
  const { data } = useFeatureFlags();
  return !!data && (data.developer_mode || data.flags.some((f) => f.can_toggle));
}
