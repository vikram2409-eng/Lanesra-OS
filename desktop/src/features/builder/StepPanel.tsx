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
 */
import { ConditionBuilder } from "./ConditionBuilder";
import { flattenSteps } from "./builderUtils";
import type { BuilderActionType, LaneBranch, LaneDefinition, LaneStep } from "../../lib/types";

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
}) {
  const flat = flattenSteps(definition);
  const stage = definition.stages.find((s) => s.id === stageId);
  const stepIndex = stage?.steps.findIndex((s) => s.id === step.id) ?? -1;
  const isProvisional = (actionType: string) =>
    actionTypes.find((a) => a.name === actionType)?.provisional ??
    !adoptedActionTypes.includes(actionType);

  function set(patch: Partial<LaneStep>) {
    onChange({ ...step, ...patch });
  }

  function targetLabel(key: string): string {
    if (key.startsWith("end:")) return `ends (${key.slice(4)})`;
    const [stageIdPart, stepIdPart] = key.split("/");
    const found = flat.find((f) => f.stage.id === stageIdPart && f.step.id === stepIdPart);
    return found ? `${found.stage.name} · ${found.step.name}` : key;
  }

  const expectedBranches =
    step.kind === "check" ? ["pass", "fail"] : step.kind === "approval" ? ["approved", "rejected"] : [];
  const branches = step.branches ?? [];

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
      </div>

      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-name">Step name</label>
        <input id="builder-step-name" type="text" value={step.name} onChange={(e) => set({ name: e.target.value })} />
      </div>
      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-kind">Kind</label>{" "}
        <select id="builder-step-kind" value={step.kind} onChange={(e) => set({ kind: e.target.value })}>
          {STEP_KINDS.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>
      </div>
      <div style={{ marginBottom: 8 }}>
        <label htmlFor="builder-step-role">Role</label>
        <input
          id="builder-step-role"
          type="text"
          value={step.role ?? ""}
          placeholder="role, or blank for anyone"
          onChange={(e) => set({ role: e.target.value || null })}
        />
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
          {(step.fields ?? []).map((f, i) => (
            <div key={i} style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 8, alignItems: "center" }}>
              <input
                aria-label={`Field ${i + 1} key`}
                value={f.key}
                onChange={(e) => set({ fields: (step.fields ?? []).map((x, j) => (j === i ? { ...x, key: e.target.value } : x)) })}
              />
              <input
                aria-label={`Field ${i + 1} label`}
                value={f.label}
                onChange={(e) => set({ fields: (step.fields ?? []).map((x, j) => (j === i ? { ...x, label: e.target.value } : x)) })}
              />
              <input
                aria-label={`Field ${i + 1} binds to`}
                value={f.binds_to}
                onChange={(e) => set({ fields: (step.fields ?? []).map((x, j) => (j === i ? { ...x, binds_to: e.target.value } : x)) })}
              />
              <button
                type="button"
                className="btn"
                onClick={() => set({ fields: (step.fields ?? []).filter((_, j) => j !== i) })}
              >
                Remove field
              </button>
            </div>
          ))}
          <button
            type="button"
            className="btn"
            onClick={() =>
              set({
                fields: [
                  ...(step.fields ?? []),
                  { key: "new_field", label: "New field", type: "string", binds_to: "run_data.new_field", required: false },
                ],
              })
            }
          >
            Add field
          </button>
        </fieldset>
      )}

      {(step.kind === "record" || step.kind === "approval" || step.kind === "agent") && (
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
                  P-12: provisional Action Type — cannot publish until the catalog adopts it.
                </p>
              )}
              {Object.entries(w.inputs).map(([k, v]) => (
                <div key={k} style={{ display: "flex", gap: 8, marginTop: 4, alignItems: "center" }}>
                  <code>{k}</code>
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
                </div>
              ))}
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
                        {flat.map((f) => (
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
                        set({
                          branches: [
                            ...branches,
                            { on, target: { stage: stageId, step: step.id } },
                          ],
                        })
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

      {step.repeat_over && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Repeats</legend>
          <p style={{ color: "var(--text-muted)" }}>
            Repeats once per {step.repeat_over.field}. A repeating record step has no filter — it
            posts for every child (Q-34 warning in the Count lane).
          </p>
          <label>
            Repeat over{" "}
            <input
              aria-label="Repeat over field"
              value={step.repeat_over.field}
              onChange={(e) => set({ repeat_over: { ...step.repeat_over!, field: e.target.value } })}
            />
          </label>
        </fieldset>
      )}

      <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
        <legend>Child lane</legend>
        <div style={{ marginBottom: 8 }}>
          <label htmlFor="builder-child-lane-id">Child lane id</label>
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
    </section>
  );
}
