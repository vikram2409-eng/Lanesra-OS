import { useEffect, useState } from "react";

// UX/UI Modernization, Phase A (issue #192): Shared Visual Builder
// Framework. Generalizes AgentTeamsAdmin.tsx's GraphEditor own click-to-
// connect state machine and per-node-type edge rules so a future builder
// (Workflow Studio 2.0, Business Rule Board 2.0) can describe its own
// connection rules without re-deriving this interaction - every Execution
// Graph node type's rule collapses into exactly one of these 5 shapes:
//   - trigger/action/agent/delay/transform/join: "single" (exactly one
//     outgoing edge, no label)
//   - parallel_split: "multi" (any number, no label, no duplicate target)
//   - condition: "fixed_labels" ["true","false"]
//   - approval: "fixed_labels" ["approved","rejected"]
//   - loop: "fixed_labels" ["body","exit"]
//   - router: "free_label" (admin types an arbitrary branch label)
//   - end: "none"
export type ConnectionRule =
  | { kind: "none" }
  | { kind: "single" }
  | { kind: "multi" }
  | { kind: "fixed_labels"; labels: [string, string] }
  | { kind: "free_label" };

export interface PendingChoice {
  from: string;
  to: string;
  /** Empty means "prompt for free text" (the `free_label` rule). */
  options: string[];
}

export function useConnectMode(opts: {
  edges: { from: string; to: string; label: string | null }[];
  ruleFor: (nodeKey: string) => ConnectionRule;
  onError: (message: string) => void;
  onComplete: (from: string, to: string, label: string | null) => void;
}) {
  const { edges, ruleFor, onError, onComplete } = opts;
  const [connectFrom, setConnectFrom] = useState<string | null>(null);
  const [pendingChoice, setPendingChoice] = useState<PendingChoice | null>(null);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setConnectFrom(null);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  function startConnecting(fromKey: string) {
    setConnectFrom(fromKey);
  }

  function cancelConnecting() {
    setConnectFrom(null);
    setPendingChoice(null);
  }

  function resolveChoice(label: string | null) {
    if (!pendingChoice) return;
    onComplete(pendingChoice.from, pendingChoice.to, label);
    setConnectFrom(null);
    setPendingChoice(null);
  }

  /** Call from a node's onClick while in connect mode. Returns true if this
   * click was consumed as part of completing/cancelling a connection -
   * the caller should fall through to its own "select this node" handling
   * only when this returns false. */
  function targetClicked(toKey: string): boolean {
    if (!connectFrom) return false;
    if (connectFrom === toKey) {
      setConnectFrom(null);
      return true;
    }
    const rule = ruleFor(connectFrom);
    const existing = edges.filter((e) => e.from === connectFrom);
    if (rule.kind === "none") {
      onError(`'${connectFrom}' cannot have an outgoing connection`);
      setConnectFrom(null);
      return true;
    }
    if (rule.kind === "single") {
      if (existing.length > 0) {
        onError(`'${connectFrom}' already has an outgoing connection - delete it first`);
        setConnectFrom(null);
        return true;
      }
      onComplete(connectFrom, toKey, null);
      setConnectFrom(null);
      return true;
    }
    if (rule.kind === "multi") {
      if (existing.some((e) => e.to === toKey)) {
        onError("That connection already exists");
        setConnectFrom(null);
        return true;
      }
      onComplete(connectFrom, toKey, null);
      setConnectFrom(null);
      return true;
    }
    if (rule.kind === "fixed_labels") {
      const remaining = rule.labels.filter((l) => !existing.some((e) => e.label === l));
      if (remaining.length === 0) {
        onError(`'${connectFrom}' already has both of its required outgoing connections`);
        setConnectFrom(null);
        return true;
      }
      if (remaining.length === 1) {
        onComplete(connectFrom, toKey, remaining[0]);
        setConnectFrom(null);
        return true;
      }
      setPendingChoice({ from: connectFrom, to: toKey, options: remaining });
      return true;
    }
    // free_label
    setPendingChoice({ from: connectFrom, to: toKey, options: [] });
    return true;
  }

  return { connectFrom, pendingChoice, startConnecting, cancelConnecting, resolveChoice, targetClicked };
}
