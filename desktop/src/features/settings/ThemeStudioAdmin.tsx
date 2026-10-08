import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { applyThemeTokens } from "../../lib/applyTheme";
import type { ThemeColorTokens, ThemePreset, ThemeTokens, WorkspaceTheme } from "../../lib/types";

const COLOR_FIELDS: { key: keyof ThemeColorTokens; label: string }[] = [
  { key: "brand_primary", label: "Brand primary" },
  { key: "brand_secondary", label: "Brand secondary" },
  { key: "surface_app", label: "App background" },
  { key: "surface_card", label: "Card surface" },
  { key: "surface_sidebar", label: "Sidebar surface" },
  { key: "border_default", label: "Border" },
  { key: "text_primary", label: "Primary text" },
  { key: "text_secondary", label: "Secondary text" },
  { key: "status_success", label: "Success" },
  { key: "status_warning", label: "Warning" },
  { key: "status_danger", label: "Danger" },
  { key: "status_info", label: "Info" },
];

type DraftState = { id: string | null; name: string; preset_key: string | null; tokens: ThemeTokens };

function cloneTokens(tokens: ThemeTokens): ThemeTokens {
  return JSON.parse(JSON.stringify(tokens));
}

function statusBadgeClass(status: string): string {
  if (status === "published") return "badge badge-success";
  if (status === "draft") return "badge badge-warning";
  return "badge";
}

/**
 * UX/UI Modernization, Phase A (issue #191): Design Tokens & Theme Studio.
 * Pick/customize brand colors, typography, shape and density, starting
 * from one of 4 curated presets or from scratch. Draft -> Published ->
 * Archived versioning with rollback, same discipline Agent Versioning
 * already established elsewhere in this admin panel - Publish is blocked
 * outright on a critical WCAG contrast failure (see theme_service.rs),
 * with no override, matching the spec's own "prefer no override in early
 * versions" instruction.
 */
export function ThemeStudioAdmin() {
  const queryClient = useQueryClient();
  const versionsQuery = useQuery({ queryKey: ["themeVersions"], queryFn: () => api.listThemeVersions() });
  const presetsQuery = useQuery({ queryKey: ["themePresets"], queryFn: () => api.listThemePresets() });
  const versions = versionsQuery.data ?? [];
  const presets = presetsQuery.data ?? [];
  const published = versions.find((v) => v.status === "published");

  const [draft, setDraft] = useState<DraftState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [issues, setIssues] = useState<{ pair_label: string; ratio: number; required_ratio: number }[] | null>(null);
  const [checking, setChecking] = useState(false);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["themeVersions"] });
  }

  function startFromPreset(preset: ThemePreset) {
    const [key, name, , tokens] = preset;
    setDraft({ id: null, name: `${name} (custom)`, preset_key: key, tokens: cloneTokens(tokens) });
    setIssues(null);
    setError(null);
  }

  function editDraft(v: WorkspaceTheme) {
    setDraft({ id: v.id, name: v.name, preset_key: v.preset_key, tokens: cloneTokens(v.tokens) });
    setIssues(null);
    setError(null);
  }

  function updateColor(key: keyof ThemeColorTokens, value: string) {
    setDraft((d) => (d ? { ...d, tokens: { ...d.tokens, color: { ...d.tokens.color, [key]: value } } } : d));
  }

  function updateChartColor(index: number, value: string) {
    setDraft((d) => {
      if (!d) return d;
      const palette = [...d.tokens.chart.palette];
      palette[index] = value;
      return { ...d, tokens: { ...d.tokens, chart: { palette } } };
    });
  }

  const save = useMutation({
    mutationFn: () => {
      if (!draft) throw new Error("no draft");
      return api.saveThemeDraft(draft.id, { name: draft.name, preset_key: draft.preset_key, tokens: draft.tokens });
    },
    onSuccess: (saved) => {
      setDraft({ id: saved.id, name: saved.name, preset_key: saved.preset_key, tokens: saved.tokens });
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this draft"),
  });

  const publish = useMutation({
    mutationFn: (id: string) => api.publishTheme(id),
    onSuccess: (theme) => {
      setDraft(null);
      setError(null);
      invalidate();
      applyThemeTokens(theme.tokens);
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not publish this theme"),
  });

  const rollback = useMutation({
    mutationFn: (fromVersion: number) => api.rollbackTheme(fromVersion),
    onSuccess: (theme) => {
      setError(null);
      invalidate();
      applyThemeTokens(theme.tokens);
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not roll back to that version"),
  });

  const remove = useMutation({
    mutationFn: (id: string) => api.deleteThemeDraft(id),
    onSuccess: (_void, id) => {
      if (draft?.id === id) setDraft(null);
      invalidate();
    },
  });

  async function checkContrast() {
    if (!draft) return;
    setChecking(true);
    try {
      const result = await api.validateThemeTokens(draft.tokens);
      setIssues(result);
    } finally {
      setChecking(false);
    }
  }

  return (
    <div>
      <div className="card">
        <h3 style={{ margin: 0 }}>Theme Studio</h3>
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          Brand colors, typography, shape and density for this workspace. Start from a curated preset or from
          scratch, then Publish once it passes a WCAG contrast check. Publishing never overwrites history - rolling
          back creates a new Draft from an older version's tokens and publishes that.
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 12, marginTop: 8 }}>
          {presets.map((preset) => {
            const [key, name, blurb, tokens] = preset;
            return (
              <button
                key={key}
                className="admin-cat-item"
                style={{ textAlign: "left", display: "block", padding: 12 }}
                onClick={() => startFromPreset(preset)}
              >
                <div style={{ display: "flex", gap: 4, marginBottom: 8 }}>
                  {[tokens.color.brand_primary, tokens.color.brand_secondary, tokens.color.surface_sidebar, tokens.color.status_success].map(
                    (c, i) => (
                      <span key={i} style={{ width: 20, height: 20, borderRadius: 4, background: c, border: "1px solid var(--border)" }} />
                    ),
                  )}
                </div>
                <strong>{name}</strong>
                <p style={{ margin: "4px 0 0", fontSize: 12, color: "var(--text-muted)" }}>{blurb}</p>
              </button>
            );
          })}
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <div className="card">
        <div className="toolbar">
          <h3 style={{ margin: 0 }}>Versions</h3>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Version</th>
                <th>Name</th>
                <th>Status</th>
                <th>Published</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {versions.map((v) => (
                <tr key={v.id}>
                  <td>v{v.version}</td>
                  <td>{v.name}</td>
                  <td>
                    <span className={statusBadgeClass(v.status)}>{v.status}</span>
                  </td>
                  <td>{v.published_at ? new Date(v.published_at).toLocaleString() : "-"}</td>
                  <td style={{ display: "flex", gap: 6, justifyContent: "flex-end" }}>
                    {v.status === "draft" && (
                      <>
                        <button className="btn" onClick={() => editDraft(v)}>
                          Edit
                        </button>
                        <button className="btn btn-primary" onClick={() => publish.mutate(v.id)} disabled={publish.isPending}>
                          Publish
                        </button>
                        <button className="btn btn-danger" onClick={() => remove.mutate(v.id)} disabled={remove.isPending}>
                          Delete
                        </button>
                      </>
                    )}
                    {v.status === "archived" && (
                      <button className="btn" onClick={() => rollback.mutate(v.version)} disabled={rollback.isPending}>
                        Roll back to this
                      </button>
                    )}
                  </td>
                </tr>
              ))}
              {versions.length === 0 && (
                <tr>
                  <td colSpan={5} style={{ color: "var(--text-muted)" }}>
                    No theme versions yet - start from a preset above. The workspace keeps its current look until you Publish one.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {draft && (
        <div className="card">
          <div className="toolbar">
            <h3 style={{ margin: 0 }}>{draft.id ? "Edit draft" : "New draft"}</h3>
            <button className="btn" onClick={() => setDraft(null)}>
              Close
            </button>
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, alignItems: "start" }}>
            <div>
              <div className="form-field">
                <label>Name</label>
                <input value={draft.name} onChange={(e) => setDraft({ ...draft, name: e.target.value })} />
              </div>

              <h4>Colors</h4>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
                {COLOR_FIELDS.map(({ key, label }) => (
                  <div className="form-field" key={key}>
                    <label>{label}</label>
                    <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      <input
                        type="color"
                        value={draft.tokens.color[key]}
                        onChange={(e) => updateColor(key, e.target.value)}
                        style={{ width: 36, height: 30, padding: 0 }}
                      />
                      <input value={draft.tokens.color[key]} onChange={(e) => updateColor(key, e.target.value)} style={{ flex: 1 }} />
                    </div>
                  </div>
                ))}
              </div>

              <h4 title="The color series a multi-row chart (Reports, dashboard chart widgets) cycles through by group index - distinct from the semantic brand/status colors above.">
                Chart palette
              </h4>
              <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
                {draft.tokens.chart.palette.map((c, i) => (
                  <div className="form-field" key={i} style={{ width: 90 }}>
                    <label>Series {i + 1}</label>
                    <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      <input
                        type="color"
                        value={c}
                        onChange={(e) => updateChartColor(i, e.target.value)}
                        style={{ width: 36, height: 30, padding: 0 }}
                      />
                      <input value={c} onChange={(e) => updateChartColor(i, e.target.value)} style={{ width: 70 }} />
                    </div>
                  </div>
                ))}
              </div>

              <h4>Typography</h4>
              <div className="form-field">
                <label>Font family</label>
                <input
                  value={draft.tokens.typography.font_family}
                  onChange={(e) =>
                    setDraft({ ...draft, tokens: { ...draft.tokens, typography: { ...draft.tokens.typography, font_family: e.target.value } } })
                  }
                />
              </div>
              <div className="form-field">
                <label>Base size (px)</label>
                <input
                  type="number"
                  value={draft.tokens.typography.base_size_px}
                  onChange={(e) =>
                    setDraft({
                      ...draft,
                      tokens: { ...draft.tokens, typography: { ...draft.tokens.typography, base_size_px: Number(e.target.value) } },
                    })
                  }
                />
              </div>

              <h4>Shape &amp; density</h4>
              <div className="form-field">
                <label>Corner radius</label>
                <select
                  value={draft.tokens.shape.radius_scale}
                  onChange={(e) => setDraft({ ...draft, tokens: { ...draft.tokens, shape: { radius_scale: e.target.value } } })}
                >
                  <option value="sharp">Sharp</option>
                  <option value="soft">Soft</option>
                  <option value="rounded">Rounded</option>
                </select>
              </div>
              <div className="form-field">
                <label>Density</label>
                <select value={draft.tokens.density} onChange={(e) => setDraft({ ...draft, tokens: { ...draft.tokens, density: e.target.value } })}>
                  <option value="comfortable">Comfortable</option>
                  <option value="compact">Compact</option>
                  <option value="dense">Dense</option>
                </select>
              </div>

              <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
                <button className="btn" onClick={checkContrast} disabled={checking}>
                  Check contrast
                </button>
                <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>
                  Save draft
                </button>
              </div>

              {issues && issues.length === 0 && <p style={{ color: "var(--success)", fontSize: 13 }}>All checked pairs pass WCAG AA (4.5:1).</p>}
              {issues && issues.length > 0 && (
                <div className="error-banner" style={{ marginTop: 8 }}>
                  <strong>Contrast issues - Publish is blocked until these pass:</strong>
                  <ul style={{ margin: "4px 0 0", paddingLeft: 18 }}>
                    {issues.map((i) => (
                      <li key={i.pair_label}>
                        {i.pair_label}: {i.ratio.toFixed(2)}:1 (needs {i.required_ratio.toFixed(1)}:1)
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>

            <div>
              <h4>Live preview</h4>
              <div
                style={{
                  background: draft.tokens.color.surface_app,
                  border: `1px solid ${draft.tokens.color.border_default}`,
                  borderRadius: 10,
                  padding: 16,
                  fontFamily: draft.tokens.typography.font_family,
                  fontSize: draft.tokens.typography.base_size_px,
                }}
              >
                <div
                  style={{
                    background: draft.tokens.color.surface_sidebar,
                    color: "#fff",
                    borderRadius: 6,
                    padding: "8px 12px",
                    marginBottom: 12,
                    fontWeight: 700,
                  }}
                >
                  Lanesra OS
                </div>
                <div
                  style={{
                    background: draft.tokens.color.surface_card,
                    border: `1px solid ${draft.tokens.color.border_default}`,
                    borderRadius: 8,
                    padding: 12,
                  }}
                >
                  <p style={{ color: draft.tokens.color.text_primary, margin: "0 0 4px", fontWeight: 600 }}>Sample card</p>
                  <p style={{ color: draft.tokens.color.text_secondary, margin: "0 0 10px" }}>Secondary text sits here.</p>
                  <button
                    style={{
                      background: draft.tokens.color.brand_primary,
                      color: "#fff",
                      border: "none",
                      borderRadius: 6,
                      padding: "6px 14px",
                      marginRight: 8,
                    }}
                  >
                    Primary action
                  </button>
                  <span
                    style={{
                      background: draft.tokens.color.status_success,
                      color: "#fff",
                      borderRadius: 999,
                      padding: "2px 10px",
                      fontSize: 12,
                      marginRight: 6,
                    }}
                  >
                    Success
                  </span>
                  <span
                    style={{
                      background: draft.tokens.color.status_warning,
                      color: "#fff",
                      borderRadius: 999,
                      padding: "2px 10px",
                      fontSize: 12,
                      marginRight: 6,
                    }}
                  >
                    Warning
                  </span>
                  <span style={{ background: draft.tokens.color.status_danger, color: "#fff", borderRadius: 999, padding: "2px 10px", fontSize: 12 }}>
                    Danger
                  </span>
                </div>
                <div style={{ display: "flex", gap: 4, marginTop: 12 }}>
                  {draft.tokens.chart.palette.map((c, i) => (
                    <div key={i} style={{ flex: 1, height: 28, background: c, borderRadius: 4 }} title={`Series ${i + 1}`} />
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {published && !draft && (
        <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
          Currently published: <strong>{published.name}</strong> (v{published.version}), applied live across the app.
        </p>
      )}
    </div>
  );
}
