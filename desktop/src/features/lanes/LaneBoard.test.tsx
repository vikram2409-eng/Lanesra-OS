/**
 * The lane board, "my steps", timing and ageing views (WP-0713,
 * REQ-LANE-019).
 * Covers: REQ-LANE-019 (WP-0713 AC-1, AC-2, AC-3, AC-4).
 *
 * The views render from fixture data props - no API, no query client -
 * so these tests prove the rendering contract: stages in order with
 * their runs, steps and approvals together oldest first, timing
 * figures as recorded, ageing longest in stage first.
 */
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
  LaneAgeingView,
  LaneBoardView,
  LaneTimingView,
  MyLaneStepsView,
  formatDuration,
} from "./LaneBoard";
import type {
  LaneBoard,
  LaneBoardItem,
  LaneTiming,
  MySteps,
} from "../../lib/types";

function boardItem(overrides: Partial<LaneBoardItem>): LaneBoardItem {
  return {
    run_id: "run-1",
    lane_id: "lane-metrics",
    lane_name: "Count lines",
    anchor_object_key: "batch",
    anchor_record_id: "BAT-1",
    anchor_label: "BAT-1",
    status: "waiting_person",
    current_stage_id: "stage-tally",
    current_stage_name: "Tally",
    current_step_id: "tally",
    current_step_name: "Count lines",
    started_at: "2026-11-19T10:00:00Z",
    stage_entered_at: "2026-11-19T10:00:00Z",
    seconds_in_stage: 3723,
    ...overrides,
  };
}

function board(): LaneBoard {
  return {
    lane_id: "lane-metrics",
    lane_name: "Count lines",
    stages: [
      {
        stage_id: "stage-tally",
        stage_name: "Tally",
        runs: [boardItem({})],
      },
      {
        stage_id: "stage-review",
        stage_name: "Review",
        runs: [
          boardItem({
            run_id: "run-3",
            anchor_record_id: "BAT-3",
            anchor_label: "BAT-3",
            status: "waiting_approval",
            current_stage_id: "stage-review",
            current_stage_name: "Review",
            current_step_id: "review",
            current_step_name: "Manager review",
            seconds_in_stage: 61,
          }),
        ],
      },
    ],
  };
}

describe("formatDuration", () => {
  it("renders seconds, minutes, hours and days", () => {
    expect(formatDuration(12)).toBe("12s");
    expect(formatDuration(61)).toBe("1m 1s");
    expect(formatDuration(3723)).toBe("1h 2m");
    expect(formatDuration(90061)).toBe("1d 1h");
    expect(formatDuration(null)).toBe("—");
  });
});

describe("LaneBoardView", () => {
  it("shows each stage with its active runs and time in stage", () => {
    const { container } = render(<LaneBoardView board={board()} />);
    expect(container.textContent).toContain("Board — Count lines");
    for (const name of ["Tally (1)", "Review (1)"]) {
      expect(container.textContent).toContain(name);
    }
    for (const label of ["BAT-1", "BAT-3"]) {
      expect(container.textContent).toContain(label);
    }
    expect(container.textContent).toContain("Count lines");
    expect(container.textContent).toContain("Manager review");
    // Time in stage, formatted.
    expect(container.textContent).toContain("1h 2m");
    expect(container.textContent).toContain("1m 1s");
  });

  it("says so when a stage has no active runs", () => {
    const empty: LaneBoard = {
      lane_id: "lane-metrics",
      lane_name: "Count lines",
      stages: [{ stage_id: "stage-tally", stage_name: "Tally", runs: [] }],
    };
    const { container } = render(<LaneBoardView board={empty} />);
    expect(container.textContent).toContain("No active runs in this stage.");
  });
});

describe("LaneAgeingView", () => {
  it("lists active runs longest in stage first", () => {
    const items = [
      boardItem({ run_id: "run-old", anchor_label: "BAT-1", seconds_in_stage: 7200 }),
      boardItem({ run_id: "run-new", anchor_label: "BAT-3", seconds_in_stage: 60 }),
    ];
    const { container } = render(<LaneAgeingView items={items} />);
    const text = container.textContent ?? "";
    expect(text.indexOf("BAT-1")).toBeLessThan(text.indexOf("BAT-3"));
    expect(text).toContain("2h 0m");
  });
});

describe("MyLaneStepsView", () => {
  it("lists steps and approvals together, oldest first", () => {
    const steps: MySteps = {
      items: [
        {
          kind: "step",
          run_id: "run-1",
          lane_id: "lane-metrics",
          lane_name: "Count lines",
          anchor_object_key: "batch",
          anchor_record_id: "BAT-1",
          anchor_label: "BAT-1",
          stage_id: "stage-tally",
          stage_name: "Tally",
          step_id: "tally",
          step_name: "Count lines",
          waiting_since: "2026-11-19T10:00:00Z",
          approval_id: null,
        },
        {
          kind: "approval",
          run_id: "run-3",
          lane_id: "lane-metrics",
          lane_name: "Count lines",
          anchor_object_key: "batch",
          anchor_record_id: "BAT-3",
          anchor_label: "BAT-3",
          stage_id: "stage-review",
          stage_name: "Review",
          step_id: "review",
          step_name: "Manager review",
          waiting_since: "2026-11-19T11:00:00Z",
          approval_id: "appr-1",
        },
      ],
    };
    const { container } = render(<MyLaneStepsView steps={steps} />);
    const text = container.textContent ?? "";
    expect(text).toContain("My steps");
    expect(text).toContain("Count lines");
    expect(text).toContain("Manager review");
    expect(text).toContain("Approval");
    // Oldest first: the tally step precedes the review approval.
    expect(text.indexOf("Count lines")).toBeLessThan(text.indexOf("Manager review"));
  });

  it("says so when nothing waits on the viewer", () => {
    const { container } = render(<MyLaneStepsView steps={{ items: [] }} />);
    expect(container.textContent).toContain("Nothing waits on you.");
  });
});

describe("LaneTimingView", () => {
  it("shows the completion rate, rework and time per stage", () => {
    const timing: LaneTiming = {
      lane_id: "lane-metrics",
      lane_name: "Count lines",
      total_runs: 3,
      completed_runs: 1,
      completion_rate: 1 / 3,
      rework_steps: 0,
      stages: [
        {
          stage_id: "stage-tally",
          stage_name: "Tally",
          runs_entered: 3,
          runs_completed: 2,
          avg_seconds: 120,
          median_seconds: 90,
        },
        {
          stage_id: "stage-review",
          stage_name: "Review",
          runs_entered: 2,
          runs_completed: 1,
          avg_seconds: 300,
          median_seconds: 300,
        },
      ],
    };
    const { container } = render(<LaneTimingView timing={timing} />);
    const text = container.textContent ?? "";
    expect(text).toContain("Timing — Count lines");
    expect(text).toContain("33%");
    expect(text).toContain("Tally");
    expect(text).toContain("Review");
    expect(text).toContain("2m 0s");
    expect(text).toContain("5m 0s");
  });
});
