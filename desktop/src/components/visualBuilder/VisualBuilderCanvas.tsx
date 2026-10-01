import type { ReactNode } from "react";

// UX/UI Modernization, Phase A (issue #192): Shared Visual Builder
// Framework - the presentational shell (pan surface, SVG edge layer, zoom
// controls) extracted from AgentTeamsAdmin.tsx's GraphEditor. Deliberately
// agnostic to node-card size/shape: the caller renders each node via
// `renderNode`, and pre-computes each edge's screen-space endpoints
// (`fromX/fromY/toX/toY`) itself, since different builders size their
// cards differently (Execution Graph cards are ~220x60; a future Business
// Rule Board card doesn't have to be).
export interface CanvasEdgeView {
  key: string;
  fromX: number;
  fromY: number;
  toX: number;
  toY: number;
  label?: string | null;
}

export function VisualBuilderCanvas<TNode extends { x: number; y: number }>({
  nodes,
  edges,
  zoom,
  canvasWidth,
  canvasHeight,
  renderNode,
  onEdgeDelete,
  editable,
}: {
  nodes: TNode[];
  edges: CanvasEdgeView[];
  zoom: number;
  canvasWidth: number;
  canvasHeight: number;
  renderNode: (node: TNode) => ReactNode;
  onEdgeDelete?: (key: string) => void;
  editable: boolean;
}) {
  // Deliberately does NOT own the scrollable ".graph-canvas-wrap" shell -
  // the caller wraps this (alongside <VisualBuilderZoomControls>) in that
  // div itself, since .graph-zoom-controls's `position: sticky` CSS
  // depends on being a sibling inside that exact scrolling container, not
  // nested one level deeper.
  return (
    <div className="graph-canvas" style={{ width: canvasWidth, height: canvasHeight, transform: `scale(${zoom})` }}>
      <svg className="graph-edges-svg" width={canvasWidth} height={canvasHeight}>
        <defs>
          <marker id="graph-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" fill="var(--text-muted)" />
          </marker>
        </defs>
        {edges.map((e) => {
          const midX = (e.fromX + e.toX) / 2;
          const midY = (e.fromY + e.toY) / 2;
          return (
            <g key={e.key}>
              <line x1={e.fromX} y1={e.fromY} x2={e.toX} y2={e.toY} stroke="var(--text-muted)" strokeWidth={1.5} markerEnd="url(#graph-arrow)" />
              {e.label && (
                <text x={midX} y={midY - 4} fontSize={10} textAnchor="middle" fill="var(--text-muted)">
                  {e.label}
                </text>
              )}
              {editable && onEdgeDelete && (
                <g transform={`translate(${midX},${midY})`} style={{ cursor: "pointer" }} onClick={() => onEdgeDelete(e.key)}>
                  <circle r={7} fill="var(--bg-elevated)" stroke="var(--border)" />
                  <text textAnchor="middle" dy={3} fontSize={9} fill="var(--text-muted)">
                    ×
                  </text>
                </g>
              )}
            </g>
          );
        })}
      </svg>
      {nodes.map((n) => renderNode(n))}
    </div>
  );
}

export function VisualBuilderZoomControls({ onZoomIn, onZoomOut }: { onZoomIn: () => void; onZoomOut: () => void }) {
  return (
    <div className="graph-zoom-controls">
      <button className="btn" type="button" title="Zoom in" onClick={onZoomIn}>
        +
      </button>
      <button className="btn" type="button" title="Zoom out" onClick={onZoomOut}>
        −
      </button>
    </div>
  );
}
