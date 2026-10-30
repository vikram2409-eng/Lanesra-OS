// WP-0112: the browser path of `subscribeRunEvents` follows one run's SSE
// stream and stops after its last event.
import { afterEach, describe, expect, it, vi } from "vitest";

import { describeRunEvent, subscribeRunEvents } from "../lib/runEvents";
import type { RunEvent } from "../lib/types";

class FakeEventSource {
  static last: FakeEventSource | null = null;
  url: string;
  withCredentials: boolean;
  closed = false;
  onmessage: ((message: { data: string }) => void) | null = null;
  constructor(url: string, init?: { withCredentials?: boolean }) {
    this.url = url;
    this.withCredentials = !!init?.withCredentials;
    FakeEventSource.last = this;
  }
  close() {
    this.closed = true;
  }
}

function event(seq: number, type: RunEvent["type"], data: Record<string, unknown> = {}): RunEvent {
  return {
    schema: "lanesra.run-event",
    v: 1,
    seq,
    type,
    at: "2026-10-30T00:00:00Z",
    workspace_id: "ws",
    run_kind: "graph",
    run_id: "run 1",
    step_id: null,
    span_id: null,
    parent_span_id: null,
    data,
  };
}

describe("subscribeRunEvents in a browser", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("follows the run's stream with the session cookie and stops after the last event", () => {
    vi.stubGlobal("EventSource", FakeEventSource);
    const seen: RunEvent[] = [];
    subscribeRunEvents("run 1", (e) => seen.push(e));
    const source = FakeEventSource.last!;
    expect(source.url).toBe("/api/v1/runs/run%201/events");
    expect(source.withCredentials).toBe(true);

    source.onmessage!({ data: JSON.stringify(event(1, "run.started")) });
    expect(source.closed).toBe(false);
    source.onmessage!({ data: JSON.stringify(event(2, "run.completed", { status: "ok" })) });
    expect(seen.map((e) => e.seq)).toEqual([1, 2]);
    expect(source.closed).toBe(true);
  });

  it("closes the stream when the caller unsubscribes", () => {
    vi.stubGlobal("EventSource", FakeEventSource);
    const stop = subscribeRunEvents("run 1", () => {});
    stop();
    expect(FakeEventSource.last!.closed).toBe(true);
  });
});

describe("describeRunEvent", () => {
  it("names what happened", () => {
    expect(describeRunEvent(event(1, "tool.called", { tool: "list_objects" }))).toBe("Called list_objects");
    expect(describeRunEvent(event(1, "run.failed", { status: "cancelled" }))).toBe("Run cancelled");
    expect(describeRunEvent(event(1, "round.completed", { round: 2 }))).toBe("Round 2 answered");
  });
});
