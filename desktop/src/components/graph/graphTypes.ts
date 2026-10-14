/**
 * PR 2 — Desktop Ontology Graph: shared graph type definitions.
 *
 * Ports the demo's graph model (`graphRender`'s `model`/`ui` contract)
 * into TypeScript. Colors are ported verbatim from the demo's
 * `SYSTEM_NODE_TYPE_COLORS` (app.js line 4636) for exact mockup parity.
 */
import type { SystemNodeType } from "../../lib/types";

/** A single node in the rendered graph. */
export interface GraphNode {
  /** Unique key: `${type}:${id}` (mirrors demo's `depNodeKey`). */
  key: string;
  /** Component id (backend identifier). */
  id: string;
  /** System node type. */
  type: SystemNodeType;
  /** Display label. */
  label: string;
  /** Node count (only set on `__cluster__` collapse nodes). */
  count?: number;
  /** True for the centered root node. */
  isRoot?: boolean;
}

/** A single directed edge in the rendered graph. */
export interface GraphEdge {
  fromKey: string;
  toKey: string;
  /** Raw edge type string (e.g. "has field", "depends_on"). */
  edgeType: string;
  /** Human-readable label. */
  label: string;
  /** True for access/permission edges (rendered dashed + 🔒). */
  access?: boolean;
}

/** The full model handed to the layout + renderer. */
export interface GraphModel {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/** Verbatim port of the demo's `SYSTEM_NODE_TYPE_COLORS` (app.js:4636). */
export const SYSTEM_NODE_TYPE_COLORS: Record<SystemNodeType, string> = {
  custom_object: "#4F46E5",
  custom_field: "#64748B",
  relationship: "#D97706",
  business_rule: "#B45309",
  workflow: "#0F766E",
  screen_layout: "#475569",
  page_layout: "#475569",
  ai_agent: "#9333EA",
  execution_graph: "#0F766E",
  business_glossary_term: "#E11D48",
  metric_definition: "#0284C7",
  test_case_definition: "#9333EA",
  action_type_definition: "#475569",
};

/** Verbatim port of the demo's `SYSTEM_NODE_TYPE_LABELS` (app.js:3754). */
export const SYSTEM_NODE_TYPE_LABELS: Record<SystemNodeType, string> = {
  custom_object: "Object",
  custom_field: "Field",
  relationship: "Relationship",
  business_rule: "Business Rule",
  workflow: "Workflow",
  screen_layout: "Screen Layout",
  page_layout: "Page Layout",
  ai_agent: "AI Agent",
  execution_graph: "Agent Team",
  business_glossary_term: "Glossary Term",
  metric_definition: "Metric",
  test_case_definition: "Test Case",
  action_type_definition: "Action Type",
};

/** Verbatim port of the demo's `DEP_CLUSTER_ORDER` (app.js:4653). */
export const GRAPH_CLUSTER_ORDER: SystemNodeType[] = [
  "custom_object",
  "custom_field",
  "relationship",
  "business_rule",
  "workflow",
  "screen_layout",
  "page_layout",
  "ai_agent",
  "execution_graph",
  "business_glossary_term",
  "metric_definition",
  "test_case_definition",
  "action_type_definition",
];

/** Builds a node's stable key from type + id (mirrors demo's `depNodeKey`). */
export function graphNodeKey(nodeType: SystemNodeType, id: string): string {
  return `${nodeType}:${id}`;
}

/** Edge types the demo treats as access/permission edges (dashed + 🔒). */
const ACCESS_EDGE_TYPES = new Set(["grants_access_to", "can_read", "can_write", "can_execute"]);

/** True when the edge should render as an access edge. */
export function isAccessEdge(edgeType: string): boolean {
  return ACCESS_EDGE_TYPES.has(edgeType);
}
