/**
 * Step panel (WP-0718 §1–§6, REQ-LANE-024): one panel per step kind.
 *
 * Form / record / check / approval / agent fields, writes, check
 * rules, branches, repeats and the child_lane editor. The branch
 * editor never lets the author type a stage or step id: every target
 * is picked from the steps that exist, and an empty arrow slot is a
 * visible gap (P-09). The approval view shows the writes and what
 * happens on each branch; a missing write is the P-05 gap, and a
 * provisional write is struck through with the P-12 reason.
 *
 * Builder-gaps batch: kind→check initialises the demo's default check
 * object; the role is a picker fed by the workspace role list (free
 * text + warning when the list is unavailable); Add field can never
 * create a P-11 trap (it binds to a declared field, or declares the
 * new one in the same action); writes have the demo's +Input editor
 * (editable name + value pairs); branch targets exclude the step
 * itself and a new branch defaults to the next step, else end held;
 * repeating steps get the Q-33 assignment control and the Q-34
 * skip-condition builder (the runner honors both once WP-0710 lands);
 * form fields get the Q-35 "resolve from latest record" control; steps
 * get a `replaces` editor fed by the object's workflow rules.
 */
import { useEffect, useState } from "react";

import { api } from "../../lib/api";
import { ConditionBuilder } from "./ConditionBuilder";
import { DEFAULT_CHECK, defaultBranchFor, flattenSteps } from "./builderUtils";
import type {
  AccessRole,
  BuilderActionType,
  LaneBranch,
  LaneDefinition,
  LaneFieldResolve,
  LaneFormField,
  LaneStep,
  WorkflowDefinition,
} from "../../lib/types";

const STEP_KINDS = ["form", "record", "check", "approval", "agent"];

export function branchTargetKey(branch: LaneBranch): string {
  if (branch.end) return `end:${branch.end.outcome}`;
  if (branch.target) return `${branch.target.stage}/${branch.target.step}`;
  return "";
}

export function StepPanel({
  step,
  stageId,
  definition,
  actionTypes,
  adoptedActionTypes,
  checkLastWarning,
  onChange,
  onJumpToStep,
  onMoveStep,
  onMoveCheckToEnd,
  onDeclareRunDataField,
  onDeleteStep,
  onActionTypeAdopted,
}: {
  step: LaneStep;
  stageId: string;
  definition: LaneDefinition;
  actionTypes: BuilderActionType[];
  adoptedActionTypes: string[];
  checkLastWarning: string | null;
  onChange: (step: LaneStep) => void;
  onJumpToStep: (stepId: string) => void;
  onMoveStep: (direction: -1 | 1) => void;
  onMoveCheckToEnd: () => void;
  onDeclareRunDataField?: (stepId: string, key: string, field: LaneFormField) => void;
  onDeleteStep?: (stepId: string) => void;
  onActionTypeAdopted?: (name: string) => void;
}) {
  const flat = flattenSteps(definition);
  const stage = definition.stages.find((s) => s.id === stageId);
  const stepIndex = stage?.steps.findIndex((s) => s.id === step.id) ?? -1;
  const declaredKeys = (definition.run_data?.fields ?? []).map((f) => f.key);
  const [roles, setRoles] = useState<AccessRole[] | null>(null);
  const [rolesLoaded, setRolesLoaded] = useState(false);
  const [laneKeys, setLaneKeys] = useState<string[] | null>(null);
  const [workflowRules, setWorkflowRules] = useState<WorkflowDefinition[] | null>(null);
  const [adopting, setAdopting] = useState<string | null>(null);
  const [adoptError, setAdoptError] = useState<string | null>(null);

  // The role picker, the child-lane picker and the replaces workflow
  // list are all fed by the workspace; each falls back to free text
  // when its source is unavailable. Guarded with `?.` — the mocked
  // api in unit tests may not provide these wrappers.
  useEffect(() => {
    let cancelled = false;
    const pending = api.listAccessRoles?.();
    if (pending) {
      pending
        .then((r) => {
          if (!cancelled) {
            setRoles(r);
            setRolesLoaded(true);
          }
        })
        .catch(() => {
          if (!cancelled) setRolesLoaded(true);
        });
    } else {
      setRolesLoaded(true);
    }
    return () => {
      cancelled = true;
    };
  }, []);
  useEffect(() => {
    let cancelled = false;
    const pending = api.listLaneBlueprints?.();
    if (pending) {
      pending
        .then((lanes) => {
          if (!cancelled) setLaneKeys(lanes.map((l) => l.lane_key));
        })
        .catch(() => {});
    }
    return () => {
      cancelled = true;
    };
  }, []);
  useEffect(() => {
    if (!definition.object_key) return;
    let cancelled = false;
    const pending = api.listWorkflowRules?.(definition.object_key);
    if (pending) {
      pending
        .then((rules) => {
          if (!cancelled) setWorkflowRules(rules);
        })
        .catch(() => {});
    }
    return () => {
      cancelled = true;
    };
  }, [definition.object_key]);

  const isProvisional = (actionType: string) =>
    actionTypes.find((a) => a.name === actionType)?.provisional ??
    !adoptedActionTypes.includes(actionType);

  function set(patch: Partial<LaneStep>) {
    onChange({ ...step, ...patch });
  }

  /// Write a field the core types carry as `unknown` (`skip_when` is
  /// a ConditionOrGroup, untagged): the cast keeps the write compiling
  /// without narrowing the core type here.
  function setExtended(patch: Record<string, unknown>) {
    onChange({ ...step, ...patch } as unknown as LaneStep);
  }

  function targetLabel(key: string): string {
    if (key.startsWith("end:")) return `ends (${key.slice(4)})`;
    const [stageIdPart, stepIdPart] = key.split("/");
    const found = flat.find((f) => f.stage.id === stageIdPart && f.step.id === stepIdPart);
    return found ? `${found.stage.name} · ${found.step.name}` : key;
  }

  /// Kind→check initialises the demo's default check object (scope /
  /// condition / message) so the check fieldset renders; an existing
  /// check step is never clobbered.
  function changeKind(kind: string) {
    if (kind === "check" && !step.check) {
      set({
        kind,
        check: { ...DEFAULT_CHECK, condition: JSON.parse(JSON.stringify(DEFAULT_CHECK.condition)) },
      });
    } else {
      set({ kind });
    }
  }

  /// Add field can never create a P-11 trap: the new field binds to
  /// the first declared run-data field, or — when nothing is declared
  /// yet — declares its own key in the same action. The declaration
  /// and the field add go through one LaneBuilder update (a single
  /// setDef), so the declaration cannot be clobbered by a stale
  /// second update.
  function addField() {
    const existingKeys = new Set((step.fields ?? []).map((f) => f.key));
    let key = "new_field";
    let n = 2;
    while (existingKeys.has(key)) key = `new_field_${n++}`;
    const field = {
      key,
      label: "New field",
      type: "string",
      binds_to: declaredKeys.length > 0 ? `run_data.${declaredKeys[0]}` : `run_data.${key}`,
      required: false,
    };
    if (declaredKeys.length > 0) {
      set({ fields: [...(step.fields ?? []), field] });
    } else {
      onDeclareRunDataField?.(step.id, key, field);
    }
  }

  function addWriteInput(writeIndex: number) {
    const inputs = (step.writes ?? [])[writeIndex]?.inputs ?? {};
    let key = "new_input";
    let n = 2;
    while (key in inputs) key = `new_input_${n++}`;
    set({
      writes: (step.writes ?? []).map((x, j) =>
        j === writeIndex ? { ...x, inputs: { ...x.inputs, [key]: "" } } : x,
      ),
    });
  }

  function renameWriteInput(writeIndex: number, oldKey: string, newKey: string) {
    set({
      writes: (step.writes ?? []).map((x, j) => {
        if (j !== writeIndex) return x;
        const inputs: Record<string, unknown> = {};
        for (const [k, v] of Object.entries(x.inputs)) inputs[k === oldKey ? newKey : k] = v;
        return { ...x, inputs };
      }),
    });
  }

  function removeWriteInput(writeIndex: number, key: string) {
    set({
      writes: (step.writes ?? []).map((x, j) => {
        if (j !== writeIndex) return x;
        const inputs = { ...x.inputs };
        delete inputs[key];
        return { ...x, inputs };
      }),
    });
  }

  const expectedBranches =
    step.kind === "check" ? ["pass", "fail"] : step.kind === "approval" ? ["approved", "rejected"] : [];
  const branches = step.branches ?? [];
  /// Branch targets exclude the step itself: a self-loop publishes
  /// today and retries forever at runtime (gap 7).
  const branchTargets = flat.filter((f) => f.step.id !== step.id);
  const skipWhen = step.skip_when ?? null;
  const repeatAssign = step.repeat_over?.assign;

  /// Adopt a provisional Action Type into the catalog (WP-0705): the
  /// write stops being provisional and P-12 clears. The parent
  /// refreshes the picker catalog after the adoption lands.
  async function adoptActionType(name: string) {
    if (!definition.object_key) return;
    setAdopting(name);
    setAdoptError(null);
    try {
      await api.adoptBuilderActionType?.(name, definition.object_key);
      onActionTypeAdopted?.(name);
    } catch (err) {
      setAdoptError(
        `Could not adopt '${name}': ${err instanceof Error ? err.message : "adoption failed."}`,
      );
    } finally {
      setAdopting(null);
    }
  }

  return (
    <section className="card" role="region" aria-label="Step panel">
      <h3 style={{ marginTop: 0 }}>
        Step panel — {step.name}
      </h3>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 12 }}>
        <button type="button" className="btn" onClick={() => onMoveStep(-1)} disabled={stepIndex <= 0}>
          Move step up
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => onMoveStep(1)}
          disabled={stage ? stepIndex >= stage.steps.length - 1 : true}
        >
          Move step down
        </button>
        {onDeleteStep && (
          <button
            type="button"
            className="btn"
            onClick={() => {
              if (
                window.confirm(
                  `Delete step '${step.name}'? Branches pointing at it will also be removed.`,
                )
              ) {
                onDeleteStep(step.id);
              }
            }}
          >
            Delete step
          </button>
        )}
      </div>

      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-name">Step name</label>
        <input id="builder-step-name" type="text" value={step.name} onChange={(e) => set({ name: e.target.value })} />
      </div>
      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-kind">Kind</label>{" "}
        <select id="builder-step-kind" value={step.kind} onChange={(e) => changeKind(e.target.value)}>
          {STEP_KINDS.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>
      </div>
      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-role">Role</label>{" "}
        {rolesLoaded && roles && roles.length > 0 ? (
          <select
            id="builder-step-role"
            value={step.role ?? ""}
            onChange={(e) => set({ role: e.target.value || null })}
          >
            <option value="">Anyone (no role)</option>
            {roles.map((r) => (
              <option key={r.id} value={r.id}>
                {r.name} ({r.id})
              </option>
            ))}
            {step.role && !roles.some((r) => r.id === step.role) && (
              <option value={step.role}>{step.role}</option>
            )}
          </select>
        ) : (
          <>
            <input
              id="builder-step-role"
              type="text"
              value={step.role ?? ""}
              placeholder="role, or blank for anyone"
              onChange={(e) => set({ role: e.target.value || null })}
            />
            {rolesLoaded && (
              <p role="note" style={{ color: "var(--warning, #8a6d00)", margin: "4px 0 0" }}>
                Role list unavailable — type the role id exactly; an unknown role cannot be checked
                here and the run would wait for a role nobody holds.
              </p>
            )}
          </>
        )}
      </div>
      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-description">Description</label>
        <textarea
          id="builder-step-description"
          value={step.description ?? ""}
          onChange={(e) => set({ description: e.target.value || null })}
        />
      </div>
      <label style={{ display: "block", marginBottom: 12 }}>
        <input
          type="checkbox"
          checked={step.required !== false}
          onChange={(e) => set({ required: e.target.checked })}
        />{" "}
        Required
      </label>

      {checkLastWarning && (
        <p role="alert" style={{ color: "var(--warning, #8a6d00)" }}>
          Check-last warning: {checkLastWarning}.{" "}
          <button type="button" className="btn" onClick={onMoveCheckToEnd}>
            Move check to end of stage
          </button>
        </p>
      )}

      {step.kind === "form" && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Fields</legend>
          {(step.fields ?? []).map((f, i) => {
            const resolve = f.resolve ?? null;
            const whereEntries = Object.entries(resolve?.where ?? {});
            const whereKey = whereEntries.length > 0 ? String(whereEntries[0][0]) : "";
            const whereValue =
              whereEntries.length > 0
                ? typeof whereEntries[0][1] === "string"
                  ? (whereEntries[0][1] as string)
                  : JSON.stringify(whereEntries[0][1])
                : "";
            function updateField(patch: Partial<LaneFormField>) {
              set({ fields: (step.fields ?? []).map((x, j) => (j === i ? { ...x, ...patch } : x)) });
            }
            function updateResolve(patch: Partial<LaneFieldResolve>) {
              const next: LaneFieldResolve = { ...(resolve as LaneFieldResolve), ...patch };
              updateField({ resolve: next });
            }
            return (
              <div key={i} style={{ marginBottom: 12, borderTop: "1px solid var(--border, #ccc)", paddingTop: 8 }}>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 8, alignItems: "center" }}>
                  <input
                    aria-label={`Field ${i + 1} key`}
                    value={f.key}
                    onChange={(e) => updateField({ key: e.target.value })}
                  />
                  <input
                    aria-label={`Field ${i + 1} label`}
                    value={f.label}
                    onChange={(e) => updateField({ label: e.target.value })}
                  />
                  <input
                    aria-label={`Field ${i + 1} binds to`}
                    value={f.binds_to}
                    onChange={(e) => updateField({ binds_to: e.target.value })}
                  />
                  {declaredKeys.length > 0 && (
                    <select
                      aria-label={`Field ${i + 1} bind to declared field`}
                      value={declaredKeys.some((k) => f.binds_to === `run_data.${k}`) ? f.binds_to : ""}
                      onChange={(e) => {
                        if (e.target.value) updateField({ binds_to: e.target.value });
                      }}
                    >
                      <option value="">Bind to declared field…</option>
                      {declaredKeys.map((k) => (
                        <option key={k} value={`run_data.${k}`}>
                          run_data.{k}
                        </option>
                      ))}
                    </select>
                  )}
                  <button
                    type="button"
                    className="btn"
                    onClick={() => set({ fields: (step.fields ?? []).filter((_, j) => j !== i) })}
                  >
                    Remove field
                  </button>
                </div>
                {resolve ? (
                  <div style={{ marginLeft: 8 }}>
                    <p style={{ color: "var(--text-muted)", margin: "0 0 4px" }}>
                      Resolves from the latest record — runtime support for resolve is a follow-up;
                      the runner does not resolve it yet.
                    </p>
                    <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
                      <label>
                        Object{" "}
                        <input
                          aria-label={`Field ${i + 1} resolve object`}
                          value={resolve.object}
                          onChange={(e) => updateResolve({ object: e.target.value })}
                        />
                      </label>
                      <label>
                        Where field{" "}
                        <input
                          aria-label={`Field ${i + 1} resolve where field`}
                          value={whereKey}
                          placeholder="product_id"
                          onChange={(e) =>
                            updateResolve({
                              where: e.target.value ? { [e.target.value]: whereValue } : {},
                            })
                          }
                        />
                      </label>
                      <label>
                        Where value{" "}
                        <input
                          aria-label={`Field ${i + 1} resolve where value`}
                          value={whereValue}
                          placeholder="$record.product_id"
                          onChange={(e) =>
                            updateResolve({ where: whereKey ? { [whereKey]: e.target.value } : {} })
                          }
                        />
                      </label>
                      <label>
                        Order by{" "}
                        <input
                          aria-label={`Field ${i + 1} resolve order by`}
                          value={resolve.order_by ?? ""}
                          onChange={(e) => updateResolve({ order_by: e.target.value })}
                        />
                      </label>
                      <label>
                        Direction{" "}
                        <select
                          aria-label={`Field ${i + 1} resolve direction`}
                          value={resolve.direction ?? "desc"}
                          onChange={(e) =>
                            updateResolve({ direction: e.target.value as "asc" | "desc" })
                          }
                        >
                          <option value="desc">desc</option>
                          <option value="asc">asc</option>
                        </select>
                      </label>
                      <label>
                        Take{" "}
                        <input
                          type="number"
                          aria-label={`Field ${i + 1} resolve take`}
                          value={resolve.take ?? 1}
                          onChange={(e) => updateResolve({ take: Number(e.target.value) })}
                        />
                      </label>
                      <label>
                        Field{" "}
                        <input
                          aria-label={`Field ${i + 1} resolve field`}
                          value={resolve.field}
                          onChange={(e) => updateResolve({ field: e.target.value })}
                        />
                      </label>
                      <button
                        type="button"
                        className="btn"
                        onClick={() => updateField({ resolve: null })}
                      >
                        Remove resolve
                      </button>
                    </div>
                  </div>
                ) : (
                  <button
                    type="button"
                    className="btn"
                    onClick={() =>
                      updateField({
                        resolve: {
                          kind: "latest_record",
                          object: definition.object_key,
                          where: {},
                          order_by: "",
                          direction: "desc",
                          take: 1,
                          field: "",
                        },
                      })
                    }
                  >
                    Resolve from latest record
                  </button>
                )}
              </div>
            );
          })}
          <button type="button" className="btn" onClick={addField}>
            Add field
          </button>
          {declaredKeys.length === 0 && (
            <p style={{ color: "var(--text-muted)" }}>
              No run-data fields declared yet — Add field declares one for you (see Run-data
              plumbing below).
            </p>
          )}
        </fieldset>
      )}

      {(step.kind === "record" || step.kind === "approval") && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Writes</legend>
          {step.kind === "approval" && (step.writes ?? []).length === 0 && (
            <p role="alert" style={{ color: "var(--danger)" }}>
              P-05: this approval has no writes — an approval with no writes cannot publish.
            </p>
          )}
          {(step.writes ?? []).map((w, i) => (
            <div key={i} style={{ marginBottom: 8 }}>
              <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                {isProvisional(w.action_type) ? (
                  <s>{w.action_type}</s>
                ) : (
                  <code>{w.action_type}</code>
                )}
                <select
                  aria-label={`Write ${i + 1} action type`}
                  value={w.action_type}
                  onChange={(e) =>
                    set({ writes: (step.writes ?? []).map((x, j) => (j === i ? { ...x, action_type: e.target.value } : x)) })
                  }
                >
                  {actionTypes.map((a) => (
                    <option key={a.name} value={a.name}>
                      {a.name}
                      {a.provisional ? " (provisional)" : ""}
                    </option>
                  ))}
                  {!actionTypes.some((a) => a.name === w.action_type) && (
                    <option value={w.action_type}>{w.action_type} (provisional)</option>
                  )}
                </select>
                <button
                  type="button"
                  className="btn"
                  onClick={() => set({ writes: (step.writes ?? []).filter((_, j) => j !== i) })}
                >
                  Remove write
                </button>
              </div>
              {isProvisional(w.action_type) && (
                <p style={{ color: "var(--text-muted)", margin: "4px 0 0" }}>
                  P-12: provisional Action Type — cannot publish until the catalog adopts it.{" "}
                  <button
                    type="button"
                    className="btn"
                    disabled={adopting === w.action_type}
                    onClick={() => adoptActionType(w.action_type)}
                  >
                    {adopting === w.action_type ? "Adopting…" : `Adopt ${w.action_type}`}
                  </button>
                </p>
              )}
              {adoptError && (
                <p role="alert" style={{ color: "var(--danger)" }}>
                  {adoptError}
                </p>
              )}
              {Object.entries(w.inputs).map(([k, v]) => (
                <div key={k} style={{ display: "flex", gap: 8, marginTop: 4, alignItems: "center" }}>
                  <input
                    aria-label={`Write ${i + 1} input name`}
                    value={k}
                    placeholder="input name"
                    onChange={(e) => renameWriteInput(i, k, e.target.value)}
                  />
                  <input
                    aria-label={`Write ${i + 1} input ${k}`}
                    value={typeof v === "string" ? v : JSON.stringify(v)}
                    onChange={(e) =>
                      set({
                        writes: (step.writes ?? []).map((x, j) =>
                          j === i ? { ...x, inputs: { ...x.inputs, [k]: e.target.value } } : x,
                        ),
                      })
                    }
                  />
                  <button
                    type="button"
                    className="btn"
                    aria-label={`Remove write ${i + 1} input ${k}`}
                    onClick={() => removeWriteInput(i, k)}
                  >
                    Remove input
                  </button>
                </div>
              ))}
              <button type="button" className="btn" onClick={() => addWriteInput(i)}>
                + Input
              </button>
            </div>
          ))}
          <button
            type="button"
            className="btn"
            onClick={() =>
              set({
                writes: [
                  ...(step.writes ?? []),
                  { action_type: actionTypes[0]?.name ?? "run_data.set", inputs: {} },
                ],
              })
            }
          >
            Add write
          </button>
        </fieldset>
      )}

      {step.kind === "approval" && (
        <div style={{ marginBottom: 12 }}>
          <h4>What happens on each branch</h4>
          {branches.map((b) => (
            <p key={b.on} style={{ margin: "4px 0" }}>
              On {b.on}:{" "}
              {b.on === "approved" || b.on === "approve"
                ? `these writes post and the run continues to ${targetLabel(branchTargetKey(b))}.`
                : `these writes do not post — the run goes to ${targetLabel(branchTargetKey(b))}.`}
            </p>
          ))}
          {branches.length === 0 && <p style={{ color: "var(--text-muted)" }}>No branches yet — add approve and reject branches below.</p>}
        </div>
      )}

      {step.kind === "check" && step.check && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Check</legend>
          <div style={{ marginBottom: 8 }}>
            <label htmlFor="builder-check-message">Message</label>
            <input
              id="builder-check-message"
              type="text"
              value={step.check.message}
              onChange={(e) => set({ check: { ...step.check!, message: e.target.value } })}
            />
          </div>
          <div style={{ marginBottom: 8 }}>
            <label htmlFor="builder-check-scope">Scope</label>{" "}
            <select
              id="builder-check-scope"
              value={step.check.scope}
              onChange={(e) => set({ check: { ...step.check!, scope: e.target.value } })}
            >
              <option value="anchor">anchor</option>
              <option value="per_item">per_item</option>
            </select>
          </div>
          <ConditionBuilder
            condition={step.check.condition ?? null}
            definition={definition}
            onChange={(condition) => set({ check: { ...step.check!, condition } })}
            onJumpToStep={onJumpToStep}
          />
          <div style={{ marginTop: 8 }}>
            <label htmlFor="builder-check-expression">Expression (single-line alternative)</label>
            <input
              id="builder-check-expression"
              type="text"
              value={step.check.expression ?? ""}
              placeholder="$field [+ $field]* (<=|<|>=|>|==|!=) ($field|number)"
              onChange={(e) => set({ check: { ...step.check!, expression: e.target.value || null } })}
            />
          </div>
        </fieldset>
      )}

      {expectedBranches.length > 0 && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Branches</legend>
          {expectedBranches.map((on) => {
            const branch = branches.find((b) => b.on === on);
            return (
              <div key={on} style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 8, flexWrap: "wrap" }}>
                <strong>{on}</strong>
                {branch ? (
                  <>
                    <label>
                      Branch {on} target{" "}
                      <select
                        aria-label={`Branch ${on} target`}
                        value={branchTargetKey(branch)}
                        onChange={(e) => {
                          const value = e.target.value;
                          const next: LaneBranch = value.startsWith("end:")
                            ? { on, end: { outcome: value.slice(4) } }
                            : (() => {
                                const [stagePart, stepPart] = value.split("/");
                                return { on, target: { stage: stagePart, step: stepPart } };
                              })();
                          set({ branches: branches.map((b) => (b.on === on ? next : b)) });
                        }}
                      >
                        {branchTargets.map((f) => (
                          <option key={`${f.stage.id}/${f.step.id}`} value={`${f.stage.id}/${f.step.id}`}>
                            {f.stage.name} · {f.step.name}
                          </option>
                        ))}
                        {["held", "not_triggered", "done", "rejected"].map((o) => (
                          <option key={o} value={`end:${o}`}>
                            End: {o}
                          </option>
                        ))}
                      </select>
                    </label>
                    <span style={{ color: "var(--text-muted)" }}>→ {targetLabel(branchTargetKey(branch))}</span>
                  </>
                ) : (
                  <span role="alert" style={{ color: "var(--danger)" }}>
                    Empty slot: no {on} branch — a branch with no exit cannot publish (P-09).{" "}
                    <button
                      type="button"
                      className="btn"
                      onClick={() =>
                        set({ branches: [...branches, defaultBranchFor(definition, step.id, on)] })
                      }
                    >
                      Add {on} branch
                    </button>
                  </span>
                )}
              </div>
            );
          })}
        </fieldset>
      )}

      {(step.kind === "form" || step.kind === "record" || step.kind === "agent") && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Repeats</legend>
          {step.repeat_over ? (
            <>
              <p style={{ color: "var(--text-muted)" }}>
                Repeats once per {step.repeat_over.field}.
                {step.kind === "record" && !skipWhen &&
                  " A repeating record step has no filter — it will post for every child, including the ones with nothing to post."}
                {step.kind === "record" && skipWhen &&
                  " Children matching the skip condition below are skipped — no write posts for them."}
              </p>
              <label>
                Repeat over{" "}
                <input
                  aria-label="Repeat over field"
                  value={step.repeat_over.field}
                  onChange={(e) =>
                    set({
                      repeat_over: e.target.value
                        ? { ...step.repeat_over!, field: e.target.value }
                        : null,
                    })
                  }
                />
              </label>
              <div style={{ marginTop: 8 }}>
                <label>
                  Per-item assignment{" "}
                  <select
                    aria-label="Per-item assignment"
                    value={repeatAssign?.by ?? "role"}
                    onChange={(e) => {
                      const by = e.target.value as "child_field" | "role";
                      set({
                        repeat_over: {
                          ...step.repeat_over!,
                          assign:
                            by === "child_field"
                              ? { by, field: repeatAssign?.field ?? "" }
                              : { by: "role" },
                        },
                      });
                    }}
                  >
                    <option value="role">Whole step: the step&apos;s role</option>
                    <option value="child_field">Per item: a field on each child</option>
                  </select>
                </label>
                {repeatAssign?.by === "child_field" && (
                  <label style={{ marginLeft: 8 }}>
                    Child field holding the assignee{" "}
                    <input
                      aria-label="Assignment child field"
                      value={repeatAssign.field ?? ""}
                      placeholder="counted_by"
                      onChange={(e) =>
                        set({
                          repeat_over: {
                            ...step.repeat_over!,
                            assign: { by: "child_field", field: e.target.value },
                          },
                        })
                      }
                    />
                  </label>
                )}
                <p style={{ color: "var(--text-muted)" }}>
                  The runner honors per-item assignment once WP-0710 lands — today the step&apos;s
                  role is the assignee for every item.
                </p>
              </div>
              <div style={{ marginTop: 8 }}>
                <h4 style={{ margin: "0 0 4px" }}>Skip condition</h4>
                <ConditionBuilder
                  condition={skipWhen}
                  definition={definition}
                  onChange={(condition) => setExtended({ skip_when: condition })}
                  onJumpToStep={onJumpToStep}
                />
                {skipWhen && (
                  <button
                    type="button"
                    className="btn"
                    onClick={() => setExtended({ skip_when: null })}
                  >
                    Remove skip condition
                  </button>
                )}
                <p style={{ color: "var(--text-muted)" }}>
                  The runner honors skip conditions once WP-0710 lands — today a repeating record
                  step posts for every child.
                </p>
              </div>
            </>
          ) : (
            <label>
              Repeat over (leave blank for once){" "}
              <input
                aria-label="Repeat over field"
                value=""
                placeholder="lines"
                onChange={(e) => {
                  if (e.target.value.trim()) {
                    set({ repeat_over: { field: e.target.value.trim(), item_id_field: "id" } });
                  }
                }}
              />
            </label>
          )}
        </fieldset>
      )}

      <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
        <legend>Child lane</legend>
        <div style={{ marginBottom: 8 }}>
          <label htmlFor="builder-child-lane-id">Child lane id</label>{" "}
          {laneKeys && laneKeys.length > 0 ? (
            <select
              id="builder-child-lane-id"
              value={step.child_lane?.lane_id ?? ""}
              onChange={(e) =>
                set({
                  child_lane: e.target.value
                    ? { lane_id: e.target.value, anchor_record: step.child_lane?.anchor_record ?? "$record.id" }
                    : null,
                })
              }
            >
              <option value="">No child lane</option>
              {laneKeys.map((id) => (
                <option key={id} value={id}>
                  {id}
                </option>
              ))}
              {step.child_lane && !laneKeys.includes(step.child_lane.lane_id) && (
                <option value={step.child_lane.lane_id}>{step.child_lane.lane_id}</option>
              )}
            </select>
          ) : (
            <input
              id="builder-child-lane-id"
              type="text"
              value={step.child_lane?.lane_id ?? ""}
              placeholder="lane id, or blank for none"
              onChange={(e) =>
                set({
                  child_lane: e.target.value
                    ? { lane_id: e.target.value, anchor_record: step.child_lane?.anchor_record ?? "$record.id" }
                    : null,
                })
              }
            />
          )}
        </div>
        {step.child_lane && (
          <div>
            <label htmlFor="builder-child-anchor">Child lane anchor record</label>
            <input
              id="builder-child-anchor"
              type="text"
              value={step.child_lane.anchor_record}
              onChange={(e) => set({ child_lane: { ...step.child_lane!, anchor_record: e.target.value } })}
            />
            <p style={{ color: "var(--text-muted)" }}>
              Runs lane {step.child_lane.lane_id} on {step.child_lane.anchor_record}; the run waits
              until it finishes.
            </p>
          </div>
        )}
      </fieldset>

      <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
        <legend>Replaces (workflow wiring)</legend>
        {(step.replaces ?? []).map((r, i) => (
          <div key={i} style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", marginBottom: 8 }}>
            {workflowRules && workflowRules.length > 0 ? (
              <label>
                Workflow{" "}
                <select
                  aria-label={`Replaces ${i + 1} workflow`}
                  value={r.workflow_id}
                  onChange={(e) =>
                    set({
                      replaces: (step.replaces ?? []).map((x, j) =>
                        j === i ? { ...x, workflow_id: e.target.value } : x,
                      ),
                    })
                  }
                >
                  {workflowRules.map((w) => (
                    <option key={w.id} value={w.id}>
                      {w.name} ({w.id})
                    </option>
                  ))}
                  {!workflowRules.some((w) => w.id === r.workflow_id) && (
                    <option value={r.workflow_id}>{r.workflow_id}</option>
                  )}
                </select>
              </label>
            ) : (
              <label>
                Workflow id{" "}
                <input
                  aria-label={`Replaces ${i + 1} workflow id`}
                  type="text"
                  value={r.workflow_id}
                  placeholder="wf-wh-01"
                  onChange={(e) =>
                    set({
                      replaces: (step.replaces ?? []).map((x, j) =>
                        j === i ? { ...x, workflow_id: e.target.value } : x,
                      ),
                    })
                  }
                />
              </label>
            )}
            <label>
              Action index{" "}
              <input
                type="number"
                aria-label={`Replaces ${i + 1} action index`}
                value={r.action_index}
                onChange={(e) =>
                  set({
                    replaces: (step.replaces ?? []).map((x, j) =>
                      j === i ? { ...x, action_index: Number(e.target.value) } : x,
                    ),
                  })
                }
              />
            </label>
            <button
              type="button"
              className="btn"
              onClick={() => set({ replaces: (step.replaces ?? []).filter((_, j) => j !== i) })}
            >
              Remove replaces
            </button>
          </div>
        ))}
        <button
          type="button"
          className="btn"
          onClick={() =>
            set({ replaces: [...(step.replaces ?? []), { workflow_id: "", action_index: 0 }] })
          }
        >
          Add replaces
        </button>
        <p style={{ color: "var(--text-muted)" }}>
          Workflows are the workflow rules for this lane&apos;s object
          {definition.object_key ? ` (${definition.object_key})` : " (no object yet)"} — pick one,
          or type an id if yours is not listed. There is no workspace-wide workflow list API.
        </p>
      </fieldset>
    </section>
  );
}
