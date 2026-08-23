import { useState } from "react";

import type { SavedViewsState } from "../lib/useSavedViews";
import { useIsAdmin } from "../lib/useCurrentUser";

/** A field a Sort by / Group by picker can offer - built-in fields are
 * supplied by the calling screen (they vary per entity), custom fields
 * come from `filters.filterableDefs` automatically. */
export type SortableField = { key: string; label: string };

export function SavedViewBar({
  views,
  fields,
  hideSortGroup,
}: {
  views: SavedViewsState;
  fields: SortableField[];
  /** Some screens (Tasks' Today/Upcoming/Overdue/Owner tabs) already have
   * their own purpose-built grouping the generic sort/group-by picker
   * would only compete with - set this to keep Saved Views to filters +
   * view management there, rather than showing a second, conflicting
   * grouping control. */
  hideSortGroup?: boolean;
}) {
  const isAdmin = useIsAdmin();
  const [mode, setMode] = useState<"idle" | "saving">("idle");
  const [name, setName] = useState("");
  const [visibility, setVisibility] = useState<"private" | "shared">("private");

  const allFields = [...fields, ...views.filters.filterableDefs.map((d) => ({ key: d.key, label: d.label }))];

  function startSave() {
    setName(views.activeView ? `${views.activeView.name} copy` : "");
    setVisibility("private");
    setMode("saving");
  }

  function confirmSave() {
    if (!name.trim()) return;
    views.saveAsNew(name.trim(), visibility).then(() => setMode("idle"));
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8, margin: "0 0 14px" }}>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 10, alignItems: "center" }}>
        <select
          value={views.activeView?.id ?? ""}
          onChange={(e) => views.selectView(views.views.find((v) => v.id === e.target.value) ?? null)}
          style={{ minWidth: 160 }}
        >
          <option value="">All records (no view)</option>
          {views.views.map((v) => (
            <option key={v.id} value={v.id}>
              {v.is_object_default ? "★ " : ""}
              {v.name}
              {v.visibility === "shared" ? " (shared)" : ""}
            </option>
          ))}
        </select>

        {!hideSortGroup && (
          <>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
              Sort by
              <select value={views.sortField ?? ""} onChange={(e) => views.setSort(e.target.value || null)}>
                <option value="">—</option>
                {allFields.map((f) => (
                  <option key={f.key} value={f.key}>
                    {f.label}
                  </option>
                ))}
              </select>
              {views.sortField && (
                <button type="button" className="link-button" onClick={() => views.setSort(views.sortField)} title="Toggle direction">
                  {views.sortDirection === "asc" ? "↑" : "↓"}
                </button>
              )}
            </label>

            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
              Group by
              <select value={views.groupByField ?? ""} onChange={(e) => views.setGroupByField(e.target.value || null)}>
                <option value="">—</option>
                {allFields.map((f) => (
                  <option key={f.key} value={f.key}>
                    {f.label}
                  </option>
                ))}
              </select>
            </label>
          </>
        )}

        {views.isDirty && (
          <button type="button" className="link-button" onClick={startSave} style={{ fontSize: 13 }}>
            Save as view…
          </button>
        )}
        {views.activeView && views.isDirty && (
          <button type="button" className="link-button" onClick={() => views.updateCurrent()} style={{ fontSize: 13 }}>
            Update "{views.activeView.name}"
          </button>
        )}
        {views.activeView && (
          <>
            {isAdmin && !views.activeView.is_object_default && (
              <button type="button" className="link-button" onClick={() => views.setDefault(views.activeView!.id)} style={{ fontSize: 13 }}>
                Set as default
              </button>
            )}
            <button
              type="button"
              className="link-button"
              onClick={() => {
                if (confirm(`Delete the view "${views.activeView!.name}"?`)) views.deleteView(views.activeView!.id);
              }}
              style={{ fontSize: 13, color: "var(--danger)" }}
            >
              Delete view
            </button>
          </>
        )}
      </div>

      {mode === "saving" && (
        <div style={{ display: "flex", gap: 8, alignItems: "center", padding: 10, border: "1px solid var(--border)", borderRadius: 8 }}>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="View name" autoFocus style={{ minWidth: 180 }} />
          <label style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 13 }}>
            <input type="radio" checked={visibility === "private"} onChange={() => setVisibility("private")} /> Only me
          </label>
          <label style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 13 }}>
            <input type="radio" checked={visibility === "shared"} onChange={() => setVisibility("shared")} /> Everyone
          </label>
          <button type="button" onClick={confirmSave} disabled={!name.trim() || views.isSaving}>
            Save
          </button>
          <button type="button" className="link-button" onClick={() => setMode("idle")}>
            Cancel
          </button>
        </div>
      )}
    </div>
  );
}
