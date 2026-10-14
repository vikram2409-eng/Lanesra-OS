/**
 * PR 3 — Shared graph fixtures for the regression suite.
 */
import type { GraphModel } from "../components/graph/graphTypes";

export const fixtureModel: GraphModel = {
  nodes: [
    { key: "custom_object:companies", id: "companies", type: "custom_object", label: "Companies", isRoot: true },
    { key: "custom_field:name", id: "name", type: "custom_field", label: "Name" },
    { key: "custom_field:revenue", id: "revenue", type: "custom_field", label: "Annual Revenue" },
    { key: "workflow:wf1", id: "wf1", type: "workflow", label: "Renewal workflow" },
    { key: "ai_agent:a1", id: "a1", type: "ai_agent", label: "Research Analyst" },
    // Deliberately unconnected: used by hover tests to assert dimming.
    { key: "metric_definition:m1", id: "m1", type: "metric_definition", label: "Pipeline coverage" },
  ],
  edges: [
    { fromKey: "custom_object:companies", toKey: "custom_field:name", edgeType: "has_field", label: "Has field" },
    { fromKey: "custom_object:companies", toKey: "custom_field:revenue", edgeType: "has_field", label: "Has field" },
    { fromKey: "custom_object:companies", toKey: "workflow:wf1", edgeType: "triggers", label: "Triggers" },
    { fromKey: "ai_agent:a1", toKey: "custom_object:companies", edgeType: "can_read", label: "Can read", access: true },
  ],
};

export const ROOT_KEY = "custom_object:companies";
