import { useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { GRID_COLUMNS, GRID_GAP, MIN_WIDGET_H, MIN_WIDGET_W, ROW_HEIGHT, gridRowCount, resolveLayouts } from "../lib/dashboardGrid";
import type { DashboardWidget, WidgetLayout } from "../lib/types";

interface DragState {
  id: string;
  mode: "move" | "resize";
  startClientX: number;
  startClientY: number;
  startLayout: WidgetLayout;
  colStep: number;
  rowStep: number;
  moved: boolean;
}

/**
 * Runtime UX Modernization (issue #198): the shared widget grid used by
 * both the live Dashboard (view mode, `editable=false`) and the dashboard-
 * layout editor's canvas (edit mode) - one drag/resize implementation
 * instead of the admin screen's old badge-chip reorder list and the live
 * Dashboard's separate hardcoded 2-column sections.
 *
 * Positioning is plain CSS Grid (`gridColumn`/`gridRow` per widget, from
 * `WidgetLayout`'s `x`/`y`/`w`/`h` cell coordinates) - no layout library,
 * matching this codebase's zero-chart-lib convention (see `Bar.tsx`).
 * Drag/resize is hand-rolled pointer-capture, the same pattern
 * `ListTable`'s column resize already established: a ref (not React state)
 * holds the in-progress gesture so every pointermove doesn't re-render the
 * whole grid, and the dragged/resized widget snaps to the nearest grid
 * cell on every move rather than following the pointer pixel-for-pixel -
 * simpler than a smooth-follow-then-snap ghost, and sufficient for a
 * dashboard (unlike a canvas you stare at while dragging, like a diagram
 * tool would need).
 */
export function DashboardGrid({
  widgets,
  renderWidget,
  renderHeader,
  editable = false,
  selectedId = null,
  onSelect,
  onLayoutChange,
}: {
  widgets: DashboardWidget[];
  renderWidget: (widget: DashboardWidget) => ReactNode;
  /** Edit-mode-only chrome rendered above each widget's content (label +
   * remove button) - `undefined` in view mode. */
  renderHeader?: (widget: DashboardWidget) => ReactNode;
  editable?: boolean;
  selectedId?: string | null;
  onSelect?: (id: string) => void;
  onLayoutChange?: (id: string, layout: WidgetLayout) => void;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const [liveLayout, setLiveLayout] = useState<{ id: string; layout: WidgetLayout } | null>(null);

  const layouts = useMemo(() => resolveLayouts(widgets), [widgets]);
  const totalRows = gridRowCount(layouts);

  function beginDrag(e: React.PointerEvent, widgetId: string, mode: "move" | "resize") {
    if (!editable) return;
    e.preventDefault();
    e.stopPropagation();
    const containerRect = containerRef.current?.getBoundingClientRect();
    const startLayout = layouts.get(widgetId);
    if (!containerRect || !startLayout) return;
    const colStep = (containerRect.width - GRID_GAP * (GRID_COLUMNS - 1)) / GRID_COLUMNS + GRID_GAP;
    const rowStep = ROW_HEIGHT + GRID_GAP;
    dragRef.current = { id: widgetId, mode, startClientX: e.clientX, startClientY: e.clientY, startLayout, colStep, rowStep, moved: false };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onDragMove(e: React.PointerEvent) {
    const active = dragRef.current;
    if (!active) return;
    const deltaX = e.clientX - active.startClientX;
    const deltaY = e.clientY - active.startClientY;
    if (Math.abs(deltaX) > 3 || Math.abs(deltaY) > 3) active.moved = true;
    if (!active.moved) return;
    const deltaCols = Math.round(deltaX / active.colStep);
    const deltaRows = Math.round(deltaY / active.rowStep);
    if (active.mode === "move") {
      const x = Math.max(0, Math.min(GRID_COLUMNS - active.startLayout.w, active.startLayout.x + deltaCols));
      const y = Math.max(0, active.startLayout.y + deltaRows);
      setLiveLayout({ id: active.id, layout: { ...active.startLayout, x, y } });
    } else {
      const w = Math.max(MIN_WIDGET_W, Math.min(GRID_COLUMNS - active.startLayout.x, active.startLayout.w + deltaCols));
      const h = Math.max(MIN_WIDGET_H, active.startLayout.h + deltaRows);
      setLiveLayout({ id: active.id, layout: { ...active.startLayout, w, h } });
    }
  }

  function onDragEnd(e: React.PointerEvent) {
    const active = dragRef.current;
    dragRef.current = null;
    if (!active) return;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    if (!active.moved) {
      setLiveLayout(null);
      onSelect?.(active.id);
      return;
    }
    const finalLayout = liveLayout?.id === active.id ? liveLayout.layout : active.startLayout;
    setLiveLayout(null);
    onLayoutChange?.(active.id, finalLayout);
  }

  return (
    <div
      ref={containerRef}
      className="dashboard-grid"
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${GRID_COLUMNS}, 1fr)`,
        gridAutoRows: `${ROW_HEIGHT}px`,
        gap: GRID_GAP,
        minHeight: totalRows > 0 ? totalRows * (ROW_HEIGHT + GRID_GAP) - GRID_GAP : undefined,
      }}
    >
      {widgets.map((w) => {
        const layout = liveLayout?.id === w.id ? liveLayout.layout : layouts.get(w.id);
        if (!layout) return null;
        return (
          <div
            key={w.id}
            className={`dashboard-grid-item${editable ? " editable" : ""}${selectedId === w.id ? " selected" : ""}`}
            style={{ gridColumn: `${layout.x + 1} / span ${layout.w}`, gridRow: `${layout.y + 1} / span ${layout.h}` }}
          >
            {editable && (
              <div
                className="dashboard-grid-item-handle"
                onPointerDown={(e) => beginDrag(e, w.id, "move")}
                onPointerMove={onDragMove}
                onPointerUp={onDragEnd}
              >
                <span aria-hidden>⠿</span>
                {renderHeader?.(w)}
              </div>
            )}
            <div
              className="dashboard-grid-item-body"
              onClick={editable ? () => onSelect?.(w.id) : undefined}
            >
              {renderWidget(w)}
            </div>
            {editable && (
              <div
                className="dashboard-grid-item-resize"
                onPointerDown={(e) => beginDrag(e, w.id, "resize")}
                onPointerMove={onDragMove}
                onPointerUp={onDragEnd}
                title="Drag to resize"
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
