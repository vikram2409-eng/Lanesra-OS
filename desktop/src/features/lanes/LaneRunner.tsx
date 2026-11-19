/**
 * The lane runner (WP-0707, REQ-LANE-014 / REQ-LANE-003 / REQ-LANE-011).
 *
 * One screen: a stage bar showing every stage, the step list for the
 * current stage, the current step's work, a summary of the anchor record
 * and of what this run has written, Back and Complete step. On a phone the
 * same parts stack (see `.lane-runner-layout` in styles.css).
 *
 * The runner builds its OWN stage-bar and form components: WP-0621 (the
 * shared Path/Form page components) is still proposed, so there is nothing
 * to reuse yet - these are to be replaced when WP-0621 ships.
 *
 * Wording matches the demo runner (WP-0702): "Complete step" (form),
 * "Post and complete step" (record), "Steps — {stage}", "Audit trail",
 * "What this run has written". Lanesra vocabulary only - never "case" or
 * "flow".
 *
 * First cut (ADR-0025): form, record and check steps on the straight
 * path. A run reaching an approval or a branch shows a plain "continue in
 * the approvals inbox" state - no approval UI in this cut (WP-0720).
 * Pause is WP-0720 too, so there is no Pause button here.
 */
import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type {
  FormFieldView,
  LaneDefinition,
  LaneRunAttachmentView,
  LaneRunNoteView,
  LaneRunReassignCandidateView,
  LaneRunView,
  LaneStep,
  LaneStepProgress,
  LaneStepView,
} from "../../lib/types";

export interface RecordSummaryRow {
  label: string;
  value: string;
}

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

function stepStatus(run: LaneRunView, stepId: string): LaneStepProgress["status"] {
  const row = run.steps?.find((s) => s.step_id === stepId);
  if (row) return row.status;
  const currentId = run.current_step_id ?? run.current?.step_id ?? null;
  if (stepId === currentId) return "current";
  return "locked";
}

function stepMark(status: LaneStepProgress["status"]): string {
  switch (status) {
    case "completed":
      return "✓";
    case "current":
      return "▶";
    case "skipped":
      return "—";
    default:
      return "·";
  }
}

/** Stages to draw the bar from: the definition when known, else the run's
 * own step progress (stage ids in first-seen order, names where known). */
function stagesFor(run: LaneRunView, blueprint?: LaneDefinition | null): { id: string; name: string }[] {
  if (blueprint) return blueprint.stages.map((s) => ({ id: s.id, name: s.name }));
  const seen = new Map<string, string>();
  for (const s of run.steps ?? []) {
    if (!seen.has(s.stage_id)) {
      seen.set(
        s.stage_id,
        s.stage_id === (run.current_stage_id ?? run.current?.stage_id)
          ? (run.current_stage_name ?? run.current?.stage_name ?? s.stage_id)
          : s.stage_id,
      );
    }
  }
  return Array.from(seen.entries()).map(([id, name]) => ({ id, name }));
}

function currentStageId(run: LaneRunView): string | null {
  return run.current_stage_id ?? run.current?.stage_id ?? null;
}

function stepKind(blueprint: LaneDefinition | null | undefined, step: LaneStepView | LaneStep): string {
  const stepId = "step_id" in step ? step.step_id : step.id;
  if ("kind" in step && step.kind) return step.kind;
  const defStep = blueprint?.stages
    .flatMap((s) => s.steps)
    .find((s) => s.id === stepId);
  if (defStep) return defStep.kind;
  if ("fields" in step && (step as LaneStepView).fields.length > 0) return "form";
  return "record";
}

function definitionStep(blueprint: LaneDefinition | null | undefined, stepId: string): LaneStep | undefined {
  return blueprint?.stages.flatMap((s) => s.steps).find((s) => s.id === stepId);
}

function isOptionalStep(
  blueprint: LaneDefinition | null | undefined,
  step: LaneStepView,
): boolean {
  if (step.required === false) return true;
  return definitionStep(blueprint, step.step_id)?.required === false;
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (typeof value === "boolean") return value ? "Yes" : "No";
  return String(value);
}

function FieldInput({
  field,
  value,
  onChange,
}: {
  field: FormFieldView;
  value: string | boolean;
  onChange: (value: string | boolean) => void;
}) {
  const id = `lane-field-${field.key}`;
  const label = `${field.label}${field.required ? " *" : ""}`;
  // A record-bound field the viewer cannot read shows value None
  // (REQ-LANE-011): the value is withheld, and the screen says so.
  if (!field.readable) {
    return (
      <div className="lane-field" style={{ marginBottom: 12 }}>
        <label htmlFor={id}>{label}</label>
        <output id={id} style={{ display: "block", color: "var(--text-muted)" }}>
          None
        </output>
        <p style={{ margin: "2px 0 0", fontSize: 12, color: "var(--text-muted)" }}>
          Not readable for this record.
        </p>
      </div>
    );
  }
  if (field.type === "boolean") {
    return (
      <div className="lane-field" style={{ marginBottom: 12 }}>
        <label htmlFor={id}>
          <input
            id={id}
            type="checkbox"
            checked={value === true}
            onChange={(e) => onChange(e.target.checked)}
            style={{ marginRight: 6 }}
          />
          {label}
        </label>
      </div>
    );
  }
  if (field.type === "textarea") {
    return (
      <div className="lane-field" style={{ marginBottom: 12 }}>
        <label htmlFor={id} style={{ display: "block", marginBottom: 4 }}>
          {label}
        </label>
        <textarea
          id={id}
          value={typeof value === "string" ? value : ""}
          onChange={(e) => onChange(e.target.value)}
          rows={3}
        />
      </div>
    );
  }
  return (
    <div className="lane-field" style={{ marginBottom: 12 }}>
      <label htmlFor={id} style={{ display: "block", marginBottom: 4 }}>
        {label}
      </label>
      <input
        id={id}
        type={field.type === "number" ? "number" : field.type === "date" ? "date" : "text"}
        value={typeof value === "string" ? value : ""}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}

function StepForm({
  step,
  onSubmit,
  disabled,
}: {
  step: LaneStepView;
  onSubmit: (values: Record<string, unknown>) => void;
  disabled: boolean;
}) {
  const [values, setValues] = useState<Record<string, string | boolean>>(() => {
    const initial: Record<string, string | boolean> = {};
    for (const f of step.fields) {
      if (!f.readable) continue;
      if (f.type === "boolean") initial[f.key] = f.value === true;
      else initial[f.key] = formatValue(f.value);
    }
    return initial;
  });
  const [error, setError] = useState<string | null>(null);

  function submit(e: FormEvent) {
    e.preventDefault();
    for (const f of step.fields) {
      if (!f.required || !f.readable) continue;
      const v = values[f.key];
      if (v === undefined || (typeof v === "string" && v.trim() === "")) {
        setError(`${f.label} is required`);
        return;
      }
    }
    const out: Record<string, unknown> = {};
    for (const f of step.fields) {
      if (!f.readable) continue;
      const v = values[f.key];
      if (f.type === "number" && typeof v === "string" && v.trim() !== "" && !Number.isNaN(Number(v))) {
        out[f.key] = Number(v);
      } else {
        out[f.key] = v ?? null;
      }
    }
    setError(null);
    onSubmit(out);
  }

  return (
    <form onSubmit={submit}>
      {step.fields.map((f) => (
        <FieldInput
          key={f.key}
          field={f}
          value={values[f.key] ?? (f.type === "boolean" ? false : "")}
          onChange={(v) => setValues((prev) => ({ ...prev, [f.key]: v }))}
        />
      ))}
      {error && (
        <p role="alert" style={{ color: "var(--danger, #b91c1c)" }}>
          {error}
        </p>
      )}
      <button className="btn btn-primary" type="submit" disabled={disabled}>
        Complete step
      </button>
    </form>
  );
}

/**
 * WP-0711 (REQ-LANE-018): the actions available at any point of a run -
 * add a note, attach a file, reassign the current step. Each is audited,
 * and the audit trail card below the run shows them. The card sits under
 * the current step's work, so it is reachable from every step.
 */
function AnytimeActions({
  run,
  onAddNote,
  onAttachFile,
  onReassign,
  onDownloadAttachment,
}: {
  run: LaneRunView;
  onAddNote?: (body: string) => void | Promise<void>;
  onAttachFile?: (fileName: string, mimeType: string | undefined, contentBase64: string) => void | Promise<void>;
  onReassign?: (toUserId: string, reason: string) => void | Promise<void>;
  onDownloadAttachment?: (attachment: LaneRunAttachmentView) => void | Promise<void>;
}) {
  const [noteText, setNoteText] = useState("");
  const [reassignTo, setReassignTo] = useState("");
  const [reassignReason, setReassignReason] = useState("");
  const [fileError, setFileError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (!onAddNote && !onAttachFile && !onReassign && !onDownloadAttachment) return null;
  const candidates: LaneRunReassignCandidateView[] = run.reassign_candidates ?? [];
  const isActive = run.status === "waiting_person";

  async function addNote() {
    const body = noteText.trim();
    if (!body || !onAddNote) return;
    setBusy(true);
    try {
      await onAddNote(body);
      setNoteText("");
    } finally {
      setBusy(false);
    }
  }

  function attachFile(file: File) {
    if (!onAttachFile) return;
    setFileError(null);
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = String(reader.result ?? "");
      const base64 = dataUrl.includes(",") ? dataUrl.split(",").slice(1).join(",") : "";
      if (!base64) {
        setFileError("Could not read that file.");
        return;
      }
      void onAttachFile(file.name, file.type || undefined, base64);
    };
    reader.onerror = () => setFileError("Could not read that file.");
    reader.readAsDataURL(file);
  }

  async function reassign() {
    if (!reassignTo.trim() || !reassignReason.trim() || !onReassign) return;
    setBusy(true);
    try {
      await onReassign(reassignTo, reassignReason.trim());
      setReassignTo("");
      setReassignReason("");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="card" role="group" aria-label="Notes, attachments and reassign">
      <h3 style={{ marginTop: 0 }}>Notes, attachments and reassign</h3>
      <p style={{ color: "var(--text-muted)" }}>
        Available at any point of this run. Every note, file and reassignment is written to the audit trail.
      </p>
      {onAddNote && (
        <div style={{ marginBottom: 12 }}>
          <label htmlFor="lane-note-text" style={{ display: "block", marginBottom: 4 }}>
            Add a note
          </label>
          <textarea
            id="lane-note-text"
            value={noteText}
            onChange={(e) => setNoteText(e.target.value)}
            rows={2}
            style={{ width: "100%" }}
            placeholder="A note for anyone working this run"
          />
          <button className="btn" type="button" onClick={() => void addNote()} disabled={busy || !noteText.trim()}>
            Add note
          </button>
        </div>
      )}
      {onAttachFile && (
        <div style={{ marginBottom: 12 }}>
          <label htmlFor="lane-attach-file" style={{ display: "block", marginBottom: 4 }}>
            Attach a file
          </label>
          <input
            id="lane-attach-file"
            type="file"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) attachFile(file);
              e.target.value = "";
            }}
          />
          {fileError && (
            <p role="alert" style={{ color: "var(--danger, #b91c1c)" }}>
              {fileError}
            </p>
          )}
        </div>
      )}
      {onReassign && isActive && candidates.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <label htmlFor="lane-reassign-to" style={{ display: "block", marginBottom: 4 }}>
            Reassign this step
          </label>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <select id="lane-reassign-to" value={reassignTo} onChange={(e) => setReassignTo(e.target.value)}>
              <option value="">Choose a person…</option>
              {candidates.map((c) => (
                <option key={c.user_id} value={c.user_id}>
                  {c.display_name}
                </option>
              ))}
            </select>
            <input
              aria-label="Reason for reassigning"
              value={reassignReason}
              onChange={(e) => setReassignReason(e.target.value)}
              placeholder="Why is this step moving?"
              style={{ flex: 1, minWidth: 180 }}
            />
            <button
              className="btn"
              type="button"
              onClick={() => void reassign()}
              disabled={busy || !reassignTo || !reassignReason.trim()}
            >
              Reassign
            </button>
          </div>
          <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
            Only people who hold this step&apos;s role can take it. A later handoff clears the reassignment.
          </p>
        </div>
      )}
      {(run.notes ?? []).length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <h4 style={{ marginBottom: 4 }}>Notes</h4>
          <ul style={{ margin: 0, paddingLeft: 20 }}>
            {(run.notes ?? []).map((n: LaneRunNoteView) => (
              <li key={n.id}>
                {n.body}{" "}
                <span style={{ color: "var(--text-muted)" }}>
                  — {n.author_display_name ?? "Someone"} · {new Date(n.created_at).toLocaleString()}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {(run.attachments ?? []).length > 0 && (
        <div>
          <h4 style={{ marginBottom: 4 }}>Attached files</h4>
          <ul style={{ margin: 0, paddingLeft: 20 }}>
            {(run.attachments ?? []).map((a: LaneRunAttachmentView) => (
              <li key={a.id}>
                {a.file_name}{" "}
                <span style={{ color: "var(--text-muted)" }}>
                  ({a.size_bytes != null ? `${Math.round(a.size_bytes / 1024)} KB` : "unknown size"} ·{" "}
                  {new Date(a.created_at).toLocaleString()})
                </span>{" "}
                {onDownloadAttachment && (
                  <button className="btn" type="button" onClick={() => void onDownloadAttachment(a)}>
                    Download
                  </button>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

export function LaneRunnerView({
  run,
  blueprint,
  recordLabel,
  recordSummary,
  onSubmit,
  onSkip,
  onAddNote,
  onAttachFile,
  onReassign,
  onDownloadAttachment,
  onClose,
}: {
  run: LaneRunView;
  blueprint?: LaneDefinition | null;
  recordLabel: string;
  recordSummary?: RecordSummaryRow[];
  onSubmit: (values: Record<string, unknown>) => void | Promise<void>;
  onSkip?: (reason: string) => void | Promise<void>;
  onAddNote?: (body: string) => void | Promise<void>;
  onAttachFile?: (fileName: string, mimeType: string | undefined, contentBase64: string) => void | Promise<void>;
  onReassign?: (toUserId: string, reason: string) => void | Promise<void>;
  onDownloadAttachment?: (attachment: LaneRunAttachmentView) => void | Promise<void>;
  onClose?: () => void;
}) {
  const stages = stagesFor(run, blueprint);
  const currentStage = currentStageId(run);
  const currentStageIndex = Math.max(
    0,
    stages.findIndex((s) => s.id === currentStage),
  );
  const laneName = run.lane_name ?? blueprint?.name ?? "Lane";
  const [reviewStepId, setReviewStepId] = useState<string | null>(null);
  const [skipReason, setSkipReason] = useState<string | null>(null);
  const [skipText, setSkipText] = useState("");

  const currentStep = run.current;
  const waitingKind = run.waiting?.kind ?? null;
  const isApprovalWait =
    run.status === "waiting_approval" ||
    waitingKind === "approval" ||
    waitingKind === "branch" ||
    (currentStep?.kind ?? null) === "approval";

  // The step list shows the current stage's steps only.
  const currentStageDef = blueprint?.stages.find((s) => s.id === currentStage);
  const stageSteps: { id: string; name: string }[] = currentStageDef
    ? currentStageDef.steps.map((s) => ({ id: s.id, name: s.name }))
    : (run.steps ?? [])
        .filter((s) => s.stage_id === currentStage)
        .map((s) => ({ id: s.step_id, name: s.step_id }));

  const completedBefore = (run.steps ?? []).filter((s) => s.status === "completed");
  const lastCompleted = completedBefore.length > 0 ? completedBefore[completedBefore.length - 1] : null;
  const reviewing = reviewStepId
    ? {
        step: definitionStep(blueprint, reviewStepId),
        progress: run.steps?.find((s) => s.step_id === reviewStepId) ?? null,
      }
    : null;

  const shownStep: LaneStepView | null = reviewing
    ? null
    : currentStep;

  function workArea() {
    if (isApprovalWait) {
      return (
        <div className="card">
          <h3 style={{ marginTop: 0 }}>{currentStep?.step_name ?? run.waiting?.step_name ?? "Waiting on approval"}</h3>
          <p>This run is waiting on an approval. Continue in the approvals inbox.</p>
          <p style={{ color: "var(--text-muted)" }}>
            Approvals are decided in the inbox; this screen does not decide them in this cut.
          </p>
        </div>
      );
    }
    if (reviewing) {
      const step = reviewing.step;
      const progress = reviewing.progress;
      return (
        <div className="card">
          <h3 style={{ marginTop: 0 }}>{step?.name ?? reviewStepId}</h3>
          <p>
            Completed
            {progress?.finished_at ? ` ${new Date(progress.finished_at).toLocaleString()}` : ""}.
          </p>
          {step?.fields && step.fields.length > 0 && (
            <dl>
              {step.fields.map((f) => {
                const key = f.binds_to.startsWith("run_data.")
                  ? f.binds_to.slice("run_data.".length)
                  : null;
                const value = key !== null ? run.run_data[key] : undefined;
                return (
                  <div key={f.key} style={{ display: "flex", gap: 8 }}>
                    <dt style={{ color: "var(--text-muted)" }}>{f.label}</dt>
                    <dd style={{ margin: 0 }}>{value === undefined ? "—" : formatValue(value)}</dd>
                  </div>
                );
              })}
            </dl>
          )}
          <p style={{ color: "var(--text-muted)" }}>
            Back reviews a finished step without changing the run.
          </p>
          <button className="btn btn-secondary" onClick={() => setReviewStepId(null)}>
            Back to current step
          </button>
        </div>
      );
    }
    if (!shownStep) {
      return (
        <div className="card">
          <p>This run has no current step.</p>
        </div>
      );
    }
    const kind = stepKind(blueprint, shownStep);
    const defStep = definitionStep(blueprint, shownStep.step_id);
    const canComplete = shownStep.can_complete;
    return (
      <div className="card">
        <div style={{ display: "flex", gap: 8, alignItems: "baseline", flexWrap: "wrap" }}>
          <h3 style={{ margin: 0 }}>{shownStep.step_name}</h3>
          <span className="badge">{stepKindLabel(kind)}</span>
          {shownStep.role && <span style={{ color: "var(--text-muted)" }}>· {shownStep.role}</span>}
        </div>
        {defStep?.description && <p>{defStep.description}</p>}
        {!canComplete && shownStep.waiting_on && (
          <p role="status">Waiting on {shownStep.waiting_on} — this step is locked for you.</p>
        )}
        {kind === "form" && (
          <StepForm key={shownStep.step_id} step={shownStep} onSubmit={onSubmit} disabled={!canComplete} />
        )}
        {kind === "record" && (
          <>
            <ul>
              {(shownStep.writes ?? defStep?.writes ?? []).map((w, i) => (
                <li key={i}>
                  <strong>{w.action_type}</strong> — {JSON.stringify(w.inputs)}
                </li>
              ))}
            </ul>
            <p style={{ color: "var(--text-muted)" }}>
              Posting is all-or-nothing for the step and never repeats: a move that already posted is not posted
              again.
            </p>
            <button className="btn btn-primary" onClick={() => void onSubmit({})} disabled={!canComplete}>
              Post and complete step
            </button>
          </>
        )}
        {kind === "check" && (
          <p role="status">
            {shownStep.check_passed === true
              ? `Passed — ${shownStep.check_reason ?? defStep?.check?.message ?? ""}`
              : shownStep.check_passed === false
                ? `Failed — ${shownStep.check_reason ?? defStep?.check?.message ?? ""}`
                : `Checking… ${defStep?.check?.message ?? ""}`}
          </p>
        )}
        {kind === "agent" && (
          <p role="status">This step is run by an agent. Nothing for you to complete here.</p>
        )}
        {kind !== "form" && kind !== "record" && kind !== "check" && kind !== "agent" && (
          <p style={{ color: "var(--text-muted)" }}>This step has no work on this screen.</p>
        )}
        {onSkip && isOptionalStep(blueprint, shownStep) && kind !== "check" && (
          <div style={{ marginTop: 16 }}>
            {skipReason === null ? (
              <button className="btn btn-secondary" onClick={() => setSkipReason("")} disabled={!canComplete}>
                Skip step
              </button>
            ) : (
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  if (skipText.trim() === "") return;
                  void onSkip(skipText.trim());
                  setSkipReason(null);
                  setSkipText("");
                }}
              >
                <label htmlFor="lane-skip-reason" style={{ display: "block", marginBottom: 4 }}>
                  Reason for skipping
                </label>
                <input
                  id="lane-skip-reason"
                  type="text"
                  value={skipText}
                  onChange={(e) => setSkipText(e.target.value)}
                  style={{ marginBottom: 8 }}
                />
                <div style={{ display: "flex", gap: 8 }}>
                  <button className="btn btn-primary" type="submit" disabled={skipText.trim() === ""}>
                    Confirm skip
                  </button>
                  <button className="btn btn-secondary" type="button" onClick={() => setSkipReason(null)}>
                    Cancel
                  </button>
                </div>
              </form>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="lane-runner">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12, flexWrap: "wrap" }}>
        <h2 style={{ margin: 0 }}>
          {recordLabel} · {laneName} · stage {currentStageIndex + 1} of {Math.max(stages.length, 1)}
        </h2>
        <div style={{ display: "flex", gap: 8 }}>
          {lastCompleted && !reviewing && (
            <button className="btn btn-secondary" onClick={() => setReviewStepId(lastCompleted.step_id)}>
              Back
            </button>
          )}
          {onClose && (
            <button className="btn btn-secondary" onClick={onClose}>
              Back to record
            </button>
          )}
        </div>
      </div>
      <p style={{ color: "var(--text-muted)" }}>
        Lane {laneName} · status {run.status}
      </p>

      <ol className="lane-stage-bar" aria-label="Stage progress">
        {stages.map((stage, i) => {
          const done = i < currentStageIndex || (stages.length > 0 && run.status === "completed");
          const isCurrent = i === currentStageIndex && run.status !== "completed";
          return (
            <li
              key={stage.id}
              className={done ? "lane-stage done" : isCurrent ? "lane-stage current" : "lane-stage"}
              aria-current={isCurrent ? "step" : undefined}
            >
              <span aria-hidden="true">{done ? "✓" : i + 1}</span> {stage.name}
            </li>
          );
        })}
      </ol>

      <div className="lane-runner-layout">
        <div className="card lane-runner-steps" role="group" aria-label={`Steps — ${stages[currentStageIndex]?.name ?? ""}`}>
          <h3 style={{ marginTop: 0 }}>Steps — {stages[currentStageIndex]?.name ?? ""}</h3>
          {stageSteps.map((step) => {
            const status = stepStatus(run, step.id);
            const isCurrent = step.id === (run.current_step_id ?? currentStep?.step_id);
            return (
              <button
                key={step.id}
                className={`lane-step ${status}`}
                disabled={status !== "completed" && !isCurrent}
                onClick={() => {
                  if (status === "completed") setReviewStepId(step.id);
                  else if (isCurrent) setReviewStepId(null);
                }}
              >
                <span aria-hidden="true">{stepMark(status)}</span> {step.name}
              </button>
            );
          })}
          <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
            A step that is not done yet is locked until the lane reaches it.
          </p>
        </div>

        <div className="lane-runner-main">
          {workArea()}
          <AnytimeActions
            run={run}
            onAddNote={onAddNote}
            onAttachFile={onAttachFile}
            onReassign={onReassign}
            onDownloadAttachment={onDownloadAttachment}
          />
        </div>

        <div className="lane-runner-side">
          <div className="card">
            <h3 style={{ marginTop: 0 }}>{recordLabel}</h3>
            {(recordSummary ?? []).map((row) => (
              <div key={row.label} style={{ display: "flex", justifyContent: "space-between", gap: 12 }}>
                <span style={{ color: "var(--text-muted)" }}>{row.label}</span>
                <strong>{row.value}</strong>
              </div>
            ))}
          </div>
          <div className="card">
            <h3 style={{ marginTop: 0 }}>What this run has written</h3>
            {(run.writes ?? []).length > 0 ? (
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {(run.writes ?? []).map((w, i) => (
                  <li key={i}>{w}</li>
                ))}
              </ul>
            ) : (
              <p style={{ color: "var(--text-muted)" }}>Nothing written yet.</p>
            )}
          </div>
          <div className="card" role="group" aria-label="Audit trail">
            <h3 style={{ marginTop: 0 }}>Audit trail</h3>
            {(run.steps ?? []).length > 0 ? (
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {(run.steps ?? []).map((s) => {
                  // Show the step's name, not its id - the trail is read by
                  // people doing the run, not by the runtime.
                  const stepName =
                    definitionStep(blueprint, s.step_id)?.name ??
                    (s.step_id === (run.current_step_id ?? run.current?.step_id)
                      ? (run.current_step_name ?? run.current?.step_name ?? s.step_id)
                      : s.step_id);
                  return (
                    <li key={s.step_id}>
                      {stepName} — {s.status}
                      {s.started_at ? ` (started ${new Date(s.started_at).toLocaleString()})` : ""}
                      {s.finished_at ? ` → finished ${new Date(s.finished_at).toLocaleString()}` : ""}
                    </li>
                  );
                })}
              </ul>
            ) : (
              <p style={{ color: "var(--text-muted)" }}>Nothing yet.</p>
            )}
            {(run.events ?? []).length > 0 && (
              <>
                <h4 style={{ marginBottom: 4 }}>Notes, files and reassignments</h4>
                <ul style={{ margin: 0, paddingLeft: 20 }}>
                  {(run.events ?? []).map((e, i) => (
                    <li key={`${e.event_type}-${i}`}>
                      {e.summary} · {new Date(e.occurred_at).toLocaleString()}
                    </li>
                  ))}
                </ul>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

/** The runner page: loads the run view (and its blueprint and anchor
 * record, best-effort) through the same `api` module the rest of the app
 * uses, so desktop and Team Workspace behave the same from the same code.
 */
export function LaneRunnerPage({
  runId,
  recordLabel,
  onClose,
}: {
  runId: string;
  recordLabel: string;
  onClose?: () => void;
}) {
  const queryClient = useQueryClient();
  const [error, setError] = useState<string | null>(null);
  const view = useQuery({ queryKey: ["laneRunView", runId], queryFn: () => api.getLaneRunView(runId) });
  // The run view carries no lane key (WP-0704 shape + WP-0707 steps/writes/
  // waiting), but it does carry the anchor record - and the list endpoint
  // returns this run's lane_id, which IS the blueprint key (the definition
  // id must equal the lane key; process_lane_service.rs). So the blueprint
  // is resolved through the record's runs when the view itself names no
  // lane; a view that does name one (lane_key / lane_id) wins.
  const runsForRecord = useQuery({
    queryKey: ["laneRunsForRecord", view.data?.anchor_object_key, view.data?.anchor_record_id],
    queryFn: () =>
      api.listLaneRunsForRecord(
        view.data?.anchor_object_key as string,
        view.data?.anchor_record_id as string,
      ),
    enabled: !!view.data?.anchor_object_key && !!view.data?.anchor_record_id,
  });
  const matchedRun = runsForRecord.data?.find((r) => r.run_id === runId) ?? null;
  const laneKey = view.data?.lane_key ?? view.data?.lane_id ?? matchedRun?.lane_id ?? null;
  const laneName = view.data?.lane_name ?? matchedRun?.lane_name ?? null;
  const blueprint = useQuery({
    queryKey: ["laneBlueprint", laneKey],
    queryFn: () => api.getLaneBlueprint(laneKey as string),
    enabled: laneKey !== null,
  });
  const anchorRecord = useQuery({
    queryKey: ["laneRunAnchorRecord", view.data?.anchor_record_id],
    queryFn: async () => {
      try {
        return await api.getCustomRecord(view.data?.anchor_record_id as string);
      } catch {
        return null;
      }
    },
    enabled: !!view.data?.anchor_record_id,
  });
  const submit = useMutation({
    mutationFn: (values: Record<string, unknown>) => api.submitLaneStep(runId, values),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["laneRunView", runId] }),
  });
  const skip = useMutation({
    mutationFn: (reason: string) => api.skipLaneStep(runId, reason),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["laneRunView", runId] }),
  });
  // WP-0711: the anytime actions invalidate the same run view, so the
  // note, file and reassignment show up the moment they land.
  const addNote = useMutation({
    mutationFn: (body: string) => api.addLaneRunNote(runId, body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["laneRunView", runId] }),
  });
  const attachFile = useMutation({
    mutationFn: (input: { fileName: string; mimeType: string | undefined; contentBase64: string }) =>
      api.attachLaneRunFile(runId, input.fileName, input.mimeType, input.contentBase64),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["laneRunView", runId] }),
  });
  const reassign = useMutation({
    mutationFn: (input: { toUserId: string; reason: string }) =>
      api.reassignLaneRunStep(runId, input.toUserId, input.reason),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["laneRunView", runId] }),
  });

  async function handleSubmit(values: Record<string, unknown>) {
    setError(null);
    try {
      await submit.mutateAsync(values);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not complete this step");
    }
  }

  async function handleSkip(reason: string) {
    setError(null);
    try {
      await skip.mutateAsync(reason);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not skip this step");
    }
  }

  async function handleAddNote(body: string) {
    setError(null);
    try {
      await addNote.mutateAsync(body);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not add this note");
    }
  }

  async function handleAttachFile(fileName: string, mimeType: string | undefined, contentBase64: string) {
    setError(null);
    try {
      await attachFile.mutateAsync({ fileName, mimeType, contentBase64 });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not attach this file");
    }
  }

  async function handleReassign(toUserId: string, reason: string) {
    setError(null);
    try {
      await reassign.mutateAsync({ toUserId, reason });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not reassign this step");
    }
  }

  async function handleDownloadAttachment(attachment: LaneRunAttachmentView) {
    setError(null);
    try {
      await api.downloadLaneRunAttachment(runId, attachment);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not download this file");
    }
  }

  if (view.isLoading) return <p>Loading run…</p>;
  if (view.error || !view.data) {
    return <p>Could not load this run: {(view.error as Error | null)?.message ?? "unknown error"}</p>;
  }

  const recordSummary: RecordSummaryRow[] = anchorRecord.data
    ? [
        { label: "Status", value: anchorRecord.data.status },
        { label: "Number", value: anchorRecord.data.display_number },
      ]
    : [];

  // The list item names the lane even when the run view does not, so the
  // run label (REQ-LANE-003) still carries the lane name.
  const enrichedView: LaneRunView = {
    ...view.data,
    lane_name: view.data.lane_name ?? laneName ?? undefined,
  };

  return (
    <>
      {error && (
        <p role="alert" style={{ color: "var(--danger, #b91c1c)" }}>
          {error}
        </p>
      )}
      <LaneRunnerView
        run={enrichedView}
        blueprint={blueprint.data?.definition ?? null}
        recordLabel={recordLabel}
        recordSummary={recordSummary}
        onSubmit={handleSubmit}
        onSkip={handleSkip}
        onAddNote={handleAddNote}
        onAttachFile={handleAttachFile}
        onReassign={handleReassign}
        onDownloadAttachment={handleDownloadAttachment}
        onClose={onClose}
      />
    </>
  );
}
