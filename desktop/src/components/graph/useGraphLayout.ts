/**
 * PR 2 — Desktop Ontology Graph: force-directed layout with per-type
 * cluster anchors.
 *
 * Line-for-line port of the demo's `graphLayoutPositions`
 * (app.js:4791). Deterministic: positions derive from a seeded RNG over
 * the sorted node keys, so the same model always lays out identically.
 */

import type { GraphModel } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";

/** Port of demo's `depHashStr` (FNV-1a, app.js:4656). */
function hashStr(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Port of demo's `depSeededRandom` (mulberry32-ish, app.js:4657). */
function seededRandom(seed: number): () => number {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export interface GraphLayoutOptions {
  /** Key of the node pinned to canvas center. */
  rootKey: string;
  /** Per-type cluster anchor order (GRAPH_CLUSTER_ORDER). */
  clusterOrder: SystemNodeType[];
  W: number;
  H: number;
}

export interface GraphPosition {
  x: number;
  y: number;
}

/**
 * Computes node positions. Port of `graphLayoutPositions` — keep the
 * constants in sync with the demo (K=82, LINK_LEN=185).
 */
export function layoutGraphPositions(
  model: GraphModel,
  layout: GraphLayoutOptions,
): Map<string, GraphPosition> {
  const { W, H } = layout;
  const visible = model.nodes;
  const rng = seededRandom(hashStr(visible.map((n) => n.key).sort().join("|")));

  const anchors: Record<string, GraphPosition> = {};
  (layout.clusterOrder || []).forEach((t, i) => {
    const a = (i / layout.clusterOrder.length) * Math.PI * 2 - Math.PI / 2;
    anchors[t] = { x: W / 2 + Math.cos(a) * W * 0.36, y: H / 2 + Math.sin(a) * H * 0.36 };
  });

  const pos = new Map<string, GraphPosition>();
  const rootKey = layout.rootKey;
  visible.forEach((n) => {
    if (n.key === rootKey) {
      pos.set(n.key, { x: W / 2, y: H / 2 });
      return;
    }
    const a = anchors[n.type] || { x: W / 2, y: H / 2 };
    pos.set(n.key, { x: a.x + (rng() - 0.5) * 90, y: a.y + (rng() - 0.5) * 90 });
  });

  const TICKS = visible.length > 260 ? 60 : 110;
  const K = 82;
  const LINK_LEN = 185;

  for (let t = 0; t < TICKS; t++) {
    const disp = new Map(visible.map((n) => [n.key, { x: 0, y: 0 }] as [string, GraphPosition]));

    // Repulsion between all pairs.
    for (let i = 0; i < visible.length; i++) {
      for (let j = i + 1; j < visible.length; j++) {
        const a = visible[i];
        const b = visible[j];
        const pa = pos.get(a.key)!;
        const pb = pos.get(b.key)!;
        let dx = pa.x - pb.x;
        let dy = pa.y - pb.y;
        let d2 = dx * dx + dy * dy;
        if (d2 < 1) {
          dx = rng() - 0.5;
          dy = rng() - 0.5;
          d2 = 1;
        }
        const d = Math.sqrt(d2);
        const f = Math.min((K * K) / d, 9);
        const fx = (f * dx) / d;
        const fy = (f * dy) / d;
        const da = disp.get(a.key)!;
        const db = disp.get(b.key)!;
        da.x += fx;
        da.y += fy;
        db.x -= fx;
        db.y -= fy;
      }
    }

    // Spring attraction along edges.
    model.edges.forEach((e) => {
      const pa = pos.get(e.fromKey);
      const pb = pos.get(e.toKey);
      if (!pa || !pb) return;
      const dx = pb.x - pa.x;
      const dy = pb.y - pa.y;
      const d = Math.sqrt(dx * dx + dy * dy) || 1;
      const f = (d - LINK_LEN) * 0.055;
      const fx = (f * dx) / d;
      const fy = (f * dy) / d;
      const da = disp.get(e.fromKey)!;
      const db = disp.get(e.toKey)!;
      da.x += fx;
      da.y += fy;
      db.x -= fx;
      db.y -= fy;
    });

    // Pull toward per-type cluster anchors.
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const p = pos.get(n.key)!;
      const d = disp.get(n.key)!;
      const a = anchors[n.type] || { x: W / 2, y: H / 2 };
      d.x += (a.x - p.x) * 0.14 + (W / 2 - p.x) * 0.006;
      d.y += (a.y - p.y) * 0.14 + (H / 2 - p.y) * 0.006;
    });

    // PR4 intra-cluster cohesion: same-type nodes pull toward their
    // type centroid each tick so every cluster reads as one tight clump.
    const centroids = new Map<string, { x: number; y: number; k: number }>();
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const c = centroids.get(n.type) || { x: 0, y: 0, k: 0 };
      const pp = pos.get(n.key)!;
      c.x += pp.x;
      c.y += pp.y;
      c.k++;
      centroids.set(n.type, c);
    });
    centroids.forEach((c) => {
      c.x /= c.k;
      c.y /= c.k;
    });
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const c = centroids.get(n.type);
      if (!c || c.k < 2) return;
      const pp = pos.get(n.key)!;
      const dd = disp.get(n.key)!;
      dd.x += (c.x - pp.x) * 0.06;
      dd.y += (c.y - pp.y) * 0.06;
    });

    // Integrate + clamp to canvas.
    visible.forEach((n) => {
      if (n.key === rootKey) return;
      const p = pos.get(n.key)!;
      const d = disp.get(n.key)!;
      p.x = Math.max(34, Math.min(W - 34, p.x + d.x * 0.5));
      p.y = Math.max(52, Math.min(H - 30, p.y + d.y * 0.5));
    });
  }

  return pos;
}
