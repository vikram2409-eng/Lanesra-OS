/**
 * Object 360 pan/fit fix — pure view math for `GraphCanvas`.
 *
 * Extracted so the fit/clamp behaviour is unit-testable without a DOM:
 * - `fitView` — scale + center a content bounding box in the viewport.
 * - `clampPan` — keep the content from ever being panned completely
 *   off-screen (the share-link `panX/panY=-1000` empty-canvas bug class).
 * - `estimatePillSize` / `contentBounds` — content bounds that include
 *   pill extents (not just node centers), so fitting never half-clips
 *   the pills at the extremes.
 *
 * Pill geometry constants here MUST match `.og-pill` in `graph.css`
 * (font sizes, padding, dot, max-width for full labels) — the fit math
 * and the rendered pills use the same numbers.
 */

export interface ContentBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export interface ViewportSize {
  w: number;
  h: number;
}

export interface Pan {
  x: number;
  y: number;
}

export interface ViewTransform {
  zoom: number;
  pan: Pan;
}

export interface FitInsets {
  left?: number;
  top?: number;
  right?: number;
  bottom?: number;
}

/** Pill geometry — mirrors `.og-pill` / `.og-pill-root` / `.og-pill-full` in graph.css. */
export const PILL_FULL_MAX_WIDTH = 260;
export const PILL_FONT_SIZE = 12;
export const PILL_FONT_SIZE_ROOT = 13;
/** Average glyph width at the pill font size (slightly generous so the fit never clips). */
export const PILL_CHAR_WIDTH = 7;
export const PILL_CHAR_WIDTH_ROOT = 7.5;
/** Horizontal padding: 12px each side (root: 16px). */
export const PILL_PADDING_X = 24;
export const PILL_PADDING_X_ROOT = 32;
/** Type dot (8px) + gap (6px); root pills have no dot. */
export const PILL_DOT_WIDTH = 14;
/** Line height: font-size * 1.2. */
export const PILL_LINE_HEIGHT = 14.4;
export const PILL_LINE_HEIGHT_ROOT = 15.6;
/** Vertical padding: 6px top + 6px bottom (root: 8px + 8px). */
export const PILL_PADDING_Y = 12;
export const PILL_PADDING_Y_ROOT = 16;
/** Default (truncated) pills show at most this many glyphs. Mirrors `truncate(label, 18)` in PillNode. */
export const PILL_TRUNCATE_CHARS = 18;

export interface PillSizeOptions {
  isRoot?: boolean;
  /** Full labels wrap at PILL_FULL_MAX_WIDTH instead of truncating. */
  fullLabels?: boolean;
  /** Cluster pills append ` · <count>` to the label. */
  count?: number;
}

/** Rendered pill size for a label, using the same geometry as the CSS. */
export function estimatePillSize(label: string, opts: PillSizeOptions = {}): { width: number; height: number } {
  const isRoot = opts.isRoot ?? false;
  const charWidth = isRoot ? PILL_CHAR_WIDTH_ROOT : PILL_CHAR_WIDTH;
  const paddingX = isRoot ? PILL_PADDING_X_ROOT : PILL_PADDING_X;
  const paddingY = isRoot ? PILL_PADDING_Y_ROOT : PILL_PADDING_Y;
  const lineHeight = isRoot ? PILL_LINE_HEIGHT_ROOT : PILL_LINE_HEIGHT;
  const dotWidth = isRoot ? 0 : PILL_DOT_WIDTH;
  const suffix = opts.count != null ? ` · ${opts.count}` : "";
  const text = `${label}${suffix}`;

  if (opts.fullLabels) {
    const textWidth = text.length * charWidth;
    const available = PILL_FULL_MAX_WIDTH - paddingX - dotWidth;
    const lines = Math.max(1, Math.ceil(textWidth / available));
    return {
      width: Math.min(PILL_FULL_MAX_WIDTH, textWidth + paddingX + dotWidth),
      height: lines * lineHeight + paddingY,
    };
  }

  const effLen = text.length > PILL_TRUNCATE_CHARS ? PILL_TRUNCATE_CHARS : text.length;
  return {
    width: effLen * charWidth + paddingX + dotWidth,
    height: lineHeight + paddingY,
  };
}

/** Edge-label size (`.ontology-edge-label`: 10px font, 2px 8px padding). */
export function estimateEdgeLabelSize(label: string, access = false): { width: number; height: number } {
  const text = access ? `🔒 ${label}` : label;
  return { width: text.length * 5.5 + 16, height: 18 };
}

export interface ContentItem {
  /** Node center in layout coordinates. */
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Bounding box of centered items, including their full extents. Null when empty. */
export function contentBounds(items: ContentItem[]): ContentBounds | null {
  if (items.length === 0) return null;
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  items.forEach((item) => {
    minX = Math.min(minX, item.x - item.width / 2);
    maxX = Math.max(maxX, item.x + item.width / 2);
    minY = Math.min(minY, item.y - item.height / 2);
    maxY = Math.max(maxY, item.y + item.height / 2);
  });
  return { minX, minY, maxX, maxY };
}

/** Grow bounds by `dx`/`dy` on every side (e.g. cluster-ellipse overhang). */
export function expandBounds(bounds: ContentBounds, dx: number, dy: number): ContentBounds {
  return { minX: bounds.minX - dx, minY: bounds.minY - dy, maxX: bounds.maxX + dx, maxY: bounds.maxY + dy };
}

export interface PlacedPill {
  key: string;
  /** Center in layout coordinates. */
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface SeparateOptions {
  /** Minimum gap between pills, in layout px. Defaults to 8. */
  gap?: number;
  /** Max relaxation passes. Defaults to 100 — a stack of pills parked
   *  on one point converges slowly (each pass only resolves local
   *  overlaps), and passes are cheap (15 nodes ≈ 100 pairs/pass). */
  iterations?: number;
}

/**
 * Push overlapping pills apart (minimum-penetration axis, split evenly)
 * and clamp each pill's rect back inside `area`. Deterministic: pairs
 * are visited in array order, ties break by index, no randomness.
 *
 * Why this exists: the force layout parks node centers (clamped to
 * W-34) without knowing pill widths, so several wide full-label pills
 * can land on the same spot — e.g. six pills stacked on the right-edge
 * clamp wall — and cover each other's labels. This pass runs after the
 * layout (Object 360 only, where pills are wide) and only moves pills
 * far enough to stop overlapping; edges and cluster ellipses follow
 * the moved centers.
 */
export function separateOverlappingPills(
  items: PlacedPill[],
  area: ViewportSize,
  opts: SeparateOptions = {},
): Map<string, { x: number; y: number }> {
  const gap = opts.gap ?? 8;
  const iterations = opts.iterations ?? 100;
  const placed = items.map((item) => ({ ...item }));

  const clampIntoArea = (p: (typeof placed)[number]) => {
    p.x = Math.min(Math.max(p.x, p.width / 2), Math.max(p.width / 2, area.w - p.width / 2));
    p.y = Math.min(Math.max(p.y, p.height / 2), Math.max(p.height / 2, area.h - p.height / 2));
  };
  placed.forEach(clampIntoArea);

  for (let pass = 0; pass < iterations; pass++) {
    let moved = false;
    for (let i = 0; i < placed.length; i++) {
      for (let j = i + 1; j < placed.length; j++) {
        const a = placed[i];
        const b = placed[j];
        const dx = b.x - a.x;
        const dy = b.y - a.y;
        const overlapX = (a.width + b.width) / 2 + gap - Math.abs(dx);
        const overlapY = (a.height + b.height) / 2 + gap - Math.abs(dy);
        if (overlapX <= 0 || overlapY <= 0) continue;
        moved = true;
        // The +0.25 nudges each push a quarter px past the boundary so
        // the pair ends with a real clearance, not a float-error tie.
        if (overlapX < overlapY) {
          // Push apart horizontally; ties (dx == 0) split by index order.
          const push = overlapX / 2 + 0.25;
          const sign = dx === 0 ? 1 : Math.sign(dx);
          a.x -= push * sign;
          b.x += push * sign;
        } else {
          // Ties (dy == 0) split by index order: the later pill goes down.
          const push = overlapY / 2 + 0.25;
          const sign = dy === 0 ? 1 : Math.sign(dy);
          a.y -= push * sign;
          b.y += push * sign;
        }
        clampIntoArea(a);
        clampIntoArea(b);
      }
    }
    if (!moved) break;
  }

  return new Map(placed.map((p) => [p.key, { x: p.x, y: p.y }]));
}

export interface FitViewOptions {
  /** Breathing room around the content, in viewport px. Defaults to 40. */
  padding?: number;
  /** Never zoom in past this (fit is a framing, not a magnifier). Defaults to 1.5. */
  maxZoom?: number;
  /** Never zoom out past this. Defaults to 0.4 (matches MIN_ZOOM in GraphCanvas). */
  minZoom?: number;
  /** Reserve viewport edges (e.g. the inspector card over the canvas). */
  insets?: FitInsets;
}

/**
 * Scale + center `bounds` inside `viewport` (minus `insets`), with
 * `padding` around it. Returns a view whose pan is already clamped.
 */
export function fitView(bounds: ContentBounds | null, viewport: ViewportSize, opts: FitViewOptions = {}): ViewTransform {
  if (!bounds) return { zoom: 1, pan: { x: 0, y: 0 } };
  const padding = opts.padding ?? 40;
  const maxZoom = opts.maxZoom ?? 1.5;
  const minZoom = opts.minZoom ?? 0.4;
  const insets = opts.insets ?? {};
  const left = insets.left ?? 0;
  const top = insets.top ?? 0;
  const right = insets.right ?? 0;
  const bottom = insets.bottom ?? 0;
  const availW = Math.max(1, viewport.w - left - right);
  const availH = Math.max(1, viewport.h - top - bottom);
  const bw = bounds.maxX - bounds.minX + padding * 2;
  const bh = bounds.maxY - bounds.minY + padding * 2;
  const zoom = Math.min(maxZoom, Math.max(minZoom, Math.min(availW / bw, availH / bh)));
  const cx = (bounds.minX + bounds.maxX) / 2;
  const cy = (bounds.minY + bounds.maxY) / 2;
  return {
    zoom,
    pan: {
      x: left + availW / 2 - cx * zoom,
      y: top + availH / 2 - cy * zoom,
    },
  };
}

export interface ClampPanOptions {
  /**
   * Minimum px of the content bounding box that must stay visible.
   * Defaults to 80 — enough to grab and drag the graph back.
   */
  margin?: number;
}

/**
 * Clamp `pan` (at `zoom`) so the content bounding box can never leave
 * the viewport entirely: at least `margin` px of it stays on screen.
 * When the content is smaller than the viewport the fitted/centered pan
 * is inside the allowed range, so this is a no-op for it. When the
 * content is wider/taller than the viewport at this zoom, the pan is
 * centered on it (still reachable from either side).
 */
export function clampPan(pan: Pan, zoom: number, bounds: ContentBounds | null, viewport: ViewportSize, opts: ClampPanOptions = {}): Pan {
  if (!bounds) return pan;
  const margin = opts.margin ?? 80;
  const clampAxis = (p: number, min: number, max: number, size: number): number => {
    // Content's screen span is [p + min*z, p + max*z]. Keep its right
    // edge >= margin from the viewport's left edge, and its left edge
    // <= size - margin from the viewport's left edge.
    const lo = margin - max * zoom;
    const hi = size - margin - min * zoom;
    if (lo > hi) return (size - (min + max) * zoom) / 2;
    return Math.min(hi, Math.max(lo, p));
  };
  return {
    x: clampAxis(pan.x, bounds.minX, bounds.maxX, viewport.w),
    y: clampAxis(pan.y, bounds.minY, bounds.maxY, viewport.h),
  };
}
