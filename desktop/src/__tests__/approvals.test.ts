// WP-0150: the approvals inbox names what each approval decides.
import { describe, expect, it } from "vitest";

import { describeApproval } from "../lib/approvals";
import type { AiApproval } from "../lib/types";

function approval(subject_type: string, proposal: Record<string, unknown>): AiApproval {
  return {
    id: "a1",
    workspace_id: "ws",
    subject_type,
    subject_id: "s1",
    proposal,
    status: "pending",
    requested_by: null,
    resolved_by: null,
    resolution_notes: null,
    created_at: "2026-11-01T00:00:00Z",
    resolved_at: null,
    consumed_at: null,
  };
}

describe("describeApproval", () => {
  it("names each kind of pause", () => {
    expect(describeApproval(approval("pipeline_run", { step_order: 0 }))).toBe("A pipeline run waits for approval of step 1");
    expect(describeApproval(approval("tool_call", { tool_name: "delete_record", risk_level: "destructive" }))).toBe(
      "An agent wants to call delete_record (risk: destructive)",
    );
    expect(describeApproval(approval("voice_plan", { transcript: "close the deal" }))).toBe('A voice command needs approval: "close the deal"');
    expect(describeApproval(approval("execution_graph_node", { node_key: "gate" }))).toBe("An Agent Team run waits at gate");
    expect(describeApproval(approval("custom", {}))).toBe("custom");
  });
});
