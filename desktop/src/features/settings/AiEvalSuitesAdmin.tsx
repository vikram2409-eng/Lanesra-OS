import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { AiAgentTargetType, AiEvalCaseInput, AiEvalRun, AiEvalSuite, AiEvalSuiteInput } from "../../lib/types";

// AI & Agentic Layer, Phase 7d: a real Evaluation Harness - a named
// Suite of golden test Cases (an input plus a plain-English success
// criteria) run against one Agent or Pipeline, graded by an LLM-as-judge
// call rather than a brittle exact-match comparison an open-ended agent
// response could never satisfy.
function emptyInput(): AiEvalSuiteInput {
  return { name: "", description: "", target_type: "agent", target_id: "", cases: [] };
}

export function AiEvalSuitesAdmin() {
  const queryClient = useQueryClient();
  const suitesQuery = useQuery({ queryKey: ["aiEvalSuites"], queryFn: () => api.listAiEvalSuites() });
  const agentsQuery = useQuery({ queryKey: ["aiAgents", "all"], queryFn: () => api.listAiAgents(true) });
  const pipelinesQuery = useQuery({ queryKey: ["aiAgentPipelines", "all"], queryFn: () => api.listAiAgentPipelines(false) });
  const suites = suitesQuery.data ?? [];
  const agents = agentsQuery.data ?? [];
  const pipelines = pipelinesQuery.data ?? [];

  const [editing, setEditing] = useState<AiEvalSuite | null>(null);
  const [creating, setCreating] = useState(false);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["aiEvalSuites"] });
  }

  const create = useMutation({
    mutationFn: (input: AiEvalSuiteInput) => api.createAiEvalSuite(input),
    onSuccess: () => {
      setCreating(false);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this suite"),
  });
  const update = useMutation({
    mutationFn: ({ id, input }: { id: string; input: AiEvalSuiteInput }) => api.updateAiEvalSuite(id, input),
    onSuccess: () => {
      setEditing(null);
      setError(null);
      invalidate();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this suite"),
  });
  const remove = useMutation({
    mutationFn: (id: string) => api.deleteAiEvalSuite(id),
    onSuccess: invalidate,
  });

  function targetName(targetType: AiAgentTargetType, targetId: string): string {
    if (targetType === "agent") {
      const a = agents.find((x) => x.id === targetId);
      return a ? `${a.icon} ${a.name}` : "(deleted agent)";
    }
    const p = pipelines.find((x) => x.id === targetId);
    return p ? p.name : "(deleted pipeline)";
  }

  return (
    <div>
      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8, flexWrap: "wrap", gap: 8 }}>
          <div>
            <h3 style={{ margin: 0 }}>Evaluations</h3>
            <p style={{ color: "var(--text-muted)", fontSize: 13, margin: "4px 0 0" }}>
              A Suite is a set of golden test cases - an input plus a plain-English success criteria - run against one Agent or Pipeline. Each run
              grades every case with an LLM-as-judge call, since an open-ended agent response is never byte-for-byte reproducible.
            </p>
          </div>
          <button className="btn btn-primary" onClick={() => setCreating(true)} disabled={agents.length === 0 && pipelines.length === 0}>
            + New suite
          </button>
        </div>
        {error && <div className="error-banner">{error}</div>}
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Target</th>
                <th>Cases</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {suites.map((s) => (
                <tr key={s.id}>
                  <td>
                    <b>{s.name}</b>
                    {s.description && <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{s.description}</div>}
                  </td>
                  <td>{targetName(s.target_type, s.target_id)}</td>
                  <td>{s.cases.length}</td>
                  <td>
                    <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                      <button className="btn btn-secondary" onClick={() => setExpanded(expanded === s.id ? null : s.id)}>
                        {expanded === s.id ? "Close" : "Run / History"}
                      </button>
                      <button className="btn btn-secondary" onClick={() => setEditing(s)}>
                        Edit
                      </button>
                      <button className="btn btn-secondary" onClick={() => remove.mutate(s.id)}>
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {suites.length === 0 && <div className="empty-state">No eval suites yet - create one to test an agent or pipeline against golden cases.</div>}
        </div>
      </div>

      {expanded && suites.find((s) => s.id === expanded) && <SuiteDetail suite={suites.find((s) => s.id === expanded)!} />}

      {(creating || editing) && (
        <AiEvalSuiteForm
          initial={editing ?? undefined}
          agents={agents}
          pipelines={pipelines}
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

function SuiteDetail({ suite }: { suite: AiEvalSuite }) {
  const [result, setResult] = useState<AiEvalRun | null>(null);
  const [error, setError] = useState<string | null>(null);
  const runsQuery = useQuery({ queryKey: ["aiEvalRuns", suite.id], queryFn: () => api.listAiEvalRuns(suite.id, 10) });

  const run = useMutation({
    mutationFn: () => api.runAiEvalSuite(suite.id),
    onSuccess: (r) => {
      setResult(r);
      setError(null);
      runsQuery.refetch();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not run this suite"),
  });

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{suite.name}</h3>
      <button className="btn btn-primary" onClick={() => run.mutate()} disabled={run.isPending}>
        {run.isPending ? "Running..." : "Run suite"}
      </button>
      {error && <div className="error-banner">{error}</div>}
      {result && <RunResult run={result} />}

      <div style={{ marginTop: 12 }}>
        <b style={{ fontSize: 13 }}>Recent runs</b>
        {(runsQuery.data ?? []).length === 0 && <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No runs yet.</p>}
        {(runsQuery.data ?? []).map((r) => (
          <div key={r.id} style={{ fontSize: 12, display: "flex", gap: 8, padding: "3px 0", alignItems: "center" }}>
            <span className={`badge${r.failed_count === 0 ? " badge-success" : " badge-danger"}`}>
              {r.passed_count}/{r.passed_count + r.failed_count} passed
            </span>
            <span style={{ color: "var(--text-muted)" }}>{new Date(r.started_at).toLocaleString()}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function RunResult({ run }: { run: AiEvalRun }) {
  return (
    <div style={{ marginTop: 12, marginBottom: 12 }}>
      <p style={{ fontSize: 13 }}>
        <span className={`badge${run.failed_count === 0 ? " badge-success" : " badge-danger"}`}>
          {run.passed_count}/{run.passed_count + run.failed_count} passed
        </span>
      </p>
      {run.results.map((r) => (
        <div key={r.id} style={{ fontSize: 13, borderTop: "1px dashed var(--border, #ddd)", padding: "6px 0" }}>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <span className={`badge${r.passed ? " badge-success" : " badge-danger"}`}>{r.passed ? "PASS" : "FAIL"}</span>
            <b>{r.input_text}</b>
          </div>
          <div style={{ color: "var(--text-muted)" }}>criteria: {r.success_criteria}</div>
          {r.actual_output && <div>response: {r.actual_output}</div>}
          {r.judge_reasoning && <div style={{ color: "var(--text-muted)" }}>judge: {r.judge_reasoning}</div>}
          {r.error && <div style={{ color: "var(--large, #b23b3b)" }}>error: {r.error}</div>}
        </div>
      ))}
    </div>
  );
}

function AiEvalSuiteForm({
  initial,
  agents,
  pipelines,
  onCancel,
  onSubmit,
  pending,
}: {
  initial?: AiEvalSuite;
  agents: { id: string; icon: string; name: string }[];
  pipelines: { id: string; name: string }[];
  onCancel: () => void;
  onSubmit: (input: AiEvalSuiteInput) => void;
  pending: boolean;
}) {
  const [input, setInput] = useState<AiEvalSuiteInput>(
    initial
      ? {
          name: initial.name,
          description: initial.description,
          target_type: initial.target_type,
          target_id: initial.target_id,
          cases: initial.cases.map((c) => ({ input_text: c.input_text, success_criteria: c.success_criteria })),
        }
      : emptyInput(),
  );

  const targetOptions = input.target_type === "agent" ? agents.map((a) => ({ id: a.id, label: `${a.icon} ${a.name}` })) : pipelines.map((p) => ({ id: p.id, label: p.name }));

  function addCase() {
    setInput((prev) => ({ ...prev, cases: [...prev.cases, { input_text: "", success_criteria: "" }] }));
  }
  function updateCase(i: number, patch: Partial<AiEvalCaseInput>) {
    setInput((prev) => ({ ...prev, cases: prev.cases.map((c, idx) => (idx === i ? { ...c, ...patch } : c)) }));
  }
  function removeCase(i: number) {
    setInput((prev) => ({ ...prev, cases: prev.cases.filter((_, idx) => idx !== i) }));
  }

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h3 style={{ marginTop: 0 }}>{initial ? `Edit ${initial.name}` : "New suite"}</h3>
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
        <div className="field">
          <label>Target type</label>
          <select
            value={input.target_type}
            onChange={(e) => setInput({ ...input, target_type: e.target.value as AiAgentTargetType, target_id: "" })}
          >
            <option value="agent">AI Agent</option>
            <option value="pipeline">Pipeline</option>
          </select>
        </div>
        <div className="field">
          <label>Target</label>
          <select value={input.target_id} onChange={(e) => setInput({ ...input, target_id: e.target.value })} required>
            <option value="" disabled>
              Select one...
            </option>
            {targetOptions.map((t) => (
              <option key={t.id} value={t.id}>
                {t.label}
              </option>
            ))}
          </select>
        </div>
        <div className="field full">
          <label>Cases</label>
          {input.cases.length === 0 && <p style={{ fontSize: 13, color: "var(--text-muted)" }}>No cases yet.</p>}
          {input.cases.map((c, i) => (
            <div key={i} style={{ display: "flex", gap: 8, alignItems: "flex-start", marginBottom: 6, flexWrap: "wrap" }}>
              <span style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 8 }}>{i + 1}.</span>
              <input style={{ flex: 1, minWidth: 200 }} value={c.input_text} onChange={(e) => updateCase(i, { input_text: e.target.value })} placeholder="Input to send the target" />
              <input
                style={{ flex: 1, minWidth: 200 }}
                value={c.success_criteria}
                onChange={(e) => updateCase(i, { success_criteria: e.target.value })}
                placeholder="What a correct response looks like"
              />
              <button type="button" className="icon-btn" onClick={() => removeCase(i)}>
                ✕
              </button>
            </div>
          ))}
          <button type="button" className="btn btn-secondary" onClick={addCase}>
            + Add case
          </button>
        </div>
        <div className="field full" style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={pending || input.cases.length === 0 || !input.target_id}>
            {pending ? "Saving..." : initial ? "Save suite" : "Create suite"}
          </button>
          <button className="btn btn-secondary" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
