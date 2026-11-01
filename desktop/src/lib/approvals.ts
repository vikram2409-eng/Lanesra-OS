import type { AiApproval } from "./types";

/** WP-0150: one line saying what an approval decides, by its subject. */
export function describeApproval(a: AiApproval): string {
  const p = (a.proposal ?? {}) as Record<string, unknown>;
  const text = (v: unknown) => (typeof v === "string" || typeof v === "number" ? String(v) : "");
  switch (a.subject_type) {
    case "pipeline_run":
      return `A pipeline run waits for approval of step ${Number(p.step_order ?? 0) + 1}`;
    case "tool_call":
      return `An agent wants to call ${text(p.tool_name)}${p.risk_level ? ` (risk: ${text(p.risk_level)})` : ""}`;
    case "voice_plan":
      return `A voice command needs approval: "${text(p.transcript)}"`;
    case "agent_version_publish":
      return "Publish an agent version";
    default:
      return p.node_key ? `An Agent Team run waits at ${text(p.node_key)}` : a.subject_type;
  }
}
