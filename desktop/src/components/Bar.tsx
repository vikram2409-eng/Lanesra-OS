import { CHART_PALETTE_SIZE } from "../lib/applyTheme";

/** A dependency-free horizontal bar, sized relative to the row's own max
 * value. Shared by the Reports screen's report tables and the dashboard's
 * chart widgets - both draw a custom report's grouped rows the same way.
 *
 * `index` (Runtime UX Modernization, issue #198) cycles the bar's color
 * through the workspace's categorical chart palette (`--chart-1..6`, see
 * applyTheme.ts/styles.css) by group position, so a multi-row chart reads
 * as distinct series instead of every bar being the same brand accent.
 * Omit it for a single-series bar (e.g. a plain progress indicator), which
 * keeps the original single-accent-color look. */
export function Bar({ value, max, index }: { value: number; max: number; index?: number }) {
  const pct = max > 0 ? Math.max(2, Math.round((value / max) * 100)) : 0;
  const color = index === undefined ? "var(--accent)" : `var(--chart-${(index % CHART_PALETTE_SIZE) + 1})`;
  return (
    <div style={{ background: "var(--surface-2, rgba(127,127,127,0.15))", borderRadius: 3, height: 8, width: 120 }}>
      <div style={{ width: `${pct}%`, height: "100%", background: color, borderRadius: 3 }} />
    </div>
  );
}
