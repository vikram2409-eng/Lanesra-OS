/** A dependency-free horizontal bar, sized relative to the row's own max
 * value. Shared by the Reports screen's report tables and, since
 * Dashboard customization Phase 2, chart widgets on the live Dashboard -
 * both draw a custom report's grouped rows the same way. */
export function Bar({ value, max }: { value: number; max: number }) {
  const pct = max > 0 ? Math.max(2, Math.round((value / max) * 100)) : 0;
  return (
    <div style={{ background: "var(--surface-2, rgba(127,127,127,0.15))", borderRadius: 3, height: 8, width: 120 }}>
      <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)", borderRadius: 3 }} />
    </div>
  );
}
