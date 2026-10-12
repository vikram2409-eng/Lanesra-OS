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

export const SYSTEM_NODE_TYPE_META: Record<SystemNodeType, { label: string; colorIndex: number }> = {
  custom_object: { label: "Object", colorIndex: 1 },
  custom_field: { label: "Field", colorIndex: 2 },
  relationship: { label: "Relationship", colorIndex: 3 },
  business_rule: { label: "Business Rule", colorIndex: 4 },
  workflow: { label: "Workflow", colorIndex: 5 },
  screen_layout: { label: "Screen Layout", colorIndex: 6 },
  page_layout: { label: "Page Layout", colorIndex: 1 },
  ai_agent: { label: "AI Agent", colorIndex: 2 },
  execution_graph: { label: "Agent Team", colorIndex: 3 },
  business_glossary_term: { label: "Glossary Term", colorIndex: 4 },
  metric_definition: { label: "Metric", colorIndex: 5 },
  test_case_definition: { label: "Test Case", colorIndex: 6 },
  action_type_definition: { label: "Action Type", colorIndex: 1 },
};

export const SYSTEM_NODE_TYPES: SystemNodeType[] = [
  "custom_object", "custom_field", "relationship", "business_rule", "workflow", "screen_layout", "page_layout", "ai_agent", "execution_graph",
  "business_glossary_term", "metric_definition", "test_case_definition", "action_type_definition",
];
