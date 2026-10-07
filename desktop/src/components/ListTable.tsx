import { useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { StatusBadge } from "./StatusBadge";
import { GroupHeaderRow } from "./GroupHeaderRow";
import { formatCents } from "../lib/money";
import type { BulkSelectionState } from "../lib/useBulkSelection";

export type ColumnFormat =
  | "text"
  | "number"
  | "status"
  | "owner"
  | "currency"
  | "percent"
  | "date"
  | "boolean"
  | "relationship"
  | "progress";

export type TableDensity = "comfortable" | "compact" | "dense";

export interface ListTableColumn<T> {
  key: string;
  label: string;
  format?: ColumnFormat;
  /** Raw value accessor - what gets formatted by `format`, and what a
   * custom `render` receives as its second argument. */
  getValue: (row: T) => unknown;
  /** Overrides the built-in formatter entirely when present. */
  render?: (row: T, value: unknown) => ReactNode;
  /** Default width in px before any user resize. */
  defaultWidth?: number;
  minWidth?: number;
  align?: "left" | "right" | "center";
}

export interface ListTableGroup<T> {
  label: string;
  rows: T[];
}

const DENSITY_OPTIONS: Array<{ key: TableDensity; label: string }> = [
  { key: "comfortable", label: "Comfortable" },
  { key: "compact", label: "Compact" },
  { key: "dense", label: "Dense" },
];

function loadJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}
function saveJson(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Best-effort only - a blocked/full localStorage just means widths
    // and density reset to default next launch, nothing is lost.
  }
}

function formatValue(format: ColumnFormat | undefined, value: unknown, resolveUser?: (id: string) => string | undefined): ReactNode {
  if (value === null || value === undefined || value === "") return "—";
  switch (format) {
    case "status":
      return <StatusBadge status={String(value)} />;
    case "owner": {
      const name = resolveUser?.(String(value));
      return name ?? "—";
    }
    case "currency":
      return formatCents(Number(value));
    case "percent":
      return `${Number(value)}%`;
    case "date": {
      const d = new Date(String(value));
      return Number.isNaN(d.getTime()) ? String(value) : d.toLocaleDateString();
    }
    case "boolean":
      return value ? "Yes" : "No";
    case "relationship": {
      const rel = value as { id: string; label: string } | null;
      return rel ? <span className="badge">{rel.label}</span> : "—";
    }
    case "progress": {
      const pct = Math.max(0, Math.min(100, Number(value)));
      return (
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div style={{ flex: 1, height: 6, borderRadius: 3, background: "var(--bg-elevated)", overflow: "hidden" }}>
            <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)" }} />
          </div>
          <span style={{ fontSize: 12, color: "var(--text-muted)", minWidth: 32, textAlign: "right" }}>{pct}%</span>
        </div>
      );
    }
    case "number":
      return String(value);
    default:
      return String(value);
  }
}

/**
 * The shared dynamic-column list table (Runtime UX Modernization, issue
 * #198): resizable/reorderable columns, sticky header, a density toggle,
 * and per-column formatting (status/owner/currency/date/relationship/
 * progress/boolean/number), driven by the `visibleKeys` a caller gets
 * from `useSavedViews().columnKeys` - this is the renderer that field was
 * added for, not a second storage shape. Column widths and the density
 * preference are a per-install convenience (localStorage), not part of
 * the Saved View record itself - only which columns show and in what
 * order is shared/synced.
 *
 * Below `mobileBreakpoint` the table collapses into one card per row
 * showing `priorityKeys` (default: the first 3 visible columns), per
 * UX-AC-18 (complex desktop column configuration must not compromise
 * runtime mobile usability). Both the table and card markup are always in
 * the DOM; only one is visible at a time via CSS so this needs no resize
 * listener.
 */
export function ListTable<T>({
  storageKey,
  columns,
  visibleKeys,
  onVisibleKeysChange,
  groups,
  getRowId,
  onRowClick,
  selection,
  resolveUser,
  density: densityProp,
  onDensityChange,
  priorityKeys,
  mobileCollapse = true,
  emptyMessage = "No records match the current filters.",
  stickyHeaderOffset = 0,
  actionsColumn,
  showGroupHeaders: showGroupHeadersProp,
}: {
  storageKey: string;
  columns: ListTableColumn<T>[];
  visibleKeys: string[] | null;
  onVisibleKeysChange: (keys: string[]) => void;
  groups: ListTableGroup<T>[];
  getRowId: (row: T) => string;
  onRowClick?: (row: T) => void;
  selection?: BulkSelectionState;
  resolveUser?: (userId: string) => string | undefined;
  density?: TableDensity;
  onDensityChange?: (d: TableDensity) => void;
  priorityKeys?: string[];
  mobileCollapse?: boolean;
  emptyMessage?: string;
  stickyHeaderOffset?: number;
  /** An always-visible trailing column (e.g. a row-level "Edit" button) -
   * not part of the toggleable/reorderable column set, so it doesn't
   * appear in the Columns picker and always renders last. */
  actionsColumn?: (row: T) => ReactNode;
  /** Overrides the default "show a group header row when there's more
   * than one group, or a single labelled one" heuristic - e.g. Tasks
   * suppresses it for its single-group tabs (the active tab button
   * already says "Today"/"Upcoming") but wants it for its "By owner" tab. */
  showGroupHeaders?: boolean;
}) {
  const [widths, setWidths] = useState<Record<string, number>>(() => loadJson(`listTableWidths:${storageKey}`, {}));
  const [internalDensity, setInternalDensity] = useState<TableDensity>(() => loadJson("listTableDensity", "comfortable" as TableDensity));
  const density = densityProp ?? internalDensity;
  const [columnPickerOpen, setColumnPickerOpen] = useState(false);
  const dragKeyRef = useRef<string | null>(null);
  const resizeRef = useRef<{ key: string; startX: number; startWidth: number } | null>(null);

  const orderedVisible = useMemo(() => {
    if (!visibleKeys || visibleKeys.length === 0) return columns;
    const byKey = new Map(columns.map((c) => [c.key, c]));
    const ordered = visibleKeys.map((k) => byKey.get(k)).filter((c): c is ListTableColumn<T> => !!c);
    return ordered.length > 0 ? ordered : columns;
  }, [columns, visibleKeys]);

  function setDensity(d: TableDensity) {
    if (onDensityChange) onDensityChange(d);
    else {
      setInternalDensity(d);
      saveJson("listTableDensity", d);
    }
  }

  function commitWidth(key: string, width: number) {
    setWidths((prev) => {
      const next = { ...prev, [key]: Math.max(width, columns.find((c) => c.key === key)?.minWidth ?? 60) };
      saveJson(`listTableWidths:${storageKey}`, next);
      return next;
    });
  }

  function onResizePointerDown(e: React.PointerEvent, key: string) {
    e.preventDefault();
    e.stopPropagation();
    const th = (e.currentTarget as HTMLElement).parentElement as HTMLElement;
    resizeRef.current = { key, startX: e.clientX, startWidth: th?.offsetWidth ?? 120 };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function onResizePointerMove(e: React.PointerEvent) {
    const active = resizeRef.current;
    if (!active) return;
    const delta = e.clientX - active.startX;
    commitWidth(active.key, active.startWidth + delta);
  }
  function onResizePointerUp(e: React.PointerEvent) {
    if (resizeRef.current) (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    resizeRef.current = null;
  }

  function reorderTo(targetKey: string) {
    const dragKey = dragKeyRef.current;
    dragKeyRef.current = null;
    if (!dragKey || dragKey === targetKey) return;
    const currentOrder = orderedVisible.map((c) => c.key);
    const from = currentOrder.indexOf(dragKey);
    const to = currentOrder.indexOf(targetKey);
    if (from === -1 || to === -1) return;
    const next = [...currentOrder];
    next.splice(from, 1);
    next.splice(to, 0, dragKey);
    onVisibleKeysChange(next);
  }

  function toggleColumnVisible(key: string) {
    const current = visibleKeys && visibleKeys.length > 0 ? visibleKeys : columns.map((c) => c.key);
    const next = current.includes(key) ? current.filter((k) => k !== key) : [...current, key];
    onVisibleKeysChange(next);
  }

  const hasRows = groups.some((g) => g.rows.length > 0);
  const showGroupHeaders = showGroupHeadersProp ?? (groups.length > 1 || (groups.length === 1 && groups[0].label !== ""));
  const colSpan = orderedVisible.length + (selection ? 1 : 0) + (actionsColumn ? 1 : 0);
  const cardPriorityKeys = priorityKeys ?? orderedVisible.slice(0, 3).map((c) => c.key);

  return (
    <div className={`list-table-wrap list-table-${density}`}>
      <div className="list-table-toolbar">
        <div className="segmented">
          {DENSITY_OPTIONS.map((opt) => (
            <button
              key={opt.key}
              type="button"
              className={`segmented-btn${density === opt.key ? " active" : ""}`}
              onClick={() => setDensity(opt.key)}
            >
              {opt.label}
            </button>
          ))}
        </div>
        <div style={{ position: "relative" }}>
          <button type="button" className="btn" onClick={() => setColumnPickerOpen((v) => !v)}>
            Columns
          </button>
          {columnPickerOpen && (
            <div className="list-table-column-picker">
              {columns.map((c) => {
                const checked = !visibleKeys || visibleKeys.length === 0 || visibleKeys.includes(c.key);
                return (
                  <label key={c.key} className="list-table-column-picker-row">
                    <input type="checkbox" checked={checked} onChange={() => toggleColumnVisible(c.key)} />
                    {c.label}
                  </label>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {!hasRows ? (
        <p className="empty-state">{emptyMessage}</p>
      ) : (
        <>
          <table className="list-table-table">
            <thead>
              <tr>
                {selection && (
                  <th style={{ width: 28, position: "sticky", top: stickyHeaderOffset, background: "var(--bg)", zIndex: 2 }}>
                    <input
                      type="checkbox"
                      checked={selection.allSelected}
                      ref={(el) => el && (el.indeterminate = selection.someSelected)}
                      onChange={selection.toggleAll}
                    />
                  </th>
                )}
                {orderedVisible.map((c) => (
                  <th
                    key={c.key}
                    draggable
                    onDragStart={() => (dragKeyRef.current = c.key)}
                    onDragOver={(e) => e.preventDefault()}
                    onDrop={() => reorderTo(c.key)}
                    style={{
                      width: widths[c.key] ?? c.defaultWidth,
                      minWidth: c.minWidth ?? 60,
                      position: "sticky",
                      top: stickyHeaderOffset,
                      background: "var(--bg)",
                      zIndex: 2,
                      cursor: "grab",
                    }}
                  >
                    <span>{c.label}</span>
                    <span
                      className="list-table-resize-handle"
                      draggable={false}
                      onDragStart={(e) => e.preventDefault()}
                      onMouseDown={(e) => e.stopPropagation()}
                      onPointerDown={(e) => onResizePointerDown(e, c.key)}
                      onPointerMove={onResizePointerMove}
                      onPointerUp={onResizePointerUp}
                    />
                  </th>
                ))}
                {actionsColumn && (
                  <th style={{ position: "sticky", top: stickyHeaderOffset, background: "var(--bg)", zIndex: 2 }} />
                )}
              </tr>
            </thead>
            <tbody>
              {groups.map((group) => (
                <RowsForGroup
                  key={group.label || "_"}
                  group={group}
                  showGroupHeaders={showGroupHeaders}
                  colSpan={colSpan}
                  orderedVisible={orderedVisible}
                  selection={selection}
                  getRowId={getRowId}
                  onRowClick={onRowClick}
                  resolveUser={resolveUser}
                  actionsColumn={actionsColumn}
                />
              ))}
            </tbody>
          </table>

          {mobileCollapse && (
            <div className="list-table-cards">
              {groups.flatMap((group) => group.rows).map((row) => (
                <div
                  key={getRowId(row)}
                  className="list-table-card"
                  onClick={() => onRowClick?.(row)}
                >
                  {selection && (
                    <input
                      type="checkbox"
                      checked={selection.isSelected(getRowId(row))}
                      onClick={(e) => e.stopPropagation()}
                      onChange={() => selection.toggle(getRowId(row))}
                    />
                  )}
                  {cardPriorityKeys.map((key) => {
                    const col = columns.find((c) => c.key === key);
                    if (!col) return null;
                    const value = col.getValue(row);
                    return (
                      <div key={key} className="list-table-card-field">
                        <span className="list-table-card-label">{col.label}</span>
                        <span className="list-table-card-value">{col.render ? col.render(row, value) : formatValue(col.format, value, resolveUser)}</span>
                      </div>
                    );
                  })}
                  {actionsColumn && (
                    <div className="list-table-card-field" onClick={(e) => e.stopPropagation()}>
                      {actionsColumn(row)}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

function RowsForGroup<T>({
  group,
  showGroupHeaders,
  colSpan,
  orderedVisible,
  selection,
  getRowId,
  onRowClick,
  resolveUser,
  actionsColumn,
}: {
  group: ListTableGroup<T>;
  showGroupHeaders: boolean;
  colSpan: number;
  orderedVisible: ListTableColumn<T>[];
  selection?: BulkSelectionState;
  getRowId: (row: T) => string;
  onRowClick?: (row: T) => void;
  resolveUser?: (id: string) => string | undefined;
  actionsColumn?: (row: T) => ReactNode;
}) {
  return (
    <>
      {showGroupHeaders && <GroupHeaderRow label={group.label} colSpan={colSpan} />}
      {group.rows.map((row) => {
        const id = getRowId(row);
        return (
          <tr key={id} style={{ cursor: onRowClick ? "pointer" : undefined }}>
            {selection && (
              <td onClick={(e) => e.stopPropagation()}>
                <input type="checkbox" checked={selection.isSelected(id)} onChange={() => selection.toggle(id)} />
              </td>
            )}
            {orderedVisible.map((c) => {
              const value = c.getValue(row);
              return (
                <td key={c.key} onClick={() => onRowClick?.(row)} style={{ textAlign: c.align }}>
                  {c.render ? c.render(row, value) : formatValue(c.format, value, resolveUser)}
                </td>
              );
            })}
            {actionsColumn && <td onClick={(e) => e.stopPropagation()}>{actionsColumn(row)}</td>}
          </tr>
        );
      })}
    </>
  );
}
