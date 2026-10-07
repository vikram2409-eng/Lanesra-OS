import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiAgentDefinition } from "../../lib/types";

const BUILTIN_OBJECT_KEYS = ["Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task"];

// Foundry-internal tools every agent can call regardless of its own
// action_names (chat_service::execute_agent_tool's own hardcoded match
// arms) - offered alongside this agent's real action_names so an admin
// can confirm the Tool-Call Firewall gap closed for these too (issue
// #245).
const AGENT_INTERNAL_TOOLS = ["update_memory", "remember", "get_memory", "search_knowledge", "use_skill", "delegate_to_agent"];

const RECORD_WRITE_TOOLS = ["create_record", "update_record", "archive_record"];

/**
 * Agent Access Governance (issue #245): a real, traceable answer to "what
 * can this agent actually touch" - the agent-shaped sibling of Access
 * Control v1's own AccessInspector.tsx. Calls the exact same
 * agent_access_inspector_service::inspect evaluator that would run for a
 * real call, so the answer here is exactly what would happen for real.
 */
export function AgentAccessInspector({ agents, onClose }: { agents: AiAgentDefinition[]; onClose: () => void }) {
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const customObjects = useQuery({ queryKey: ["customObjects"], queryFn: () => api.listCustomObjects(true) });

  const [agentId, setAgentId] = useState(agents[0]?.id ?? "");
  const agent = agents.find((a) => a.id === agentId);
  const toolOptions = [...(agent?.action_names ?? []), ...AGENT_INTERNAL_TOOLS];
  const [toolName, setToolName] = useState(toolOptions[0] ?? "");
  const [objectKey, setObjectKey] = useState(BUILTIN_OBJECT_KEYS[0]);
  const [recordId, setRecordId] = useState("");
  const [simulateAsUserId, setSimulateAsUserId] = useState("");
  const [result, setResult] = useState<Awaited<ReturnType<typeof api.inspectAgentAccess>> | null>(null);
  const [error, setError] = useState<string | null>(null);

  const objectKeys = [...BUILTIN_OBJECT_KEYS, ...(customObjects.data ?? []).map((o) => o.key)];
  const isRecordWriteTool = RECORD_WRITE_TOOLS.includes(toolName);

  const check = useMutation({
    mutationFn: () =>
      api.inspectAgentAccess(agentId, toolName, isRecordWriteTool ? objectKey : null, isRecordWriteTool ? recordId.trim() || null : null, simulateAsUserId || null),
    onSuccess: setResult,
    onError: (err) => {
      setResult(null);
      setError(err instanceof ApiError ? err.message : "Could not run the Agent Access Inspector");
    },
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <div className="toolbar">
        <h4 style={{ margin: 0 }}>Agent Access Inspector</h4>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Pick an agent and a tool - this runs the exact same Tool-Call Firewall decision (and, for a record-write tool
        once a policy enforces it, the exact same Access Control v1 trace) a real call would, so the answer here is
        exactly what would happen for real.
      </p>
      {error && <div className="error-banner">{error}</div>}

      <div className="form-grid">
        <div className="form-field">
          <label>Agent</label>
          <select
            value={agentId}
            onChange={(e) => {
              setAgentId(e.target.value);
              const next = agents.find((a) => a.id === e.target.value);
              setToolName(next?.action_names[0] ?? AGENT_INTERNAL_TOOLS[0]);
            }}
          >
            {agents.map((a) => (
              <option key={a.id} value={a.id}>
                {a.icon} {a.name}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Tool</label>
          <select value={toolName} onChange={(e) => setToolName(e.target.value)}>
            {toolOptions.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        {isRecordWriteTool && (
          <>
            <div className="form-field">
              <label>Object</label>
              <select value={objectKey} onChange={(e) => setObjectKey(e.target.value)}>
                {objectKeys.map((k) => (
                  <option key={k} value={k}>
                    {k}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-field">
              <label>Record id (optional)</label>
              <input value={recordId} onChange={(e) => setRecordId(e.target.value)} placeholder="Leave blank to check the capability generally" />
            </div>
            <div className="form-field">
              <label>Simulate as user (if this agent has no Acts As identity of its own)</label>
              <select value={simulateAsUserId} onChange={(e) => setSimulateAsUserId(e.target.value)}>
                <option value="">— None —</option>
                {(users.data ?? []).map((u) => (
                  <option key={u.id} value={u.id}>
                    {u.display_name}
                  </option>
                ))}
              </select>
            </div>
          </>
        )}
        <div className="form-field full">
          <button className="btn btn-primary" onClick={() => check.mutate()} disabled={check.isPending || !agentId || !toolName}>
            Inspect
          </button>
        </div>
      </div>

      {result && (
        <div style={{ marginTop: 12 }}>
          <p>
            <span className={`badge${result.tool_in_action_list ? " badge-success" : ""}`}>
              {result.tool_in_action_list ? "In this agent's tool list" : "Not in this agent's tool list"}
            </span>
          </p>
          <p>
            <span
              className={`badge${result.policy_decision.outcome === "allow" ? " badge-success" : ""}`}
            >
              {result.policy_decision.outcome === "allow" ? "Allowed" : result.policy_decision.outcome === "deny" ? "Denied" : "Requires approval"}
            </span>{" "}
            {result.policy_decision.outcome !== "allow" && <>(risk level: {result.policy_decision.risk_level}) </>}
            by {result.policy_source}
          </p>
          {isRecordWriteTool && (
            <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
              {result.record_access_enforced
                ? `Access Control v1 is enforced for this agent - traced as ${result.acting_as_user_id ?? "(no identity resolvable - set an Acts As identity or pick a user to simulate)"}.`
                : "Access Control v1 is not enforced on this agent's record writes yet - once Allowed above, it can act on any record of this type. Turn on \"Enforce Access Control on record writes\" in this agent's (or the workspace default) policy to scope it."}
            </p>
          )}
          {result.record_access && (
            <>
              <p>
                <span className={`badge${result.record_access.decision.allowed ? " badge-success" : ""}`}>
                  {result.record_access.decision.allowed ? "Allowed" : "Denied"}
                </span>{" "}
                {result.record_access.decision.reason}
              </p>
              {result.record_access.record_summary && (
                <p style={{ color: "var(--text-muted)", fontSize: 13 }}>Record: {result.record_access.record_summary}</p>
              )}
              <table>
                <thead>
                  <tr>
                    <th>Access Role</th>
                    <th>Matched grant</th>
                    <th>Grants this capability?</th>
                    <th>Scope</th>
                  </tr>
                </thead>
                <tbody>
                  {result.record_access.roles_checked.map((c, i) => (
                    <tr key={i}>
                      <td>{c.role_name}</td>
                      <td>{c.matched_object_key ?? "—"}</td>
                      <td>{c.capability_granted ? "Yes" : "No"}</td>
                      <td>{c.scope ?? "—"}</td>
                    </tr>
                  ))}
                  {result.record_access.roles_checked.length === 0 && (
                    <tr>
                      <td colSpan={4} className="empty-state">
                        This user holds no Access Roles.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </>
          )}
        </div>
      )}
    </div>
  );
}
