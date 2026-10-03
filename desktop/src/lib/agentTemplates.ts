import type { AiAgentInput, AiAgentPolicyInput, RiskLevel } from "./types";

/**
 * Agent Studio 2.0 (issue #196): "Agent Templates gallery... each a
 * preconfigured starting tool/policy/output-schema combination, not a new
 * execution mode" - the same client-side starter-shape pattern
 * `workflowTemplates.ts`/`businessRuleTemplates.ts` already use for their
 * own 5 starter templates, applied here to an agent's base fields plus
 * its optional Policy and first Draft version's Output Schema.
 *
 * `build()` returns exactly what `AiAgentInput` can carry; `policy` and
 * `outputSchema` are applied as follow-up calls once the agent exists
 * (policy and output schema are both per-existing-agent concepts -
 * `upsertAiAgentPolicy`/`createAiAgentVersionDraft` - a brand-new agent
 * has no id yet for either to attach to). See `AiAgentsAdmin.tsx`'s
 * `applyAgentTemplate` for the 3-step apply sequence.
 */
export type AgentTemplateDef = {
  key: string;
  label: string;
  description: string;
  build: () => AiAgentInput;
  policy?: AiAgentPolicyInput;
  outputSchema?: unknown;
};

function base(name: string, icon: string, description: string, tone: string, persona: string, action_names: string[]): AiAgentInput {
  return {
    name,
    description,
    icon,
    system_prompt: `Tone: ${tone}\n\n${persona}`,
    action_names,
    delegate_agent_ids: [],
    skill_ids: [],
  };
}

const READ_ONLY = ["list_objects", "get_object_metadata", "list_records", "get_record", "search_records"];

export const AGENT_TEMPLATES: AgentTemplateDef[] = [
  {
    key: "business_assistant",
    label: "Business Assistant",
    description: "A general-purpose, read-only assistant for day-to-day questions about records and the workspace. No approval gate - it can't change anything to approve.",
    build: () => base(
      "Business Assistant",
      "🧠",
      "Answers questions about records and the workspace",
      "Friendly",
      "You are a general-purpose assistant. Help the person asking with questions about their records and this workspace, using the tools available to you. Be concise and cite the specific record or setting you're referencing.",
      READ_ONLY,
    ),
  },
  {
    key: "record_triage",
    label: "Record Triage",
    description: "Reviews incoming records and flags which ones need attention, with a structured yes/no + reason output - read-only, never changes a record itself.",
    build: () => base(
      "Record Triage",
      "📥",
      "Flags records that need attention",
      "Concise",
      "You review records and decide whether each one needs human attention right now. You never create, update or delete anything - you only read and report.",
      READ_ONLY,
    ),
    outputSchema: {
      type: "object",
      properties: {
        needs_attention: { type: "boolean" },
        reason: { type: "string" },
      },
      required: ["needs_attention", "reason"],
    },
  },
  {
    key: "action_agent",
    label: "Action Agent",
    description: "Creates and updates records under a human-set guardrail - anything destructive queues for Administrator approval instead of running immediately.",
    build: () => base(
      "Action Agent",
      "⚡",
      "Creates and updates records under guardrails",
      "Professional",
      "You take well-defined actions on records - creating and updating them as asked. Confirm what you changed and why in your reply.",
      ["list_records", "get_record", "search_records", "create_record", "update_record"],
    ),
    policy: { require_approval_at_or_above: "destructive" as RiskLevel, blocked_tool_names: [], exclude_restricted_memory: true },
  },
  {
    key: "research_agent",
    label: "Research Agent",
    description: "Researches across records and produces a structured summary with sources - read-only.",
    build: () => base(
      "Research Agent",
      "🔎",
      "Researches records and produces a structured summary",
      "Technical",
      "You research a topic across the records and data available to you, then produce a clear, well-sourced summary. Always note which records your summary draws from.",
      READ_ONLY,
    ),
    outputSchema: {
      type: "object",
      properties: {
        summary: { type: "string" },
        sources: { type: "array", items: { type: "string" } },
      },
      required: ["summary"],
    },
  },
  {
    key: "supervisor_agent",
    label: "Supervisor Agent",
    description: "Delegates work to other agents and synthesizes their results rather than acting directly - pick which agents it can delegate to after creating it.",
    build: () => base(
      "Supervisor Agent",
      "🗂️",
      "Delegates to other agents and synthesizes their results",
      "Professional",
      "You coordinate other agents rather than acting directly yourself. Break the request into sub-tasks, delegate each to the right agent, then synthesize their results into one clear answer.",
      ["list_records", "get_record"],
    ),
  },
  {
    key: "document_agent",
    label: "Document Agent",
    description: "Extracts structured fields from records with a confidence score - read-only.",
    build: () => base(
      "Document Agent",
      "📄",
      "Extracts structured fields from records",
      "Technical",
      "You extract specific structured fields from the records you're given, and report your confidence in the extraction. If a field isn't present, omit it rather than guessing.",
      READ_ONLY,
    ),
    outputSchema: {
      type: "object",
      properties: {
        extracted_fields: { type: "object" },
        confidence: { type: "number" },
      },
      required: ["extracted_fields"],
    },
  },
];

export function agentTemplate(key: string): AgentTemplateDef | undefined {
  return AGENT_TEMPLATES.find((t) => t.key === key);
}
