/**
 * Lane builder page (WP-0718, REQ-LANE-024): the entry point the App
 * renders. Loads a saved lane by key through getLaneBlueprint, starts
 * from a blank definition for "New lane", or renders the definition
 * it is handed (tests, and the blueprint page's "Open in builder").
 *
 * Gap 5: the page keeps the lane key in its own state — when a new
 * lane is saved the builder adopts the returned key (LaneBuilder's
 * onLaneKeyChange) and the page holds it, so the saved lane stays
 * addressable after the save without closing and reopening it. A lane
 * created in this session keeps the definition it was built with even
 * after that save: reloading it from the server would remount the
 * builder and wipe the save confirmation, the dry-run report and the
 * fixture inputs the author just filled in. Only a lane opened by key
 * is loaded from the server.
 */
import { useState } from "react";
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
  const [pageLaneKey, setPageLaneKey] = useState<string | null>(laneKey ?? null);
  // Created once per mount: the definition a brand-new lane is built
  // with. Stable across renders — and across the first save adopting a
  // key — so the builder below never remounts for it.
  const [newLaneDefinition] = useState<LaneDefinition | null>(() =>
    laneKey == null && !initialDefinition ? blankLaneDefinition() : null,
  );
  const detail = useQuery({
    queryKey: ["laneBlueprint", pageLaneKey],
    queryFn: () => api.getLaneBlueprint(pageLaneKey as string),
    enabled:
      !initialDefinition && !newLaneDefinition && pageLaneKey !== null && pageLaneKey !== undefined,
  });

  const definition = initialDefinition ?? newLaneDefinition ?? detail.data?.definition;
  if (!definition) {
    if (detail.isLoading) return <p>Loading lane…</p>;
    if (detail.error) return <p>Could not load the lane: {(detail.error as Error).message}</p>;
    return <p>No lane selected.</p>;
  }
  return (
    <div>
      {newLaneDefinition && pageLaneKey === null && <h2>New lane</h2>}
      <LaneBuilder
        definition={definition}
        laneKey={pageLaneKey}
        onLaneKeyChange={setPageLaneKey}
        onClose={onClose}
      />
    </div>
  );
}
