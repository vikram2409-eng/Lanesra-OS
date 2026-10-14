/**
 * PR 2 — Desktop Ontology Graph: type legend with counts.
 *
 * React port of the demo's legend (app.js:4641). Chips double as
 * toggle-filters (click dims the type) and as the spotlight mirror:
 * `highlightedTypes` (from `useHoverDimming`) adds `legend-highlight`
 * / `legend-dim` classes, porting the demo's `__syncLegend`.
 */
import { memo } from "react";

import { cn } from "../ui/cn";
import { SYSTEM_NODE_TYPE_COLORS, SYSTEM_NODE_TYPE_LABELS, type GraphModel } from "./graphTypes";
import { GRAPH_CLUSTER_ORDER } from "./graphTypes";
import type { SystemNodeType } from "../../lib/types";

export interface GraphLegendProps {
  model: GraphModel;
  hiddenTypes: Set<SystemNodeType>;
  onToggleType: (t: SystemNodeType) => void;
  /** Types in the hover spotlight's neighborhood (legend sync). */
  highlightedTypes?: Set<SystemNodeType>;
  /** Spotlight active (any hover). */
  spotlightActive?: boolean;
}

export const GraphLegend = memo(function GraphLegend(props: GraphLegendProps) {
  const { model, hiddenTypes, onToggleType, highlightedTypes, spotlightActive } = props;

  // Count visible nodes per type.
  const counts = new Map<SystemNodeType, number>();
  model.nodes.forEach((n) => {
    if (n.id === "__cluster__") return;
    counts.set(n.type, (counts.get(n.type) ?? 0) + 1);
  });

  // Only show types present in the model (plus keep cluster order).
  const types = GRAPH_CLUSTER_ORDER.filter((t) => counts.has(t));

  if (!types.length) return null;

  return (
    <div className="ontology-legend" role="group" aria-label="Node type legend">
      {types.map((t) => {
        const hidden = hiddenTypes.has(t);
        const highlighted = spotlightActive && highlightedTypes?.has(t);
        const dimmed = spotlightActive && highlightedTypes && !highlightedTypes.has(t);
        return (
          <button
            key={t}
            type="button"
            className={cn(
              "ontology-legend-item",
              hidden && "ontology-legend-item-off",
              highlighted && "legend-highlight",
              dimmed && "legend-dim",
            )}
            data-legend-toggle={t}
            onClick={() => onToggleType(t)}
            title={`${hidden ? "Show" : "Hide"} ${SYSTEM_NODE_TYPE_LABELS[t]}${counts.get(t) ? ` (${counts.get(t)} shown)` : ""}`}
          >
            <span
              style={{
                display: "inline-block",
                flexShrink: 0,
                width: 10,
                height: 10,
                borderRadius: "50%",
                background: SYSTEM_NODE_TYPE_COLORS[t],
              }}
            />
            {SYSTEM_NODE_TYPE_LABELS[t]}
            {counts.get(t) ? <span className="ontology-legend-count">{counts.get(t)}</span> : null}
          </button>
        );
      })}
    </div>
  );
});
