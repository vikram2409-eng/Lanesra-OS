/**
 * "Start lane run" (WP-0707, REQ-LANE-014 / REQ-LANE-010).
 *
 * The entry point from a record page-header and from a list row action.
 * Lists the published lanes for the record's object; with one lane it
 * starts directly, with several it lets the person pick.
 *
 * One active run per lane per record (REQ-LANE-010): when an active run
 * already exists for the lane, the button opens that run instead of
 * starting a second - including when the start itself is refused as a
 * duplicate, in which case the existing run is looked up and opened.
 */
import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { LaneRunListItem } from "../../lib/types";

const FINISHED_STATUSES = new Set(["completed", "completed_partial", "failed", "cancelled"]);

function isActiveRun(run: LaneRunListItem): boolean {
  return run.finished_at === null && !FINISHED_STATUSES.has(run.status);
}

export function StartLaneRunButton({
  objectKey,
  recordId,
  recordLabel,
  onOpenRun,
}: {
  objectKey: string;
  recordId: string;
  recordLabel: string;
  onOpenRun: (runId: string, recordLabel: string) => void;
}) {
  const queryClient = useQueryClient();
  const [picking, setPicking] = useState(false);
  const [selectedLane, setSelectedLane] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const blueprints = useQuery({ queryKey: ["laneBlueprints"], queryFn: () => api.listLaneBlueprints() });
  const runs = useQuery({
    queryKey: ["laneRunsForRecord", objectKey, recordId],
    queryFn: () => api.listLaneRunsForRecord(objectKey, recordId),
  });

  const lanes = (blueprints.data ?? []).filter(
    (b) => b.object_key === objectKey && b.latest_status === "published",
  );
  const activeRuns = (runs.data ?? []).filter(isActiveRun);
  const activeFor = (laneKey: string, laneName: string) =>
    activeRuns.find((r) => r.lane_id === laneKey || r.lane_name === laneName) ?? null;

  async function start(laneKey: string, laneName: string) {
    const existing = activeFor(laneKey, laneName);
    if (existing) {
      onOpenRun(existing.run_id, recordLabel);
      return;
    }
    setStarting(true);
    setError(null);
    try {
      const run = await api.startLaneRun(laneKey, recordId);
      setPicking(false);
      onOpenRun(run.id, recordLabel);
    } catch {
      // A refusal is the one-active-run guard firing (REQ-LANE-010):
      // look the existing run up and open it instead of failing. Read
      // the list straight from the api - the query cache still holds the
      // pre-start (empty) answer, so a fetchQuery would return that.
      const refetched = await api.listLaneRunsForRecord(objectKey, recordId);
      await queryClient.invalidateQueries({ queryKey: ["laneRunsForRecord", objectKey, recordId] });
      const found = refetched.filter(isActiveRun).find((r) => r.lane_id === laneKey || r.lane_name === laneName);
      if (found) {
        setPicking(false);
        onOpenRun(found.run_id, recordLabel);
      } else {
        setError("Could not start this lane run.");
      }
    } finally {
      setStarting(false);
    }
  }

  const loading = blueprints.isLoading || runs.isLoading;
  const single = lanes.length === 1 ? lanes[0] : null;
  const singleActive = single ? activeFor(single.lane_key, single.name) : null;

  if (picking) {
    const selected = selectedLane ?? lanes[0]?.lane_key ?? null;
    const selectedLaneRow = lanes.find((l) => l.lane_key === selected) ?? null;
    const selectedActive = selectedLaneRow ? activeFor(selectedLaneRow.lane_key, selectedLaneRow.name) : null;
    return (
      <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <label htmlFor="lane-run-picker" style={{ fontSize: 13 }}>
          Lane
        </label>
        <select
          id="lane-run-picker"
          value={selected ?? ""}
          onChange={(e) => setSelectedLane(e.target.value)}
          disabled={starting}
        >
          {lanes.map((l) => (
            <option key={l.lane_key} value={l.lane_key}>
              {l.name}
            </option>
          ))}
        </select>
        <button
          className="btn btn-secondary"
          disabled={starting || !selectedLaneRow}
          onClick={() => selectedLaneRow && void start(selectedLaneRow.lane_key, selectedLaneRow.name)}
        >
          {selectedActive ? `Open ${selectedLaneRow?.name ?? ""}` : `Start ${selectedLaneRow?.name ?? ""}`}
        </button>
        <button className="btn btn-secondary" onClick={() => setPicking(false)} disabled={starting}>
          Cancel
        </button>
        {error && <span role="alert">{error}</span>}
      </div>
    );
  }

  return (
    <div style={{ display: "inline-flex", gap: 8, alignItems: "center" }}>
      <button
        className="btn btn-secondary"
        disabled={loading || starting || lanes.length === 0}
        title={lanes.length === 0 && !loading ? "No published lane runs on this record" : undefined}
        onClick={() => {
          if (single) void start(single.lane_key, single.name);
          else setPicking(true);
        }}
      >
        {singleActive ? "Open lane run" : "Start lane run"}
      </button>
      {error && <span role="alert">{error}</span>}
    </div>
  );
}
