import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { KPI_DEFS, kpiLabel } from "../dashboard/kpis";

/**
 * Admin screen for FR-KPI - lets an Administrator choose which Dashboard
 * KPI tiles show, out of the full catalog in dashboard/kpis.tsx. An empty
 * selection resets to "show every KPI, default order". Reordering isn't
 * exposed yet (see product backlog) - the stored preference is already an
 * ordered list, so a future phase can add drag-to-reorder without a
 * schema change.
 */
export function DashboardKpiAdmin() {
  const queryClient = useQueryClient();
  const workspace = useQuery({ queryKey: ["workspaceStatus"], queryFn: () => api.workspaceStatus() });
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (workspace.data && !loaded) {
      const prefsJson = workspace.data.dashboard_kpi_prefs;
      if (prefsJson) {
        try {
          const keys: unknown = JSON.parse(prefsJson);
          if (Array.isArray(keys)) setSelected(new Set(keys.map(String)));
        } catch {
          // malformed prefs - fall through to "everything selected"
        }
      }
      if (!prefsJson) setSelected(new Set(KPI_DEFS.map((k) => k.key)));
      setLoaded(true);
    }
  }, [workspace.data, loaded]);

  const save = useMutation({
    mutationFn: (keys: string[]) => api.setDashboardKpis({ keys }),
    onSuccess: () => {
      setError(null);
      setSuccess(true);
      queryClient.invalidateQueries({ queryKey: ["workspaceStatus"] });
    },
    onError: (err) => {
      setSuccess(false);
      setError(err instanceof ApiError ? err.message : "Could not save KPI preferences");
    },
  });

  function toggle(key: string) {
    setSuccess(false);
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Dashboard KPIs</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Choose which KPI tiles show at the top of the Dashboard. Unchecking all of them resets to showing every KPI.
      </p>
      {error && <div className="error-banner">{error}</div>}
      {success && <div className="success-banner">Saved.</div>}
      <div className="form-grid">
        {KPI_DEFS.map((kpi) => (
          <div className="form-field" key={kpi.key}>
            <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input type="checkbox" checked={selected.has(kpi.key)} onChange={() => toggle(kpi.key)} />
              {kpiLabel(kpi.key)}
            </label>
          </div>
        ))}
      </div>
      <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
        <button
          className="btn btn-primary"
          onClick={() => {
            const ordered = KPI_DEFS.filter((k) => selected.has(k.key)).map((k) => k.key);
            save.mutate(selected.size === KPI_DEFS.length ? [] : ordered);
          }}
          disabled={save.isPending}
        >
          Save
        </button>
        <button
          className="btn"
          type="button"
          onClick={() => {
            setSelected(new Set(KPI_DEFS.map((k) => k.key)));
            save.mutate([]);
          }}
          disabled={save.isPending}
        >
          Show all
        </button>
      </div>
    </div>
  );
}
