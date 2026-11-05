import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import type { Guardrail } from "../../lib/types";

/**
 * WP-0133: output guardrails as versioned policy objects (REQ-GOV-007).
 *
 * Lists the built-in library and any custom guardrails, lets an admin add a
 * custom one, and edit a guardrail's configuration (which writes a new
 * version) or turn it off. Four checker kinds: rule, pii, schema, judge.
 */
const CHECKERS = ["rule", "pii", "schema", "judge"] as const;

/**
 * WP-0128: where in the agent run loop a guardrail runs. `off` is not wired
 * in; the three loop hook points are a step boundary, a model call and a
 * tool call. A failing guardrail at a point blocks the run there.
 */
const HOOK_POINTS = ["off", "step", "model", "tool"] as const;
const HOOK_LABEL: Record<string, string> = {
  off: "Off (not in loop)",
  step: "Step boundary",
  model: "Model call",
  tool: "Tool call",
};

export const GUARDRAILS_FLAG = "guardrails.admin";

export function GuardrailsAdmin() {
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draftConfig, setDraftConfig] = useState("");
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const [newChecker, setNewChecker] = useState<(typeof CHECKERS)[number]>("rule");
  const [newConfig, setNewConfig] = useState('{"mode":"must_contain","text":""}');
  const [newHookPoint, setNewHookPoint] = useState<(typeof HOOK_POINTS)[number]>("off");

  const guardrails = useQuery({
    queryKey: ["guardrails"],
    queryFn: () => api.listGuardrails(),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["guardrails"] });
  const fail = (e: unknown, msg: string) =>
    setError(e instanceof ApiError ? e.message : msg);

  const saveConfig = useMutation({
    mutationFn: (g: Guardrail) => api.editGuardrail(g.id, { config_json: draftConfig }),
    onSuccess: () => {
      invalidate();
      setEditingId(null);
    },
    onError: (e) => fail(e, "Could not save the guardrail"),
  });

  const toggle = useMutation({
    mutationFn: (g: Guardrail) => api.editGuardrail(g.id, { enabled: !g.enabled }),
    onSuccess: invalidate,
    onError: (e) => fail(e, "Could not change the guardrail"),
  });

  const create = useMutation({
    mutationFn: () =>
      api.createGuardrail({
        name: newName,
        checker_type: newChecker,
        config_json: newConfig,
        hook_point: newHookPoint,
      }),
    onSuccess: () => {
      invalidate();
      setCreating(false);
      setNewName("");
      setNewConfig('{"mode":"must_contain","text":""}');
      setNewHookPoint("off");
    },
    onError: (e) => fail(e, "Could not create the guardrail"),
  });

  const setHook = useMutation({
    mutationFn: ({ g, hook_point }: { g: Guardrail; hook_point: string }) =>
      api.editGuardrail(g.id, { hook_point }),
    onSuccess: invalidate,
    onError: (e) => fail(e, "Could not change where the guardrail runs"),
  });

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Guardrails</h3>
        {!creating && (
          <button className="btn btn-primary" onClick={() => { setCreating(true); setError(null); }}>
            New guardrail
          </button>
        )}
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Reusable output checks: PII leak, required disclaimer, external domain allow-list, JSON
        schema conformance, and an LLM tone judge. Editing a guardrail's configuration saves a new
        version.
      </p>

      {error && <div className="error-banner">{error}</div>}

      {creating && (
        <form
          className="panel"
          style={{ marginBottom: 16 }}
          onSubmit={(e) => {
            e.preventDefault();
            create.mutate();
          }}
        >
          <div className="form-row">
            <label htmlFor="g-name">Name</label>
            <input id="g-name" value={newName} onChange={(e) => setNewName(e.target.value)} />
          </div>
          <div className="form-row">
            <label htmlFor="g-checker">Checker</label>
            <select
              id="g-checker"
              value={newChecker}
              onChange={(e) => setNewChecker(e.target.value as (typeof CHECKERS)[number])}
            >
              {CHECKERS.map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label htmlFor="g-hook">Runs at</label>
            <select
              id="g-hook"
              value={newHookPoint}
              onChange={(e) => setNewHookPoint(e.target.value as (typeof HOOK_POINTS)[number])}
            >
              {HOOK_POINTS.map((h) => (
                <option key={h} value={h}>
                  {HOOK_LABEL[h]}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label htmlFor="g-config">Configuration (JSON)</label>
            <textarea
              id="g-config"
              rows={3}
              value={newConfig}
              onChange={(e) => setNewConfig(e.target.value)}
            />
          </div>
          <div className="toolbar">
            <button type="submit" className="btn btn-primary" disabled={create.isPending}>
              {create.isPending ? "Creating..." : "Create"}
            </button>
            <button type="button" className="btn" onClick={() => setCreating(false)}>
              Cancel
            </button>
          </div>
        </form>
      )}

      {guardrails.isLoading && <p>Loading...</p>}
      {guardrails.data && (
        <table>
          <thead>
            <tr>
              <th style={{ textAlign: "left" }}>Name</th>
              <th style={{ textAlign: "left" }}>Checker</th>
              <th style={{ textAlign: "left" }}>Runs at</th>
              <th style={{ textAlign: "left" }}>Version</th>
              <th style={{ textAlign: "left" }}>Enabled</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {guardrails.data.map((g) => (
              <tr key={g.id}>
                <td>
                  {g.name}
                  {g.is_builtin && <span className="badge" style={{ marginLeft: 8 }}>Built-in</span>}
                  {editingId === g.id && (
                    <div style={{ marginTop: 8 }}>
                      <textarea
                        aria-label={`Configuration for ${g.name}`}
                        rows={3}
                        style={{ width: "100%" }}
                        value={draftConfig}
                        onChange={(e) => setDraftConfig(e.target.value)}
                      />
                      <div className="toolbar" style={{ marginTop: 6 }}>
                        <button
                          className="btn btn-primary"
                          disabled={saveConfig.isPending}
                          onClick={() => saveConfig.mutate(g)}
                        >
                          Save new version
                        </button>
                        <button className="btn" onClick={() => setEditingId(null)}>
                          Cancel
                        </button>
                      </div>
                    </div>
                  )}
                </td>
                <td>{g.checker_type}</td>
                <td>
                  <select
                    aria-label={`Where ${g.name} runs`}
                    value={g.hook_point}
                    disabled={setHook.isPending}
                    onChange={(e) => {
                      setError(null);
                      setHook.mutate({ g, hook_point: e.target.value });
                    }}
                  >
                    {HOOK_POINTS.map((h) => (
                      <option key={h} value={h}>
                        {HOOK_LABEL[h]}
                      </option>
                    ))}
                  </select>
                </td>
                <td>v{g.current_version}</td>
                <td>{g.enabled ? "Yes" : "No"}</td>
                <td style={{ textAlign: "right", whiteSpace: "nowrap" }}>
                  {editingId !== g.id && (
                    <button
                      className="btn"
                      onClick={() => {
                        setEditingId(g.id);
                        setDraftConfig(g.config_json);
                        setError(null);
                      }}
                    >
                      Edit config
                    </button>
                  )}
                  <button
                    className="btn"
                    style={{ marginLeft: 6 }}
                    disabled={toggle.isPending}
                    onClick={() => toggle.mutate(g)}
                  >
                    {g.enabled ? "Disable" : "Enable"}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
