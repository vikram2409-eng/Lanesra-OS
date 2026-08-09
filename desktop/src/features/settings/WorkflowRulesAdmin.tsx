import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  INVOICE_STATUSES,
  OPPORTUNITY_STAGES,
  WORKFLOW_ENTITY_TYPES,
  type WorkflowEntityType,
  type WorkflowRule,
  type WorkflowRuleInput,
} from "../../lib/types";

function emptyInput(entityType: WorkflowEntityType): WorkflowRuleInput {
  return {
    entity_type: entityType,
    trigger_status: statusesFor(entityType)[0],
    task_title: "",
    task_description: null,
    due_in_days: 0,
    assignee_user_id: null,
  };
}

function statusesFor(entityType: WorkflowEntityType): readonly string[] {
  return entityType === "Opportunity" ? OPPORTUNITY_STAGES : INVOICE_STATUSES;
}

function triggerLabel(entityType: WorkflowEntityType): string {
  return entityType === "Opportunity" ? "stage" : "status";
}

/** Plain-English summary, e.g. "When stage reaches Won, create task 'Send onboarding kit' (due in 3 days, assigned to owner)." */
function describeRule(rule: WorkflowRule, nameByUserId: Map<string, string>): string {
  const field = triggerLabel(rule.entity_type as WorkflowEntityType);
  const due = rule.due_in_days === 0 ? "due immediately" : `due in ${rule.due_in_days} day${rule.due_in_days === 1 ? "" : "s"}`;
  const assignee = rule.assignee_user_id ? nameByUserId.get(rule.assignee_user_id) ?? "an unknown user" : "the record's owner";
  return `When ${field} reaches "${rule.trigger_status}", create task "${rule.task_title}" (${due}, assigned to ${assignee}).`;
}

/**
 * Admin screen for FR-WFL Phase 1 workflow automation - lets an
 * Administrator auto-create a follow-up Task when an Opportunity's stage
 * or an Invoice's status transitions to a chosen value. Enforced entirely
 * server-side at the moment of the transition (opportunity_service::update,
 * invoice_service's status transitions) - there is nothing for the client
 * to evaluate live, unlike FR-RUL's field rules.
 */
export function WorkflowRulesAdmin() {
  const [entityType, setEntityType] = useState<WorkflowEntityType>("Opportunity");
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const rules = useQuery({
    queryKey: ["workflowRules", entityType],
    queryFn: () => api.listWorkflowRules(entityType),
  });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["workflowRules"] });
  }

  const nameByUserId = new Map((users.data ?? []).map((u) => [u.id, u.display_name]));
  const editing = rules.data?.find((r) => r.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Workflow automation</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New rule
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Automatically create a follow-up task when an Opportunity's stage or an Invoice's status changes - e.g. a
        task to send an onboarding kit the moment a deal is Won. Every matching active rule fires, so more than one
        can create more than one task.
      </p>

      <div className="tab-row">
        {WORKFLOW_ENTITY_TYPES.map((t) => (
          <button
            key={t}
            className={`tab${entityType === t ? " active" : ""}`}
            onClick={() => {
              setEntityType(t);
              setCreating(false);
              setEditingId(null);
            }}
          >
            {t === "Opportunity" ? "Opportunities" : "Invoices"}
          </button>
        ))}
      </div>

      {creating && (
        <RuleForm
          entityType={entityType}
          users={users.data ?? []}
          initial={emptyInput(entityType)}
          submitLabel="Add rule"
          onSubmit={(input) => api.createWorkflowRule(input)}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <RuleForm
          entityType={entityType}
          users={users.data ?? []}
          initial={{
            entity_type: entityType,
            trigger_status: editing.trigger_status,
            task_title: editing.task_title,
            task_description: editing.task_description,
            due_in_days: editing.due_in_days,
            assignee_user_id: editing.assignee_user_id,
            is_active: editing.is_active,
          }}
          submitLabel="Save"
          onSubmit={(input, isActive) => api.updateWorkflowRule(editing.id, { ...input, is_active: isActive })}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
          showActiveToggle
        />
      )}

      {rules.isLoading && <p>Loading...</p>}
      {rules.data && rules.data.length === 0 && (
        <p className="empty-state">No workflow rules defined for {entityType}s yet.</p>
      )}
      {rules.data && rules.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Rule</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rules.data.map((r) => (
              <tr key={r.id}>
                <td>{describeRule(r, nameByUserId)}</td>
                <td>
                  <span className={`badge${r.is_active ? " badge-success" : ""}`}>{r.is_active ? "Active" : "Inactive"}</span>
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(r.id)}>
                    Edit
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

function RuleForm({
  entityType,
  users,
  initial,
  submitLabel,
  onSubmit,
  onDone,
  onCancel,
  showActiveToggle,
}: {
  entityType: WorkflowEntityType;
  users: { id: string; display_name: string; is_active: boolean }[];
  initial: WorkflowRuleInput & { is_active?: boolean };
  submitLabel: string;
  onSubmit: (input: WorkflowRuleInput, isActive: boolean) => Promise<unknown>;
  onDone: () => void;
  onCancel: () => void;
  showActiveToggle?: boolean;
}) {
  const [triggerStatus, setTriggerStatus] = useState(initial.trigger_status);
  const [taskTitle, setTaskTitle] = useState(initial.task_title);
  const [taskDescription, setTaskDescription] = useState(initial.task_description ?? "");
  const [dueInDays, setDueInDays] = useState(initial.due_in_days);
  const [assigneeUserId, setAssigneeUserId] = useState(initial.assignee_user_id ?? "");
  const [isActive, setIsActive] = useState(initial.is_active ?? true);
  const [error, setError] = useState<string | null>(null);

  const activeUsers = users.filter((u) => u.is_active);

  const save = useMutation({
    mutationFn: () =>
      onSubmit(
        {
          entity_type: entityType,
          trigger_status: triggerStatus,
          task_title: taskTitle,
          task_description: taskDescription.trim() || null,
          due_in_days: dueInDays,
          assignee_user_id: assigneeUserId || null,
        },
        isActive,
      ),
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
          <label>When {triggerLabel(entityType)} reaches</label>
          <select value={triggerStatus} onChange={(e) => setTriggerStatus(e.target.value)}>
            {statusesFor(entityType).map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field full">
          <label>Task title</label>
          <input value={taskTitle} onChange={(e) => setTaskTitle(e.target.value)} required />
        </div>
        <div className="form-field full">
          <label>Task description</label>
          <textarea value={taskDescription} onChange={(e) => setTaskDescription(e.target.value)} />
        </div>
        <div className="form-field">
          <label>Due in (days)</label>
          <input
            type="number"
            min={0}
            value={dueInDays}
            onChange={(e) => setDueInDays(Math.max(0, Number(e.target.value)))}
          />
        </div>
        <div className="form-field">
          <label>Assign to</label>
          <select value={assigneeUserId} onChange={(e) => setAssigneeUserId(e.target.value)}>
            <option value="">The record's owner</option>
            {activeUsers.map((u) => (
              <option key={u.id} value={u.id}>
                {u.display_name}
              </option>
            ))}
          </select>
        </div>
        {showActiveToggle && (
          <div className="form-field">
            <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
              Active
            </label>
          </div>
        )}
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            {submitLabel}
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
