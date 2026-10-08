import type { ThemeTokens } from "./types";

// Runtime UX Modernization (issue #198): how many --chart-N slots the rest
// of the app (Bar.tsx) cycles through - see that file and styles.css's
// :root defaults. Fixed, not derived from a theme's own palette length, so
// every chart-coloring call site can cycle through a stable range.
export const CHART_PALETTE_SIZE = 6;

// UX/UI Modernization, Phase A (issue #191): applies a Published theme's
// tokens as inline CSS custom properties on :root, overriding styles.css's
// static defaults - see that file's own comment on which vars this covers
// (color + typography) and which it doesn't yet (shape/density). A
// workspace with no Published theme never calls this, so it keeps exactly
// the pre-Theme-Studio look.
export function applyThemeTokens(tokens: ThemeTokens): void {
  const root = document.documentElement.style;
  root.setProperty("--bg", tokens.color.surface_app);
  root.setProperty("--bg-elevated", tokens.color.surface_card);
  root.setProperty("--surface-sidebar", tokens.color.surface_sidebar);
  root.setProperty("--border", tokens.color.border_default);
  root.setProperty("--text", tokens.color.text_primary);
  root.setProperty("--text-muted", tokens.color.text_secondary);
  root.setProperty("--accent", tokens.color.brand_primary);
  root.setProperty("--accent-secondary", tokens.color.brand_secondary);
  root.setProperty("--success", tokens.color.status_success);
  root.setProperty("--warning", tokens.color.status_warning);
  root.setProperty("--danger", tokens.color.status_danger);
  root.setProperty("--info", tokens.color.status_info);
  root.setProperty("--font-family", tokens.typography.font_family);
  root.setProperty("--base-font-size", `${tokens.typography.base_size_px}px`);
  // A palette shorter than CHART_PALETTE_SIZE (the Rust-side minimum is 3)
  // cycles to fill every slot, so no --chart-N var is ever left undefined.
  const palette = tokens.chart.palette;
  for (let i = 0; i < CHART_PALETTE_SIZE; i++) {
    if (palette.length > 0) root.setProperty(`--chart-${i + 1}`, palette[i % palette.length]);
  }
}
