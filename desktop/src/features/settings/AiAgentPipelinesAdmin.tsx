import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { AiTriggersPanel } from "./AiTriggersPanel";
import type { AiAgentPipeline, AiAgentPipelineInput, AiAgentRun, AiAgentRunStep, PipelineStepInput } from "../../lib/types";

// AI & Agentic Layer, Phase 6b: Orchestration - a deterministic, ordered
// Pipeline of Agents (complementary to an Agent's own dynamic
// delegate_to_agent tool - see chat_service.rs's own doc comment on why
// both exist), plus each Pipeline's Triggers and its run history.
function emptyInput(): AiAgentPipelineInput {
  return { name: "", description: "", topology: "sequential", steps: [] };
}

const TOPOLOGY_LABELS: Record<AiAgentPipeline["topology"], string> = {
  sequential: "Sequential",
  consensus: "Consensus",
  peer_review: "Peer review",
};

// Phase 7g: mirrors `validate_pipeline_input`'s own rule exactly - a
// human-approval gate is only meaningful at the one step each topology
// has a well-defined resume point for.
function stepSupportsApproval(topology: AiAgentPipelineInput["topology"], i: number, total: number): boolean {
  if (topology === "sequential") return true;
  if (topology === "consensus") return i === total - 1;
  if (topology === "peer_review") return i === 1;
  return false;
}

// Phase 7g: `paused_at_step_order` means something different per
// topology (see `ai_orchestration_service.rs`'s own doc comments on
// `run_consensus`/`run_peer_review_from`) - sequential's own step-count
// framing only reads correctly for sequential itself.
function awaitingApprovalMessage(topology: AiAgentPipelineInput["topology"], pausedAtStepOrder: number | null | undefined, totalSteps: number): string {
  if (topology === "consensus") {
    return "Paused for approval before the synthesizer step. Edit the joined candidate outputs below before continuing, or leave them as-is to synthesize unchanged.";
  }
  if (topology === "peer_review") {
    return 'Paused for approval on this round’s reviewer verdict. Edit the feedback below before it’s acted on – or start it with "APPROVED" to force approval – or leave it as-is to continue unchanged.';
  }
  return `Paused for approval before step ${(pausedAtStepOrder ?? 0) + 1} of ${totalSteps}. Edit the output below before continuing, or leave it as-is to approve unchanged.`;
}

function stepRoleLabel(topology: AiAgentPipelineInput["topology"], i: number, total: number): string {
  if (topology === "consensus") return i === total - 1 ? "Synthesizer" : `Candidate ${i + 1}`;
  if (topology === "peer_review") return i === 0 ? "Drafter" : "Reviewer";
  return `Step ${i + 1}`;
}

function runStatusBadgeClass(status: string): string {
  if (status === "succeeded") return " badge-success";
  if (status === "awaiting_approval") return " badge-warning";
  return " badge-danger";
}

// Phase 7e: real wall-clock duration for a step, from its started_at/
// finished_at columns - null for a step recorded before that migration.
function stepDuration(s: AiAgentRunStep): string | null {
  if (!s.started_at || !s.finished_at) return null;
  const ms = new Date(s.finished_at).getTime() - new Date(s.started_at).getTime();
  if (!Number.isFinite(ms) || ms < 0) return null;
  return ms < 1000 ? `${ms}ms` : `${(ms / 1000).toFixed(1)}s`;
}

export function AiAgentPipelinesAdmin() {
  const queryClient = useQueryClient();
  const pipelinesQuery = useQuery({ queryKey: ["aiAgentPipelines"], queryFn: () => api.listAiAgentPipelines(false) });
  const agentsQuery = useQuery({ queryKey: ["aiAgents", "all"], queryFn: () => api.listAiAgents(true) });
  const pipelines = pipelinesQuery.data ?? [];
  const agents = agentsQuery.data ?? [];

  const [editing, setEditing] = useState<AiAgentPipeline | null>(null);
  const [creating, setCreating] = useState(false);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["aiAgentPipelines"] });
  }

  const create = useMutation({
    mutationFn: (input: AiAgentPipelineInput) => api.createAiAgentPipeline(input),
    onSuccess: () => {
      setCreating(false);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this pipeline"),
  });
  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiAgentPipelineInput }) => api.updateAiAgentPipeline(id, input),
    onSuccess: () => {
      setEditing(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this pipeline"),
  });
  const toggleActive = useMutation({
    mutationFn: ({ id, isActive }: { id: string; isActive: boolean }) => api.setAiAgentPipelineActive(id, isActive),
    onSuccess: invalidate,
  });

  function agentName(id: string): string {
    const a = agents.find((x) => x.id === id);
    return a ? `${a.icon} ${a.name}` : "(deleted agent)";
  }

  function stepsSummary(p: AiAgentPipeline): string {
    const names = p.steps.map((s) => agentName(s.agent_id));
    if (p.topology === "consensus" && names.length > 0) {
      const synth = names[names.length - 1];
      const candidates = names.slice(0, -1);
      return `${candidates.join(" + ")} → ${synth}`;
    }
    if (p.topology === "peer_review" && names.length === 2) {
      return `${names[0]} ⇄ ${names[1]}`;
    }
    return names.join(" → ");
  }

  return (
    <div>
      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
          <div>
            <h3 style={{ margin: 0 }}>Orchestration</h3>
            <p style={{ color: "var(--text-muted)", fontSize: 13, margin: "4px 0 0" }}>
              A Pipeline runs a fixed, ordered chain of Agents - step 2 onward can reference the prior step's answer as
              <code>{" {{previous_output}} "}</code>. Reliable, repeatable automation; for a supervisor Agent that decides at chat time
              who to delegate to, see AI Agents' own "Can delegate to" instead.
            </p>
          </div>
          <button className="btn btn-primary" onClick={() => setCreating(true)}>
            + New pipeline
          </button>
        </div>
        {error && <div className="error-banner">{error}</div>}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Steps</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {pipelines.map((p) => (
                <tr key={p.id}>
                  <td>
                    <b>{p.name}</b>
                    <div style={{ marginTop: 2 }}>
                      <span className="badge">{TOPOLOGY_LABELS[p.topology]}</span>
                    </div>
                    {p.description && <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{p.description}</div>}
                  </td>
                  <td>{stepsSummary(p)}</td>
                  <td>
                    <span className={`badge${p.is_active ? " badge-success" : ""}`}>{p.is_active ? "Active" : "Inactive"}</span>
                  </td>
                  <td>
                    <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                      <button className="btn btn-secondary" onClick={() => setExpanded(expanded === p.id ? null : p.id)}>
                        {expanded === p.id ? "Close" : "Run / Triggers / History"}
                      </button>
                      <button className="btn btn-secondary" onClick={() => setEditing(p)}>
                        Edit
                      </button>
                      <button className="btn btn-secondary" onClick={() => toggleActive.mutate({ id: p.id, isActive: !p.is_active })}>
                        {p.is_active ? "Deactivate" : "Reactivate"}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {pipelines.length === 0 && <div className="empty-state">No pipelines yet - create one to chain agents together.</div>}
        </div>
      </div>

      {expanded && pipelines.find((p) => p.id === expanded) && <PipelineDetail pipeline={pipelines.find((p) => p.id === expanded)!} agentName={agentName} />}

      {(creating || editing) && (
        <AiAgentPipelineForm
          initial={editing ?? undefined}
          agents={agents}
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

function PipelineDetail({ pipeline, agentName }: { pipeline: AiAgentPipeline; agentName: (id: string) => string }) {
  const [input, setInput] = useState("");
  const [activeRun, setActiveRun] = useState<AiAgentRun | null>(null);
  const [editedOutput, setEditedOutput] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [otlpJson, setOtlpJson] = useState<string | null>(null);
  const runsQuery = useQuery({ queryKey: ["aiAgentRuns", "pipeline", pipeline.id], queryFn: () => api.listAiAgentRuns("pipeline", pipeline.id, 10) });

  function reviewRun(r: AiAgentRun) {
    setActiveRun(r);
    setEditedOutput(r.resume_previous_output ?? "");
    setError(null);
  }

  const run = useMutation({
    mutationFn: () => api.runAiAgentPipeline(pipeline.id, input),
    onSuccess: (r) => {
      reviewRun(r);
      runsQuery.refetch();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not run this pipeline"),
  });
  const approve = useMutation({
    mutationFn: () => api.approveAiAgentPendingStep(activeRun!.id, editedOutput.trim() ? editedOutput : null),
    onSuccess: (r) => {
      reviewRun(r);
      runsQuery.refetch();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not approve this step"),
  });
  const reject = useMutation({
    mutationFn: (reason: string) => api.rejectAiAgentPendingRun(activeRun!.id, reason),
    onSuccess: (r) => {
      reviewRun(r);
      runsQuery.refetch();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not reject this run"),
  });
  const viewOtlp = useMutation({
    mutationFn: (runId: string) => api.exportAiAgentRunOtlp(runId),
    onSuccess: (data) => setOtlpJson(JSON.stringify(data, null, 2)),
  });
  const [pushResult, setPushResult] = useState<string | null>(null);
  const pushOtlp = useMutation({
    mutationFn: (runId: string) => api.pushAiAgentRunOtlp(runId),
    onSuccess: (message) => {
      setError(null);
      setPushResult(message);
    },
    onError: (err) => {
      setPushResult(null);
      setError(err instanceof ApiError ? err.message : "Could not push this trace");
    },
  });

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{pipeline.name}</h3>

      <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <input style={{ flex: 1 }} placeholder="Trigger input for step 1..." value={input} onChange={(e) => setInput(e.target.value)} />
        <button className="btn btn-primary" onClick={() => run.mutate()} disabled={run.isPending}>
          {run.isPending ? "Running..." : "Run now"}
        </button>
      </div>
      {error && <div className="error-banner">{error}</div>}
      {activeRun && (
        <div style={{ marginBottom: 12 }}>
          <p style={{ fontSize: 13 }}>
            Result: <span className={`badge${runStatusBadgeClass(activeRun.status)}`}>{activeRun.status}</span>
          </p>
          {activeRun.steps.map((s) => (
            <div key={s.id} style={{ fontSize: 13, borderTop: "1px dashed var(--border, #ddd)", padding: "6px 0" }}>
              <b>{agentName(s.agent_id)}</b>
              {stepDuration(s) && <span style={{ color: "var(--text-muted)" }}> ({stepDuration(s)})</span>}
              <div style={{ color: "var(--text-muted)" }}>in: {s.input_text}</div>
              {s.output_text && <div>out: {s.output_text}</div>}
              {s.error && <div style={{ color: "var(--large, #b23b3b)" }}>error: {s.error}</div>}
            </div>
          ))}
          {activeRun.status === "awaiting_approval" && (
            <div className="panel" style={{ marginTop: 8 }}>
              <p style={{ fontSize: 13, margin: "0 0 6px" }}>{awaitingApprovalMessage(pipeline.topology, activeRun.paused_at_step_order, pipeline.steps.length)}</p>
              <textarea style={{ width: "100%", minHeight: 80 }} value={editedOutput} onChange={(e) => setEditedOutput(e.target.value)} />
              <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
                <button className="btn btn-primary" onClick={() => approve.mutate()} disabled={approve.isPending}>
                  {approve.isPending ? "Approving..." : "Approve & continue"}
                </button>
                <button
                  className="btn btn-secondary"
                  disabled={reject.isPending}
                  onClick={() => {
                    const reason = window.prompt("Reason for rejecting this run?") ?? "";
                    reject.mutate(reason);
                  }}
                >
                  Reject
                </button>
              </div>
            </div>
          )}
          <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 8, flexWrap: "wrap" }}>
            <button className="btn btn-secondary" onClick={() => viewOtlp.mutate(activeRun.id)}>
              View OTLP trace
            </button>
            <button className="btn btn-secondary" disabled={pushOtlp.isPending} onClick={() => pushOtlp.mutate(activeRun.id)}>
              {pushOtlp.isPending ? "Pushing..." : "Push to collector"}
            </button>
            {pushResult && <span style={{ fontSize: 12, color: "var(--text-muted)" }}>{pushResult}</span>}
          </div>
        </div>
      )}
      {otlpJson && (
        <div className="panel" style={{ marginTop: 8, marginBottom: 12 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
            <b style={{ fontSize: 13 }}>OTLP trace JSON</b>
            <button className="icon-btn" onClick={() => setOtlpJson(null)}>
              ✕
            </button>
          </div>
          <pre style={{ maxHeight: 240, overflow: "auto", fontSize: 11 }}>{otlpJson}</pre>
        </div>
      )}

      <AiTriggersPanel targetType="pipeline" targetId={pipeline.id} />

      <div style={{ marginTop: 12 }}>
        <b style={{ fontSize: 13 }}>Recent runs</b>
        {(runsQuery.data ?? []).length === 0 && <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No runs yet.</p>}
        {(runsQuery.data ?? []).map((r) => (
          <div key={r.id} style={{ fontSize: 12, display: "flex", gap: 8, padding: "3px 0", alignItems: "center" }}>
            <span className={`badge${runStatusBadgeClass(r.status)}`}>{r.status}</span>
            <span>{r.triggered_by ?? "manual"}</span>
            <span style={{ color: "var(--text-muted)" }}>{new Date(r.started_at).toLocaleString()}</span>
            {r.status === "awaiting_approval" && (
              <button className="icon-btn" onClick={() => reviewRun(r)}>
                Review
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function AiAgentPipelineForm({
  initial,
  agents,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiAgentPipeline;
  agents: { id: string; icon: string; name: string }[];
  onCancel: () => void;
  onSubmit: (input: AiAgentPipelineInput) => void;
  pending: boolean;
}) {
  const [input, setInput] = useState<AiAgentPipelineInput>(
    initial
      ? {
          name: initial.name,
          description: initial.description,
          topology: initial.topology,
          steps: initial.steps.map((s) => ({ agent_id: s.agent_id, input_template: s.input_template, requires_approval: s.requires_approval })),
        }
      : emptyInput(),
  );

  function addStep() {
    if (agents.length === 0) return;
    setInput((prev) => ({
      ...prev,
      steps: [...prev.steps, { agent_id: agents[0].id, input_template: prev.steps.length === 0 || prev.topology === "consensus" ? "{{trigger_input}}" : "{{previous_output}}", requires_approval: false }],
    }));
  }
  function updateStep(i: number, patch: Partial<PipelineStepInput>) {
    setInput((prev) => ({ ...prev, steps: prev.steps.map((s, idx) => (idx === i ? { ...s, ...patch } : s)) }));
  }
  function removeStep(i: number) {
    setInput((prev) => ({ ...prev, steps: prev.steps.filter((_, idx) => idx !== i) }));
  }
  function moveStep(i: number, dir: -1 | 1) {
    setInput((prev) => {
      const steps = [...prev.steps];
      const j = i + dir;
      if (j < 0 || j >= steps.length) return prev;
      [steps[i], steps[j]] = [steps[j], steps[i]];
      return { ...prev, steps };
    });
  }

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Edit ${initial.name}` : "New pipeline"}</h3>
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
          <label>Description (optional)</label>
          <input value={input.description ?? ""} onChange={(e) => setInput({ ...input, description: e.target.value || null })} />
        </div>
        <div className="field full">
          <label>Topology</label>
          <select
            value={input.topology}
            onChange={(e) => {
              const topology = e.target.value as AiAgentPipelineInput["topology"];
              setInput((prev) => ({
                ...prev,
                topology,
                // A step's "needs approval" flag only makes sense at the
                // one step the new topology actually supports it on -
                // drop it from every other step rather than carry a flag
                // forward that the backend would now reject on save.
                steps: prev.steps.map((s, i) => (stepSupportsApproval(topology, i, prev.steps.length) ? s : { ...s, requires_approval: false })),
              }));
            }}
          >
            <option value="sequential">Sequential - a fixed chain, each step sees the prior step's answer</option>
            <option value="consensus">Consensus - every step but the last runs independently; the last synthesizes them all</option>
            <option value="peer_review">Peer review - exactly 2 steps: a drafter and a reviewer, looping until approved</option>
          </select>
          {input.topology === "consensus" && (
            <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0 0" }}>
              Every step above the last is a candidate (runs on <code>{"{{trigger_input}}"}</code> only); the last step is the synthesizer and can
              reference every candidate's answer via <code>{"{{candidate_outputs}}"}</code>.
            </p>
          )}
          {input.topology === "peer_review" && (
            <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0 0" }}>
              Exactly 2 steps: step 1 is the drafter (its <code>{"{{previous_output}}"}</code> is the reviewer's latest feedback, empty on round 1),
              step 2 is the reviewer (its <code>{"{{previous_output}}"}</code> is the latest draft). The reviewer approves by starting its answer with
              "APPROVED"; up to 3 rounds before giving up.
            </p>
          )}
        </div>
        <div className="field full">
          <label>Steps</label>
          {input.steps.length === 0 && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>No steps yet.</p>}
          {input.steps.map((step, i) => (
            <div key={i} style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 6, flexWrap: "wrap" }}>
              <span style={{ fontSize: 12, color: "var(--text-muted)", minWidth: 90 }}>{stepRoleLabel(input.topology, i, input.steps.length)}</span>
              <select value={step.agent_id} onChange={(e) => updateStep(i, { agent_id: e.target.value })}>
                {agents.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.icon} {a.name}
                  </option>
                ))}
              </select>
              <input
                style={{ flex: 1, minWidth: 180 }}
                value={step.input_template}
                onChange={(e) => updateStep(i, { input_template: e.target.value })}
                placeholder={i === 0 ? "{{trigger_input}}" : "{{previous_output}}"}
              />
              {stepSupportsApproval(input.topology, i, input.steps.length) && (
                <label style={{ fontSize: 12, display: "flex", alignItems: "center", gap: 4, whiteSpace: "nowrap" }} title="Pause the run here for an Administrator to approve or reject before continuing">
                  <input type="checkbox" checked={step.requires_approval} onChange={(e) => updateStep(i, { requires_approval: e.target.checked })} />
                  Needs approval
                </label>
              )}
              <button type="button" className="icon-btn" onClick={() => moveStep(i, -1)} disabled={i === 0}>
                ↑
              </button>
              <button type="button" className="icon-btn" onClick={() => moveStep(i, 1)} disabled={i === input.steps.length - 1}>
                ↓
              </button>
              <button type="button" className="icon-btn" onClick={() => removeStep(i)}>
                ✕
              </button>
            </div>
          ))}
          <button type="button" className="btn btn-secondary" onClick={addStep} disabled={agents.length === 0}>
            + Add step
          </button>
        </div>
        <div className="field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={pending || input.steps.length === 0}>
            {pending ? "Saving..." : initial ? "Save pipeline" : "Create pipeline"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
