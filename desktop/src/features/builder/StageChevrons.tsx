/**
 * Stage chevrons for the lane builder (WP-0718 §1, REQ-LANE-024).
 *
 * The builder's OWN chevron component: the Path component it would
 * reuse (WP-0621) is STILL proposed, so — like the runner, which built
 * its own stage bar for the same reason — the builder draws its stages
 * as chevrons here instead of importing a component that does not
 * exist. A stage is one chevron; stages reorder by drag and by the
 * keyboard move buttons beside each one.
 */
import type { LaneStage } from "../../lib/types";

export function StageChevrons({
  stages,
  selectedStageId,
  onSelectStage,
  onMoveStage,
  onAddStage,
  onDeleteStage,
}: {
  stages: LaneStage[];
  selectedStageId: string | null;
  onSelectStage: (stageId: string) => void;
  onMoveStage: (stageId: string, direction: -1 | 1) => void;
  onAddStage: () => void;
  onDeleteStage?: (stageId: string) => void;
}) {
  return (
    <div>
      <ol
        aria-label="Stage order"
        style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center", listStyle: "none", padding: 0, margin: "0 0 8px" }}
      >
        {stages.map((stage, i) => (
          <li key={stage.id} style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <button
              type="button"
              className="btn"
              aria-label={`Stage ${i + 1}: ${stage.name}`}
              aria-pressed={selectedStageId === stage.id}
              onClick={() => onSelectStage(stage.id)}
              style={{
                clipPath: "polygon(0 0, calc(100% - 12px) 0, 100% 50%, calc(100% - 12px) 100%, 0 100%, 12px 50%)",
                paddingLeft: 20,
                paddingRight: 20,
                fontWeight: selectedStageId === stage.id ? 700 : undefined,
              }}
            >
              {i + 1}. {stage.name}
            </button>
            <button
              type="button"
              className="btn"
              aria-label={`Move stage ${stage.name} earlier`}
              disabled={i === 0}
              onClick={() => onMoveStage(stage.id, -1)}
            >
              ‹
            </button>
            <button
              type="button"
              className="btn"
              aria-label={`Move stage ${stage.name} later`}
              disabled={i === stages.length - 1}
              onClick={() => onMoveStage(stage.id, 1)}
            >
              ›
            </button>
            {onDeleteStage && (
              <button
                type="button"
                className="btn"
                aria-label={`Delete stage ${stage.name}`}
                onClick={() => onDeleteStage(stage.id)}
              >
                Delete stage
              </button>
            )}
          </li>
        ))}
      </ol>
      <button type="button" className="btn" onClick={onAddStage}>
        Add stage
      </button>
    </div>
  );
}
