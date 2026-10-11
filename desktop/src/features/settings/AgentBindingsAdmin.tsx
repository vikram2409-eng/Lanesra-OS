import { Fragment, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { ActionCapability, AiAgentDefinition, AiAgentInput, ObjectTypeScope } from "../../lib/types";
import { AgentOntologyScopeEditor } from "./AgentOntologyScopeEditor";

function inputFromAgent(agent: AiAgentDefinition): AiAgentInput {
  return {
    name: agent.name,
    description: agent.description,
    icon: agent.icon,
    system_prompt: agent.system_prompt,
    action_names: agent.action_names,
    delegate_agent_ids: agent.delegate_agent_ids,
    skill_ids: agent.skill_ids,
    object_type_scopes: agent.object_type_scopes,
    action_type_ids: agent.action_type_ids,
  };
}

/**
 * Ontology Layer epic, issue #345: a standalone screen listing every AI
 * agent with its Ontology scope at a glance, inline-editable through the
 * exact same `AgentOntologyScopeEditor` the Agent Foundry's own agent form
 * uses - same component, same `["aiAgents"]` query, so an edit made here
 * and one made from Agent Foundry can never drift out of sync with each
 * other.
 */
export function AgentBindingsAdmin() {
  const queryClient = useQueryClient();
  const agentsQuery = useQuery({ queryKey: ["aiAgents"], queryFn: () => api.listAiAgents(false) });
  const agents = agentsQuery.data ?? [];

  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState<{ object_type_scopes: ObjectTypeScope[]; action_type_ids: string[] } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiAgentInput }) => api.updateAiAgent(id, input),
    onSuccess: () => {
      setEditingId(null);
      setDraft(null);
      setError(null);
      queryClient.invalidateQueries({ queryKey: ["aiAgents"] });
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this agent's Ontology scope"),
  });

  function startEditing(agent: AiAgentDefinition) {
    setEditingId(agent.id);
    setDraft({ object_type_scopes: agent.object_type_scopes, action_type_ids: agent.action_type_ids });
    setError(null);
  }

  function toggleObjectTypeScope(objectType: string) {
    setDraft((prev) =>
      prev
        ? {
            ...prev,
            object_type_scopes: prev.object_type_scopes.some((s) => s.object_type === objectType)
              ? prev.object_type_scopes.filter((s) => s.object_type !== objectType)
              : [...prev.object_type_scopes, { object_type: objectType, capabilities: ["read"] as ActionCapability[] }],
          }
        : prev,
    );
  }
  function toggleScopeCapability(objectType: string, capability: ActionCapability) {
    setDraft((prev) =>
      prev
        ? {
            ...prev,
            object_type_scopes: prev.object_type_scopes.map((s) =>
              s.object_type === objectType
                ? { ...s, capabilities: s.capabilities.includes(capability) ? s.capabilities.filter((c) => c !== capability) : [...s.capabilities, capability] }
                : s,
            ),
          }
        : prev,
    );
  }
  function toggleActionType(id: string) {
    setDraft((prev) =>
      prev ? { ...prev, action_type_ids: prev.action_type_ids.includes(id) ? prev.action_type_ids.filter((a) => a !== id) : [...prev.action_type_ids, id] } : prev,
    );
  }

  function save(agent: AiAgentDefinition) {
    if (!draft) return;
    update.mutate({ id: agent.id, input: { ...inputFromAgent(agent), object_type_scopes: draft.object_type_scopes, action_type_ids: draft.action_type_ids } });
  }

  return (
    <div>
      <h3>Agent Bindings</h3>
      <p style={{ color: "var(--text-muted)", maxWidth: 720 }}>
        Every AI agent's Ontology scope in one place - which object types and capabilities it may touch, and which
        Action Types it may invoke. Editing here or from an agent's own Agent Foundry form saves to the exact same
        place; there is no separate copy to drift out of sync.
      </p>
      {error && <div className="error-banner">{error}</div>}

      <table className="data-table">
        <thead>
          <tr>
            <th></th>
            <th>Agent</th>
            <th>Ontology scope</th>
            <th>Action Types</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {agents.map((agent) => (
            <Fragment key={agent.id}>
              <tr>
                <td style={{ width: 28, textAlign: "center" }}>{agent.icon}</td>
                <td>
                  <strong>{agent.name}</strong>
                  {!agent.is_active && <span className="badge" style={{ marginLeft: 8 }}>Inactive</span>}
                </td>
                <td>{agent.object_type_scopes.length === 0 ? <span className="muted">Unscoped</span> : `${agent.object_type_scopes.length} object type(s)`}</td>
                <td>{agent.action_type_ids.length === 0 ? <span className="muted">None</span> : `${agent.action_type_ids.length} granted`}</td>
                <td>
                  {editingId === agent.id ? (
                    <button type="button" className="btn btn-secondary" onClick={() => { setEditingId(null); setDraft(null); }}>
                      Cancel
                    </button>
                  ) : (
                    <button type="button" className="btn btn-secondary" onClick={() => startEditing(agent)}>
                      Edit scope
                    </button>
                  )}
                </td>
              </tr>
              {editingId === agent.id && draft && (
                <tr>
                  <td colSpan={5}>
                    <div className="card" style={{ margin: "4px 0 12px" }}>
                      <AgentOntologyScopeEditor
                        objectTypeScopes={draft.object_type_scopes}
                        actionTypeIds={draft.action_type_ids}
                        onToggleObjectTypeScope={toggleObjectTypeScope}
                        onToggleScopeCapability={toggleScopeCapability}
                        onToggleActionType={toggleActionType}
                      />
                      <div className="toolbar">
                        <button type="button" className="btn btn-primary" disabled={update.isPending} onClick={() => save(agent)}>
                          {update.isPending ? "Saving..." : "Save"}
                        </button>
                        <button type="button" className="btn btn-secondary" onClick={() => { setEditingId(null); setDraft(null); }}>
                          Cancel
                        </button>
                      </div>
                    </div>
                  </td>
                </tr>
              )}
            </Fragment>
          ))}
          {agents.length === 0 && !agentsQuery.isLoading && (
            <tr>
              <td colSpan={5} style={{ color: "var(--text-muted)" }}>
                No AI agents yet - create one in AI Agent Foundry first.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
