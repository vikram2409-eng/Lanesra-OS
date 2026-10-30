// WP-0112: live run events (ADR-0008 §5). The desktop app receives them on
// the Tauri `run-event` channel; a browser on Team Workspace follows
// `/api/v1/runs/:id/events` with its session cookie. Both carry the same
// envelope, so callers do not care which edition they run in.
import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

import type { RunEvent } from "./types";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function isTerminal(event: RunEvent): boolean {
  return event.type === "run.completed" || event.type === "run.failed";
}

/** Calls `onEvent` for each new event of `runId`; returns an unsubscribe. */
export function subscribeRunEvents(runId: string, onEvent: (event: RunEvent) => void): () => void {
  if (isTauriRuntime()) {
    const stop = listen<RunEvent>("run-event", (message) => {
      if (message.payload.run_id === runId) onEvent(message.payload);
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }
  if (typeof EventSource === "undefined") return () => {};
  const source = new EventSource(`/api/v1/runs/${encodeURIComponent(runId)}/events`, { withCredentials: true });
  source.onmessage = (message) => {
    const event = JSON.parse(message.data) as RunEvent;
    onEvent(event);
    // The server ends the stream after the last event; do not reconnect.
    if (isTerminal(event)) source.close();
  };
  return () => source.close();
}

/** Follows `runId` while it is set, calling the latest `onEvent`. */
export function useRunEvents(runId: string | null, onEvent: (event: RunEvent) => void): void {
  const latest = useRef(onEvent);
  latest.current = onEvent;
  useEffect(() => {
    if (!runId) return;
    return subscribeRunEvents(runId, (event) => latest.current(event));
  }, [runId]);
}

/** A one-line description of an event, for a progress line. */
export function describeRunEvent(event: RunEvent): string {
  const d = event.data;
  const text = (v: unknown) => (typeof v === "string" || typeof v === "number" ? String(v) : "");
  switch (event.type) {
    case "step.started":
      return `Step ${text(d.name)} started`;
    case "step.completed":
      return `Step ${text(d.name)} ${d.status === "ok" ? "finished" : "failed"}`;
    case "round.completed":
      return `Round ${text(d.round)} answered`;
    case "tool.called":
      return `Called ${text(d.tool)}`;
    case "tool.denied":
      return `Refused ${text(d.tool)}`;
    case "approval.requested":
      return "Waiting for approval";
    case "run.started":
      return "Run started";
    case "run.completed":
      return "Run finished";
    case "run.failed":
      return d.status === "cancelled" ? "Run cancelled" : "Run failed";
    default:
      return event.type;
  }
}
