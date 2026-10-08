import type { DashboardWidget, WidgetLayout } from "./types";

/**
 * Runtime UX Modernization (issue #198): geometry for the dashboard's
 * responsive drag/resize grid. Hand-rolled (no layout library, matching
 * this codebase's zero-chart-lib convention) - CSS Grid does the actual
 * column math via `gridColumn`/`gridRow`, this module only owns the
 * "what layout does an unplaced widget get" and "how tall is the grid"
 * questions.
 */
export const GRID_COLUMNS = 12;
export const ROW_HEIGHT = 90;
export const GRID_GAP = 16;
export const MIN_WIDGET_W = 2;
export const MIN_WIDGET_H = 2;

const DEFAULT_SIZE: Record<string, { w: number; h: number }> = {
  kpi: { w: 3, h: 2 },
  chart: { w: 6, h: 4 },
  record_list: { w: 6, h: 4 },
  table: { w: 8, h: 5 },
  saved_view: { w: 6, h: 4 },
  task_queue: { w: 4, h: 4 },
  agent_insight: { w: 3, h: 3 },
};

export function defaultSizeFor(kind: string): { w: number; h: number } {
  return DEFAULT_SIZE[kind] ?? { w: 4, h: 3 };
}

/**
 * Fills in a layout for every widget that doesn't have one yet (`layout:
 * null` - never placed, or saved before this field existed), via simple
 * shelf-packing: left-to-right in `widgets`' own order, wrapping at
 * `GRID_COLUMNS`, starting below the lowest already-placed widget. This is
 * not a general bin-packing solver - it only has to turn "never positioned"
 * into "somewhere reasonable," the same bar `screen_layout`'s initial-field
 * placement already set.  Widgets that already have an explicit `layout`
 * keep it untouched, including any overlap a user deliberately created.
 */
export function resolveLayouts(widgets: DashboardWidget[]): Map<string, WidgetLayout> {
  const result = new Map<string, WidgetLayout>();
  let maxY = 0;
  for (const w of widgets) {
    if (w.layout) {
      result.set(w.id, w.layout);
      maxY = Math.max(maxY, w.layout.y + w.layout.h);
    }
  }
  let cursorX = 0;
  let cursorY = maxY;
  let rowHeight = 0;
  for (const w of widgets) {
    if (w.layout) continue;
    const { w: width, h: height } = defaultSizeFor(w.kind);
    if (cursorX > 0 && cursorX + width > GRID_COLUMNS) {
      cursorX = 0;
      cursorY += rowHeight;
      rowHeight = 0;
    }
    result.set(w.id, { x: cursorX, y: cursorY, w: width, h: height });
    cursorX += width;
    rowHeight = Math.max(rowHeight, height);
  }
  return result;
}

/** Total grid row count spanned by every widget's layout - used to give the
 * grid container an explicit height so it doesn't collapse when every
 * widget is positioned absolutely via `gridRow`. */
export function gridRowCount(layouts: Map<string, WidgetLayout>): number {
  let max = 0;
  for (const l of layouts.values()) max = Math.max(max, l.y + l.h);
  return max;
}
