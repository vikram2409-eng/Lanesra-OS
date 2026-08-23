import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "./api";
import type { CustomFieldDefinition } from "./types";

/**
 * List-view filtering (roadmap "Global search & list-view filtering",
 * second half): drives filter controls for whichever custom fields of
 * `entityType` an admin has flagged `is_filterable`, and filters client
 * side against the bulk fetch `api.listFilterableCustomFieldValues` -
 * matching global search's own "just enough to be useful, not a real
 * query engine" scope rather than pushing filter predicates to SQL.
 *
 * A `select`/`boolean` field filters by exact match (a dropdown of the
 * field's own options - there's nothing to be fuzzy about); `text` filters
 * by case-insensitive substring, the same rule global search itself uses;
 * `number`/`date` filter by exact match too - a range picker is more than
 * this first cut needs and can follow if it turns out to matter.
 */
export function useCustomFieldFilters(entityType: string) {
  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType],
    queryFn: () => api.listCustomFieldDefinitions(entityType, true),
  });
  const values = useQuery({
    queryKey: ["filterableCustomFieldValues", entityType],
    queryFn: () => api.listFilterableCustomFieldValues(entityType),
  });
  const [filters, setFilters] = useState<Record<string, string>>({});

  const filterableDefs = useMemo(() => (defs.data ?? []).filter((d) => d.is_filterable), [defs.data]);

  function setFilter(key: string, value: string) {
    setFilters((current) => {
      if (value === "") {
        const { [key]: _drop, ...rest } = current;
        return rest;
      }
      return { ...current, [key]: value };
    });
  }

  function clearFilters() {
    setFilters({});
  }

  function matches(entityId: string): boolean {
    const activeKeys = Object.keys(filters);
    if (activeKeys.length === 0) return true;
    const recordValues = values.data?.[entityId];
    if (!recordValues) return false;
    return activeKeys.every((key) => {
      const actual = recordValues[key];
      if (actual === undefined) return false;
      const def = filterableDefs.find((d) => d.key === key) as CustomFieldDefinition | undefined;
      if (def?.field_type === "text") return actual.toLowerCase().includes(filters[key].toLowerCase());
      return actual === filters[key];
    });
  }

  return {
    filterableDefs,
    filters,
    setFilter,
    clearFilters,
    matches,
    /** Raw per-record custom field values, keyed by entity id then custom
     * field key - exposed (in addition to `matches`) so a caller building
     * a generic sort/group-by (Saved Views) can look up a custom field's
     * value the same way `matches` itself does internally. */
    values: values.data ?? {},
    isActive: Object.keys(filters).length > 0,
    isLoading: defs.isLoading || values.isLoading,
  };
}

export type CustomFieldFilters = ReturnType<typeof useCustomFieldFilters>;
