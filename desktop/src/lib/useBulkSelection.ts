import { useMemo, useState } from "react";

/** Multi-select state for a list screen's Bulk Actions bar - selection is
 * keyed by id, so it survives a re-sort/re-group of the same underlying
 * rows without losing track of which records are checked. */
export function useBulkSelection<T>(rows: T[], getId: (row: T) => string) {
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const allIds = useMemo(() => rows.map(getId), [rows, getId]);
  const selectedIds = useMemo(() => allIds.filter((id) => selected.has(id)), [allIds, selected]);
  const allSelected = allIds.length > 0 && selectedIds.length === allIds.length;
  const someSelected = selectedIds.length > 0 && !allSelected;

  function isSelected(id: string) {
    return selected.has(id);
  }
  function toggle(id: string) {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }
  function toggleAll() {
    setSelected((current) => (current.size === allIds.length ? new Set() : new Set(allIds)));
  }
  function clear() {
    setSelected(new Set());
  }

  return { selectedIds, isSelected, toggle, toggleAll, clear, allSelected, someSelected, count: selectedIds.length };
}

export type BulkSelectionState = ReturnType<typeof useBulkSelection>;
