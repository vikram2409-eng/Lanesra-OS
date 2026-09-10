import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { ChatPanel } from "../../components/ChatPanel";
import { agentRequiresAdmin } from "../../lib/aiAgents";
import type { AiAgentDefinition, AiAgentInput, AiSkill } from "../../lib/types";

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

export function AiAgentsAdmin() {
  const queryClient = useQueryClient();
  const agentsQuery = useQuery({ queryKey: ["aiAgents"], queryFn: () => api.listAiAgents(false) });
  const skillsQuery = useQuery({ queryKey: ["aiSkills"], queryFn: () => api.listAiSkills(true) });
  const agents = agentsQuery.data ?? [];
  const skills = skillsQuery.data ?? [];

  const [editing, setEditing] = useState<AiAgentDefinition | null>(null);
  const [creating, setCreating] = useState(false);
  const [chatWith, setChatWith] = useState<AiAgentDefinition | null>(null);
  const [memoryFor, setMemoryFor] = useState<AiAgentDefinition | null>(null);
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
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            + New agent
          </button>
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
    </div>
  );
}

function AiAgentForm({
  initial,
  agents,
  skills,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiAgentDefinition;
  agents: AiAgentDefinition[];
  skills: AiSkill[];
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
