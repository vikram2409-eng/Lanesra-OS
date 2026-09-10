import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiSkill, AiSkillInput } from "../../lib/types";

// AI & Agentic Layer, Phase 6: the AI Agent Foundry's Skills tab - a
// reusable library, not tied to any one agent (see AiAgentsAdmin.tsx's
// Skills multi-select). Same "short description up front, full content
// only on demand" shape this session's own Skill tool uses.
function emptyInput(): AiSkillInput {
  return { name: "", description: "", instructions_md: "" };
}

export function AiSkillsAdmin() {
  const queryClient = useQueryClient();
  const skillsQuery = useQuery({ queryKey: ["aiSkills", "all"], queryFn: () => api.listAiSkills(false) });
  const skills = skillsQuery.data ?? [];

  const [editing, setEditing] = useState<AiSkill | null>(null);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["aiSkills"] });
  }

  const create = useMutation({
    mutationFn: (input: AiSkillInput) => api.createAiSkill(input),
    onSuccess: () => {
      setCreating(false);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this skill"),
  });
  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiSkillInput }) => api.updateAiSkill(id, input),
    onSuccess: () => {
      setEditing(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this skill"),
  });
  const toggleActive = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => api.setAiSkillActive(id, isActive),
    onSuccess: invalidate,
  });

  return (
    <div>
      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
          <div>
            <h3 style={{ margin: 0 }}>Skills</h3>
            <p style={{ color: "var(--text-muted)", fontSize: 13, margin: "4px 0 0" }}>
              A reusable library any Agent can attach - only the name and description are shown to an agent up front; the full
              instructions load only when it decides to use one, via its use_skill tool.
            </p>
          </div>
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            + New skill
          </button>
        </div>
        {error && <div className="error-banner">{error}</div>}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Description</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {skills.map((s) => (
                <tr key={s.id}>
                  <td>
                    <b>{s.name}</b>
                  </td>
                  <td style={{ fontSize: 13, color: "var(--text-muted)" }}>{s.description}</td>
                  <td>
                    <span className={`badge${s.is_active ? " badge-success" : ""}`}>{s.is_active ? "Active" : "Inactive"}</span>
                  </td>
                  <td>
                    <div style={{ display: "flex", gap: 6 }}>
                      <button className="btn btn-secondary" onClick={() => setEditing(s)}>
                        Edit
                      </button>
                      <button className="btn btn-secondary" onClick={() => toggleActive.mutate({ id: s.id, isActive: !s.is_active })}>
                        {s.is_active ? "Deactivate" : "Reactivate"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {skills.length === 0 && <div className="empty-state">No skills yet - create one to attach to an agent.</div>}
        </div>
      </div>

      {(creating || editing) && (
        <AiSkillForm
          initial={editing ?? undefined}
          onCancel={() => {
            setCreating(false);
            setEditing(null);
          }}
          onSubmit={(input) => (editing ? update.mutate({ id: editing.id, input }) : create.mutate(input))}
          pending={create.isPending || update.isPending}
        />
      )}
    </div>
  );
}

function AiSkillForm({
  initial,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiSkill;
  onCancel: () => void;
  onSubmit: (input: AiSkillInput) => void;
  pending: boolean;
}) {
  const [input, setInput] = useState<AiSkillInput>(
    initial ? { name: initial.name, description: initial.description, instructions_md: initial.instructions_md } : emptyInput(),
  );

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Edit ${initial.name}` : "New skill"}</h3>
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          onSubmit(input);
        }}
      >
        <div className="field full">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="field full">
          <label>Description</label>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "0 0 4px" }}>What an agent sees up front, before deciding to use this.</p>
          <input value={input.description} onChange={(e) => setInput({ ...input, description: e.target.value })} required />
        </div>
        <div className="field full">
          <label>Instructions</label>
          <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "0 0 4px" }}>Only loaded into context when an agent calls use_skill.</p>
          <textarea
            style={{ width: "100%", minHeight: 160, fontFamily: "ui-monospace, monospace", fontSize: 13 }}
            value={input.instructions_md}
            onChange={(e) => setInput({ ...input, instructions_md: e.target.value })}
            required
          />
        </div>
        <div className="field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={pending}>
            {pending ? "Saving..." : initial ? "Save skill" : "Create skill"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
