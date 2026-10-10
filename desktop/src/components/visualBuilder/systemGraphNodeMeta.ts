// Next-Gen program, Domain A, FND-01: per-node-type metadata for the
// Dependency Explorer - pure data, no JSX/hooks, same split
// `graphNodeMeta.ts` already establishes. Scoped to exactly the 9 node
// types `system_graph.rs`'s `NODE_TYPES` syncs in this v1 slice.

import type { SystemNodeType } from "../../lib/types";

export const SYSTEM_NODE_TYPE_META: Record<SystemNodeType, { label: string; color: string }> = {
  custom_object: { label: "Object", color: "#4f7cff" },
  custom_field: { label: "Field", color: "#0891b2" },
  relationship: { label: "Relationship", color: "#db2777" },
  business_rule: { label: "Business Rule", color: "#d97706" },
  workflow: { label: "Workflow", color: "#16a34a" },
  screen_layout: { label: "Screen Layout", color: "#64748b" },
  page_layout: { label: "Page Layout", color: "#0284c7" },
  ai_agent: { label: "AI Agent", color: "#9333ea" },
  execution_graph: { label: "Agent Team", color: "#9333ea" },
};

export const SYSTEM_NODE_TYPES: SystemNodeType[] = [
  "custom_object", "custom_field", "relationship", "business_rule", "workflow", "screen_layout", "page_layout", "ai_agent", "execution_graph",
];
