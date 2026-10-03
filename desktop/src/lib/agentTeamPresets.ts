import type { EditEdge, EditNode } from "../components/visualBuilder/graphNodeMeta";

/**
 * Agent Studio 2.0 (issue #196): "5 orchestration presets... as one-click
 * starting graphs on the existing canvas" - the same client-side
 * starter-shape pattern `workflowTemplates.ts`/`businessRuleTemplates.ts`/
 * `agentTemplates.ts` already use, targeting the Execution Graph runtime
 * `AgentTeamsAdmin.tsx` actually drives (not `ai_agent_pipeline.rs`'s
 * separate, still-current 3 fixed topologies - confirmed those are a
 * smaller, unmerged system this preset mechanism doesn't touch). Applying
 * a preset seeds a brand-new, unsaved graph's node/edge state exactly the
 * way the blank trigger->end skeleton already does - "Save draft" still
 * makes the real `createExecutionGraph` call, so the result is an
 * ordinary, independent graph from then on, nothing preset-bound.
 *
 * Every `agent` node's `agent_id` is left blank - which real agent plays
 * which role is a per-workspace choice no template can guess, picked in
 * the property panel after applying, same as a Page Builder template
 * leaving a field picker blank when it has nothing real to suggest.
 */
export type AgentTeamPresetDef = {
  key: string;
  label: string;
  description: string;
  build: () => { nodes: EditNode[]; edges: EditEdge[] };
};

function node(node_key: string, node_type: EditNode["node_type"], x: number, y: number, config: Record<string, unknown> = {}): EditNode {
  return { node_key, node_type, config, x, y };
}
function edge(from_node_key: string, to_node_key: string, branch_label: string | null = null): EditEdge {
  return { from_node_key, to_node_key, branch_label };
}

export const AGENT_TEAM_PRESETS: AgentTeamPresetDef[] = [
  {
    key: "sequential_team",
    label: "Sequential Team",
    description: "Two agents working one after the other - the first agent's output becomes the second's input.",
    build: () => ({
      nodes: [
        node("trigger_1", "trigger", 40, 40),
        node("agent_1", "agent", 300, 40, { agent_id: "", input_template: "" }),
        node("agent_2", "agent", 560, 40, { agent_id: "", input_template: "{{agent_1.output}}" }),
        node("end_1", "end", 820, 40),
      ],
      edges: [edge("trigger_1", "agent_1"), edge("agent_1", "agent_2"), edge("agent_2", "end_1")],
    }),
  },
  {
    key: "parallel_research",
    label: "Parallel Research",
    description: "Three agents research independently and in parallel, then their findings join back into one result.",
    build: () => ({
      nodes: [
        node("trigger_1", "trigger", 40, 120),
        node("parallel_split_1", "parallel_split", 300, 120),
        node("agent_1", "agent", 560, 20, { agent_id: "", input_template: "" }),
        node("agent_2", "agent", 560, 140, { agent_id: "", input_template: "" }),
        node("agent_3", "agent", 560, 260, { agent_id: "", input_template: "" }),
        node("join_1", "join", 820, 140, { mode: "all", required_count: 3 }),
        node("end_1", "end", 1080, 140),
      ],
      edges: [
        edge("trigger_1", "parallel_split_1"),
        edge("parallel_split_1", "agent_1"),
        edge("parallel_split_1", "agent_2"),
        edge("parallel_split_1", "agent_3"),
        edge("agent_1", "join_1"),
        edge("agent_2", "join_1"),
        edge("agent_3", "join_1"),
        edge("join_1", "end_1"),
      ],
    }),
  },
  {
    key: "supervisor_team",
    label: "Supervisor Team",
    description: "A supervisor agent decides which specialist should handle the request, then its answer joins back for a final result.",
    build: () => ({
      nodes: [
        node("trigger_1", "trigger", 40, 120),
        node("agent_supervisor", "agent", 300, 120, { agent_id: "", input_template: "" }),
        node("router_1", "router", 560, 120, { branches: ["specialist_a", "specialist_b"] }),
        node("agent_specialist_1", "agent", 820, 20, { agent_id: "", input_template: "" }),
        node("agent_specialist_2", "agent", 820, 220, { agent_id: "", input_template: "" }),
        node("join_1", "join", 1080, 120, { mode: "all", required_count: 1 }),
        node("end_1", "end", 1340, 120),
      ],
      edges: [
        edge("trigger_1", "agent_supervisor"),
        edge("agent_supervisor", "router_1"),
        edge("router_1", "agent_specialist_1", "specialist_a"),
        edge("router_1", "agent_specialist_2", "specialist_b"),
        edge("agent_specialist_1", "join_1"),
        edge("agent_specialist_2", "join_1"),
        edge("join_1", "end_1"),
      ],
    }),
  },
  {
    key: "review_loop",
    label: "Review Loop",
    description: "An agent drafts a response, a judge grades it, and a failing draft is sent back for another attempt, bounded to a few tries.",
    // The back-edge must close directly onto the `loop` node itself, not
    // onto a node inside its body - `execution_graph_service.rs`'s own
    // cycle check (`find_unbounded_cycle`) only recognizes a bounded
    // iteration when the re-entry edge's target is the loop node, exactly
    // the shape its own `peer_review` Pipeline-to-graph mapping uses
    // (`loop --body--> drafter -> reviewer -> check --fail--> loop`).
    build: () => ({
      nodes: [
        node("trigger_1", "trigger", 40, 40),
        node("loop_1", "loop", 300, 40, { max_iterations: 3 }),
        node("agent_1", "agent", 560, 40, { agent_id: "", input_template: "" }),
        node("evaluate_result_1", "evaluate_result", 820, 40, { source_node_key: "agent_1", success_criteria: "The response fully and correctly answers the request" }),
        node("end_1", "end", 1080, 40),
      ],
      edges: [
        edge("trigger_1", "loop_1"),
        edge("loop_1", "agent_1", "body"),
        edge("agent_1", "evaluate_result_1"),
        edge("evaluate_result_1", "end_1", "pass"),
        edge("evaluate_result_1", "loop_1", "fail"),
        edge("loop_1", "end_1", "exit"),
      ],
    }),
  },
  {
    key: "plan_and_execute",
    label: "Plan & Execute",
    description: "A planning agent breaks the request into a plan, which hands off to an execution agent that carries it out.",
    build: () => ({
      nodes: [
        node("trigger_1", "trigger", 40, 40),
        node("agent_planner", "agent", 300, 40, { agent_id: "", input_template: "" }),
        node("transform_1", "transform", 560, 40, { set: { plan: "{{agent_planner.output}}" } }),
        node("agent_executor", "agent", 820, 40, { agent_id: "", input_template: "{{transform_1.plan}}" }),
        node("end_1", "end", 1080, 40),
      ],
      edges: [edge("trigger_1", "agent_planner"), edge("agent_planner", "transform_1"), edge("transform_1", "agent_executor"), edge("agent_executor", "end_1")],
    }),
  },
];

export function agentTeamPreset(key: string): AgentTeamPresetDef | undefined {
  return AGENT_TEAM_PRESETS.find((p) => p.key === key);
}
