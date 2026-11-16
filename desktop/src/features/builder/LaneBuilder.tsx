/**
 * The visual lane builder (WP-0718, REQ-LANE-024).
 *
 * Canvas (stages as chevrons, steps as cards, branches as drawn
 * arrows), step panel, condition builder, reads/writes map,
 * approval-writes view, Action-Type picker, live publish checks and
 * the dry-run preview — all editing the one LaneDefinition, in the
 * same JSON format the demo and the core share (AC-1). There is no
 * builder-only format and no builder-only rule: the checks are the
 * WP-0714 list, evaluated on every change through builder_check_lane
 * with the local checks (builderUtils.localPublishChecks) as the
 * fallback while that command is still landing (AC-2). Every canvas
 * action has a keyboard equivalent — select, move, draw an arrow
 * (branch-target select), open a panel — and the surface is
 * axe-checked (AC-3). All five builder commands go through the shared
 * dual-transport `call<T>` (AC-4).
 */
import { useEffect, useMemo, useState } from "react";

import { api } from "../../lib/api";
import type {
  BuilderActionType,
  BuilderReadsWrites,
  DryRunReport,
  LaneDefinition,
  LaneStep,
  PublishReport,
} from "../../lib/types";
import { useNodeDrag } from "../../components/visualBuilder/useNodeDrag";
import { StageChevrons } from "./StageChevrons";
import { StepPanel } from "./StepPanel";
import {
  checkLastWarnings,
  flattenSteps,
  localPublishChecks,
  plumbingFromLocal,
  provisionalActionTypes,
  repeatingRecordStepsWithoutFilter,
  stepCount,
} from "./builderUtils";

const STEP_KIND_LABELS: Record<string, string> = {
  form: "Form",
  record: "Record",
  check: "Check",
  approval: "Approval",
  agent: "Agent",
};

function stepKindLabel(kind: string): string {
  return STEP_KIND_LABELS[kind] ?? kind;
}

/// Branch diagram: drawn SVG arrows from each check/approval step to
/// its branch targets (pass/fail, approve/reject) and to end markers
/// labelled with their outcomes. Positions come from definition order,
/// so the diagram always matches the canvas.
function BranchDiagram({ definition }: { definition: LaneDefinition }) {
  const flat = flattenSteps(definition);
  const rowH = 64;
  const colW = 220;
  const width = Math.max(definition.stages.length, 1) * colW + 120;
  const height = Math.max(flat.length, 1) * rowH + 40;
  const pos = new Map<string, { x: number; y: number }>();
  flat.forEach((f) => {
    const stageIndex = definition.stages.findIndex((s) => s.id === f.stage.id);
    pos.set(`${f.stage.id}/${f.step.id}`, { x: 60 + stageIndex * colW, y: 30 + (f.order - 1) * rowH });
  });
  const lines: { key: string; x1: number; y1: number; x2: number; y2: number; label: string; end?: string }[] = [];
  for (const f of flat) {
    for (const branch of f.step.branches ?? []) {
      const from = pos.get(`${f.stage.id}/${f.step.id}`);
      if (!from) continue;
      if (branch.target) {
        const to = pos.get(`${branch.target.stage}/${branch.target.step}`);
        if (to) lines.push({ key: `${f.step.id}:${branch.on}`, x1: from.x, y1: from.y, x2: to.x, y2: to.y, label: branch.on });
      } else if (branch.end) {
        lines.push({ key: `${f.step.id}:${branch.on}`, x1: from.x, y1: from.y, x2: width - 40, y2: from.y, label: branch.on, end: branch.end.outcome });
      }
    }
  }
  return (
    <svg
      role="img"
      aria-label="Branch diagram"
      width={width}
      height={height}
      style={{ display: "block", maxWidth: "100%", overflowX: "auto", background: "var(--bg)" }}
    >
      <defs>
        <marker id="builder-arrow" markerWidth="8" markerHeight="8" refX="7" refY="3" orient="auto">
          <path d="M0,0 L8,3 L0,6 Z" fill="currentColor" />
        </marker>
      </defs>
      {flat.map((f) => {
        const p = pos.get(`${f.stage.id}/${f.step.id}`)!;
        return (
          <g key={f.step.id}>
            <rect x={p.x - 50} y={p.y - 14} width="100" height="28" rx="4" fill="var(--card, #fff)" stroke="currentColor" />
            <text x={p.x} y={p.y + 4} textAnchor="middle" fontSize="11">
              {f.step.name.slice(0, 16)}
            </text>
          </g>
        );
      })}
      {lines.map((l) => (
        <g key={l.key}>
          <line x1={l.x1 + 50} y1={l.y1} x2={l.x2 - 50} y2={l.y2} stroke="currentColor" strokeWidth="1.5" markerEnd="url(#builder-arrow)" />
          <text x={(l.x1 + l.x2) / 2} y={(l.y1 + l.y2) / 2 - 4} textAnchor="middle" fontSize="11">
            {l.label}
          </text>
          {l.end && (
            <g>
              <circle cx={l.x2 - 30} cy={l.y2} r="12" fill="none" stroke="currentColor" />
              <text x={l.x2 - 30} y={l.y2 + 4} textAnchor="middle" fontSize="9">
                {l.end}
              </text>
            </g>
          )}
        </g>
      ))}
    </svg>
  );
}

/// Per-line expected / counted / variance table for the Count lane
/// (§8). The seed is the example from the lane description
/// (CC-2026-001, Zone A pick face: three variances, HG-1001 -2,
/// HG-1003 +1, HG-1005 -2); both numbers are editable and the variance
/// is computed, highlighted when it is not zero.
const COUNT_SEED = [
  { line: "HG-1001", expected: 10, counted: 8 },
  { line: "HG-1002", expected: 8, counted: 8 },
  { line: "HG-1003", expected: 6, counted: 7 },
  { line: "HG-1004", expected: 4, counted: 4 },
  { line: "HG-1005", expected: 2, counted: 0 },
];

function CountPanel({ definition }: { definition: LaneDefinition }) {
  const [rows, setRows] = useState(COUNT_SEED);
  const noFilter = repeatingRecordStepsWithoutFilter(definition);
  return (
    <section className="card" aria-labelledby="count-panel-heading">
      <h3 id="count-panel-heading">Count — per-line table</h3>
      <p style={{ color: "var(--text-muted)" }}>
        Example seed from the lane description (CC-2026-001, Zone A pick face). Edit the counted
        quantities; the variance is computed. The approval step sees this whole table at once, not
        one line at a time.
      </p>
      {noFilter.length > 0 && (
        <p role="alert" style={{ color: "var(--warning, #8a6d00)" }}>
          Warning: repeating record step{noFilter.length === 1 ? "" : "s"} {noFilter.join(", ")}{" "}
          {noFilter.length === 1 ? "has" : "have"} no filter — it will post for every child,
          including the ones with nothing to post.
        </p>
      )}
      <table aria-label="Count lines">
        <thead>
          <tr>
            <th>Line</th>
            <th>Expected</th>
            <th>Counted</th>
            <th>Variance</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => {
            const variance = row.counted - row.expected;
            return (
              <tr key={row.line}>
                <td>{row.line}</td>
                <td>
                  <input
                    type="number"
                    aria-label={`Expected quantity for ${row.line}`}
                    value={row.expected}
                    onChange={(e) =>
                      setRows(rows.map((r, j) => (j === i ? { ...r, expected: Number(e.target.value) } : r)))
                    }
                  />
                </td>
                <td>
                  <input
                    type="number"
                    aria-label={`Counted quantity for ${row.line}`}
                    value={row.counted}
                    onChange={(e) =>
                      setRows(rows.map((r, j) => (j === i ? { ...r, counted: Number(e.target.value) } : r)))
                    }
                  />
                </td>
                <td style={variance !== 0 ? { fontWeight: 700, color: "var(--danger)" } : undefined}>
                  {variance > 0 ? `+${variance}` : `${variance}`}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <p style={{ color: "var(--text-muted)" }}>
        Whole-count approval view:{" "}
        {rows
          .filter((r) => r.counted - r.expected !== 0)
          .map((r) => `${r.line} ${r.counted - r.expected > 0 ? "+" : ""}${r.counted - r.expected}`)
          .join(", ") || "no variances — the count closes straight away."}
      </p>
    </section>
  );
}

export function LaneBuilder({
  definition: initialDefinition,
  laneKey,
  onDefinitionChange,
  onClose,
}: {
  definition: LaneDefinition;
  laneKey?: string | null;
  onDefinitionChange?: (definition: LaneDefinition) => void;
  onClose?: () => void;
}) {
  const [def, setDefState] = useState(initialDefinition);
  const [selectedStepId, setSelectedStepId] = useState<string | null>(null);
  const [selectedStageId, setSelectedStageId] = useState<string | null>(null);
  const [report, setReport] = useState<PublishReport | null>(null);
  const [checksNote, setChecksNote] = useState<string | null>(null);
  const [dryRun, setDryRun] = useState<DryRunReport | null>(null);
  const [dryRunError, setDryRunError] = useState<string | null>(null);
  const [dryRunning, setDryRunning] = useState(false);
  const [anchorRecordId, setAnchorRecordId] = useState("");
  const [records, setRecords] = useState<{ id: string; primary_name: string; display_number: string }[]>([]);
  const [actionTypes, setActionTypes] = useState<BuilderActionType[]>([]);
  const [adoptedActionTypes, setAdoptedActionTypes] = useState<string[]>([]);
  const [plumbing, setPlumbing] = useState<BuilderReadsWrites | null>(null);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [publishMessage, setPublishMessage] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [publishing, setPublishing] = useState(false);

  // Visual canvas positions for the step cards, driven by useNodeDrag
  // (the shared drag hook). Positions are visual only — definition
  // order is what the lane stores, and the drag commits a reorder.
  const [nodes, setNodes] = useState<{ id: string; x: number; y: number }[]>([]);
  const { startDrag } = useNodeDrag(setNodes, 1, false, (n) => n.id);

  const flat = useMemo(() => flattenSteps(def), [def]);
  const total = stepCount(def);
  const selected = flat.find((f) => f.step.id === selectedStepId) ?? null;
  const failedStepIds = useMemo(
    () => new Set((report?.checks ?? []).filter((c) => c.status !== "pass" && c.step).map((c) => c.step as string)),
    [report],
  );
  const refuses = (report?.checks ?? []).filter((c) => c.status === "refuse");
  const firstRefuse = refuses[0] ?? null;
  const warnings = useMemo(() => checkLastWarnings(def), [def]);
  const warningForSelected = selected ? warnings.find((w) => w.stepId === selected.step.id) ?? null : null;

  function setDef(next: LaneDefinition) {
    setDefState(next);
    onDefinitionChange?.(next);
  }

  function updateStep(stepId: string, next: LaneStep) {
    setDef({
      ...def,
      stages: def.stages.map((s) => ({
        ...s,
        steps: s.steps.map((st) => (st.id === stepId ? next : st)),
      })),
    });
  }

  function moveStep(stepId: string, direction: -1 | 1) {
    const next: LaneDefinition = JSON.parse(JSON.stringify(def));
    for (const stage of next.stages) {
      const i = stage.steps.findIndex((s) => s.id === stepId);
      if (i >= 0) {
        const j = i + direction;
        if (j < 0 || j >= stage.steps.length) return;
        [stage.steps[i], stage.steps[j]] = [stage.steps[j], stage.steps[i]];
        break;
      }
    }
    setDef(next);
  }

  function moveCheckToEnd(stepId: string) {
    const next: LaneDefinition = JSON.parse(JSON.stringify(def));
    for (const stage of next.stages) {
      const i = stage.steps.findIndex((s) => s.id === stepId);
      if (i >= 0) {
        const [step] = stage.steps.splice(i, 1);
        stage.steps.push(step);
        break;
      }
    }
    setDef(next);
  }

  function moveStage(stageId: string, direction: -1 | 1) {
    const next: LaneDefinition = JSON.parse(JSON.stringify(def));
    const i = next.stages.findIndex((s) => s.id === stageId);
    const j = i + direction;
    if (i < 0 || j < 0 || j >= next.stages.length) return;
    [next.stages[i], next.stages[j]] = [next.stages[j], next.stages[i]];
    setDef(next);
  }

  // Keep the visual nodes in step with the definition's step ids.
  const stepIdsKey = flat.map((f) => f.step.id).join(",");
  useEffect(() => {
    setNodes(
      flat.map((f) => {
        const stageIndex = def.stages.findIndex((s) => s.id === f.stage.id);
        const stepIndex = def.stages[stageIndex].steps.findIndex((s) => s.id === f.step.id);
        return { id: f.step.id, x: stageIndex * 260, y: stepIndex * 110 };
      }),
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stepIdsKey]);

  // Live publish checks on every change (AC-2), and the reads/writes
  // map from the service — both fall back to the local computation
  // while the builder commands are still landing.
  useEffect(() => {
    let cancelled = false;
    api
      .builderCheckLane(def)
      .then((r) => {
        if (!cancelled) {
          setReport(r);
          setChecksNote(null);
        }
      })
      .catch(() => {
        if (cancelled) return;
        const local = localPublishChecks(def, adoptedActionTypes);
        setReport({ lane_key: def.id, version: null, refused: local.some((c) => c.status === "refuse"), checks: local });
        setChecksNote("builder_check_lane is not available yet — showing the local checks.");
      });
    api
      .builderReadsWrites(def)
      .then((r) => {
        // An empty service answer for a lane that declares run-data
        // fields means the service has nothing to say yet — the local
        // computation is the map, not an empty table.
        if (!cancelled) setPlumbing(r.plumbing.length > 0 ? r : plumbingFromLocal(def));
      })
      .catch(() => {
        if (!cancelled) setPlumbing(plumbingFromLocal(def));
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [def, adoptedActionTypes]);

  // The adopted Action-Type catalog (WP-0705): the local P-12 check
  // needs it, so a lane that only uses adopted types can go green even
  // while builder_check_lane is still landing. Guarded with `?.` — the
  // mocked api in unit tests does not provide listActionTypes.
  useEffect(() => {
    if (!def.object_key) return;
    let cancelled = false;
    const pending = api.listActionTypes?.(def.object_key, true);
    if (pending) {
      pending
        .then((types) => {
          if (!cancelled) setAdoptedActionTypes(types.map((t) => t.name));
        })
        .catch(() => {});
    }
    return () => {
      cancelled = true;
    };
  }, [def.object_key]);

  // Action-Type picker: the adopted catalog when the service answers,
  // else the provisional ids this lane already uses, marked provisional.
  useEffect(() => {
    let cancelled = false;
    api
      .builderActionTypes(def.object_key || undefined)
      .then((types) => {
        if (!cancelled) setActionTypes(types);
      })
      .catch(() => {
        if (!cancelled) {
          setActionTypes(
            provisionalActionTypes(def).map((id) => ({
              name: id,
              target_object_type: def.object_key,
              adopted: false,
              provisional: true,
              effect: null,
              description: "",
              parameters: [],
            })),
          );
        }
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [def.object_key]);

  useEffect(() => {
    if (!def.object_key) return;
    let cancelled = false;
    api
      .listCustomRecords(def.object_key)
      .then((r) => {
        if (!cancelled) setRecords(r as { id: string; primary_name: string; display_number: string }[]);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [def.object_key]);

  function commitDragReorder() {
    const next: LaneDefinition = JSON.parse(JSON.stringify(def));
    let changed = false;
    for (const stage of next.stages) {
      const ordered = [...stage.steps].sort((a, b) => {
        const ya = nodes.find((n) => n.id === a.id)?.y ?? 0;
        const yb = nodes.find((n) => n.id === b.id)?.y ?? 0;
        return ya - yb;
      });
      if (ordered.some((s, i) => s.id !== stage.steps[i].id)) {
        stage.steps = ordered;
        changed = true;
      }
    }
    if (changed) setDef(next);
  }

  async function save() {
    setSaving(true);
    setSaveMessage(null);
    try {
      const version = await api.saveBuilderLane(def, laneKey ?? null);
      setSaveMessage(`Saved as version ${version.version}.`);
    } catch (err) {
      setSaveMessage(
        `Could not save: ${err instanceof Error ? err.message : "save_builder_lane is not available yet."}`,
      );
    } finally {
      setSaving(false);
    }
  }

  /// Publish publishes the EDITED definition, not the last saved one:
  /// save first (a new draft version), then publish that version.
  /// Saving first matters because the checks that enabled the button
  /// ran on the edited definition — publishing the stale saved lane
  /// while the editor shows green would be a lie, so a save failure
  /// stops the publish and says so instead of publishing stale state.
  async function publish() {
    if (!laneKey) return;
    setPublishing(true);
    setPublishMessage(null);
    try {
      await api.saveBuilderLane(def, laneKey);
    } catch (err) {
      setPublishMessage(
        `Could not save the edited lane before publishing: ${err instanceof Error ? err.message : "save_builder_lane is not available yet."} The saved lane is unchanged.`,
      );
      setPublishing(false);
      return;
    }
    try {
      const version = await api.publishLane(laneKey);
      setPublishMessage(`Published version ${version.version}.`);
    } catch (err) {
      setPublishMessage(`Publish refused: ${err instanceof Error ? err.message : "publishing failed."}`);
    } finally {
      setPublishing(false);
    }
  }

  async function runDryRun() {
    if (!anchorRecordId) return;
    setDryRunning(true);
    setDryRunError(null);
    try {
      setDryRun(await api.builderDryRunLane(def, anchorRecordId, { run_data: {} }));
    } catch (err) {
      setDryRun(null);
      setDryRunError(
        `The builder dry run is not available yet (builder_dry_run_lane): ${err instanceof Error ? err.message : ""} Save the lane and dry-run the saved version from the blueprint page.`,
      );
    } finally {
      setDryRunning(false);
    }
  }

  const publishTitle = !laneKey
    ? "Save the lane first — publish works on a saved lane."
    : firstRefuse
      ? `Cannot publish — ${firstRefuse.id}: ${firstRefuse.reason ?? "failing check"}`
      : "Publish this lane";
  const stockMoves = (dryRun?.writes ?? []).filter((w) => w.action_type.includes("stock_move"));
  const draftPoWrite = flat
    .find((f) => f.step.id === "create-draft-po")
    ?.step.writes?.find((w) => w.action_type === "purchase_order.create");

  return (
    <div>
      <div style={{ display: "flex", gap: 8, alignItems: "baseline", flexWrap: "wrap" }}>
        <label htmlFor="builder-lane-name">Lane name</label>
        <input
          id="builder-lane-name"
          type="text"
          value={def.name}
          onChange={(e) => setDef({ ...def, name: e.target.value })}
          aria-label="Lane name"
        />
        <h2 style={{ margin: 0 }}>{def.name}</h2>
        <span className="badge">
          {total} steps · {def.stages.length} stages · {def.object_key || "no object yet"}
        </span>
        <span style={{ flex: 1 }} />
        {onClose && (
          <button type="button" className="btn" onClick={onClose}>
            Close builder
          </button>
        )}
      </div>
      <p style={{ color: "var(--text-muted)" }}>
        The builder edits the lane definition as data — a lane built here equals the same lane
        written as data. Drag a step card to reorder it, or use its Move buttons; every canvas
        action has a keyboard equivalent.
      </p>

      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 12 }}>
        <button type="button" className="btn" onClick={save} disabled={saving}>
          {saving ? "Saving…" : "Save lane"}
        </button>
        <button
          type="button"
          className="btn"
          onClick={publish}
          disabled={publishing || !laneKey || refuses.length > 0}
          title={publishTitle}
        >
          {publishing ? "Publishing…" : "Publish"}
        </button>
        {saveMessage && <span>{saveMessage}</span>}
        {publishMessage && <span>{publishMessage}</span>}
      </div>

      <StageChevrons
        stages={def.stages}
        selectedStageId={selectedStageId}
        onSelectStage={setSelectedStageId}
        onMoveStage={moveStage}
        onAddStage={() =>
          setDef({
            ...def,
            stages: [...def.stages, { id: `stage-${def.stages.length + 1}`, name: `Stage ${def.stages.length + 1}`, steps: [] }],
          })
        }
      />

      <section className="card" aria-labelledby="builder-canvas-heading" style={{ marginTop: 12 }}>
        <h3 id="builder-canvas-heading">Canvas</h3>
        {def.stages.map((stage) => (
          <div key={stage.id} style={{ marginBottom: 16 }}>
            <h4 style={{ margin: "8px 0" }}>
              {stage.name}{" "}
              <button
                type="button"
                className="btn"
                onClick={() =>
                  setDef({
                    ...def,
                    stages: def.stages.map((s) =>
                      s.id === stage.id
                        ? {
                            ...s,
                            steps: [
                              ...s.steps,
                              { id: `step-${Date.now().toString(36)}`, name: "New step", kind: "form", required: true },
                            ],
                          }
                        : s,
                    ),
                  })
                }
              >
                Add step
              </button>
            </h4>
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              {stage.steps.map((step) => {
                const order = flat.find((f) => f.step.id === step.id)?.order ?? 0;
                const node = nodes.find((n) => n.id === step.id);
                const isCheckLast = warnings.some((w) => w.stepId === step.id);
                return (
                  <button
                    key={step.id}
                    type="button"
                    className="card"
                    aria-label={`Step ${order} of ${total}: ${step.name}`}
                    aria-pressed={selectedStepId === step.id}
                    data-check-failed={failedStepIds.has(step.id) ? "true" : "false"}
                    onClick={() => setSelectedStepId(step.id)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") setSelectedStepId(step.id);
                    }}
                    onPointerDown={(e) => node && startDrag(e, node)}
                    onPointerUp={commitDragReorder}
                    style={{
                      textAlign: "left",
                      cursor: "grab",
                      border: selectedStepId === step.id ? "2px solid var(--accent, #6c5ce7)" : undefined,
                      outline: failedStepIds.has(step.id) ? "2px solid var(--danger)" : undefined,
                      minWidth: 220,
                    }}
                  >
                    <span className="badge">{stepKindLabel(step.kind)}</span>{" "}
                    <span className="badge">{order}</span>{" "}
                    <strong>{step.name}</strong>
                    {step.role && <span style={{ color: "var(--text-muted)" }}> · {step.role}</span>}
                    <span style={{ display: "block", color: "var(--text-muted)" }}>
                      Writes: {(step.writes ?? []).map((w) => w.action_type).join(", ") || "none"}
                    </span>
                    {step.child_lane && (
                      <span style={{ display: "block", color: "var(--text-muted)" }}>
                        Child lane: {step.child_lane.lane_id} on {step.child_lane.anchor_record}
                      </span>
                    )}
                    {isCheckLast && (
                      <span role="alert" style={{ display: "block", color: "var(--warning, #8a6d00)" }}>
                        Check is not the last step of its stage — the step after it runs on the pass path.
                      </span>
                    )}
                    {failedStepIds.has(step.id) && (
                      <span role="alert" style={{ display: "block", color: "var(--danger)" }}>
                        Failing publish check — see the checks panel.
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </section>

      <section className="card" aria-labelledby="builder-branches-heading" style={{ marginTop: 12 }}>
        <h3 id="builder-branches-heading">Branches</h3>
        <BranchDiagram definition={def} />
        <p style={{ color: "var(--text-muted)" }}>
          Every outcome gets an arrow: pass / fail for a check, approve / reject for an approval.
          An empty slot is a visible gap on the canvas and a P-09 failure — pick the target in the
          step panel; a target that does not exist cannot be picked.
        </p>
      </section>

      {selected && (
        <div style={{ marginTop: 12 }}>
          <StepPanel
            step={selected.step}
            stageId={selected.stage.id}
            definition={def}
            actionTypes={actionTypes}
            adoptedActionTypes={adoptedActionTypes}
            checkLastWarning={
              warningForSelected
                ? `check '${selected.step.id}' is not the last step of stage '${selected.stage.id}' — '${warningForSelected.nextStepName}' runs on the pass path`
                : null
            }
            onChange={(next) => updateStep(selected.step.id, next)}
            onJumpToStep={setSelectedStepId}
            onMoveStep={(dir) => moveStep(selected.step.id, dir)}
            onMoveCheckToEnd={() => moveCheckToEnd(selected.step.id)}
          />
        </div>
      )}

      <section className="card" aria-labelledby="builder-checks-heading" style={{ marginTop: 12 }}>
        <h3 id="builder-checks-heading">Publish checks</h3>
        <p style={{ color: "var(--text-muted)" }}>
          The WP-0714 checks run live while editing; failures name the step and the reason, and the
          failing step&apos;s card is marked. Publish stays disabled until the REFUSE checks pass.
        </p>
        {checksNote && <p style={{ color: "var(--text-muted)" }}>{checksNote}</p>}
        {report ? (
          <ul aria-label="Builder publish checks" style={{ paddingLeft: 20, margin: 0 }}>
            {report.checks.map((check) => (
              <li key={`${check.id}:${check.step ?? ""}`} style={{ marginBottom: 8 }}>
                <span className={`badge ${check.status === "pass" ? "badge-success" : check.status === "warn" ? "badge-warning" : "badge-danger"}`}>
                  {check.status}
                </span>{" "}
                <strong>{check.id}</strong>
                {check.step && (
                  <>
                    {" · "}
                    <span>{check.step}</span>
                  </>
                )}
                {check.reason ? <> — {check.reason}</> : ""}
              </li>
            ))}
          </ul>
        ) : (
          <p>Running checks…</p>
        )}
      </section>

      <section className="card" aria-labelledby="builder-plumbing-heading" style={{ marginTop: 12 }}>
        <h3 id="builder-plumbing-heading">Run-data plumbing</h3>
        <p style={{ color: "var(--text-muted)" }}>
          Which run-data fields each step reads and writes — the flag the check depends on and the
          approval that sets it, visibly connected.
        </p>
        <table aria-label="Run-data plumbing">
          <thead>
            <tr>
              <th>Field</th>
              <th>Type</th>
              <th>Per line</th>
              <th>Steps</th>
            </tr>
          </thead>
          <tbody>
            {(plumbing ?? plumbingFromLocal(def)).plumbing.map((row) => (
              <tr key={row.field}>
                <td>
                  <code>{row.field}</code>
                </td>
                <td>{row.field_type ?? "—"}</td>
                <td>{row.per_child ? "Yes" : "No"}</td>
                <td>
                  {row.steps.length === 0
                    ? "—"
                    : row.steps.map((c) => `${c.step_id} (${c.access})`).join(", ")}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card" aria-labelledby="builder-dryrun-heading" style={{ marginTop: 12 }}>
        <h3 id="builder-dryrun-heading">Dry-run preview</h3>
        <p style={{ color: "var(--text-muted)" }}>
          The dry run walks this (unsaved) lane against a seed record and writes nothing — stages
          visited, checks passed or failed, writes it would post.
        </p>
        <div style={{ marginBottom: 8 }}>
          <label htmlFor="builder-dryrun-anchor">Anchor record</label>
          <select id="builder-dryrun-anchor" value={anchorRecordId} onChange={(e) => setAnchorRecordId(e.target.value)}>
            <option value="">Choose a record…</option>
            {records.map((r) => (
              <option key={r.id} value={r.id}>
                {r.primary_name} ({r.display_number})
              </option>
            ))}
          </select>
        </div>
        <button type="button" className="btn" onClick={runDryRun} disabled={!anchorRecordId || dryRunning}>
          {dryRunning ? "Running dry run…" : "Run dry run"}
        </button>
        {dryRunError && (
          <p role="alert" style={{ color: "var(--danger)" }}>
            {dryRunError}
          </p>
        )}
        {dryRun && (
          <div role="region" aria-label="Builder dry-run report" style={{ marginTop: 12 }}>
            <p>Stages visited: {dryRun.stages_visited.join(", ") || "none"}.</p>
            <p>End outcome: {dryRun.end_outcome ?? "none"}.</p>
            <h4>Steps</h4>
            <ul style={{ paddingLeft: 20, margin: 0 }}>
              {dryRun.steps.map((s) => (
                <li key={s.step_id}>
                  {s.step_id} — {s.outcome}
                  {s.reason ? ` — ${s.reason}` : ""}
                </li>
              ))}
            </ul>
            <h4>Writes it would post</h4>
            {dryRun.writes.length === 0 ? (
              <p style={{ color: "var(--text-muted)" }}>No writes on this walk.</p>
            ) : (
              <ul style={{ paddingLeft: 20, margin: 0 }}>
                {dryRun.writes.map((w, i) => (
                  <li key={`${w.step_id}:${i}`}>
                    {w.step_id}
                    {w.child_id ? ` (line ${w.child_id})` : ""} — <code>{w.action_type}</code> —{" "}
                    {Object.entries(w.inputs)
                      .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
                      .join(", ")}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </section>

      {def.id === "lane-wh-receiving" && (
        <section className="card" aria-labelledby="receiving-panel-heading" style={{ marginTop: 12 }}>
          <h3 id="receiving-panel-heading">Receive Shipment — live preview</h3>
          <p style={{ color: "var(--text-muted)" }}>
            Pick the seed record (PO-2026-001 with its two lines) in the dry-run preview above to
            see the quantities the steps will post. The discrepancy branch — check → approval →
            back to the record stage with adjusted quantities, reject → end held — is edited in
            the step panel:
          </p>
          <button
            type="button"
            className="btn"
            onClick={() => setSelectedStepId("discrepancy-check")}
          >
            Edit discrepancy check
          </button>
        </section>
      )}

      {def.id === "lane-wh-pickpack" && (
        <section className="card" aria-labelledby="pickpack-panel-heading" style={{ marginTop: 12 }}>
          <h3 id="pickpack-panel-heading">Pick &amp; Pack — StockMove preview</h3>
          <p style={{ color: "var(--text-muted)" }}>
            What each step posts, per line: the PICK move posts only the quantity picked in this
            run. Run the dry-run preview above to fill this in.
          </p>
          {dryRun ? (
            stockMoves.length === 0 ? (
              <p>No StockMoves on this walk.</p>
            ) : (
              <ul style={{ paddingLeft: 20, margin: 0 }}>
                {stockMoves.map((w, i) => (
                  <li key={i}>
                    Line {w.child_id ?? "?"} — <code>{w.action_type}</code> —{" "}
                    {Object.entries(w.inputs)
                      .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
                      .join(", ")}
                  </li>
                ))}
              </ul>
            )
          ) : (
            <p style={{ color: "var(--text-muted)" }}>No dry run yet.</p>
          )}
        </section>
      )}

      {def.id === "lane-wh-count" && (
        <div style={{ marginTop: 12 }}>
          <CountPanel definition={def} />
        </div>
      )}

      {def.id === "lane-wh-replenish" && (
        <section className="card" aria-labelledby="replenish-panel-heading" style={{ marginTop: 12 }}>
          <h3 id="replenish-panel-heading">Replenish — related record &amp; draft PO preview</h3>
          <p style={{ color: "var(--text-muted)" }}>
            Pick the related record (the supplier) from existing records — no new lookup field, no
            schema change. The draft PO preview shows what the manager is approving before it is
            created.
          </p>
          <div style={{ marginBottom: 8 }}>
            <label htmlFor="builder-replenish-supplier">Related record (supplier)</label>
            <select
              id="builder-replenish-supplier"
              value={anchorRecordId}
              onChange={(e) => setAnchorRecordId(e.target.value)}
            >
              <option value="">Choose a record…</option>
              {records.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.primary_name} ({r.display_number})
                </option>
              ))}
            </select>
          </div>
          {draftPoWrite ? (
            <ul style={{ paddingLeft: 20, margin: 0 }}>
              {Object.entries(draftPoWrite.inputs).map(([k, v]) => (
                <li key={k}>
                  {k}: <code>{JSON.stringify(v)}</code>
                </li>
              ))}
              <li>Total: not in the definition — no price field, so the preview shows supplier, product and quantity only.</li>
            </ul>
          ) : (
            <p style={{ color: "var(--text-muted)" }}>No draft-PO write on this lane yet.</p>
          )}
          <p>
            What happens on reject: the draft stays DRAFT — nothing is submitted.
          </p>
        </section>
      )}
    </div>
  );
}
