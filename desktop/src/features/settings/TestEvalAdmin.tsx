import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { CUSTOM_FIELD_ENTITY_TYPES, entityTypeLabel, type TestCaseDefinition, type TestCaseDefinitionInput, type TestRun, type TestType } from "../../lib/types";

const TEST_TYPES: { value: TestType; label: string; placeholder: string }[] = [
  {
    value: "business_rule",
    label: "Business Rule",
    placeholder: '{\n  "ctx": {"field_key": "value"},\n  "expect_field_effects": {"field_key": "hide"},\n  "expect_blocked": false,\n  "expect_errors": []\n}',
  },
  { value: "workflow", label: "Workflow", placeholder: '{\n  "ctx": {"field_key": "value"},\n  "expect_matched_workflow_names": ["Notify owner"]\n}' },
  {
    value: "access_security",
    label: "Access / Security",
    placeholder: '{\n  "actor_user_id": "...",\n  "capability": "update",\n  "record_id": null,\n  "expect_allowed": true\n}',
  },
  {
    value: "screen_visibility",
    label: "Screen Visibility",
    placeholder: '{\n  "actor_user_id": "...",\n  "expect_visible_fields": ["name"],\n  "expect_hidden_fields": ["internal_notes"]\n}',
  },
  {
    value: "integration_mapping",
    label: "Integration Mapping",
    placeholder: '{\n  "source_row": {"Column A": "raw value"},\n  "expect_target_fields": {"target_field": "expected value"}\n}',
  },
  { value: "agent_eval", label: "Agent Evaluation", placeholder: '{\n  "input_text": "...",\n  "success_criteria": "..."\n}' },
  { value: "agent_team_eval", label: "Agent Team Evaluation", placeholder: '{\n  "input_text": "...",\n  "success_criteria": "..."\n}' },
];

const TEST_TYPE_LABELS: Record<TestType, string> = Object.fromEntries(TEST_TYPES.map((t) => [t.value, t.label])) as Record<TestType, string>;

const ENTITY_SCOPED_TYPES: TestType[] = ["business_rule", "workflow", "access_security", "screen_visibility"];

function emptyInput(): TestCaseDefinitionInput {
  return { name: "", description: null, test_type: "business_rule", target_id: "", dataset_json: "{}" };
}

/**
 * Next-Gen program, Domain A (Intelligence Foundation), FND-03: the
 * Unified Test & Evaluation Framework. One framework for mixed
 * deterministic (Business Rule/Workflow/Access-Security/Screen
 * Visibility/Integration Mapping) and AI (Agent/Agent Team) test cases -
 * every executor wraps a dry-run/read-only function this platform
 * already has (see `test_eval_service`'s own doc comment, Rust), this
 * screen just defines cases and runs them together as one Test Run with
 * one readiness verdict. A Solution's own "Validate" action (Deployment
 * Management) runs exactly the cases curated into it the same way.
 */
export function TestEvalAdmin() {
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [lastRun, setLastRun] = useState<TestRun | null>(null);
  const [error, setError] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const cases = useQuery({ queryKey: ["testCaseDefinitions", "all"], queryFn: () => api.listTestCaseDefinitions(false) });
  const runs = useQuery({ queryKey: ["testRuns"], queryFn: () => api.listTestRuns(10) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["testCaseDefinitions"] });
  }

  const editing = cases.data?.find((c) => c.id === editingId) ?? null;
  const deactivate = useMutation({ mutationFn: (id: string) => api.deactivateTestCaseDefinition(id), onSuccess: invalidate });
  const remove = useMutation({ mutationFn: (id: string) => api.deleteTestCaseDefinition(id), onSuccess: invalidate });
  const runSelected = useMutation({
    mutationFn: () => api.runTests([...selected]),
    onSuccess: (run) => {
      setLastRun(run);
      setError(null);
      queryClient.invalidateQueries({ queryKey: ["testRuns"] });
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not run these test cases"),
  });

  function toggle(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Test & Evaluation Framework</h3>
        <div className="actions">
          <button className="btn btn-primary" onClick={() => runSelected.mutate()} disabled={selected.size === 0 || runSelected.isPending}>
            Run selected ({selected.size})
          </button>
          <button className="btn btn-primary" onClick={() => { setCreating((v) => !v); setEditingId(null); }}>
            + New test case
          </button>
        </div>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        One execution framework for deterministic tests and AI evaluations: pick any mix of cases below and run them together as
        one Test Run with one pass/fail readiness result - the same thing a Solution's own "Validate" action runs automatically
        during deployment. No case re-invents its own checking logic; each wraps a dry-run tool this platform already has (Test
        Rules, Test Workflows, the Access Inspector, the resolved screen layout, a Mapping's own field transform, or a real
        agent/agent-team run graded by the Evaluation Harness's judge call).
      </p>
      {error && <div className="error-banner">{error}</div>}

      {creating && <TestCaseForm onDone={() => { invalidate(); setCreating(false); }} onCancel={() => setCreating(false)} />}
      {editing && <TestCaseForm testCase={editing} onDone={() => { invalidate(); setEditingId(null); }} onCancel={() => setEditingId(null)} />}

      <div className="table-wrap">
        <table className="table">
          <thead>
            <tr>
              <th></th>
              <th>Name</th>
              <th>Type</th>
              <th>Target</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {(cases.data ?? []).map((c) => (
              <tr key={c.id}>
                <td><input type="checkbox" checked={selected.has(c.id)} onChange={() => toggle(c.id)} /></td>
                <td><b>{c.name}</b>{c.description && <><br /><small className="muted">{c.description}</small></>}</td>
                <td>{TEST_TYPE_LABELS[c.test_type]}</td>
                <td><code>{c.target_id}</code></td>
                <td>{c.is_active ? <span className="badge badge-success">Active</span> : <span className="badge">Inactive</span>}</td>
                <td>
                  <div className="actions">
                    <button className="icon-btn" onClick={() => { setEditingId(c.id); setCreating(false); }}>Edit</button>
                    {c.is_active && <button className="icon-btn" onClick={() => deactivate.mutate(c.id)}>Deactivate</button>}
                    <button className="icon-btn" onClick={() => { if (confirm(`Delete test case '${c.name}'?`)) remove.mutate(c.id); }}>Delete</button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {!cases.data?.length && <div className="empty">No test cases yet</div>}
      </div>

      {lastRun && <RunResultPanel run={lastRun} />}

      <div className="panel" style={{ marginTop: 16 }}>
        <h4 style={{ marginTop: 0 }}>Recent runs</h4>
        {(runs.data ?? []).map((r) => (
          <div key={r.id} style={{ marginBottom: 8 }}>
            <button className="icon-btn" onClick={() => setLastRun(r)}>
              {new Date(r.started_at).toLocaleString()} - {r.passed_count} passed / {r.failed_count} failed
              {r.triggered_by === "deployment_validation" ? " (deployment validation)" : ""}
            </button>
          </div>
        ))}
        {!runs.data?.length && <p className="muted" style={{ fontSize: 13 }}>No runs yet.</p>}
      </div>
    </div>
  );
}

function RunResultPanel({ run }: { run: TestRun }) {
  const ready = run.status === "completed" && run.failed_count === 0;
  return (
    <div className="panel" style={{ marginTop: 16 }}>
      <h4 style={{ marginTop: 0 }}>
        Run result: {ready ? "Ready" : "Not ready"}{" "}
        <span className={run.failed_count === 0 ? "badge badge-success" : "badge"}>
          {run.passed_count} passed / {run.failed_count} failed
        </span>
      </h4>
      <div className="table-wrap">
        <table className="table">
          <thead>
            <tr>
              <th>Case</th>
              <th>Type</th>
              <th>Result</th>
              <th>Runtime</th>
              <th>Policy</th>
              <th>Trace</th>
            </tr>
          </thead>
          <tbody>
            {run.results.map((r) => (
              <tr key={r.id}>
                <td>{r.test_case_name}</td>
                <td>{TEST_TYPE_LABELS[r.test_type]}</td>
                <td>{r.passed ? <span className="badge badge-success">Pass</span> : <span className="badge">Fail</span>}</td>
                <td>{r.runtime_ms}ms</td>
                <td>{r.policy_outcome}</td>
                <td style={{ maxWidth: 320 }}><small>{r.trace_text ?? "—"}</small></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function TestCaseForm({ testCase, onDone, onCancel }: { testCase?: TestCaseDefinition; onDone: () => void; onCancel: () => void }) {
  const [input, setInput] = useState<TestCaseDefinitionInput>(
    testCase
      ? {
          name: testCase.name, description: testCase.description, test_type: testCase.test_type, target_id: testCase.target_id,
          dataset_json: testCase.dataset_json, cost_threshold_usd: testCase.cost_threshold_usd, latency_threshold_ms: testCase.latency_threshold_ms,
        }
      : emptyInput(),
  );
  const [error, setError] = useState<string | null>(null);

  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const mappings = useQuery({ queryKey: ["mappings"], queryFn: () => api.listMappings(), enabled: input.test_type === "integration_mapping" });
  const agents = useQuery({ queryKey: ["aiAgents", "active"], queryFn: () => api.listAiAgents(true), enabled: input.test_type === "agent_eval" });
  const graphs = useQuery({ queryKey: ["executionGraphs"], queryFn: () => api.listExecutionGraphs(), enabled: input.test_type === "agent_team_eval" });

  const entityTypes = [...CUSTOM_FIELD_ENTITY_TYPES, ...(customObjects.data?.map((o) => o.key) ?? [])];
  const labelFor = (t: string) => customObjects.data?.find((o) => o.key === t)?.plural_label ?? entityTypeLabel(t);
  const selectedType = TEST_TYPES.find((t) => t.value === input.test_type)!;

  const save = useMutation({
    mutationFn: () => (testCase ? api.updateTestCaseDefinition(testCase.id, input) : api.createTestCaseDefinition(input)),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this test case"),
  });

  return (
    <div className="panel" style={{ marginBottom: 16 }}>
      {error && <div className="error-banner">{error}</div>}
      <div className="form-grid">
        <div className="form-field">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} />
        </div>
        <div className="form-field">
          <label>Test type</label>
          <select value={input.test_type} onChange={(e) => setInput({ ...input, test_type: e.target.value as TestType, target_id: "" })}>
            {TEST_TYPES.map((t) => <option key={t.value} value={t.value}>{t.label}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Target</label>
          {ENTITY_SCOPED_TYPES.includes(input.test_type) && (
            <select value={input.target_id} onChange={(e) => setInput({ ...input, target_id: e.target.value })}>
              <option value="">Select an object...</option>
              {entityTypes.map((t) => <option key={t} value={t}>{labelFor(t)}</option>)}
            </select>
          )}
          {input.test_type === "integration_mapping" && (
            <select value={input.target_id} onChange={(e) => setInput({ ...input, target_id: e.target.value })}>
              <option value="">Select a mapping...</option>
              {(mappings.data ?? []).map((m) => <option key={m.id} value={m.id}>{m.name}</option>)}
            </select>
          )}
          {input.test_type === "agent_eval" && (
            <select value={input.target_id} onChange={(e) => setInput({ ...input, target_id: e.target.value })}>
              <option value="">Select an agent...</option>
              {(agents.data ?? []).map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
            </select>
          )}
          {input.test_type === "agent_team_eval" && (
            <select value={input.target_id} onChange={(e) => setInput({ ...input, target_id: e.target.value })}>
              <option value="">Select an agent team...</option>
              {(graphs.data ?? []).map((g) => <option key={g.id} value={g.id}>{g.name}</option>)}
            </select>
          )}
        </div>
        <div className="form-field">
          <label>Cost threshold (USD, optional)</label>
          <input type="number" step="0.01" value={input.cost_threshold_usd ?? ""} onChange={(e) => setInput({ ...input, cost_threshold_usd: e.target.value ? Number(e.target.value) : null })} />
        </div>
        <div className="form-field">
          <label>Latency threshold (ms, optional)</label>
          <input type="number" value={input.latency_threshold_ms ?? ""} onChange={(e) => setInput({ ...input, latency_threshold_ms: e.target.value ? Number(e.target.value) : null })} />
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Description</label>
          <textarea rows={2} value={input.description ?? ""} onChange={(e) => setInput({ ...input, description: e.target.value || null })} />
        </div>
        <div className="form-field" style={{ gridColumn: "1 / -1" }}>
          <label>Dataset (JSON)</label>
          <textarea
            rows={6}
            placeholder={selectedType.placeholder}
            value={input.dataset_json ?? "{}"}
            onChange={(e) => setInput({ ...input, dataset_json: e.target.value })}
            style={{ fontFamily: "monospace", fontSize: 12 }}
          />
        </div>
      </div>
      <div className="actions">
        <button className="btn btn-primary" onClick={() => save.mutate()} disabled={save.isPending}>{testCase ? "Save" : "Create"}</button>
        <button className="btn" onClick={onCancel}>Cancel</button>
      </div>
    </div>
  );
}
