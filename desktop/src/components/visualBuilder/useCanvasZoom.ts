import { useState } from "react";

// UX/UI Modernization, Phase A (issue #192): Shared Visual Builder
// Framework - CSS-zoom-trick state, extracted verbatim from
// AgentTeamsAdmin.tsx's GraphEditor (the only canvas this codebase has
// ever shipped) so a future Workflow Studio 2.0 / Business Rule Board 2.0
// canvas can reuse the identical zoom behavior instead of re-deriving it.
export function useCanvasZoom(initial = 1, min = 0.5, max = 1.5, step = 0.1) {
  const [zoom, setZoom] = useState(initial);
  const zoomIn = () => setZoom((z) => Math.min(max, Math.round((z + step) * 10) / 10));
  const zoomOut = () => setZoom((z) => Math.max(min, Math.round((z - step) * 10) / 10));
  return { zoom, zoomIn, zoomOut };
}
