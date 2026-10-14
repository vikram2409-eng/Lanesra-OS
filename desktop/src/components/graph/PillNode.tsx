/**
 * PR 2 — Desktop Ontology Graph: pill node.
 *
 * React port of the demo's `og-pill` renderer (app.js ~4951-4960).
 * Renders as an absolutely-positioned pill: type dot + label (root
 * pills are solid type-color with white text). Carries `data-node-key`
 * and `data-og-type` for hover/legend wiring.
 */
import { memo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphNode } from "./graphTypes";

export interface PillNodeProps {
  node: GraphNode;
  x: number;
  y: number;
  /** Currently inspected node key (outline ring). */
  inspecting?: boolean;
  /** Dimmed by hidden-type filter. */
  dimmed?: boolean;
  /** In the hover spotlight's connected set. */
  hoverOn?: boolean;
  /** Spotlight active on the canvas. */
  spotlightActive?: boolean;
  onClick?: (node: GraphNode) => void;
  onDoubleClick?: (node: GraphNode) => void;
  onMouseEnter?: (key: string) => void;
  onMouseLeave?: () => void;
}

function truncate(label: string, max = 18): string {
  return label.length > max ? label.slice(0, max - 1) + "…" : label;
}

export const PillNode = memo(function PillNode(props: PillNodeProps) {
  const { node, x, y, inspecting, dimmed, hoverOn, spotlightActive, onClick, onDoubleClick, onMouseEnter, onMouseLeave } = props;
  const color = SYSTEM_NODE_TYPE_COLORS[node.type] ?? "#888";
  const typeLabel = SYSTEM_NODE_TYPE_LABELS[node.type] ?? node.type;

  const cls = cn(
    "og-pill",
    node.isRoot && "og-pill-root",
    inspecting && "og-pill-inspecting",
    dimmed && "og-pill-dimmed",
    hoverOn && "graph-hover-on",
    node.id === "__cluster__" && "og-pill-cluster",
  );

  const style: React.CSSProperties = {
    left: x,
    top: y,
    ...(node.isRoot ? { background: color, borderColor: color } : {}),
    // When the spotlight is active but this pill isn't connected, the
    // CSS `.graph-hover-dim .og-pill:not(.graph-hover-on)` handles the
    // fade — no inline opacity needed.
    ...(spotlightActive && !hoverOn ? {} : {}),
  };

  return (
    <div
      className={cls}
      data-node-key={node.key}
      data-og-type={node.type}
      style={style}
      title={`${typeLabel}: ${node.label}`}
      role="button"
      tabIndex={0}
      onClick={() => onClick?.(node)}
      onDoubleClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        onDoubleClick?.(node);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick?.(node);
        }
      }}
      onMouseEnter={() => onMouseEnter?.(node.key)}
      onMouseLeave={() => onMouseLeave?.()}
    >
      {!node.isRoot && <span className="og-pill-dot" style={{ background: color }} />}
      <span className="og-pill-label">
        {truncate(node.label)}
        {node.id === "__cluster__" && node.count != null ? ` · ${node.count}` : ""}
      </span>
    </div>
  );
});
