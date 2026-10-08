import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "./api";
import { useCustomFieldFilters } from "./useCustomFieldFilters";
import type { SavedView, SavedViewInput } from "./types";

export type SortDirection = "asc" | "desc";

/**
 * Saved Views (product backlog "Saved Views & Bulk Actions"): persists a
 * named filter/sort/grouping combination for `objectKey` and lets it be
 * reloaded, updated, or (for an Administrator) set as the object-wide
 * default. Wraps `useCustomFieldFilters` rather than replacing it, so a
 * screen keeps using the exact same `filters` object (and
 * `CustomFieldFilterBar`) it already had - a saved view is "remember what
 * I had set," not a new query capability.
 *
 * Column visibility/order (Runtime UX Modernization, issue #198): the
 * `columns` array is the display order of visible column keys; `null`
 * means "use the table's own default set." `ListTable` is the one real
 * reader of it now - see that component for the resizable/reorderable
 * dynamic-column table itself.
 */
export function useSavedViews(objectKey: string) {
  const queryClient = useQueryClient();
  const filters = useCustomFieldFilters(objectKey);
  const [activeViewId, setActiveViewId] = useState<string | null>(null);
  const [sortField, setSortFieldState] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");
  const [groupByField, setGroupByField] = useState<string | null>(null);
  const [columnKeys, setColumnKeys] = useState<string[] | null>(null);

  const viewsQuery = useQuery({
    queryKey: ["savedViews", objectKey],
    queryFn: () => api.listSavedViews(objectKey),
  });
  const views = viewsQuery.data ?? [];
  const activeView = views.find((v) => v.id === activeViewId) ?? null;

  function selectView(view: SavedView | null) {
    setActiveViewId(view?.id ?? null);
    if (!view) {
      filters.clearFilters();
      setSortFieldState(null);
      setSortDirection("asc");
      setGroupByField(null);
      setColumnKeys(null);
      return;
    }
    Object.entries(view.filters).forEach(([key, value]) => filters.setFilter(key, value));
    setSortFieldState(view.sort_field);
    setSortDirection(view.sort_direction);
    setGroupByField(view.group_by_field);
    setColumnKeys(view.columns);
  }

  function setSort(field: string | null) {
    if (field === sortField) {
      setSortDirection((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortFieldState(field);
      setSortDirection("asc");
    }
  }

  function currentInput(name: string, visibility: "private" | "shared"): SavedViewInput {
    return {
      object_key: objectKey,
      name,
      visibility,
      filters: filters.filters,
      sort_field: sortField,
      sort_direction: sortDirection,
      columns: columnKeys,
      group_by_field: groupByField,
    };
  }

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["savedViews", objectKey] });

  const createMutation = useMutation({
    mutationFn: (input: SavedViewInput) => api.createSavedView(input),
    onSuccess: (created) => {
      invalidate();
      setActiveViewId(created.id);
    },
  });
  const updateMutation = useMutation({
    mutationFn: ({ id, input }: { id: string; input: SavedViewInput }) => api.updateSavedView(id, input),
    onSuccess: invalidate,
  });
  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteSavedView(id),
    onSuccess: () => {
      invalidate();
      setActiveViewId(null);
    },
  });
  const setDefaultMutation = useMutation({
    mutationFn: (id: string) => api.setSavedViewDefault(id),
    onSuccess: invalidate,
  });

  function saveAsNew(name: string, visibility: "private" | "shared") {
    return createMutation.mutateAsync(currentInput(name, visibility));
  }
  function updateCurrent() {
    if (!activeView) return Promise.resolve();
    return updateMutation.mutateAsync({ id: activeView.id, input: currentInput(activeView.name, activeView.visibility) });
  }
  function deleteView(id: string) {
    return deleteMutation.mutateAsync(id);
  }
  function setDefault(id: string) {
    return setDefaultMutation.mutateAsync(id);
  }

  const isDirty = useMemo(() => {
    if (!activeView) return sortField !== null || groupByField !== null || columnKeys !== null || filters.isActive;
    return (
      JSON.stringify(activeView.filters) !== JSON.stringify(filters.filters) ||
      activeView.sort_field !== sortField ||
      activeView.sort_direction !== sortDirection ||
      activeView.group_by_field !== groupByField ||
      JSON.stringify(activeView.columns) !== JSON.stringify(columnKeys)
    );
  }, [activeView, filters.filters, filters.isActive, sortField, sortDirection, groupByField, columnKeys]);

  /**
   * Applies the current sort/group (not filters - the caller already
   * filters its own rows via `filters.matches`) to `rows`, always
   * returning grouped buckets so the render path is uniform: no grouping
   * chosen is just one bucket with an empty label.
   */
  function transform<T>(rows: T[], getValue: (row: T, fieldKey: string) => string): Array<{ label: string; rows: T[] }> {
    let working = rows;
    if (sortField) {
      const dir = sortDirection === "desc" ? -1 : 1;
      working = [...rows].sort((a, b) => getValue(a, sortField).localeCompare(getValue(b, sortField)) * dir);
    }
    if (!groupByField) return [{ label: "", rows: working }];
    const buckets = new Map<string, T[]>();
    for (const row of working) {
      const key = getValue(row, groupByField) || "—";
      const bucket = buckets.get(key);
      if (bucket) bucket.push(row);
      else buckets.set(key, [row]);
    }
    return [...buckets.entries()].map(([label, bucketRows]) => ({ label, rows: bucketRows }));
  }

  return {
    views,
    activeView,
    selectView,
    filters,
    sortField,
    sortDirection,
    setSort,
    groupByField,
    setGroupByField,
    columnKeys,
    setColumnKeys,
    saveAsNew,
    updateCurrent,
    deleteView,
    setDefault,
    isDirty,
    isSaving: createMutation.isPending || updateMutation.isPending,
    transform,
  };
}

export type SavedViewsState = ReturnType<typeof useSavedViews>;
