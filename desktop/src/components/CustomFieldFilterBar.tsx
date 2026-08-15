import type { CustomFieldFilters } from "../lib/useCustomFieldFilters";

/**
 * Filter controls for one list screen's `is_filterable` custom fields -
 * renders nothing when the entity type has none, so every list screen can
 * mount it unconditionally without an extra "any filterable fields?"
 * check at each call site.
 */
export function CustomFieldFilterBar({ filters }: { filters: CustomFieldFilters }) {
  if (filters.filterableDefs.length === 0) return null;

  return (
    <div style={{ display: "flex", flexWrap: "wrap", gap: 8, alignItems: "center", margin: "0 0 12px" }}>
      {filters.filterableDefs.map((def) => (
        <div key={def.id} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          <label style={{ fontSize: 11, color: "var(--text-muted)" }}>{def.label}</label>
          {def.field_type === "select" && (
            <select value={filters.filters[def.key] ?? ""} onChange={(e) => filters.setFilter(def.key, e.target.value)}>
              <option value="">All</option>
              {def.options.map((o) => (
                <option key={o} value={o}>
                  {o}
                </option>
              ))}
            </select>
          )}
          {def.field_type === "boolean" && (
            <select value={filters.filters[def.key] ?? ""} onChange={(e) => filters.setFilter(def.key, e.target.value)}>
              <option value="">All</option>
              <option value="true">Yes</option>
              <option value="false">No</option>
            </select>
          )}
          {(def.field_type === "text" || def.field_type === "number" || def.field_type === "date") && (
            <input
              type={def.field_type === "date" ? "date" : def.field_type === "number" ? "number" : "text"}
              value={filters.filters[def.key] ?? ""}
              onChange={(e) => filters.setFilter(def.key, e.target.value)}
              placeholder={def.field_type === "text" ? "Contains..." : undefined}
              style={{ minWidth: 120 }}
            />
          )}
        </div>
      ))}
      {filters.isActive && (
        <button type="button" className="link-button" style={{ fontSize: 12, alignSelf: "flex-end" }} onClick={filters.clearFilters}>
          Clear filters
        </button>
      )}
    </div>
  );
}
