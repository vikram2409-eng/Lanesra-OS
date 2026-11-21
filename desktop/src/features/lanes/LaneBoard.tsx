/**
 * The lane board, "my steps", lane timing and ageing views (WP-0713,
 * REQ-LANE-019).
 *
 * The board shows each active run of a lane in its stage, only records
 * the viewer may read; "my steps" lists the signed-in person's lane
 * steps and approvals together, oldest first; timing reports time per
 * stage, completion rate and rework from the recorded timestamps; the
 * ageing view lists active runs longest in their current stage first.
 *
 * Pure views take data props (tested directly, like LaneBlueprintView);
 * the `*Section` components below fetch through the Tauri commands.
 * Styling follows the lane pages: `card` sections, plain tables,
 * `var(--text-muted)` for secondary text, no new CSS.
 */
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type {
  LaneBoard,
  LaneBoardItem,
  LaneTiming,
  MySteps,
} from "../../lib/types";

/** Seconds as a short human duration: 12s, 3m 12s, 2h 5m, 3d 2h. */
export function formatDuration(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined || Number.isNaN(seconds)) return "—";
  const total = Math.max(0, Math.floor(seconds));
  if (total < 60) return `${total}s`;
  const minutes = Math.floor(total / 60);
  if (minutes < 60) return `${minutes}m ${total % 60}s`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ${minutes % 60}m`;
  const days = Math.floor(hours / 24);
  return `${days}d ${hours % 24}h`;
}

function formatTimestamp(iso: string | null | undefined): string {
  if (!iso) return "—";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString();
}

function RunLink({
  item,
  onOpenRun,
}: {
  item: LaneBoardItem;
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  if (!onOpenRun) return <strong>{item.anchor_label}</strong>;
  return (
    <button
      className="btn"
      style={{ padding: "2px 8px" }}
      onClick={() => onOpenRun(item.run_id, item.anchor_label)}
    >
      {item.anchor_label}
    </button>
  );
}

/// The board for one lane: its stages in definition order, each with the
/// active runs currently in it. Terminal runs never appear here.
export function LaneBoardView({
  board,
  onOpenRun,
}: {
  board: LaneBoard;
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  return (
    <section className="card" aria-labelledby="lane-board-heading">
      <h3 id="lane-board-heading" style={{ marginTop: 0 }}>
        Board — {board.lane_name}
      </h3>
      {board.stages.length === 0 && <p>No stages yet.</p>}
      {board.stages.map((stage) => (
        <div key={stage.stage_id} style={{ marginBottom: 16 }}>
          <h4 style={{ marginBottom: 6 }}>
            {stage.stage_name} ({stage.runs.length})
          </h4>
          {stage.runs.length === 0 ? (
            <p style={{ color: "var(--text-muted)" }}>No active runs in this stage.</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Record</th>
                  <th>Current step</th>
                  <th>In stage</th>
                </tr>
              </thead>
              <tbody>
                {stage.runs.map((run) => (
                  <tr key={run.run_id}>
                    <td>
                      <RunLink item={run} onOpenRun={onOpenRun} />
                    </td>
                    <td>{run.current_step_name ?? run.status}</td>
                    <td>{formatDuration(run.seconds_in_stage)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      ))}
    </section>
  );
}

/// Active runs longest in their current stage first. The items arrive
/// sorted; the view renders them in order.
export function LaneAgeingView({
  items,
  onOpenRun,
}: {
  items: LaneBoardItem[];
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  return (
    <section className="card" aria-labelledby="lane-ageing-heading">
      <h3 id="lane-ageing-heading" style={{ marginTop: 0 }}>
        Ageing — longest in stage first
      </h3>
      {items.length === 0 ? (
        <p style={{ color: "var(--text-muted)" }}>No active runs to age.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Record</th>
              <th>Stage</th>
              <th>Current step</th>
              <th>In stage</th>
            </tr>
          </thead>
          <tbody>
            {items.map((run) => (
              <tr key={run.run_id}>
                <td>
                  <RunLink item={run} onOpenRun={onOpenRun} />
                </td>
                <td>{run.current_stage_name ?? "—"}</td>
                <td>{run.current_step_name ?? run.status}</td>
                <td>{formatDuration(run.seconds_in_stage)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/// The signed-in person's lane steps and approvals together, oldest
/// first.
export function MyLaneStepsView({
  steps,
  onOpenRun,
}: {
  steps: MySteps;
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  return (
    <section className="card" aria-labelledby="my-lane-steps-heading">
      <h3 id="my-lane-steps-heading" style={{ marginTop: 0 }}>
        My steps
      </h3>
      {steps.items.length === 0 ? (
        <p style={{ color: "var(--text-muted)" }}>Nothing waits on you.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Step</th>
              <th>Lane</th>
              <th>Record</th>
              <th>Waiting since</th>
            </tr>
          </thead>
          <tbody>
            {steps.items.map((item) => (
              <tr key={`${item.kind}-${item.run_id}-${item.step_id}`}>
                <td>
                  <strong>{item.step_name}</strong>{" "}
                  <span style={{ color: "var(--text-muted)" }}>
                    ({item.kind === "approval" ? "Approval" : "Step"})
                  </span>
                </td>
                <td>{item.lane_name}</td>
                <td>
                  {onOpenRun ? (
                    <button
                      className="btn"
                      style={{ padding: "2px 8px" }}
                      onClick={() => onOpenRun(item.run_id, item.anchor_label)}
                    >
                      {item.anchor_label}
                    </button>
                  ) : (
                    <strong>{item.anchor_label}</strong>
                  )}
                </td>
                <td>{formatTimestamp(item.waiting_since)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/// A lane's timing figures: completion rate and rework up top, time per
/// stage below. Averages come from runs that left the stage; a stage no
/// run has left yet shows "—".
export function LaneTimingView({ timing }: { timing: LaneTiming }) {
  const rate = `${Math.round(timing.completion_rate * 100)}%`;
  return (
    <section className="card" aria-labelledby="lane-timing-heading">
      <h3 id="lane-timing-heading" style={{ marginTop: 0 }}>
        Timing — {timing.lane_name}
      </h3>
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 12 }}>
        <div>
          <div style={{ fontSize: 24, fontWeight: 700 }}>{timing.total_runs}</div>
          <div style={{ color: "var(--text-muted)" }}>Total runs</div>
        </div>
        <div>
          <div style={{ fontSize: 24, fontWeight: 700 }}>{timing.completed_runs}</div>
          <div style={{ color: "var(--text-muted)" }}>Completed</div>
        </div>
        <div>
          <div style={{ fontSize: 24, fontWeight: 700 }}>{rate}</div>
          <div style={{ color: "var(--text-muted)" }}>Completion rate</div>
        </div>
        <div>
          <div style={{ fontSize: 24, fontWeight: 700 }}>{timing.rework_steps}</div>
          <div style={{ color: "var(--text-muted)" }}>Rework steps</div>
        </div>
      </div>
      {timing.stages.length === 0 ? (
        <p style={{ color: "var(--text-muted)" }}>No stage figures yet.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Stage</th>
              <th>Entered</th>
              <th>Completed</th>
              <th>Average</th>
              <th>Median</th>
            </tr>
          </thead>
          <tbody>
            {timing.stages.map((stage) => (
              <tr key={stage.stage_id}>
                <td>
                  <strong>{stage.stage_name}</strong>
                </td>
                <td>{stage.runs_entered}</td>
                <td>{stage.runs_completed}</td>
                <td>{formatDuration(stage.avg_seconds)}</td>
                <td>{formatDuration(stage.median_seconds)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

/// Board + ageing + timing for one lane, fetched through the Tauri
/// commands. Renders nothing while the lane key is null.
export function LaneBoardSection({
  laneKey,
  onOpenRun,
}: {
  laneKey: string | null;
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  const board = useQuery({
    queryKey: ["laneBoard", laneKey],
    queryFn: () => api.listLaneBoard(laneKey as string),
    enabled: laneKey !== null,
  });
  const timing = useQuery({
    queryKey: ["laneTiming", laneKey],
    queryFn: () => api.getLaneTiming(laneKey as string),
    enabled: laneKey !== null,
  });
  const ageing = useQuery({
    queryKey: ["laneAgeing", laneKey],
    queryFn: () => api.listLaneAgeing(laneKey as string),
    enabled: laneKey !== null,
  });
  if (laneKey === null) return null;
  return (
    <div style={{ display: "grid", gap: 16 }}>
      {board.isLoading && <p>Loading board…</p>}
      {board.error && <p>Could not load the board: {(board.error as Error).message}</p>}
      {board.data && <LaneBoardView board={board.data} onOpenRun={onOpenRun} />}
      {ageing.isLoading && <p>Loading ageing…</p>}
      {ageing.error && <p>Could not load ageing: {(ageing.error as Error).message}</p>}
      {ageing.data && <LaneAgeingView items={ageing.data} onOpenRun={onOpenRun} />}
      {timing.isLoading && <p>Loading timing…</p>}
      {timing.error && <p>Could not load timing: {(timing.error as Error).message}</p>}
      {timing.data && <LaneTimingView timing={timing.data} />}
    </div>
  );
}

/// "My steps" for the signed-in person, fetched through the Tauri
/// command. Shown on the Process Lanes page, above the lane list.
export function MyLaneStepsSection({
  onOpenRun,
}: {
  onOpenRun?: (runId: string, recordLabel: string) => void;
}) {
  const mine = useQuery({
    queryKey: ["myLaneSteps"],
    queryFn: () => api.listMyLaneSteps(),
  });
  return (
    <>
      {mine.isLoading && <p>Loading my steps…</p>}
      {mine.error && <p>Could not load my steps: {(mine.error as Error).message}</p>}
      {mine.data && <MyLaneStepsView steps={mine.data} onOpenRun={onOpenRun} />}
    </>
  );
}
