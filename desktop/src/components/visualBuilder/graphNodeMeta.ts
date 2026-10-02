// Shared Visual Builder Framework: per-node-type metadata and pure helpers
// for editing an Execution Graph in `node_key` space (never the
// server-assigned row `id` - only a load from an existing graph needs to
// map `GraphNode.id` back to its own `node_key`, see `nodesFromGraph`/
// `edgesFromGraph`). Extracted out of `AgentTeamsAdmin.tsx` (the original
// consumer, issue #170) when Workflow Studio 2.0 (issue #193) became the
// second - pure data/functions only, no JSX or hooks, so both screens'
// own canvas/property-panel markup stays free to differ where their
// actual needs differ (Workflow Studio reuses the existing rich
// condition/action builders AgentTeamsAdmin deliberately left as raw
// JSON; this module is only the part that's genuinely identical).

import type { ConnectionRule } from "./useConnectMode";
import type { ExecutionGraph, ExecutionGraphInput, GraphEdgeInput, GraphNode, GraphNodeInput, GraphNodeType, WorkflowActionType } from "../../lib/types";

export type EditNode = { node_key: string; node_type: GraphNodeType; config: Record<string, unknown>; x: number; y: number };
export type EditEdge = { from_node_key: string; to_node_key: string; branch_label: string | null };

export const NODE_TYPE_META: Record<GraphNodeType, { label: string; color: string; rule: ConnectionRule }> = {
  trigger: { label: "Trigger", color: "#16a34a", rule: { kind: "single" } },
  condition: { label: "Condition", color: "#4f7cff", rule: { kind: "fixed_labels", labels: ["true", "false"] } },
  router: { label: "Switch", color: "#0891b2", rule: { kind: "free_label" } },
  action: { label: "Action", color: "#d97706", rule: { kind: "single" } },
  agent: { label: "Run Agent", color: "#9333ea", rule: { kind: "single" } },
  run_agent_team: { label: "Run Agent Team", color: "#9333ea", rule: { kind: "single" } },
  evaluate_result: { label: "Evaluate Result", color: "#4f7cff", rule: { kind: "fixed_labels", labels: ["pass", "fail"] } },
  approval: { label: "Approval", color: "#dc2626", rule: { kind: "fixed_labels", labels: ["approved", "rejected"] } },
  delay: { label: "Delay", color: "#64748b", rule: { kind: "single" } },
  transform: { label: "Transform", color: "#0284c7", rule: { kind: "single" } },
  loop: { label: "Loop", color: "#65a30d", rule: { kind: "fixed_labels", labels: ["body", "exit"] } },
  parallel_split: { label: "Parallel", color: "#db2777", rule: { kind: "multi" } },
  join: { label: "Join", color: "#db2777", rule: { kind: "single" } },
  end: { label: "End", color: "#94a3b8", rule: { kind: "none" } },
};

export const FULL_PALETTE: GraphNodeType[] = [
  "trigger", "condition", "router", "action", "agent", "run_agent_team", "evaluate_result",
  "approval", "delay", "transform", "loop", "parallel_split", "join", "end",
];

export function defaultConfig(type: GraphNodeType): Record<string, unknown> {
  switch (type) {
    case "condition":
      return { match_type: "all", conditions: [] };
    case "router":
      return { branches: [] };
    case "action":
      return { action_type: "create_task", params_json: "{}" };
    case "agent":
      return { agent_id: "", input_template: "" };
    case "run_agent_team":
      return { pipeline_id: "", input_template: "" };
    case "evaluate_result":
      return { source_node_key: "", success_criteria: "" };
    case "approval":
      return { subject_type: "execution_graph_node" };
    case "delay":
      return { delay_seconds: 60 };
    case "transform":
      return { set: {} };
    case "loop":
      return { max_iterations: 3 };
    case "join":
      return { mode: "all", required_count: 1 };
    default:
      return {};
  }
}

// Mirrors workflow_service::apply_action's own per-action-type param
// struct field names exactly, so the raw JSON an admin types for an
// Action node's `params_json` is real, runnable config - not a guess.
export const ACTION_PARAM_HINTS: Partial<Record<WorkflowActionType, string>> = {
  create_task: '{"title":"Follow up","description":null,"assignee_user_id":null,"due_in_days":1}',
  create_reminder: '{"title":"Reminder","description":null,"assignee_user_id":null,"remind_in_days":1}',
  update_field: '{"target_field_source":"builtin","target_field_key":"status","value_kind":"literal","literal_value":"Active"}',
  set_default_field: '{"target_field_source":"builtin","target_field_key":"status","value_kind":"literal","literal_value":"Active"}',
  clear_field: '{"target_field_source":"builtin","target_field_key":"notes"}',
  assign_owner: '{"user_id":null}',
  create_record: '{"entity_type":"Task","name_template":null}',
  add_notification: '{"audience":"owner","message":"Something happened"}',
  run_ai_agent: '{"agent_id":""}',
};

export function gridPosition(index: number): { x: number; y: number } {
  return { x: 40 + (index % 4) * 260, y: 40 + Math.floor(index / 4) * 160 };
}

export function nodesFromGraph(graph: ExecutionGraph): EditNode[] {
  return graph.nodes.map((n: GraphNode, i) => {
    const pos = n.position_x != null && n.position_y != null ? { x: n.position_x, y: n.position_y } : gridPosition(i);
    let config: Record<string, unknown> = {};
    try {
      config = JSON.parse(n.config_json || "{}");
    } catch {
      config = {};
    }
    return { node_key: n.node_key, node_type: n.node_type, config, x: pos.x, y: pos.y };
  });
}

export function edgesFromGraph(graph: ExecutionGraph): EditEdge[] {
  const idToKey = new Map(graph.nodes.map((n) => [n.id, n.node_key]));
  return graph.edges.map((e) => ({
    from_node_key: idToKey.get(e.from_node_id) ?? e.from_node_id,
    to_node_key: idToKey.get(e.to_node_id) ?? e.to_node_id,
    branch_label: e.branch_label,
  }));
}

export function nextNodeKey(nodes: EditNode[], type: GraphNodeType): string {
  let i = 1;
  while (nodes.some((n) => n.node_key === `${type}_${i}`)) i++;
  return `${type}_${i}`;
}

export function summarizeNode(n: EditNode): string {
  switch (n.node_type) {
    case "condition":
      return `${(n.config.conditions as unknown[] | undefined)?.length ?? 0} condition(s), match ${String(n.config.match_type ?? "all")}`;
    case "router":
      return `${(n.config.branches as unknown[] | undefined)?.length ?? 0} branch(es)`;
    case "action":
      return String(n.config.action_type ?? "");
    case "agent":
      return n.config.agent_id ? "agent selected" : "no agent selected";
    case "run_agent_team":
      return n.config.pipeline_id ? "team selected" : "no team selected";
    case "evaluate_result":
      return n.config.source_node_key ? `evaluates ${String(n.config.source_node_key)}` : "no node selected";
    case "approval":
      return String(n.config.subject_type ?? "");
    case "delay":
      return `${String(n.config.delay_seconds ?? 0)}s`;
    case "transform":
      return `${Object.keys((n.config.set as Record<string, unknown> | undefined) ?? {}).length} field(s)`;
    case "loop":
      return `max ${String(n.config.max_iterations ?? 1)} iteration(s)`;
    case "join":
      return `mode ${String(n.config.mode ?? "all")}`;
    default:
      return "";
  }
}

export function buildGraphInput(name: string, description: string | null, nodes: EditNode[], edges: EditEdge[]): ExecutionGraphInput {
  const nodeInputs: GraphNodeInput[] = nodes.map((n) => ({
    node_key: n.node_key,
    node_type: n.node_type,
    config_json: JSON.stringify(n.config ?? {}),
    position_x: n.x,
    position_y: n.y,
    sort_order: 0,
  }));
  const edgeInputs: GraphEdgeInput[] = edges.map((e) => ({ from_node_key: e.from_node_key, to_node_key: e.to_node_key, branch_label: e.branch_label ?? undefined }));
  return { name, description, nodes: nodeInputs, edges: edgeInputs };
}
