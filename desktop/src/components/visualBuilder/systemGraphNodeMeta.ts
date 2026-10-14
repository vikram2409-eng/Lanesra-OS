// Next-Gen program, Domain A, FND-01/FND-02: per-node-type metadata for
// the Dependency Explorer - pure data, no JSX/hooks, same split
// `graphNodeMeta.ts` already establishes. Scoped to exactly the node
// types `system_graph.rs`'s `NODE_TYPES` syncs.
//
// Ontology Explorer visual language (issue #339): `colorIndex` replaces
// this map's old hardcoded hex `color` field - every node now renders
// through Theme Studio's own categorical `--chart-1`..`--chart-6` tokens
// (the same `ontologyColorIndex`-style convention Object Types' own
// per-type color already uses), so a Theme Studio preset switch re-colors
// this graph too instead of a second, untouchable palette.
import type { SystemNodeType } from "../../lib/types";

export const SYSTEM_NODE_TYPE_META: Record<SystemNodeType, { label: string; colorIndex: number; color: string }> = {
  custom_object: { label: "Object", colorIndex: 1, color: "#4F46E5" },
  custom_field: { label: "Field", colorIndex: 2, color: "#64748B" },
  relationship: { label: "Relationship", colorIndex: 3, color: "#D97706" },
  business_rule: { label: "Business Rule", colorIndex: 4, color: "#B45309" },
  workflow: { label: "Workflow", colorIndex: 5, color: "#0F766E" },
  screen_layout: { label: "Screen Layout", colorIndex: 6, color: "#475569" },
  page_layout: { label: "Page Layout", colorIndex: 1, color: "#475569" },
  ai_agent: { label: "AI Agent", colorIndex: 2, color: "#9333EA" },
  execution_graph: { label: "Agent Team", colorIndex: 3, color: "#0F766E" },
  business_glossary_term: { label: "Glossary Term", colorIndex: 4, color: "#E11D48" },
  metric_definition: { label: "Metric", colorIndex: 5, color: "#0284C7" },
  test_case_definition: { label: "Test Case", colorIndex: 6, color: "#9333EA" },
  action_type_definition: { label: "Action Type", colorIndex: 1, color: "#475569" },
};

export const SYSTEM_NODE_TYPES: SystemNodeType[] = [
  "custom_object", "custom_field", "relationship", "business_rule", "workflow", "screen_layout", "page_layout", "ai_agent", "execution_graph",
  "business_glossary_term", "metric_definition", "test_case_definition", "action_type_definition",
];
