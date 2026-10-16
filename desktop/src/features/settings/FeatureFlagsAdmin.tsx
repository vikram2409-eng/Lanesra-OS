import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { FEATURE_FLAGS_QUERY_KEY, useFeatureFlags } from "../../lib/useFeature";

const STAGE_LABEL: Record<string, string> = {
  building: "Being built",
  preview: "Preview",
  ga: "Released",
};

/**
 * Admin → Workspace → Feature flags (WP-0010, ADR-0012). Lists the flags
 * this workspace can see and lets an Administrator override the edition
 * default. `building` flags appear only in a developer build with
 * developer mode on; release builds never show or enable them. The admin
 * landing page hides this screen when there is nothing to change.
 */
export function FeatureFlagsAdmin() {
  const queryClient = useQueryClient();
  const flags = useFeatureFlags();
  const change = useMutation({
    mutationFn: (input: { key: string; enabled: boolean | null }) => api.setFeatureFlagOverride(input),
    onSuccess: (data) => queryClient.setQueryData(FEATURE_FLAGS_QUERY_KEY, data),
  });

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Feature flags</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Turn features that are still in preview on or off for this workspace. Every change is recorded in the audit
        log. Features ship in desktop and Team Workspace together, and appear in the online demo in the same release.
      </p>
      {flags.data?.developer_mode && (
        <p className="callout callout-warning" style={{ fontSize: 13 }}>
          Developer mode is on: features still being built are listed. Release builds never show or enable them.
        </p>
      )}
      {change.error && (
        <p className="error">{change.error instanceof ApiError ? change.error.message : String(change.error)}</p>
      )}
      {flags.isLoading && <p>Loading...</p>}
      {flags.data && flags.data.flags.length === 0 && <p>No features are waiting to be turned on.</p>}
      {flags.data && flags.data.flags.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Feature</th>
              <th>Stage</th>
              <th>On</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {flags.data.flags.map((f) => (
              <tr key={f.key}>
                <td>
                  <div>{f.description}</div>
                  <code style={{ fontSize: 12 }}>{f.key}</code>
                </td>
                <td>{STAGE_LABEL[f.stage] ?? f.stage}</td>
                <td>
                  <input
                    type="checkbox"
                    aria-label={`Turn ${f.key} on`}
                    checked={f.enabled}
                    disabled={!f.can_toggle || change.isPending}
                    onChange={(e) => change.mutate({ key: f.key, enabled: e.target.checked })}
                  />
                </td>
                <td>
                  {f.overridden && (
                    <button
                      className="link-button"
                      disabled={change.isPending}
                      onClick={() => change.mutate({ key: f.key, enabled: null })}
                    >
                      Reset to default
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
