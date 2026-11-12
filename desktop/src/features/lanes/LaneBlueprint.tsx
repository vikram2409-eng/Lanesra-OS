import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type {
  LaneBlueprint,
  LaneBranch,
  LaneDefinition,
  LaneStep,
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

/// The Process Lanes page: pick a lane, read its blueprint, print it.
/// Desktop and Team Workspace run the same component against the same
/// commands (WP-0712 AC-4).
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
