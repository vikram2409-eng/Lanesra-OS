import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type {
  DryRunFixtures,
  DryRunReport,
  LaneBlueprint,
  LaneBranch,
  LaneDefinition,
  LaneRunDataField,
  LaneStep,
  PublishReport,
} from "../../lib/types";
import { PrintOverlay } from "../../components/PrintOverlay";

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

/// One condition (or group) in plain words, so the blueprint says when a
/// lane starts and when a stage applies instead of showing raw JSON.
function describeCondition(condition: unknown): string {
  if (!condition || typeof condition !== "object") return "";
  const obj = condition as Record<string, unknown>;
  if (typeof obj.field === "string") {
    const compare =
      typeof obj.compare_field === "string"
        ? obj.compare_field
        : obj.value === undefined
          ? ""
          : JSON.stringify(obj.value);
    return `${obj.field} ${obj.operator ?? ""} ${compare}`.trim();
  }
  if (typeof obj.match === "string" && Array.isArray(obj.conditions)) {
    const parts = (obj.conditions as unknown[]).map(describeCondition).filter(Boolean);
    return `${obj.match} of (${parts.join("; ")})`;
  }
  return "";
}

function branchTargetLabel(
  branch: LaneBranch,
  definition: LaneDefinition,
): string {
  if (branch.end) return `ends the run (${branch.end.outcome})`;
  if (!branch.target) return "no target";
  const stage = definition.stages.find((s) => s.id === branch.target?.stage);
  const step = stage?.steps.find((s) => s.id === branch.target?.step);
  if (stage && step) return `${stage.name} · ${step.name}`;
  if (stage) return stage.name;
  return `${branch.target.stage} · ${branch.target.step}`;
}

function StepDetail({ step, definition }: { step: LaneStep; definition: LaneDefinition }) {
  return (
    <div className="card" style={{ marginBottom: 12 }}>
      <div style={{ display: "flex", gap: 8, alignItems: "baseline", flexWrap: "wrap" }}>
        <span className="badge">{stepKindLabel(step.kind)}</span>
        <strong>{step.name}</strong>
        {step.role && <span style={{ color: "var(--text-muted)" }}>· {step.role}</span>}
        {step.required === false && <span style={{ color: "var(--text-muted)" }}>· optional</span>}
      </div>
      {step.description && <p style={{ margin: "8px 0 0" }}>{step.description}</p>}
      {step.repeat_over && (
        <p style={{ margin: "8px 0 0", color: "var(--text-muted)" }}>
          Repeats once per {step.repeat_over.field}.
        </p>
      )}
      {step.assistance && (
        <p style={{ margin: "8px 0 0", color: "var(--text-muted)" }}>
          Agent assistance: {step.assistance.autonomy}
          {step.assistance.agent ? ` (${step.assistance.agent})` : ""}
        </p>
      )}
      {step.check && (
        <p style={{ margin: "8px 0 0" }}>
          Checks: {step.check.message} ({step.check.scope}
          {step.check.aggregate ? `, ${step.check.aggregate}` : ""})
        </p>
      )}
      {step.fields && step.fields.length > 0 && (
        <table style={{ marginTop: 8 }}>
          <thead>
            <tr>
              <th>Field</th>
              <th>Type</th>
              <th>Stored as</th>
            </tr>
          </thead>
          <tbody>
            {step.fields.map((f) => (
              <tr key={f.key}>
                <td>
                  {f.label}
                  {f.required ? " *" : ""}
                </td>
                <td>{f.type}</td>
                <td>
                  <code>{f.binds_to}</code>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {step.writes && step.writes.length > 0 && (
        <div style={{ marginTop: 8 }}>
          <strong>Writes</strong>
          <ul style={{ margin: "4px 0 0", paddingLeft: 20 }}>
            {step.writes.map((w, i) => (
              <li key={i}>
                <code>{w.action_type}</code>
                {Object.entries(w.inputs).map(([k, v]) => (
                  <span key={k}>
                    {" "}
                    · {k}: <code>{JSON.stringify(v)}</code>
                  </span>
                ))}
              </li>
            ))}
          </ul>
        </div>
      )}
      {step.branches && step.branches.length > 0 && (
        <div style={{ marginTop: 8 }}>
          <strong>Branches</strong>
          <ul style={{ margin: "4px 0 0", paddingLeft: 20 }}>
            {step.branches.map((b, i) => (
              <li key={i}>
                {b.on} → {branchTargetLabel(b, definition)}
              </li>
            ))}
          </ul>
        </div>
      )}
      {step.ends_run && (
        <p style={{ margin: "8px 0 0" }}>Ends the run with outcome {step.ends_run.outcome}.</p>
      )}
      {step.notify &&
        step.notify.map((n, i) => (
          <p key={i} style={{ margin: "8px 0 0", color: "var(--text-muted)" }}>
            Notifies {n.to_role}: {n.message}
          </p>
        ))}
      {step.replaces &&
        step.replaces.map((r, i) => (
          <p key={i} style={{ margin: "8px 0 0", color: "var(--text-muted)" }}>
            Replaces workflow {r.workflow_id}, action {r.action_index}.
          </p>
        ))}
      {step.child_lane && (
        <p style={{ margin: "8px 0 0", color: "var(--text-muted)" }}>
          Starts lane {step.child_lane.lane_id} on {step.child_lane.anchor_record}.
        </p>
      )}
    </div>
  );
}

/// The read-only blueprint of one lane: purpose, stage chevrons with their
/// steps (kind, role, writes), run-data fields and mapped record fields.
/// Rendered from the definition only - no run, no record values - so a
/// Viewer reads exactly what an Administrator reads. The same component
/// renders on screen and inside PrintOverlay, so the export matches the
/// view (WP-0712 AC-2).
export function LaneBlueprintView({ blueprint }: { blueprint: LaneBlueprint }) {
  const definition = blueprint.definition;
  const mappedFields = definition.stages.flatMap((stage) =>
    stage.steps.flatMap((step) =>
      (step.fields ?? [])
        .filter((f) => f.binds_to.startsWith("record."))
        .map((f) => ({ step: step.name, field: f })),
    ),
  );
  const startCondition = describeCondition(definition.start_condition);

  return (
    <div>
      <div style={{ display: "flex", gap: 8, alignItems: "baseline", flexWrap: "wrap" }}>
        <h2 style={{ margin: 0 }}>{definition.name}</h2>
        <span className="badge">
          v{blueprint.version} · {blueprint.status}
        </span>
        <span style={{ color: "var(--text-muted)" }}>Runs on {definition.object_key}</span>
      </div>
      {definition.description && <p>{definition.description}</p>}
      {startCondition && (
        <p style={{ color: "var(--text-muted)" }}>Starts when {startCondition}.</p>
      )}
      {definition.excludes && definition.excludes.length > 0 && (
        <p style={{ color: "var(--text-muted)" }}>
          Never runs at the same time as: {definition.excludes.join(", ")}.
        </p>
      )}

      <h3>Stages</h3>
      <div
        style={{ display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center", marginBottom: 16 }}
        aria-label="Stage order"
      >
        {definition.stages.map((stage, i) => (
          <span key={stage.id} style={{ display: "flex", alignItems: "center", gap: 6 }}>
            {i > 0 && <span aria-hidden="true">›</span>}
            <span className="badge">
              {i + 1}. {stage.name}
            </span>
          </span>
        ))}
      </div>

      {definition.stages.map((stage, i) => {
        const entry = describeCondition(stage.entry_condition);
        const skip = describeCondition(stage.skip_condition);
        return (
          <section key={stage.id} style={{ marginBottom: 20 }}>
            <h3 style={{ marginBottom: 4 }}>
              Stage {i + 1} of {definition.stages.length} — {stage.name}
            </h3>
            {entry && (
              <p style={{ margin: "0 0 8px", color: "var(--text-muted)" }}>Runs when {entry}.</p>
            )}
            {skip && (
              <p style={{ margin: "0 0 8px", color: "var(--text-muted)" }}>Skipped when {skip}.</p>
            )}
            {stage.steps.map((step) => (
              <StepDetail key={step.id} step={step} definition={definition} />
            ))}
          </section>
        );
      })}

      {definition.run_data && definition.run_data.fields.length > 0 && (
        <section style={{ marginBottom: 20 }}>
          <h3>Run data</h3>
          <p style={{ color: "var(--text-muted)" }}>
            Values the lane collects as it runs, declared up front - every entered value lands
            here or on the record, never nowhere.
          </p>
          <table>
            <thead>
              <tr>
                <th>Field</th>
                <th>Type</th>
                <th>Required</th>
                <th>Per line</th>
                <th>What it holds</th>
              </tr>
            </thead>
            <tbody>
              {definition.run_data.fields.map((f) => (
                <tr key={f.key}>
                  <td>
                    <code>{f.key}</code>
                  </td>
                  <td>{f.type}</td>
                  <td>{f.required ? "Yes" : "No"}</td>
                  <td>{f.per_child ? "Yes" : "No"}</td>
                  <td>{f.description ?? ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}

      {mappedFields.length > 0 && (
        <section style={{ marginBottom: 20 }}>
          <h3>Mapped record fields</h3>
          <p style={{ color: "var(--text-muted)" }}>
            Fields a step writes straight onto the anchor record. The blueprint names the
            mapping, never a record's values.
          </p>
          <table>
            <thead>
              <tr>
                <th>Field</th>
                <th>Step</th>
                <th>Maps to</th>
              </tr>
            </thead>
            <tbody>
              {mappedFields.map((m) => (
                <tr key={`${m.step}:${m.field.key}`}>
                  <td>{m.field.label}</td>
                  <td>{m.step}</td>
                  <td>
                    <code>{m.field.binds_to}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}

      <p style={{ color: "var(--text-muted)" }}>
        This blueprint shows the lane definition only - no run, no record data.
      </p>
    </div>
  );
}

const PUBLISH_STATUS_BADGE: Record<string, string> = {
  pass: "badge-success",
  warn: "badge-warning",
  refuse: "badge-danger",
};

/// WP-0714 (REQ-LANE-021 AC-2): the publish checks P-01..P-16 for one
/// lane, and the Publish action. Publish is refused while any check
/// refuses, and the refusal names every failed check - the backend's
/// refusal message lists them all, and the check list below names each
/// one with its id, step, status and reason. A warning never blocks
/// publish; it is listed for the author to review.
function PublishChecksSection({ blueprint }: { blueprint: LaneBlueprint }) {
  const queryClient = useQueryClient();
  const [report, setReport] = useState<PublishReport | null>(null);
  const [checking, setChecking] = useState(false);
  const [publishing, setPublishing] = useState(false);
  const [publishedVersion, setPublishedVersion] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function runChecks() {
    setChecking(true);
    setError(null);
    try {
      setReport(await api.checkLanePublish(blueprint.lane_key));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not run the publish checks.");
    } finally {
      setChecking(false);
    }
  }

  async function publish() {
    setPublishing(true);
    setError(null);
    setPublishedVersion(null);
    try {
      const version = await api.publishLane(blueprint.lane_key);
      setPublishedVersion(version.version);
      await queryClient.invalidateQueries({ queryKey: ["laneBlueprint", blueprint.lane_key] });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Publishing failed.");
      // A refusal must show the full list of named reasons, not just the
      // first: if no report is loaded yet, run the checks so the refused
      // checks are listed below alongside the refusal message.
      if (!report) {
        try {
          setReport(await api.checkLanePublish(blueprint.lane_key));
        } catch {
          // The refusal message itself is the evidence; no report, no list.
        }
      }
    } finally {
      setPublishing(false);
    }
  }

  const refuses = report?.checks.filter((c) => c.status === "refuse") ?? [];
  const warns = report?.checks.filter((c) => c.status === "warn") ?? [];

  return (
    <section className="card" aria-labelledby="publish-checks-heading">
      <h3 id="publish-checks-heading">Publish checks</h3>
      <p style={{ color: "var(--text-muted)" }}>
        Publishing stays a separate action by a person, from the blueprint view or the lane
        commands. Run the checks first: publish is refused while any check refuses, and the
        refusal names every failed check.
      </p>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
        <button className="btn" onClick={runChecks} disabled={checking}>
          {checking ? "Running checks…" : "Run publish checks"}
        </button>
        <button className="btn" onClick={publish} disabled={publishing}>
          {publishing ? "Publishing…" : "Publish"}
        </button>
      </div>
      {publishedVersion !== null && (
        <p style={{ color: "var(--success)" }}>Published version {publishedVersion}.</p>
      )}
      {error && (
        <p role="alert" style={{ color: "var(--danger)" }}>
          <strong>Publish refused.</strong> {error}
        </p>
      )}
      {report && (
        <>
          <p style={{ marginBottom: 8 }}>
            {report.refused
              ? `Cannot publish — ${refuses.length} check${refuses.length === 1 ? "" : "s"} refuse${refuses.length === 1 ? "s" : ""}.`
              : "Ready to publish — every check passes."}
            {warns.length > 0 &&
              ` ${warns.length} warning${warns.length === 1 ? "" : "s"} to review.`}
          </p>
          <ul aria-label="Publish checks" style={{ paddingLeft: 20, margin: 0 }}>
            {report.checks.map((check) => (
              <li key={check.id} style={{ marginBottom: 8 }}>
                <span className={`badge ${PUBLISH_STATUS_BADGE[check.status] ?? ""}`}>
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
        </>
      )}
    </section>
  );
}

function fixtureInputType(field: LaneRunDataField): string {
  if (field.type === "integer" || field.type === "number") return "number";
  if (field.type === "date") return "date";
  if (field.type === "boolean") return "checkbox";
  return "text";
}

function parseFixtureValue(field: LaneRunDataField, raw: string): unknown {
  if (field.type === "integer" || field.type === "number") {
    const n = Number(raw);
    return raw.trim() !== "" && Number.isFinite(n) ? n : raw;
  }
  if (field.type === "boolean") return raw === "true";
  return raw;
}

/// WP-0714 (REQ-LANE-021 AC-1): the dry run. The author picks the anchor
/// record, supplies the values a person would type (run-data fields,
/// per-line values for repeating steps, approval decisions) and names the
/// expected outcome; the report shows the walk - stages visited, steps
/// executed or skipped with reasons, checks passed or failed with reasons,
/// branches taken, writes simulated with resolved inputs, and the end
/// outcome against the expectation. A dry run writes nothing: no status
/// change, no stock move, no ledger row, no approval created.
function DryRunSection({ blueprint }: { blueprint: LaneBlueprint }) {
  const definition = blueprint.definition;
  const runDataFields = definition.run_data?.fields ?? [];
  const approvalSteps = definition.stages
    .flatMap((stage) => stage.steps)
    .filter((step) => step.kind === "approval");
  const [anchorRecordId, setAnchorRecordId] = useState("");
  const [fixtureText, setFixtureText] = useState<Record<string, string>>({});
  const [approvalValues, setApprovalValues] = useState<Record<string, "approved" | "rejected">>({});
  const [expectedOutcome, setExpectedOutcome] = useState("");
  const [report, setReport] = useState<DryRunReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const records = useQuery({
    queryKey: ["laneDryRunRecords", definition.object_key],
    queryFn: () => api.listCustomRecords(definition.object_key),
  });

  function stageName(stageId: string): string {
    return definition.stages.find((s) => s.id === stageId)?.name ?? stageId;
  }

  /// The report carries step ids, not names - resolve them against the
  /// definition this section already renders from.
  function stepName(stepId: string): string {
    for (const stage of definition.stages) {
      const step = stage.steps.find((s) => s.id === stepId);
      if (step) return step.name;
    }
    return stepId;
  }

  /// A branch target is `"stage/step"` or `"end:<outcome>"`.
  function branchTargetLabel(target: string): string {
    if (target.startsWith("end:")) return `ends (${target.slice(4)})`;
    const [stageId, stepId] = target.split("/");
    if (stepId) {
      const stage = definition.stages.find((s) => s.id === stageId);
      const step = stage?.steps.find((s) => s.id === stepId);
      if (step) return step.name;
    }
    return target;
  }

  function setFixture(key: string, value: string) {
    setFixtureText((prev) => ({ ...prev, [key]: value }));
  }

  async function runDryRun() {
    setFormError(null);
    setError(null);
    const fixtures: DryRunFixtures = { run_data: {} };
    // Per-line values arrive as one JSON object per field
    // (`{"line-1": 40}`) and are transposed into the shape the dry run
    // takes: per_child[child_id][field].
    const perChild: Record<string, Record<string, unknown>> = {};
    for (const field of runDataFields) {
      const raw = fixtureText[field.key];
      if (raw === undefined || raw === "") continue;
      if (field.per_child) {
        // Per-line values are keyed by child id, so the author enters
        // them as one JSON object; a bare value cannot say which line
        // it belongs to, so it is a form error, not a guess.
        let parsed: unknown;
        try {
          parsed = JSON.parse(raw);
        } catch {
          setFormError(
            `Per-line values for '${field.key}' must be JSON keyed by line id, for example {"line-1": 40}.`,
          );
          return;
        }
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
          setFormError(
            `Per-line values for '${field.key}' must be a JSON object keyed by line id, for example {"line-1": 40}.`,
          );
          return;
        }
        for (const [childId, value] of Object.entries(parsed as Record<string, unknown>)) {
          if (!perChild[childId]) perChild[childId] = {};
          perChild[childId][field.key] = value;
        }
      } else {
        fixtures.run_data[field.key] = parseFixtureValue(field, raw);
      }
    }
    if (Object.keys(perChild).length > 0) fixtures.per_child = perChild;
    if (approvalSteps.length > 0) {
      fixtures.approvals = {};
      for (const step of approvalSteps) {
        fixtures.approvals[step.id] = approvalValues[step.id] ?? "approved";
      }
    }
    setRunning(true);
    try {
      setReport(
        await api.dryRunLane(
          blueprint.lane_key,
          anchorRecordId,
          fixtures,
          expectedOutcome.trim() || undefined,
        ),
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : "The dry run failed.");
    } finally {
      setRunning(false);
    }
  }

  return (
    <section className="card" aria-labelledby="dry-run-heading">
      <h3 id="dry-run-heading">Dry run</h3>
      <p style={{ color: "var(--text-muted)" }}>
        A dry run walks the lane against a seed record and writes nothing — no status change, no
        stock move, no ledger row, no approval created. Supply the values a person would type,
        then read the walk before publishing.
      </p>

      <div style={{ marginBottom: 12 }}>
        <label htmlFor="dry-run-anchor-record">Anchor record</label>
        <select
          id="dry-run-anchor-record"
          value={anchorRecordId}
          onChange={(e) => setAnchorRecordId(e.target.value)}
        >
          <option value="">Choose a record…</option>
          {records.data?.map((record) => (
            <option key={record.id} value={record.id}>
              {record.primary_name} ({record.display_number})
            </option>
          ))}
        </select>
        {records.isLoading && (
          <p style={{ color: "var(--text-muted)" }}>Loading records…</p>
        )}
        {records.error && (
          <p style={{ color: "var(--danger)" }}>
            Could not load records: {(records.error as Error).message}
          </p>
        )}
        {records.data && records.data.length === 0 && (
          <p style={{ color: "var(--text-muted)" }}>
            No records of type {definition.object_key} yet — create one before running a dry run.
          </p>
        )}
      </div>

      {runDataFields.length > 0 && (
        <fieldset style={{ border: 0, padding: 0, margin: "0 0 12px" }}>
          <legend>Fixture values</legend>
          {runDataFields.map((field) => {
            const inputId = `dry-run-fixture-${field.key}`;
            const label = field.per_child ? `${field.key} (per line)` : field.key;
            // A per-line value is a JSON object keyed by line id, so its
            // input is always text - a number input would reject the JSON.
            const type = field.per_child ? "text" : fixtureInputType(field);
            if (type === "checkbox") {
              return (
                <div key={field.key} style={{ marginBottom: 8 }}>
                  <label htmlFor={inputId}>{label}</label>{" "}
                  <input
                    id={inputId}
                    type="checkbox"
                    checked={fixtureText[field.key] === "true"}
                    onChange={(e) => setFixture(field.key, e.target.checked ? "true" : "false")}
                  />
                  {field.description && (
                    <p style={{ margin: "4px 0 0", color: "var(--text-muted)" }}>
                      {field.description}
                    </p>
                  )}
                </div>
              );
            }
            return (
              <div key={field.key} style={{ marginBottom: 8 }}>
                <label htmlFor={inputId}>{label}</label>
                <input
                  id={inputId}
                  type={type}
                  value={fixtureText[field.key] ?? ""}
                  placeholder={field.per_child ? '{"line-1": value, ...}' : undefined}
                  onChange={(e) => setFixture(field.key, e.target.value)}
                />
                {field.description && (
                  <p style={{ margin: "4px 0 0", color: "var(--text-muted)" }}>
                    {field.description}
                  </p>
                )}
              </div>
            );
          })}
        </fieldset>
      )}

      {approvalSteps.map((step) => (
        <div key={step.id} style={{ marginBottom: 8 }}>
          <label htmlFor={`dry-run-approval-${step.id}`}>Approval: {step.name}</label>
          <select
            id={`dry-run-approval-${step.id}`}
            value={approvalValues[step.id] ?? "approved"}
            onChange={(e) =>
              setApprovalValues((prev) => ({
                ...prev,
                [step.id]: e.target.value === "approved" ? "approved" : "rejected",
              }))
            }
          >
            <option value="approved">Approve</option>
            <option value="rejected">Reject</option>
          </select>
        </div>
      ))}

      <div style={{ marginBottom: 12 }}>
        <label htmlFor="dry-run-expected-outcome">Expected outcome</label>
        <input
          id="dry-run-expected-outcome"
          type="text"
          value={expectedOutcome}
          placeholder="for example received"
          onChange={(e) => setExpectedOutcome(e.target.value)}
        />
      </div>

      <button className="btn" onClick={runDryRun} disabled={!anchorRecordId || running}>
        {running ? "Running dry run…" : "Run dry run"}
      </button>
      {formError && (
        <p role="alert" style={{ color: "var(--danger)" }}>
          {formError}
        </p>
      )}
      {error && (
        <p role="alert" style={{ color: "var(--danger)" }}>
          {error}
        </p>
      )}

      {report && (
        <div role="region" aria-label="Dry-run report" style={{ marginTop: 16 }}>
          <h4>Dry-run report</h4>
          <p style={{ color: "var(--text-muted)" }}>
            Version {report.lane_version} on {report.anchor_object_key}. Simulated only — nothing
            was written.
          </p>
          {!report.would_start && (
            <p role="alert" style={{ color: "var(--danger)" }}>
              Would refuse to start: {report.refusal_reason ?? "start condition is not met"}
            </p>
          )}
          {report.stopped_reason && <p>Stopped: {report.stopped_reason}</p>}
          <p>
            Stages visited: {report.stages_visited.map(stageName).join(", ") || "none"}.
          </p>
          <h4>Steps</h4>
          <ul aria-label="Dry-run steps" style={{ paddingLeft: 20, margin: 0 }}>
            {report.steps.map((step) => (
              <li key={step.step_id} style={{ marginBottom: 4 }}>
                <strong>{stepName(step.step_id)}</strong> — {step.outcome}
                {step.reason ? ` — ${step.reason}` : ""}
              </li>
            ))}
          </ul>
          <h4>Checks</h4>
          {report.checks.length === 0 ? (
            <p style={{ color: "var(--text-muted)" }}>No checks evaluated on this walk.</p>
          ) : (
            <ul aria-label="Dry-run checks" style={{ paddingLeft: 20, margin: 0 }}>
              {report.checks.map((check) => (
                <li key={check.step_id} style={{ marginBottom: 4 }}>
                  {stepName(check.step_id)} — {check.outcome}:{" "}
                  {check.reason ?? "No reason given."}
                  {check.per_child && check.per_child.length > 0 && (
                    <ul style={{ paddingLeft: 20, margin: "4px 0 0" }}>
                      {check.per_child.map((child) => (
                        <li key={child.child_id}>
                          {child.child_id} — {child.outcome}
                          {child.reason ? `: ${child.reason}` : ""}
                        </li>
                      ))}
                    </ul>
                  )}
                </li>
              ))}
            </ul>
          )}
          <h4>Branches taken</h4>
          {report.branches_taken.length === 0 ? (
            <p style={{ color: "var(--text-muted)" }}>No branches taken on this walk.</p>
          ) : (
            <ul aria-label="Dry-run branches" style={{ paddingLeft: 20, margin: 0 }}>
              {report.branches_taken.map((branch, i) => (
                <li key={`${branch.step_id}:${i}`} style={{ marginBottom: 4 }}>
                  {branch.step_id} — {branch.on} → {branchTargetLabel(branch.target)}
                </li>
              ))}
            </ul>
          )}
          <h4>Writes simulated</h4>
          {report.writes.length === 0 ? (
            <p style={{ color: "var(--text-muted)" }}>No writes simulated on this walk.</p>
          ) : (
            <ul aria-label="Dry-run writes" style={{ paddingLeft: 20, margin: 0 }}>
              {report.writes.map((write, i) => (
                <li key={`${write.step_id}:${i}`} style={{ marginBottom: 4 }}>
                  <strong>{stepName(write.step_id)}</strong> — <code>{write.action_type}</code>
                  {write.child_id ? ` (line ${write.child_id})` : ""} —{" "}
                  {Object.entries(write.inputs)
                    .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
                    .join(", ")}
                  {write.zero_quantity ? " — zero-quantity, would post nothing" : ""}
                </li>
              ))}
            </ul>
          )}
          <p>
            End outcome: {report.end_outcome ?? "none"}.
            {report.expected_outcome !== null &&
              (report.matches_expected
                ? " Matches the expected outcome."
                : ` Does not match the expected outcome (expected ${report.expected_outcome}, got ${report.end_outcome ?? "none"}).`)}
          </p>
          {report.zero_quantity_flags.length > 0 && (
            <>
              <h4>Zero-quantity writes flagged</h4>
              <ul aria-label="Zero-quantity writes" style={{ paddingLeft: 20, margin: 0 }}>
                {report.zero_quantity_flags.map((flag, i) => (
                  <li key={`${flag.step_id}:${i}`} style={{ marginBottom: 4 }}>
                    <strong>{stepName(flag.step_id)}</strong> — {flag.message} (
                    {flag.skipped_children} child{flag.skipped_children === 1 ? "" : "ren"}{" "}
                    skipped)
                  </li>
                ))}
              </ul>
            </>
          )}
          {report.unevaluated.length > 0 && (
            <>
              <h4>Unevaluated checks</h4>
              <ul aria-label="Unevaluated checks" style={{ paddingLeft: 20, margin: 0 }}>
                {report.unevaluated.map((check) => (
                  <li key={check.step_id} style={{ marginBottom: 4 }}>
                    <strong>{stepName(check.step_id)}</strong> — {check.reason}
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
      )}
    </section>
  );
}

/// The Process Lanes page: pick a lane, read its blueprint, print it.
/// Desktop and Team Workspace run the same component against the same
/// commands (WP-0712 AC-4).
///
/// WP-0714 (REQ-LANE-021): the blueprint page is also the publish
/// surface. The publish-checks section runs the P-01..P-16 checks and
/// publishes the lane; the dry-run section walks the lane against a seed
/// record and writes nothing. Neither section renders inside PrintOverlay
/// - the print export stays the definition only (WP-0712 AC-2).
export function LaneBlueprintPage() {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [printing, setPrinting] = useState(false);

  const list = useQuery({
    queryKey: ["laneBlueprints"],
    queryFn: () => api.listLaneBlueprints(),
  });
  const selected = selectedKey ?? list.data?.[0]?.lane_key ?? null;
  const detail = useQuery({
    queryKey: ["laneBlueprint", selected],
    queryFn: () => api.getLaneBlueprint(selected as string),
    enabled: selected !== null,
  });

  return (
    <div>
      <h2>Process Lanes</h2>
      <p style={{ color: "var(--text-muted)" }}>
        A read-only blueprint of each lane - what it does, stage by stage, before anyone runs
        it. No record data is shown here.
      </p>

      {list.isLoading && <p>Loading lanes…</p>}
      {list.error && <p>Could not load lanes: {(list.error as Error).message}</p>}
      {list.data && list.data.length === 0 && (
        <p>No lanes yet. Import a lane definition to see its blueprint here.</p>
      )}
      {list.data && list.data.length > 0 && (
        <div className="card" style={{ marginBottom: 16 }}>
          <table>
            <thead>
              <tr>
                <th>Lane</th>
                <th>Runs on</th>
                <th>Version</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {list.data.map((lane) => (
                <tr
                  key={lane.lane_key}
                  onClick={() => setSelectedKey(lane.lane_key)}
                  style={{
                    cursor: "pointer",
                    background: lane.lane_key === selected ? "var(--bg)" : undefined,
                  }}
                >
                  <td>
                    <strong>{lane.name}</strong>
                  </td>
                  <td>{lane.object_key}</td>
                  <td>v{lane.latest_version}</td>
                  <td>{lane.latest_status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {detail.isLoading && <p>Loading blueprint…</p>}
      {detail.error && <p>Could not load the blueprint: {(detail.error as Error).message}</p>}
      {detail.data && (
        <>
          <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 12 }}>
            <button className="btn" onClick={() => setPrinting(true)}>
              Print / Save as PDF
            </button>
          </div>
          <LaneBlueprintView blueprint={detail.data} />
          <div className="lane-publish-layout">
            <PublishChecksSection blueprint={detail.data} />
            <DryRunSection blueprint={detail.data} />
          </div>
          {printing && (
            <PrintOverlay onClose={() => setPrinting(false)}>
              <LaneBlueprintView blueprint={detail.data} />
            </PrintOverlay>
          )}
        </>
      )}
    </div>
  );
}
