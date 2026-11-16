/**
 * Lane builder page (WP-0718, REQ-LANE-024): the entry point the App
 * renders. Loads a saved lane by key through getLaneBlueprint, starts
 * from a blank definition for "New lane", or renders the definition
 * it is handed (tests, and the blueprint page's "Open in builder").
 */
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { LaneBuilder } from "./LaneBuilder";
import { blankLaneDefinition } from "./builderUtils";
import type { LaneDefinition } from "../../lib/types";

export function LaneBuilderPage({
  laneKey,
  initialDefinition,
  onClose,
}: {
  laneKey?: string | null;
  initialDefinition?: LaneDefinition;
  onClose?: () => void;
}) {
  const detail = useQuery({
    queryKey: ["laneBlueprint", laneKey],
    queryFn: () => api.getLaneBlueprint(laneKey as string),
    enabled: !initialDefinition && laneKey !== null && laneKey !== undefined,
  });

  if (initialDefinition) {
    return <LaneBuilder definition={initialDefinition} laneKey={laneKey ?? null} onClose={onClose} />;
  }
  if (laneKey === null || laneKey === undefined) {
    return (
      <div>
        <h2>New lane</h2>
        <LaneBuilder definition={blankLaneDefinition()} laneKey={null} onClose={onClose} />
      </div>
    );
  }
  if (detail.isLoading) return <p>Loading lane…</p>;
  if (detail.error) return <p>Could not load the lane: {(detail.error as Error).message}</p>;
  if (!detail.data) return <p>No lane selected.</p>;
  return <LaneBuilder definition={detail.data.definition} laneKey={laneKey} onClose={onClose} />;
}
