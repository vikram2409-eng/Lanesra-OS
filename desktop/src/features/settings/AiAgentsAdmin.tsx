import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { ChatPanel } from "../../components/ChatPanel";
import { AiTriggersPanel } from "./AiTriggersPanel";
import { agentRequiresAdmin } from "../../lib/aiAgents";
import { RISK_LEVELS, RISK_LEVEL_LABELS } from "../../lib/types";
import type {
  AgentConnectorToolOption,
  AiAgentDefinition,
  AiAgentInput,
  AiAgentModelRouting,
  AiAgentPolicy,
  AiAgentPolicyInput,
  AiAgentVersion,
  AiAgentVersionInput,
  AiApproval,
  AiProvider,
  AiSkill,
  AiToolRegistryOverride,
  RiskLevel,
  KnowledgeCollection,
  KnowledgeCollectionInput,
  KnowledgeSource,
  KnowledgeSourceInput,
  KnowledgeSearchHit,
  MemoryItem,
  MemoryType,
} from "../../lib/types";

// Phase 7a: the DLP classes an agent's forced-air-gap list can name -
// hand-mirrored from `dlp_service::CLASSES`, the same "hardcoded mirror
// of the server's own catalog" convention every other tool/action list on
// this page already follows.
const DLP_CLASSES: [string, string][] = [
  ["ssn", "Social Security Number"],
  ["credit_card", "Credit card number"],
  ["bank_account", "Bank account / routing number"],
  ["phone", "Phone number"],
  ["email", "Email address"],
];

// AI & Agentic Layer, Phase 6: the AI Agent Foundry's Agents tab.
// `action_names` is a per-tool checklist, hand-mirrored from
// chat_service.rs's own `record_tools()`/`admin_tools()` catalogs - the
// same "real shape, hardcoded to match" convention the online demo's own
// MCP_TOOLS list already uses for server/src/mcp.rs, since there's no
// dynamic "list every tool" endpoint to fetch this from instead.
const RECORD_ACTIONS: [string, string][] = [
  ["list_objects", "List objects"],
  ["get_object_metadata", "Get object metadata"],
  ["list_records", "List records"],
  ["get_record", "Get a record"],
  ["create_record", "Create a record"],
  ["update_record", "Update a record"],
  ["archive_record", "Archive a record"],
  ["search_records", "Search records (ranked, Custom Objects only)"],
];
const ADMIN_ACTIONS: [string, string][] = [
  ["list_business_rules", "List business rules"],
  ["create_business_rule", "Create business rule"],
  ["list_workflows", "List workflows"],
  ["create_workflow", "Create workflow"],
  ["list_custom_objects", "List custom objects"],
  ["create_custom_object", "Create custom object"],
  ["list_custom_fields", "List custom fields"],
  ["create_custom_field", "Create custom field"],
  ["list_relationships", "List relationships"],
  ["create_relationship", "Create relationship"],
  ["list_status_transitions", "List status transitions"],
  ["create_status_transition", "Create status transition"],
  ["list_connections", "List connections"],
  ["create_connection", "Create connection (no secret)"],
  ["list_webhooks", "List webhooks"],
  ["create_webhook", "Create webhook"],
  ["list_integration_jobs", "List integration jobs"],
  ["create_integration_job", "Create integration job"],
  ["list_api_clients", "List API clients"],
  ["create_api_client", "Create API client"],
  ["list_dashboards", "List dashboards"],
  ["create_dashboard", "Create dashboard"],
  ["list_apps", "List apps"],
  ["create_app", "Create app"],
  ["list_custom_reports", "List custom reports"],
  ["create_custom_report", "Create custom report"],
  ["list_saved_views", "List saved views"],
  ["create_saved_view", "Create saved view"],
  ["list_users", "List users"],
  ["create_user", "Create user"],
  ["list_numbering", "List numbering"],
  ["set_numbering", "Set numbering"],
  ["get_workspace_profile", "Get workspace profile"],
  ["update_workspace_profile", "Update workspace profile"],
  ["list_connectors", "List connectors"],
  ["list_ai_agents", "List AI agents"],
  ["create_ai_agent", "Create AI agent"],
  ["list_ai_skills", "List AI skills"],
  ["create_ai_skill", "Create AI skill"],
];
const ICON_CHOICES = ["🤖", "🧠", "🛠️", "📊", "📥", "🔎", "✉️", "📅", "🗂️", "⚡"];

function emptyInput(): AiAgentInput {
  return { name: "", description: "", icon: "🤖", system_prompt: "", action_names: [], delegate_agent_ids: [], skill_ids: [] };
}

export function AiAgentsAdmin({ onOpenHelp }: { onOpenHelp: (slug: string) => void }) {
  const queryClient = useQueryClient();
  const agentsQuery = useQuery({ queryKey: ["aiAgents"], queryFn: () => api.listAiAgents(false) });
  const skillsQuery = useQuery({ queryKey: ["aiSkills"], queryFn: () => api.listAiSkills(true) });
  const providersQuery = useQuery({ queryKey: ["aiProviders"], queryFn: () => api.listAiProviders(true) });
  // Integration Hub Tool Bridge: unlike RECORD_ACTIONS/ADMIN_ACTIONS
  // above, this one genuinely is workspace-scoped data (which connectors
  // an admin has opted into agent-tool use), so it's fetched rather than
  // hand-mirrored - see `connector_tool_service::list_options`.
  const connectorToolsQuery = useQuery({ queryKey: ["agentConnectorTools"], queryFn: () => api.listAgentConnectorTools() });
  const agents = agentsQuery.data ?? [];
  const skills = skillsQuery.data ?? [];
  const providers = providersQuery.data ?? [];
  const connectorTools = connectorToolsQuery.data ?? [];

  const [editing, setEditing] = useState<AiAgentDefinition | null>(null);
  const [creating, setCreating] = useState(false);
  const [chatWith, setChatWith] = useState<AiAgentDefinition | null>(null);
  const [memoryFor, setMemoryFor] = useState<AiAgentDefinition | null>(null);
  const [guardrailsFor, setGuardrailsFor] = useState<AiAgentDefinition | null>(null);
  const [routingFor, setRoutingFor] = useState<AiAgentDefinition | null>(null);
  const [versionsFor, setVersionsFor] = useState<AiAgentDefinition | null>(null);
  const [triggersFor, setTriggersFor] = useState<AiAgentDefinition | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["aiAgents"] });
  }

  const create = useMutation({
    mutationFn: (input: AiAgentInput) => api.createAiAgent(input),
    onSuccess: () => {
      setCreating(false);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this agent"),
  });
  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiAgentInput }) => api.updateAiAgent(id, input),
    onSuccess: () => {
      setEditing(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this agent"),
  });
  const toggleActive = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => api.setAiAgentActive(id, isActive),
    onSuccess: invalidate,
  });
  const saveMemory = useMutation({
    mutationFn: ({ id, memory_md }: { id: string; memory_md: string }) => api.setAiAgentMemory(id, { memory_md }),
    onSuccess: () => {
      setMemoryFor(null);
      invalidate();
    },
  });
  const saveGuardrails = useMutation({
    mutationFn: ({ id, guardrails_md }: { id: string; guardrails_md: string }) => api.setAiAgentGuardrails(id, { guardrails_md }),
    onSuccess: () => {
      setGuardrailsFor(null);
      invalidate();
    },
  });
  const saveRouting = useMutation({
    mutationFn: ({ id, routing }: { id: string; routing: AiAgentModelRouting | null }) => api.setAiAgentModelRouting(id, routing),
    onSuccess: () => {
      setRoutingFor(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this agent's routing"),
  });

  return (
    <div>
      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
          <div>
            <h3 style={{ margin: 0 }}>AI Agents</h3>
            <p style={{ color: "var(--text-muted)", fontSize: 13, margin: "4px 0 0" }}>
              Each agent is a persona layered over a set of Actions it can take, with its own persistent Memory, attached Skills, and
              optionally other agents it can delegate to.
            </p>
          </div>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn btn-secondary" onClick={() => onOpenHelp("build-your-first-agent")}>
              📖 Help
            </button>
            <button className="btn btn-primary" onClick={() => setCreating(true)}>
              + New agent
            </button>
          </div>
        </div>
        {error && <div className="error-banner">{error}</div>}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th></th>
                <th>Name</th>
                <th>Actions</th>
                <th>Skills</th>
                <th>Delegates</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {agents.map((a) => (
                <tr key={a.id}>
                  <td>{a.icon}</td>
                  <td>
                    <b>{a.name}</b>
                    {a.description && <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{a.description}</div>}
                  </td>
                  <td>{a.action_names.length}</td>
                  <td>{a.skill_ids.length}</td>
                  <td>{a.delegate_agent_ids.length}</td>
                  <td>
                    <span className={`badge${a.is_active ? " badge-success" : ""}`}>{a.is_active ? "Active" : "Inactive"}</span>
                  </td>
                  <td>
                    <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                      <button className="btn btn-secondary" onClick={() => setChatWith(a)} disabled={!a.is_active}>
                        Chat
                      </button>
                      <button className="btn btn-secondary" onClick={() => setEditing(a)}>
                        Edit
                      </button>
                      <button className="btn btn-secondary" onClick={() => setMemoryFor(a)}>
                        Memory
                      </button>
                      <button className="btn btn-secondary" onClick={() => setGuardrailsFor(a)}>
                        Guardrails
                      </button>
                      <button className="btn btn-secondary" onClick={() => setRoutingFor(a)}>
                        Routing{a.model_routing && <span className="badge badge-success" style={{ marginLeft: 4 }}>on</span>}
                      </button>
                      <button className="btn btn-secondary" onClick={() => setVersionsFor(a)}>
                        Versions
                      </button>
                      <button className="btn btn-secondary" onClick={() => setTriggersFor(triggersFor?.id === a.id ? null : a)}>
                        Triggers
                      </button>
                      <button className="btn btn-secondary" onClick={() => toggleActive.mutate({ id: a.id, isActive: !a.is_active })}>
                        {a.is_active ? "Deactivate" : "Reactivate"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {agents.length === 0 && <div className="empty-state">No agents yet - create one to get started.</div>}
        </div>
      </div>

      {(creating || editing) && (
        <AiAgentForm
          initial={editing ?? undefined}
          agents={agents}
          skills={skills}
          connectorTools={connectorTools}
          onCancel={() => {
            setCreating(false);
            setEditing(null);
          }}
          onSubmit={(input) => (editing ? update.mutate({ id: editing.id, input }) : create.mutate(input))}
          pending={create.isPending || update.isPending}
        />
      )}

      {chatWith && (
        <div className="card" style={{ marginTop: 16 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
            <h3 style={{ margin: 0 }}>
              {chatWith.icon} Chat with {chatWith.name}
            </h3>
            <button className="btn btn-secondary" onClick={() => setChatWith(null)}>
              Close
            </button>
          </div>
          <ChatPanel agentId={chatWith.id} />
        </div>
      )}

      {memoryFor && (
        <MemoryEditor
          agent={memoryFor}
          onCancel={() => setMemoryFor(null)}
          onSave={(memory_md) => saveMemory.mutate({ id: memoryFor.id, memory_md })}
          pending={saveMemory.isPending}
        />
      )}

      {guardrailsFor && (
        <GuardrailsEditor
          agent={guardrailsFor}
          onCancel={() => setGuardrailsFor(null)}
          onSave={(guardrails_md) => saveGuardrails.mutate({ id: guardrailsFor.id, guardrails_md })}
          pending={saveGuardrails.isPending}
        />
      )}

      {routingFor && (
        <ModelRoutingEditor
          agent={routingFor}
          providers={providers}
          onCancel={() => setRoutingFor(null)}
          onSave={(routing) => saveRouting.mutate({ id: routingFor.id, routing })}
          pending={saveRouting.isPending}
        />
      )}

      {versionsFor && <VersionsPanel agent={versionsFor} agents={agents} skills={skills} connectorTools={connectorTools} />}

      <ApprovalsPanel />

      <PolicyEnginePanel agents={agents} />

      <KnowledgeAndMemoryPanel />

      {triggersFor && (
        <div className="card" style={{ marginTop: 16 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <h3 style={{ margin: 0 }}>
              {triggersFor.icon} {triggersFor.name}'s triggers
            </h3>
            <button className="btn btn-secondary" onClick={() => setTriggersFor(null)}>
              Close
            </button>
          </div>
          <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
            Runs unattended (schedule or webhook) have no human actor - see this agent's own Actions: only a record-only agent can run this
            way, unless the trigger runs with a real Administrator behind it.
          </p>
          <AiTriggersPanel targetType="agent" targetId={triggersFor.id} />
        </div>
      )}
    </div>
  );
}

function MemoryEditor({
  agent,
  onCancel,
  onSave,
  pending,
}: {
  agent: AiAgentDefinition;
  onCancel: () => void;
  onSave: (memoryMd: string) => void;
  pending: boolean;
}) {
  const [value, setValue] = useState(agent.memory_md);
  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>
        {agent.icon} {agent.name}'s memory
      </h3>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        A living document this agent reads every run and can revise itself via its own update_memory tool. Edit it directly to seed or
        correct what it knows.
      </p>
      <textarea
        style={{ width: "100%", minHeight: 200, fontFamily: "ui-monospace, monospace", fontSize: 13 }}
        value={value}
        onChange={(e) => setValue(e.target.value)}
      />
      <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
        <button className="btn btn-primary" onClick={() => onSave(value)} disabled={pending}>
          {pending ? "Saving..." : "Save memory"}
        </button>
        <button className="btn btn-secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
      <MemoryHistoryPanel agentId={agent.id} />
    </div>
  );
}

/**
 * Phase 7b: read-only, most-recent-first history of this agent's prior
 * memory_md values - every snapshot taken just before an overwrite,
 * whichever path changed it (the agent's own update_memory tool, or an
 * admin's direct edit above). Collapsed by default since most agents will
 * have an empty or short history and this is a review/audit aid, not
 * something needed on every open.
 */
function MemoryHistoryPanel({ agentId }: { agentId: string }) {
  const [open, setOpen] = useState(false);
  const historyQuery = useQuery({
    queryKey: ["aiAgentMemoryHistory", agentId],
    queryFn: () => api.listAiAgentMemoryHistory(agentId),
    enabled: open,
  });
  const history = historyQuery.data ?? [];
  return (
    <div style={{ marginTop: 16, borderTop: "1px solid var(--border)", paddingTop: 12 }}>
      <button className="btn btn-secondary" onClick={() => setOpen((o) => !o)}>
        {open ? "Hide" : "Show"} memory history{history.length > 0 ? ` (${history.length})` : ""}
      </button>
      {open && (
        <div style={{ marginTop: 10 }}>
          {historyQuery.isLoading && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Loading...</p>}
          {!historyQuery.isLoading && history.length === 0 && (
            <p style={{ fontSize: 13, color: "var(--text-muted)" }}>No prior memory changes recorded yet.</p>
          )}
          {history.map((snap) => (
            <div key={snap.id} style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 10, marginBottom: 8 }}>
              <div style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 6 }}>
                {new Date(snap.created_at).toLocaleString()} · changed by {snap.changed_by === "agent" ? "the agent itself" : snap.changed_by}
              </div>
              <pre style={{ margin: 0, whiteSpace: "pre-wrap", fontFamily: "ui-monospace, monospace", fontSize: 12 }}>{snap.memory_md}</pre>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Phase 7c: an agent's operational-boundary statement - injected into its
 * system prompt alongside persona/memory (advisory/prompted, not
 * independently code-enforced; the one guard this phase does enforce in
 * code, consecutive-identical-tool-call loop detection, needs no editable
 * field here).
 */
function GuardrailsEditor({
  agent,
  onCancel,
  onSave,
  pending,
}: {
  agent: AiAgentDefinition;
  onCancel: () => void;
  onSave: (guardrailsMd: string) => void;
  pending: boolean;
}) {
  const [value, setValue] = useState(agent.guardrails_md);
  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>
        {agent.icon} {agent.name}'s guardrails
      </h3>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        An operational-boundary statement included in this agent's system prompt - the model is instructed to respect it, but it is not
        independently enforced. Loop detection (the same tool called 3 times in a row with identical inputs) is enforced in code
        regardless of what's written here.
      </p>
      <textarea
        style={{ width: "100%", minHeight: 160, fontFamily: "ui-monospace, monospace", fontSize: 13 }}
        value={value}
        onChange={(e) => setValue(e.target.value)}
      />
      <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
        <button className="btn btn-primary" onClick={() => onSave(value)} disabled={pending}>
          {pending ? "Saving..." : "Save guardrails"}
        </button>
        <button className="btn btn-secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

/**
 * Phase 7a: an agent's optional Gateway routing policy - primary/
 * fallback/local_fallback provider tiers, temperature/max_tokens
 * overrides, its own daily token budget, and which sensitive-data classes
 * force it straight to `local_fallback`. Enabling routing at all starts
 * every tier at "workspace default" (`null`), matching the backend's own
 * "None means the workspace default, not skip" rule - see
 * `AiAgentModelRouting`'s own doc comment in `core::models::ai`.
 */
function ModelRoutingEditor({
  agent,
  providers,
  onCancel,
  onSave,
  pending,
}: {
  agent: AiAgentDefinition;
  providers: AiProvider[];
  onCancel: () => void;
  onSave: (routing: AiAgentModelRouting | null) => void;
  pending: boolean;
}) {
  const [enabled, setEnabled] = useState(agent.model_routing !== null);
  const [routing, setRouting] = useState<AiAgentModelRouting>(
    agent.model_routing ?? {
      primary_provider_id: null,
      fallback_provider_id: null,
      local_fallback_provider_id: null,
      temperature: null,
      max_tokens: null,
      daily_token_budget: null,
      force_air_gapped_for: [],
    },
  );
  const usage = useQuery({ queryKey: ["aiAgentTokenUsage", agent.id], queryFn: () => api.getAiAgentTokenUsage(agent.id) });

  function toggleClass(name: string) {
    setRouting((prev) => ({
      ...prev,
      force_air_gapped_for: prev.force_air_gapped_for.includes(name) ? prev.force_air_gapped_for.filter((c) => c !== name) : [...prev.force_air_gapped_for, name],
    }));
  }

  function providerSelect(label: string, value: string | null, onChange: (v: string | null) => void) {
    return (
      <div className="field">
        <label>{label}</label>
        <select value={value ?? ""} onChange={(e) => onChange(e.target.value || null)}>
          <option value="">— Workspace default —</option>
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name} ({p.provider})
            </option>
          ))}
        </select>
      </div>
    );
  }

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>
        {agent.icon} {agent.name}'s model routing
      </h3>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        With no routing policy this agent uses the workspace's plain LLM &amp; MCP → LLM settings, unchanged. Turning
        this on lets it try a named provider first, fail over to another, and - for sensitive data classes below -
        route straight to a local/air-gapped provider instead of the cloud, regardless of the normal order.
      </p>
      <label style={{ fontSize: 13, display: "block", marginBottom: 8 }}>
        <input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} /> Configure a routing policy for this agent
      </label>

      {enabled && (
        <div className="form-grid">
          {providerSelect("Primary provider", routing.primary_provider_id, (v) => setRouting({ ...routing, primary_provider_id: v }))}
          {providerSelect("Fallback provider", routing.fallback_provider_id, (v) => setRouting({ ...routing, fallback_provider_id: v }))}
          {providerSelect("Local / air-gapped fallback", routing.local_fallback_provider_id, (v) => setRouting({ ...routing, local_fallback_provider_id: v }))}
          <div className="field">
            <label>Temperature (optional)</label>
            <input
              type="number"
              min={0}
              max={2}
              step={0.1}
              value={routing.temperature ?? ""}
              onChange={(e) => setRouting({ ...routing, temperature: e.target.value === "" ? null : Number(e.target.value) })}
            />
          </div>
          <div className="field">
            <label>Max tokens (optional)</label>
            <input
              type="number"
              min={1}
              value={routing.max_tokens ?? ""}
              onChange={(e) => setRouting({ ...routing, max_tokens: e.target.value === "" ? null : Number(e.target.value) })}
            />
          </div>
          <div className="field">
            <label>Daily token budget (optional)</label>
            <input
              type="number"
              min={1}
              value={routing.daily_token_budget ?? ""}
              onChange={(e) => setRouting({ ...routing, daily_token_budget: e.target.value === "" ? null : Number(e.target.value) })}
            />
            {usage.data && (
              <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0 0" }}>
                Used today: {usage.data.today_input_tokens + usage.data.today_output_tokens} tokens
              </p>
            )}
          </div>
          <div className="field full">
            <label>Force air-gapped routing for</label>
            <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 6px" }}>
              If a run's outbound payload matches any class checked here, it's sent straight to the local/air-gapped
              fallback above, never the cloud primary/fallback - checked before every dispatch.
            </p>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {DLP_CLASSES.map(([key, label]) => (
                <label key={key} style={{ fontSize: 13 }}>
                  <input type="checkbox" checked={routing.force_air_gapped_for.includes(key)} onChange={() => toggleClass(key)} /> {label}
                </label>
              ))}
            </div>
          </div>
        </div>
      )}

      <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
        <button className="btn btn-primary" onClick={() => onSave(enabled ? routing : null)} disabled={pending}>
          {pending ? "Saving..." : "Save routing"}
        </button>
        <button className="btn btn-secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}

function AiAgentForm({
  initial,
  agents,
  skills,
  connectorTools,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiAgentDefinition;
  agents: AiAgentDefinition[];
  skills: AiSkill[];
  connectorTools: AgentConnectorToolOption[];
  onCancel: () => void;
  onSubmit: (input: AiAgentInput) => void;
  pending: boolean;
}) {
  const [input, setInput] = useState<AiAgentInput>(
    initial
      ? {
          name: initial.name,
          description: initial.description,
          icon: initial.icon,
          system_prompt: initial.system_prompt,
          action_names: initial.action_names,
          delegate_agent_ids: initial.delegate_agent_ids,
          skill_ids: initial.skill_ids,
        }
      : emptyInput(),
  );

  function toggleAction(name: string) {
    setInput((prev) => ({
      ...prev,
      action_names: prev.action_names.includes(name) ? prev.action_names.filter((n) => n !== name) : [...prev.action_names, name],
    }));
  }
  function toggleSkill(id: string) {
    setInput((prev) => ({ ...prev, skill_ids: prev.skill_ids.includes(id) ? prev.skill_ids.filter((s) => s !== id) : [...prev.skill_ids, id] }));
  }
  function toggleDelegate(id: string) {
    setInput((prev) => ({
      ...prev,
      delegate_agent_ids: prev.delegate_agent_ids.includes(id) ? prev.delegate_agent_ids.filter((d) => d !== id) : [...prev.delegate_agent_ids, id],
    }));
  }

  const delegateChoices = agents.filter((a) => a.is_active && a.id !== initial?.id);
  const requiresAdmin = agentRequiresAdmin(input.action_names);

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Edit ${initial.name}` : "New agent"}</h3>
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          onSubmit(input);
        }}
      >
        <div className="field">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="field">
          <label>Icon</label>
          <select value={input.icon} onChange={(e) => setInput({ ...input, icon: e.target.value })}>
            {ICON_CHOICES.map((i) => (
              <option key={i} value={i}>
                {i}
              </option>
            ))}
          </select>
        </div>
        <div className="field full">
          <label>Description (optional)</label>
          <input value={input.description ?? ""} onChange={(e) => setInput({ ...input, description: e.target.value || null })} />
        </div>
        <div className="field full">
          <label>Persona / instructions</label>
          <textarea
            style={{ width: "100%", minHeight: 100 }}
            value={input.system_prompt}
            onChange={(e) => setInput({ ...input, system_prompt: e.target.value })}
            required
          />
        </div>
        <div className="field full">
          <label>
            Actions{" "}
            {requiresAdmin && (
              <span className="badge badge-danger" style={{ marginLeft: 6 }}>
                Administrator-only
              </span>
            )}
          </label>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 6px" }}>
            Granting any admin action makes this agent usable by administrators only - checked every time it runs, not just here.
          </p>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
            <div>
              <b style={{ fontSize: 12 }}>Records</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {RECORD_ACTIONS.map(([name, label]) => (
                  <label key={name} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={input.action_names.includes(name)} onChange={() => toggleAction(name)} /> {label}
                  </label>
                ))}
              </div>
            </div>
            <div>
              <b style={{ fontSize: 12 }}>Admin</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {ADMIN_ACTIONS.map(([name, label]) => (
                  <label key={name} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={input.action_names.includes(name)} onChange={() => toggleAction(name)} /> {label}
                  </label>
                ))}
              </div>
            </div>
          </div>
          {connectorTools.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <b style={{ fontSize: 12 }}>Connector Actions</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {connectorTools.map((t) => (
                  <label key={t.tool_name} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={input.action_names.includes(t.tool_name)} onChange={() => toggleAction(t.tool_name)} />{" "}
                    {t.connector_name}: {t.action_display_name} ({t.http_method.toUpperCase()})
                    {t.requires_admin && (
                      <span className="badge badge-danger" style={{ marginLeft: 6, fontSize: 10 }}>
                        Administrator
                      </span>
                    )}
                  </label>
                ))}
              </div>
            </div>
          )}
        </div>
        {skills.length > 0 && (
          <div className="field full">
            <label>Skills</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {skills.map((s) => (
                <label key={s.id} style={{ fontSize: 13 }}>
                  <input type="checkbox" checked={input.skill_ids.includes(s.id)} onChange={() => toggleSkill(s.id)} /> {s.name}
                </label>
              ))}
            </div>
          </div>
        )}
        {delegateChoices.length > 0 && (
          <div className="field full">
            <label>Can delegate to</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {delegateChoices.map((a) => (
                <label key={a.id} style={{ fontSize: 13 }}>
                  <input type="checkbox" checked={input.delegate_agent_ids.includes(a.id)} onChange={() => toggleDelegate(a.id)} /> {a.icon}{" "}
                  {a.name}
                </label>
              ))}
            </div>
          </div>
        )}
        <div className="field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={pending}>
            {pending ? "Saving..." : initial ? "Save agent" : "Create agent"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

// --- AI Agent Platform v2, Phase 1: version lifecycle + approvals ---------

/**
 * Draft -> Test -> Published -> Deprecated -> Disabled for one agent's
 * versions - see `agent_version_service`'s own doc comment for the legal
 * transition table. A Published version is an immutable historical
 * snapshot (its content can no longer be edited), which is why "Edit"
 * only appears for a Draft/Test row.
 */
const VERSION_NEXT_ACTIONS: Record<string, [string, string][]> = {
  draft: [
    ["test", "Move to Test"],
    ["disabled", "Disable"],
  ],
  test: [
    ["draft", "Back to Draft"],
    ["published", "Publish"],
    ["disabled", "Disable"],
  ],
  published: [["deprecated", "Deprecate"]],
  deprecated: [
    ["published", "Re-publish"],
    ["disabled", "Disable"],
  ],
  disabled: [],
};

function VersionsPanel({
  agent,
  agents,
  skills,
  connectorTools,
}: {
  agent: AiAgentDefinition;
  agents: AiAgentDefinition[];
  skills: AiSkill[];
  connectorTools: AgentConnectorToolOption[];
}) {
  const queryClient = useQueryClient();
  const versionsQuery = useQuery({ queryKey: ["aiAgentVersions", agent.id], queryFn: () => api.listAiAgentVersions(agent.id) });
  const versions = versionsQuery.data ?? [];
  const [drafting, setDrafting] = useState(false);
  const [editingVersion, setEditingVersion] = useState<AiAgentVersion | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["aiAgentVersions", agent.id] });
    queryClient.invalidateQueries({ queryKey: ["aiAgents"] });
  }

  const createDraft = useMutation({
    mutationFn: (input: AiAgentVersionInput) => api.createAiAgentVersionDraft(agent.id, input),
    onSuccess: () => {
      setDrafting(false);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this draft"),
  });
  const updateDraft = useMutation({
    mutationFn: ({ versionId, input }: { versionId: string; input: AiAgentVersionInput }) => api.updateAiAgentVersionDraft(agent.id, versionId, input),
    onSuccess: () => {
      setEditingVersion(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this draft"),
  });
  const transition = useMutation({
    mutationFn: ({ versionId, newStatus }: { versionId: string; newStatus: string }) => api.transitionAiAgentVersionStatus(agent.id, versionId, newStatus),
    onSuccess: () => {
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not change this version's status"),
  });

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 8 }}>
        <h3 style={{ margin: 0 }}>
          {agent.icon} {agent.name}'s versions
        </h3>
        <button className="btn btn-primary" onClick={() => setDrafting(true)}>
          + New draft
        </button>
      </div>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        Draft → Test → Published → Deprecated → Disabled. A Published version is an immutable snapshot of exactly what ran - publishing a
        new one automatically deprecates whichever version was Published before it.
      </p>
      {error && <div className="error-banner">{error}</div>}
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>v</th>
              <th>Status</th>
              <th>Name</th>
              <th>Created</th>
              <th>Published</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {versions.map((v) => (
              <tr key={v.id}>
                <td>{v.version_number}</td>
                <td>
                  <span className={`badge${v.status === "published" ? " badge-success" : ""}`}>{v.status}</span>
                  {agent.current_version_id === v.id && (
                    <span className="badge badge-success" style={{ marginLeft: 4 }}>
                      current
                    </span>
                  )}
                </td>
                <td>{v.name}</td>
                <td style={{ fontSize: 12 }}>{new Date(v.created_at).toLocaleString()}</td>
                <td style={{ fontSize: 12 }}>{v.published_at ? new Date(v.published_at).toLocaleString() : "—"}</td>
                <td>
                  <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                    {(v.status === "draft" || v.status === "test") && (
                      <button className="btn btn-secondary" onClick={() => setEditingVersion(v)}>
                        Edit
                      </button>
                    )}
                    {VERSION_NEXT_ACTIONS[v.status]?.map(([status, label]) => (
                      <button
                        key={status}
                        className="btn btn-secondary"
                        disabled={transition.isPending}
                        onClick={() => transition.mutate({ versionId: v.id, newStatus: status })}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {versionsQuery.isLoading && <div className="empty-state">Loading versions...</div>}
        {!versionsQuery.isLoading && versions.length === 0 && <div className="empty-state">No versions yet.</div>}
      </div>

      {(drafting || editingVersion) && (
        <VersionDraftForm
          agent={agent}
          agents={agents}
          skills={skills}
          connectorTools={connectorTools}
          initial={editingVersion ?? undefined}
          onCancel={() => {
            setDrafting(false);
            setEditingVersion(null);
          }}
          onSubmit={(input) => (editingVersion ? updateDraft.mutate({ versionId: editingVersion.id, input }) : createDraft.mutate(input))}
          pending={createDraft.isPending || updateDraft.isPending}
        />
      )}
    </div>
  );
}

function VersionDraftForm({
  agent,
  agents,
  skills,
  connectorTools,
  initial,
  onCancel,
  onSubmit,
  pending,
}: {
  agent: AiAgentDefinition;
  agents: AiAgentDefinition[];
  skills: AiSkill[];
  connectorTools: AgentConnectorToolOption[];
  initial?: AiAgentVersion;
  onCancel: () => void;
  onSubmit: (input: AiAgentVersionInput) => void;
  pending: boolean;
}) {
  const [name, setName] = useState(initial?.name ?? agent.name);
  const [description, setDescription] = useState(initial?.description ?? agent.description ?? "");
  const [icon, setIcon] = useState(initial?.icon ?? agent.icon);
  const [systemPrompt, setSystemPrompt] = useState(initial?.system_prompt ?? agent.system_prompt);
  const [actionNames, setActionNames] = useState<string[]>(initial?.action_names ?? agent.action_names);
  const [delegateIds, setDelegateIds] = useState<string[]>(initial?.delegate_agent_ids ?? agent.delegate_agent_ids);
  const [skillIds, setSkillIds] = useState<string[]>(initial?.skill_ids ?? agent.skill_ids);
  const [outputSchemaText, setOutputSchemaText] = useState(initial?.output_schema ? JSON.stringify(initial.output_schema, null, 2) : "");
  const [schemaError, setSchemaError] = useState<string | null>(null);

  function toggle(list: string[], setList: (v: string[]) => void, value: string) {
    setList(list.includes(value) ? list.filter((v) => v !== value) : [...list, value]);
  }

  const delegateChoices = agents.filter((a) => a.is_active && a.id !== agent.id);

  function submit() {
    let output_schema: unknown | null = null;
    if (outputSchemaText.trim()) {
      try {
        output_schema = JSON.parse(outputSchemaText);
      } catch {
        setSchemaError("Output schema must be valid JSON (or left blank for free-form text)");
        return;
      }
    }
    setSchemaError(null);
    onSubmit({
      name,
      description: description || null,
      icon,
      system_prompt: systemPrompt,
      action_names: actionNames,
      delegate_agent_ids: delegateIds,
      skill_ids: skillIds,
      model_routing: initial?.model_routing ?? agent.model_routing,
      output_schema,
    });
  }

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Editing v${initial.version_number} (${initial.status})` : "New draft version"}</h3>
      <div className="form-grid">
        <div className="field">
          <label>Name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} required />
        </div>
        <div className="field">
          <label>Icon</label>
          <select value={icon} onChange={(e) => setIcon(e.target.value)}>
            {ICON_CHOICES.map((i) => (
              <option key={i} value={i}>
                {i}
              </option>
            ))}
          </select>
        </div>
        <div className="field full">
          <label>Description (optional)</label>
          <input value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
        <div className="field full">
          <label>Persona / instructions</label>
          <textarea style={{ width: "100%", minHeight: 100 }} value={systemPrompt} onChange={(e) => setSystemPrompt(e.target.value)} required />
        </div>
        <div className="field full">
          <label>Actions</label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
            <div>
              <b style={{ fontSize: 12 }}>Records</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {RECORD_ACTIONS.map(([n, label]) => (
                  <label key={n} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={actionNames.includes(n)} onChange={() => toggle(actionNames, setActionNames, n)} /> {label}
                  </label>
                ))}
              </div>
            </div>
            <div>
              <b style={{ fontSize: 12 }}>Admin</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {ADMIN_ACTIONS.map(([n, label]) => (
                  <label key={n} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={actionNames.includes(n)} onChange={() => toggle(actionNames, setActionNames, n)} /> {label}
                  </label>
                ))}
              </div>
            </div>
          </div>
          {connectorTools.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <b style={{ fontSize: 12 }}>Connector Actions</b>
              <div style={{ maxHeight: 160, overflowY: "auto", border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 6, marginTop: 4 }}>
                {connectorTools.map((t) => (
                  <label key={t.tool_name} style={{ display: "block", fontSize: 13 }}>
                    <input type="checkbox" checked={actionNames.includes(t.tool_name)} onChange={() => toggle(actionNames, setActionNames, t.tool_name)} />{" "}
                    {t.connector_name}: {t.action_display_name}
                  </label>
                ))}
              </div>
            </div>
          )}
        </div>
        {skills.length > 0 && (
          <div className="field full">
            <label>Skills</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {skills.map((s) => (
                <label key={s.id} style={{ fontSize: 13 }}>
                  <input type="checkbox" checked={skillIds.includes(s.id)} onChange={() => toggle(skillIds, setSkillIds, s.id)} /> {s.name}
                </label>
              ))}
            </div>
          </div>
        )}
        {delegateChoices.length > 0 && (
          <div className="field full">
            <label>Can delegate to</label>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {delegateChoices.map((a) => (
                <label key={a.id} style={{ fontSize: 13 }}>
                  <input type="checkbox" checked={delegateIds.includes(a.id)} onChange={() => toggle(delegateIds, setDelegateIds, a.id)} /> {a.icon}{" "}
                  {a.name}
                </label>
              ))}
            </div>
          </div>
        )}
        <div className="field full">
          <label>Structured Output JSON Schema (optional)</label>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "2px 0 6px" }}>
            When set, this version's final answer must conform to this JSON Schema - checked automatically, with one repair attempt if it
            doesn't. Leave blank for today's plain free-form text.
          </p>
          <textarea
            style={{ width: "100%", minHeight: 100, fontFamily: "ui-monospace, monospace", fontSize: 12 }}
            placeholder={'{\n  "type": "object",\n  "required": ["answer"],\n  "properties": { "answer": { "type": "string" } }\n}'}
            value={outputSchemaText}
            onChange={(e) => setOutputSchemaText(e.target.value)}
          />
          {schemaError && <div className="error-banner" style={{ marginTop: 6 }}>{schemaError}</div>}
        </div>
        <div className="field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="button" onClick={submit} disabled={pending}>
            {pending ? "Saving..." : "Save draft"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * The workspace-wide durable approval inbox (`ai_approvals`) - Phase 1
 * ships the table and this review surface; nothing yet auto-creates an
 * approval (Phase 2's Tool-Call Firewall and later phases are what
 * actually route sensitive actions through it). Collapsed by default,
 * same reasoning as `MemoryHistoryPanel` above.
 */
function ApprovalsPanel() {
  const queryClient = useQueryClient();
  const [open, setOpen] = useState(false);
  const pendingQuery = useQuery({
    queryKey: ["aiApprovals", "pending"],
    queryFn: () => api.listAiApprovals("pending"),
    enabled: open,
  });
  const approvals = pendingQuery.data ?? [];

  const resolve = useMutation({
    mutationFn: ({ id, approve }: { id: string; approve: boolean }) => api.resolveAiApproval(id, { approve, resolution_notes: null }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["aiApprovals"] }),
  });

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0 }}>Pending Approvals</h3>
        <button className="btn btn-secondary" onClick={() => setOpen((o) => !o)}>
          {open ? "Hide" : "Show"}
        </button>
      </div>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        A durable, workspace-wide queue of proposed actions awaiting a human decision - the foundation the Tool-Call Firewall and other
        later phases route sensitive actions through.
      </p>
      {open && (
        <div>
          {pendingQuery.isLoading && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Loading...</p>}
          {!pendingQuery.isLoading && approvals.length === 0 && <div className="empty-state">Nothing pending right now.</div>}
          {approvals.map((a: AiApproval) => (
            <div key={a.id} style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 10, marginBottom: 8 }}>
              <div style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 6 }}>
                {a.subject_type} · {new Date(a.created_at).toLocaleString()}
                {a.requested_by && ` · requested by ${a.requested_by}`}
              </div>
              <pre style={{ margin: "0 0 8px", whiteSpace: "pre-wrap", fontFamily: "ui-monospace, monospace", fontSize: 12 }}>
                {JSON.stringify(a.proposal, null, 2)}
              </pre>
              <div style={{ display: "flex", gap: 6 }}>
                <button className="btn btn-primary" disabled={resolve.isPending} onClick={() => resolve.mutate({ id: a.id, approve: true })}>
                  Approve
                </button>
                <button className="btn btn-secondary" disabled={resolve.isPending} onClick={() => resolve.mutate({ id: a.id, approve: false })}>
                  Reject
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * AI Agent Platform v2, Phase 2: the Tool-Call Firewall's own decision,
 * editable per-agent or as a workspace-wide default (agentId "" ->
 * `null`), plus the risk-classification overrides it checks. No policy
 * anywhere in a workspace means every tool call is Allowed, unchanged
 * from before this feature existed - the same "purely additive" default
 * every other governance surface in this codebase (Voice Governance,
 * DLP routing) already follows.
 */
function PolicyEnginePanel({ agents }: { agents: AiAgentDefinition[] }) {
  const [open, setOpen] = useState(false);
  const [agentId, setAgentId] = useState("");

  const policyQuery = useQuery({
    queryKey: ["aiAgentPolicy", agentId || null],
    queryFn: () => api.getAiAgentPolicy(agentId || null),
    enabled: open,
  });

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0 }}>Policy Engine</h3>
        <button className="btn btn-secondary" onClick={() => setOpen((o) => !o)}>
          {open ? "Hide" : "Show"}
        </button>
      </div>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        Checked before every tool call dispatches - the exact same dispatcher a manual edit already uses, this only decides whether it
        runs at all. A blocklist denies a tool name outright; a risk threshold queues anything at or above it as a durable Pending Approval
        instead of running immediately.
      </p>
      {open && (
        <div>
          <label style={{ fontSize: 13, display: "block", marginBottom: 10 }}>
            Policy for{" "}
            <select value={agentId} onChange={(e) => setAgentId(e.target.value)}>
              <option value="">Workspace default</option>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.icon} {a.name}
                </option>
              ))}
            </select>
          </label>
          {policyQuery.isLoading && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Loading...</p>}
          {policyQuery.isSuccess && <PolicyEditor key={agentId} agentId={agentId || null} policy={policyQuery.data ?? null} />}
          <ToolRegistryOverridesSection />
        </div>
      )}
    </div>
  );
}

function PolicyEditor({ agentId, policy }: { agentId: string | null; policy: AiAgentPolicy | null }) {
  const queryClient = useQueryClient();
  const [threshold, setThreshold] = useState<RiskLevel | "">(policy?.require_approval_at_or_above ?? "");
  const [blockedText, setBlockedText] = useState((policy?.blocked_tool_names ?? []).join(", "));
  const [excludeRestrictedMemory, setExcludeRestrictedMemory] = useState(policy?.exclude_restricted_memory ?? true);

  const save = useMutation({
    mutationFn: () => {
      const input: AiAgentPolicyInput = {
        require_approval_at_or_above: threshold === "" ? null : threshold,
        blocked_tool_names: blockedText
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
        exclude_restricted_memory: excludeRestrictedMemory,
      };
      return api.upsertAiAgentPolicy(agentId, input);
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["aiAgentPolicy"] }),
  });

  return (
    <div style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 10, marginBottom: 16 }}>
      <label style={{ fontSize: 13, display: "block", marginBottom: 10 }}>
        Require administrator approval at or above{" "}
        <select value={threshold} onChange={(e) => setThreshold(e.target.value as RiskLevel | "")}>
          <option value="">Never (default)</option>
          {RISK_LEVELS.map((r) => (
            <option key={r} value={r}>
              {RISK_LEVEL_LABELS[r]}
            </option>
          ))}
        </select>
      </label>
      <label style={{ fontSize: 13, display: "block", marginBottom: 10 }}>
        Blocked tool names (comma-separated, denied outright regardless of risk level)
        <input
          style={{ width: "100%", marginTop: 4 }}
          value={blockedText}
          onChange={(e) => setBlockedText(e.target.value)}
          placeholder="e.g. create_api_client, archive_record"
        />
      </label>
      <label style={{ fontSize: 13, display: "block", marginBottom: 10 }}>
        <input type="checkbox" checked={excludeRestrictedMemory} onChange={(e) => setExcludeRestrictedMemory(e.target.checked)} style={{ marginRight: 6 }} />
        Exclude 'restricted'-classified content from durable memory (Session/Working/Entity - remember tool)
      </label>
      <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>
        {save.isPending ? "Saving..." : "Save policy"}
      </button>
    </div>
  );
}

function ToolRegistryOverridesSection() {
  const queryClient = useQueryClient();
  const overridesQuery = useQuery({ queryKey: ["aiToolRegistryOverrides"], queryFn: () => api.listAiToolRegistryOverrides() });
  const [toolName, setToolName] = useState("");
  const [riskLevel, setRiskLevel] = useState<RiskLevel>("write");

  const setOverride = useMutation({
    mutationFn: () => api.setAiToolRegistryOverride({ tool_name: toolName.trim(), risk_level: riskLevel }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["aiToolRegistryOverrides"] });
      setToolName("");
    },
  });
  const clearOverride = useMutation({
    mutationFn: (name: string) => api.clearAiToolRegistryOverride(name),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["aiToolRegistryOverrides"] }),
  });

  const overrides = overridesQuery.data ?? [];

  return (
    <div>
      <h4 style={{ marginBottom: 4 }}>Tool risk overrides</h4>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        Every tool name already has a sensible built-in risk classification (a list_*/get_* lookup is Read, an ordinary record create/
        update is Low write, a workspace-configuration change is Write, and so on) - a row here reclassifies one specific tool name away
        from that default.
      </p>
      {overrides.length === 0 && <div className="empty-state">No overrides - every tool uses its built-in default.</div>}
      {overrides.map((o: AiToolRegistryOverride) => (
        <div key={o.id} style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
          <code style={{ fontSize: 12 }}>{o.tool_name}</code>
          <span style={{ fontSize: 12, color: "var(--text-muted)" }}>{RISK_LEVEL_LABELS[o.risk_level]}</span>
          <button
            className="btn btn-secondary"
            style={{ fontSize: 12 }}
            onClick={() => clearOverride.mutate(o.tool_name)}
            disabled={clearOverride.isPending}
          >
            Revert to default
          </button>
        </div>
      ))}
      <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
        <input
          style={{ flex: 1 }}
          placeholder="tool name, e.g. create_record"
          value={toolName}
          onChange={(e) => setToolName(e.target.value)}
        />
        <select value={riskLevel} onChange={(e) => setRiskLevel(e.target.value as RiskLevel)}>
          {RISK_LEVELS.map((r) => (
            <option key={r} value={r}>
              {RISK_LEVEL_LABELS[r]}
            </option>
          ))}
        </select>
        <button className="btn btn-primary" onClick={() => setOverride.mutate()} disabled={!toolName.trim() || setOverride.isPending}>
          Set override
        </button>
      </div>
    </div>
  );
}

const MEMORY_TYPE_LABELS: Record<MemoryType, string> = { session: "Session", working: "Working", entity: "Entity" };

/// AI Agent Platform v2, Phase 4: Knowledge Collections/Sources (Document
/// RAG) and the admin Memory Inspector for Session/Working/Entity Memory -
/// same collapsible-card convention as PolicyEnginePanel above. Agent
/// Memory (memory_md) already has its own editor/history panel elsewhere
/// on this page - unaffected by this section.
function KnowledgeAndMemoryPanel() {
  const [open, setOpen] = useState(false);
  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0 }}>Knowledge & Memory</h3>
        <button className="btn btn-secondary" onClick={() => setOpen((o) => !o)}>
          {open ? "Hide" : "Show"}
        </button>
      </div>
      <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
        Document RAG (curated Knowledge Sources any agent's search_knowledge tool can cite) and the three itemized memory types
        (Session/Working/Entity) a remember/get_memory tool call writes and reads - distinct from an agent's own single persistent Memory
        document above.
      </p>
      {open && (
        <div>
          <KnowledgeSection />
          <MemoryInspectorSection />
        </div>
      )}
    </div>
  );
}

function KnowledgeSection() {
  const queryClient = useQueryClient();
  const collectionsQuery = useQuery({ queryKey: ["knowledgeCollections"], queryFn: () => api.listKnowledgeCollections() });
  const [selectedCollectionId, setSelectedCollectionId] = useState<string>("");
  const sourcesQuery = useQuery({
    queryKey: ["knowledgeSources", selectedCollectionId || null],
    queryFn: () => api.listKnowledgeSources(selectedCollectionId || null),
  });

  const [newCollectionName, setNewCollectionName] = useState("");
  const createCollection = useMutation({
    mutationFn: () => api.createKnowledgeCollection({ name: newCollectionName.trim() } as KnowledgeCollectionInput),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["knowledgeCollections"] });
      setNewCollectionName("");
    },
  });
  const deleteCollection = useMutation({
    mutationFn: (id: string) => api.deleteKnowledgeCollection(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["knowledgeCollections"] });
      setSelectedCollectionId("");
    },
  });

  const [sourceName, setSourceName] = useState("");
  const [sourceContent, setSourceContent] = useState("");
  const createSource = useMutation({
    mutationFn: () =>
      api.createKnowledgeSource({ name: sourceName.trim(), content: sourceContent, collection_id: selectedCollectionId || null } as KnowledgeSourceInput),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["knowledgeSources"] });
      setSourceName("");
      setSourceContent("");
    },
  });
  const deleteSource = useMutation({
    mutationFn: (id: string) => api.deleteKnowledgeSource(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["knowledgeSources"] }),
  });

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<KnowledgeSearchHit[] | null>(null);
  const runSearch = useMutation({
    mutationFn: () => api.searchKnowledgePreview(searchQuery, selectedCollectionId || null),
    onSuccess: (hits) => setSearchResults(hits),
  });

  const collections = collectionsQuery.data ?? [];
  const sources = sourcesQuery.data ?? [];

  return (
    <div style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 10, marginBottom: 16 }}>
      <h4 style={{ marginTop: 0, marginBottom: 4 }}>Document RAG - Knowledge Collections & Sources</h4>
      <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
        Text content only in this pass - paste or generate the source's own content, no file upload/parsing yet. Chunked and embedded via
        this workspace's configured provider; every search_knowledge result cites its source name.
      </p>
      <div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 10, flexWrap: "wrap" }}>
        <label style={{ fontSize: 13 }}>
          Collection{" "}
          <select value={selectedCollectionId} onChange={(e) => setSelectedCollectionId(e.target.value)}>
            <option value="">All / unfiled</option>
            {collections.map((c: KnowledgeCollection) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </label>
        {selectedCollectionId && (
          <button className="btn btn-secondary" style={{ fontSize: 12 }} onClick={() => deleteCollection.mutate(selectedCollectionId)} disabled={deleteCollection.isPending}>
            Delete this collection
          </button>
        )}
        <input style={{ flex: 1, minWidth: 160 }} placeholder="New collection name" value={newCollectionName} onChange={(e) => setNewCollectionName(e.target.value)} />
        <button className="btn btn-secondary" onClick={() => createCollection.mutate()} disabled={!newCollectionName.trim() || createCollection.isPending}>
          Add collection
        </button>
      </div>

      {sourcesQuery.isLoading && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Loading sources...</p>}
      {sources.length === 0 && !sourcesQuery.isLoading && <div className="empty-state">No knowledge sources yet.</div>}
      {sources.map((s: KnowledgeSource) => (
        <div key={s.id} style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
          <strong style={{ fontSize: 13 }}>{s.name}</strong>
          <span style={{ fontSize: 12, color: "var(--text-muted)" }}>
            {s.chunk_count} chunk{s.chunk_count === 1 ? "" : "s"} - {s.status}
          </span>
          <button className="btn btn-secondary" style={{ fontSize: 12 }} onClick={() => deleteSource.mutate(s.id)} disabled={deleteSource.isPending}>
            Delete
          </button>
        </div>
      ))}

      <div style={{ marginTop: 10 }}>
        <input style={{ width: "100%", marginBottom: 6 }} placeholder="New source name" value={sourceName} onChange={(e) => setSourceName(e.target.value)} />
        <textarea
          style={{ width: "100%", minHeight: 80, marginBottom: 6 }}
          placeholder="Source content (plain text)"
          value={sourceContent}
          onChange={(e) => setSourceContent(e.target.value)}
        />
        <button className="btn btn-primary" onClick={() => createSource.mutate()} disabled={!sourceName.trim() || !sourceContent.trim() || createSource.isPending}>
          {createSource.isPending ? "Chunking & embedding..." : "Add source"}
        </button>
        {createSource.isError && <p style={{ fontSize: 12, color: "var(--danger, #c0392b)" }}>{(createSource.error as ApiError)?.message ?? "Failed to add source"}</p>}
      </div>

      <div style={{ marginTop: 14, borderTop: "1px solid var(--border)", paddingTop: 10 }}>
        <div style={{ display: "flex", gap: 8 }}>
          <input style={{ flex: 1 }} placeholder="Test search_knowledge with a query..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} />
          <button className="btn btn-secondary" onClick={() => runSearch.mutate()} disabled={!searchQuery.trim() || runSearch.isPending}>
            Search
          </button>
        </div>
        {searchResults && (
          <div style={{ marginTop: 8 }}>
            {searchResults.length === 0 && <div className="empty-state">No matches.</div>}
            {searchResults.map((hit, i) => (
              <div key={i} style={{ fontSize: 12, marginBottom: 8, padding: 6, background: "var(--surface-2, #f5f5f5)", borderRadius: 6 }}>
                <div style={{ fontWeight: 600 }}>
                  {hit.source_name} (chunk {hit.chunk_index}, similarity {hit.similarity.toFixed(3)})
                </div>
                <div>{hit.content}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function MemoryInspectorSection() {
  const queryClient = useQueryClient();
  const [memoryType, setMemoryType] = useState<MemoryType | "">("");
  const [entityType, setEntityType] = useState("");
  const [entityId, setEntityId] = useState("");

  const itemsQuery = useQuery({
    queryKey: ["memoryItems", memoryType || null, entityType || null, entityId || null],
    queryFn: () => api.listMemoryItems(memoryType || null, entityType || null, entityId || null),
  });

  const forget = useMutation({
    mutationFn: (id: string) => api.forgetMemoryItem(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["memoryItems"] }),
  });

  const items = itemsQuery.data ?? [];

  return (
    <div style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 10 }}>
      <h4 style={{ marginTop: 0, marginBottom: 4 }}>Memory Inspector</h4>
      <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
        Every Session/Working/Entity item any agent's remember tool has written - inspect and delete retained memory, per this issue's own
        governance requirement.
      </p>
      <div style={{ display: "flex", gap: 8, marginBottom: 10, flexWrap: "wrap" }}>
        <select value={memoryType} onChange={(e) => setMemoryType(e.target.value as MemoryType | "")}>
          <option value="">All types</option>
          {(Object.keys(MEMORY_TYPE_LABELS) as MemoryType[]).map((t) => (
            <option key={t} value={t}>
              {MEMORY_TYPE_LABELS[t]}
            </option>
          ))}
        </select>
        <input style={{ width: 140 }} placeholder="Entity type (e.g. Company)" value={entityType} onChange={(e) => setEntityType(e.target.value)} />
        <input style={{ width: 160 }} placeholder="Entity id" value={entityId} onChange={(e) => setEntityId(e.target.value)} />
      </div>
      {itemsQuery.isLoading && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>Loading...</p>}
      {items.length === 0 && !itemsQuery.isLoading && <div className="empty-state">No memory items match these filters.</div>}
      {items.map((item: MemoryItem) => (
        <div key={item.id} style={{ fontSize: 12, marginBottom: 8, padding: 6, background: "var(--surface-2, #f5f5f5)", borderRadius: 6 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <span>
              <strong>{MEMORY_TYPE_LABELS[item.memory_type]}</strong>
              {item.entity_type && item.entity_id && (
                <span>
                  {" "}
                  - {item.entity_type} #{item.entity_id}
                </span>
              )}
              {item.classification !== "standard" && <span> - {item.classification}</span>}
            </span>
            <button className="btn btn-secondary" style={{ fontSize: 11 }} onClick={() => forget.mutate(item.id)} disabled={forget.isPending}>
              Forget
            </button>
          </div>
          <div style={{ marginTop: 4 }}>{item.content}</div>
          <div style={{ color: "var(--text-muted)", marginTop: 2 }}>
            {item.source} - {item.created_at}
            {item.expires_at ? ` - expires ${item.expires_at}` : ""}
          </div>
        </div>
      ))}
    </div>
  );
}
