import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  entityTypeLabel,
  transitionFieldFor,
  transitionValuesForEntity,
  TRANSITION_ENTITY_TYPES,
  type StatusTransitionInput,
} from "../../lib/types";

/**
 * Admin Automation & Customization addendum, Phase 2 (spec §2.5): a
 * dedicated Status Transition editor rather than forcing administrators to
 * model every allowed From -> To state change as a generic business rule.
 * With zero rules for an entity type, transitions stay fully unrestricted
 * (today's behavior) - adding the first rule turns that entity's
 * transitions into an allow-list enforced server-side wherever its
 * status/stage actually changes.
 */
export function StatusTransitionsAdmin() {
  const [entityType, setEntityType] = useState<string>(TRANSITION_ENTITY_TYPES[0]);
  const [adding, setAdding] = useState(false);
  const queryClient = useQueryClient();

  const rules = useQuery({ queryKey: ["statusTransitions", entityType], queryFn: () => api.listStatusTransitions(entityType) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["statusTransitions"] });
  }

  const field = transitionFieldFor(entityType);
  const values = transitionValuesForEntity(entityType);

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Status transitions</h3>
        <button className="btn btn-primary" onClick={() => setAdding((v) => !v)}>
          + New rule
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Restrict which {field} changes are allowed. With no rules for an object, every {field} change is allowed, same
        as today. Add the first rule and only the listed From → To pairs are allowed from then on.
      </p>

      <div className="tab-row">
        {TRANSITION_ENTITY_TYPES.map((t) => (
          <button
            key={t}
            className={`tab${entityType === t ? " active" : ""}`}
            onClick={() => {
              setEntityType(t);
              setAdding(false);
            }}
          >
            {entityTypeLabel(t)}
          </button>
        ))}
      </div>

      {adding && (
        <RuleForm
          entityType={entityType}
          values={values}
          onDone={() => {
            invalidate();
            setAdding(false);
          }}
          onCancel={() => setAdding(false)}
        />
      )}

      {rules.isLoading && <p>Loading...</p>}
      {rules.data && rules.data.length === 0 && (
        <p className="empty-state">
          No rules on {entityTypeLabel(entityType)} yet - every {field} change is currently allowed.
        </p>
      )}
      {rules.data && rules.data.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>From</th>
              <th>To</th>
              <th>Active</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rules.data.map((r) => (
              <RuleRow key={r.id} rule={r} onChanged={invalidate} />
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function RuleRow({
  rule,
  onChanged,
}: {
  rule: { id: string; from_status: string | null; to_status: string; is_active: boolean };
  onChanged: () => void;
}) {
  const toggle = useMutation({
    mutationFn: () => api.setStatusTransitionActive(rule.id, !rule.is_active),
    onSuccess: onChanged,
  });
  const remove = useMutation({
    mutationFn: () => api.deleteStatusTransition(rule.id),
    onSuccess: onChanged,
  });

  return (
    <tr>
      <td>{rule.from_status ?? <em>Any status</em>}</td>
      <td>{rule.to_status}</td>
      <td>
        <button className={`badge${rule.is_active ? " badge-success" : ""}`} onClick={() => toggle.mutate()} disabled={toggle.isPending}>
          {rule.is_active ? "Active" : "Inactive"}
        </button>
      </td>
      <td>
        <button className="btn" onClick={() => remove.mutate()} disabled={remove.isPending}>
          Delete
        </button>
      </td>
    </tr>
  );
}

function RuleForm({
  entityType,
  values,
  onDone,
  onCancel,
}: {
  entityType: string;
  values: readonly string[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [fromAny, setFromAny] = useState(true);
  const [fromStatus, setFromStatus] = useState(values[0] ?? "");
  const [toStatus, setToStatus] = useState(values[0] ?? "");
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () => {
      const input: StatusTransitionInput = { entity_type: entityType, from_status: fromAny ? null : fromStatus, to_status: toStatus };
      return api.createStatusTransition(input);
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this rule"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field">
          <label>From</label>
          <select
            value={fromAny ? "any" : fromStatus}
            onChange={(e) => {
              if (e.target.value === "any") {
                setFromAny(true);
              } else {
                setFromAny(false);
                setFromStatus(e.target.value);
              }
            }}
          >
            <option value="any">Any status</option>
            {values.map((v) => <option key={v} value={v}>{v}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>To</label>
          <select value={toStatus} onChange={(e) => setToStatus(e.target.value)}>
            {values.map((v) => <option key={v} value={v}>{v}</option>)}
          </select>
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Add rule
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
