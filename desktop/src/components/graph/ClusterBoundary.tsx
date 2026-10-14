/**
 * PR 2 — Desktop Ontology Graph: dashed-ellipse cluster boundary.
 *
 * React port of the demo's `__clusterBoundaries` ellipse renderer
 * (app.js ~4903-4911). Geometry ported verbatim: padX=58, padY=44,
 * min rx=70/ry=48, stroke-dasharray "8 6", type color at 60% opacity,
 * 4% fill. The label button sits at the top-center of the ellipse.
 */
import { memo, useMemo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphNode } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";
import type { GraphPosition } from "./useGraphLayout";

export interface ClusterBoundaryProps {
  type: SystemNodeType;
  members: GraphNode[];
  positions: Map<string, GraphPosition>;
  /** In the hover spotlight's connected set. */
  hoverOn?: boolean;
  /** Spotlight active on the canvas. */
  spotlightActive?: boolean;
  onCollapse?: (type: SystemNodeType) => void;
  onMouseEnter?: (type: SystemNodeType) => void;
  onMouseLeave?: () => void;
}

const PAD_X = 58;
const PAD_Y = 44;
const MIN_RX = 70;
const MIN_RY = 48;

export const ClusterBoundary = memo(function ClusterBoundary(props: ClusterBoundaryProps) {
  const { type, members, positions, hoverOn, spotlightActive, onCollapse, onMouseEnter, onMouseLeave } = props;

  const geom = useMemo(() => {
    const pts = members
      .map((n) => positions.get(n.key))
      .filter((p): p is GraphPosition => !!p);
    if (!pts.length) return null;
    const xs = pts.map((p) => p.x);
    const ys = pts.map((p) => p.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return {
      cx: (minX + maxX) / 2,
      cy: (minY + maxY) / 2,
      rx: Math.max(MIN_RX, (maxX - minX) / 2 + PAD_X),
      ry: Math.max(MIN_RY, (maxY - minY) / 2 + PAD_Y),
      labelX: (minX + maxX) / 2,
      labelY: minY - 14,
    };
  }, [members, positions]);

  if (!geom) return null;

  const color = SYSTEM_NODE_TYPE_COLORS[type] ?? "#888";
  const label = `${SYSTEM_NODE_TYPE_LABELS[type] ?? type} · ${members.length}`;

  // Dim non-hovered clusters while the spotlight is active (demo sets
  // inline opacity 0.25 on unrelated ellipses).
  const dimmed = spotlightActive && !hoverOn;

  return (
    <>
      <ellipse
        data-cluster-type={type}
        cx={geom.cx}
        cy={geom.cy}
        rx={geom.rx}
        ry={geom.ry}
        fill={color}
        fillOpacity={0.04}
        stroke={color}
        strokeOpacity={0.6}
        strokeWidth={1.5}
        strokeDasharray="8 6"
        style={{ opacity: dimmed ? 0.25 : undefined, transition: "opacity 0.2s ease" }}
        className={cn(hoverOn && "graph-hover-on")}
      />
      <button
        type="button"
        className="ontology-halo-label og-cluster-label"
        data-cluster-type={type}
        style={{ left: geom.labelX, top: geom.labelY, color }}
        title={`Collapse ${label} into a single node`}
        onClick={() => onCollapse?.(type)}
        onMouseEnter={() => onMouseEnter?.(type)}
        onMouseLeave={() => onMouseLeave?.()}
      >
        {label} ▾
      </button>
    </>
  );
});
