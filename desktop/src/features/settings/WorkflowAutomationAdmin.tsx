import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { BuiltinValueInput } from "../../components/BuiltinValueInput";
import {
  builtinFieldsFor,
  builtinTriggerFieldFor,
  CONDITION_OPERATORS,
  CUSTOM_FIELD_ENTITY_TYPES,
  dateFieldsFor,
  entityTypeLabel,
  MATCH_TYPES,
  NOTIFICATION_AUDIENCES,
  transitionFieldFor,
  transitionValuesForEntity,
  TRIGGER_TYPES,
  TRIGGER_SOURCES,
  VALUELESS_OPERATORS,
  WORKFLOW_ACTION_TYPES,
  type ConditionOperator,
  type MatchType,
  type NotificationAudience,
  type TriggerSource,
  type TriggerType,
  type WorkflowActionInput,
  type WorkflowActionType,
  type WorkflowConditionInput,
  type WorkflowDefinitionInput,
} from "../../lib/types";

const TRIGGER_LABELS: Record<TriggerType, string> = {
  record_created: "Record created",
  record_updated: "Record updated",
  status_changed: "Status/stage reaches",
  field_changed: "Custom field changed",
  date_reached: "Date reached",
  due_overdue: "Overdue",
  scheduled: "Recurring schedule",
};

const ACTION_LABELS: Record<WorkflowActionType, string> = {
  create_task: "Create task",
  update_field: "Update a field",
  assign_owner: "Assign owner",
  create_record: "Create a record",
  update_related_record: "Update a linked record",
  add_notification: "Send notification",
  create_reminder: "Create reminder",
};

const ACTION_ICONS: Record<WorkflowActionType, string> = {
  create_task: "📋", create_reminder: "⏰", update_field: "✏️", assign_owner: "👤",
  create_record: "➕", update_related_record: "🔗", add_notification: "🔔",
};

const TRIGGER_ICONS: Record<TriggerType, string> = {
  record_created: "🆕", record_updated: "🔄", status_changed: "🏆",
  field_changed: "✏️", date_reached: "📅", due_overdue: "⏳", scheduled: "🔁",
};

const OPERATOR_LABELS: Record<ConditionOperator, string> = {
  equals: "equals", not_equals: "does not equal", contains: "contains", not_contains: "does not contain",
  starts_with: "starts with", ends_with: "ends with", in_list: "is one of", not_in_list: "is not one of",
  is_empty: "is empty", is_not_empty: "is not empty", greater_than: "is greater than", less_than: "is less than",
  on_or_after: "is on or after", on_or_before: "is on or before",
};

function emptyCondition(entityType: string): WorkflowConditionInput {
  return {
    field_source: "builtin", field_key: builtinTriggerFieldFor(entityType), operator: "equals", value: "",
    compare_field_source: null, compare_field_key: null,
  };
}

function defaultParamsFor(actionType: WorkflowActionType, customFieldKey: string): Record<string, unknown> {
  switch (actionType) {
    case "create_task": return { title: "", description: null, due_in_days: 0, assignee_user_id: null };
    case "create_reminder": return { title: "", description: null, remind_in_days: 0, assignee_user_id: null };
    case "update_field": return { target_field_key: customFieldKey, target_field_source: "custom", value: "", copy_from_field_key: null };
    case "assign_owner": return { user_id: null };
    case "create_record": return { entity_type: "", relationship_definition_id: null, name_template: null };
    case "update_related_record": return { relationship_definition_id: "", target_field_key: "", target_field_source: "custom", value: "", copy_from_field_key: null };
    case "add_notification": return { message: "", audience: "owner" };
  }
}

function emptyAction(customFieldKey: string): WorkflowActionInput {
  return { action_type: "create_task", params_json: JSON.stringify(defaultParamsFor("create_task", customFieldKey)) };
}

/** Label for a condition/action field regardless of source - mirrors
 * BusinessRulesAdmin's identical helper (kept as a separate copy since
 * it's typed against this file's own condition/action shapes). */
function fieldLabel(entityType: string, source: TriggerSource, key: string, labelByKey: Map<string, string>): string {
  if (source === "builtin") {
    return builtinFieldsFor(entityType).find((f) => f.key === key)?.label ?? key;
  }
  return labelByKey.get(key) ?? key;
}

function describeCondition(entityType: string, c: WorkflowConditionInput, labelByKey: Map<string, string>): string {
  const label = fieldLabel(entityType, c.field_source, c.field_key, labelByKey);
  const needsValue = !VALUELESS_OPERATORS.includes(c.operator);
  const comparand = c.compare_field_key && c.compare_field_source
    ? fieldLabel(entityType, c.compare_field_source, c.compare_field_key, labelByKey)
    : `"${c.value}"`;
  return `${label} ${OPERATOR_LABELS[c.operator]}${needsValue ? ` ${comparand}` : ""}`;
}

/** Plain-language description of when a workflow fires - used both for
 * the trigger canvas node and the collapsed "1 Trigger" section summary. */
function describeTrigger(
  entityType: string,
  triggerType: TriggerType,
  triggerStatus: string,
  triggerFieldKey: string,
  triggerOffsetDays: number,
): string {
  switch (triggerType) {
    case "record_created": return `When a ${entityTypeLabel(entityType)} record is created`;
    case "record_updated": return `When a ${entityTypeLabel(entityType)} record is updated`;
    case "status_changed": return `When ${transitionFieldFor(entityType)} changes to "${triggerStatus}"`;
    case "field_changed": return `When ${triggerFieldKey || "a field"} changes`;
    case "date_reached": return `When ${triggerFieldKey || "a date field"} is reached (offset ${triggerOffsetDays} day(s))`;
    case "due_overdue": return `When ${triggerFieldKey || "a date field"} is overdue`;
    case "scheduled": return `Every ${triggerOffsetDays} day(s)`;
  }
}

function describeAction(entityType: string, a: WorkflowActionInput, labelByKey: Map<string, string>): string {
  try {
    const p = JSON.parse(a.params_json) as Record<string, unknown>;
    switch (a.action_type) {
      case "create_task": return `create task "${p.title}"`;
      case "create_reminder": return `create reminder "${p.title}"`;
      case "update_field": {
        const isBuiltin = p.target_field_source === "builtin";
        const label = isBuiltin
          ? builtinFieldsFor(entityType).find((f) => f.key === p.target_field_key)?.label ?? String(p.target_field_key)
          : labelByKey.get(String(p.target_field_key)) ?? p.target_field_key;
        return `set ${label} = "${p.value ?? `{${p.copy_from_field_key}}`}"`;
      }
      case "assign_owner": return "assign owner";
      case "create_record": return `create a new ${p.entity_type}${p.relationship_definition_id ? " and link it" : ""}`;
      case "update_related_record": return `set ${p.target_field_key} on linked record${p.value ? ` = "${p.value}"` : p.copy_from_field_key ? ` = {${p.copy_from_field_key}}` : ""}`;
      case "add_notification": return `notify ${p.audience === "all_admins" ? "all admins" : "owner"}: "${p.message}"`;
      default: return a.action_type;
    }
  } catch {
    return a.action_type;
  }
}

/**
 * Admin extensibility Phase D (spec §23/ADM-WF): a no-code Trigger ->
 * Conditions -> Actions workflow builder - more trigger types than the
 * original status-transition-only engine, AND/OR conditions, and actions
 * beyond task creation.
 */
export function WorkflowAutomationAdmin() {
  const [entityType, setEntityType] = useState<string>(CUSTOM_FIELD_ENTITY_TYPES[0]);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];
  const currentLabel = entityTabs.find((t) => t.key === entityType)?.label ?? entityType;

  const workflows = useQuery({ queryKey: ["workflowRules", entityType], queryFn: () => api.listWorkflowRules(entityType) });
  const defs = useQuery({ queryKey: ["customFieldDefinitions", entityType, "all"], queryFn: () => api.listCustomFieldDefinitions(entityType, false) });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const relationshipDefs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["workflowRules"] });
  }

  const activeDefs = (defs.data ?? []).filter((d) => d.is_active);
  const labelByKey = new Map((defs.data ?? []).map((d) => [d.key, d.label]));
  const editing = workflows.data?.find((w) => w.id === editingId) ?? null;

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
          + New workflow
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Trigger an action - create a task, update a field, assign an owner, create a record (optionally linked),
        update a field on a linked record, or send a notification - when a record is created, updated, changes
        status, a date is reached, or on a recurring schedule.
      </p>

      <div className="tab-row">
        {entityTabs.map((t) => (
          <button
            key={t.key}
            className={`tab${entityType === t.key ? " active" : ""}`}
            onClick={() => {
              setEntityType(t.key);
              setCreating(false);
              setEditingId(null);
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {creating && (
        <WorkflowForm
          entityType={entityType}
          customFields={activeDefs}
          customObjects={customObjects.data ?? []}
          users={users.data ?? []}
          relationshipDefs={relationshipDefs.data ?? []}
          initial={{
            entity_type: entityType, name: "", description: null, trigger_type: "status_changed",
            trigger_status: transitionValuesForEntity(entityType)[0] ?? null, trigger_field_key: null, trigger_field_source: "custom",
            trigger_offset_days: 0, match_type: "all", priority: 0, conditions: [], actions: [emptyAction(activeDefs[0]?.key ?? "")],
          }}
          submitLabel="Add workflow"
          onSubmit={(input) => api.createWorkflowRule(input)}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <WorkflowForm
          entityType={entityType}
          customFields={activeDefs}
          customObjects={customObjects.data ?? []}
          users={users.data ?? []}
          relationshipDefs={relationshipDefs.data ?? []}
          initial={{
            entity_type: entityType, name: editing.name, description: editing.description, trigger_type: editing.trigger_type,
            trigger_status: editing.trigger_status, trigger_field_key: editing.trigger_field_key, trigger_field_source: editing.trigger_field_source,
            trigger_offset_days: editing.trigger_offset_days, match_type: editing.match_type, priority: editing.priority,
            conditions: editing.conditions.map((c) => ({
              field_source: c.field_source, field_key: c.field_key, operator: c.operator, value: c.value,
              compare_field_source: c.compare_field_source, compare_field_key: c.compare_field_key,
            })),
            actions: editing.actions.map((a) => ({ action_type: a.action_type, params_json: a.params_json })),
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

      {workflows.isLoading && !creating && !editing && <p>Loading...</p>}
      {workflows.data && workflows.data.length === 0 && !creating && !editing && (
        <p className="empty-state">No workflows defined for {currentLabel} yet.</p>
      )}
      {workflows.data && workflows.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Trigger</th>
              <th>Then</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {workflows.data.map((w) => (
              <tr key={w.id}>
                <td>{w.name}</td>
                <td>
                  {TRIGGER_LABELS[w.trigger_type]}
                  {w.trigger_status ? ` "${w.trigger_status}"` : ""}
                  {w.trigger_field_key ? ` (${w.trigger_field_key})` : ""}
                </td>
                <td>{w.actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}</td>
                <td>
                  <span className={`badge${w.is_active ? " badge-success" : ""}`}>{w.is_active ? "Active" : "Inactive"}</span>
                  {w.is_protected && <span className="badge" style={{ marginLeft: 4 }}>System</span>}
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(w.id)} disabled={w.is_protected}>
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

function ConditionRow({
  entityType,
  customFields,
  condition,
  onChange,
  onRemove,
}: {
  entityType: string;
  customFields: { key: string; label: string }[];
  condition: WorkflowConditionInput;
  onChange: (c: WorkflowConditionInput) => void;
  onRemove: () => void;
}) {
  const builtinFields = builtinFieldsFor(entityType);
  const isBuiltin = condition.field_source === "builtin";
  const needsValue = !VALUELESS_OPERATORS.includes(condition.operator);
  const isListOp = condition.operator === "in_list" || condition.operator === "not_in_list";
  const comparesToField = condition.compare_field_key !== null;
  const selectedBuiltin = isBuiltin ? builtinFields.find((f) => f.key === condition.field_key) : undefined;
  const compareFields = condition.compare_field_source === "builtin" ? builtinFields : customFields;

  return (
    <div style={{ display: "flex", gap: 6, alignItems: "center", flexWrap: "wrap", marginBottom: 6 }}>
      <select
        value={condition.field_source}
        onChange={(e) => {
          const source = e.target.value as TriggerSource;
          onChange({ ...condition, field_source: source, field_key: source === "builtin" ? builtinFields[0]?.key ?? "" : customFields[0]?.key ?? "", value: "" });
        }}
      >
        {TRIGGER_SOURCES.map((s) => <option key={s} value={s}>{s === "builtin" ? "Built-in field" : "Custom field"}</option>)}
      </select>
      {isBuiltin ? (
        <select value={condition.field_key} onChange={(e) => onChange({ ...condition, field_key: e.target.value, value: "" })}>
          {builtinFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
        </select>
      ) : (
        <select value={condition.field_key} onChange={(e) => onChange({ ...condition, field_key: e.target.value })}>
          {customFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
        </select>
      )}
      <select value={condition.operator} onChange={(e) => onChange({ ...condition, operator: e.target.value as ConditionOperator })}>
        {CONDITION_OPERATORS.map((o) => <option key={o} value={o}>{OPERATOR_LABELS[o]}</option>)}
      </select>
      {needsValue && !isListOp && (
        <select
          value={comparesToField ? "field" : "literal"}
          onChange={(e) => {
            if (e.target.value === "field") {
              onChange({ ...condition, compare_field_source: "custom", compare_field_key: customFields[0]?.key ?? builtinFields[0]?.key ?? "" });
            } else {
              onChange({ ...condition, compare_field_source: null, compare_field_key: null });
            }
          }}
        >
          <option value="literal">a fixed value</option>
          <option value="field">another field</option>
        </select>
      )}
      {needsValue && (comparesToField ? (
        <>
          <select
            value={condition.compare_field_source ?? "custom"}
            onChange={(e) => {
              const source = e.target.value as TriggerSource;
              const list = source === "builtin" ? builtinFields : customFields;
              onChange({ ...condition, compare_field_source: source, compare_field_key: list[0]?.key ?? "" });
            }}
          >
            {TRIGGER_SOURCES.map((s) => <option key={s} value={s}>{s === "builtin" ? "Built-in field" : "Custom field"}</option>)}
          </select>
          <select value={condition.compare_field_key ?? ""} onChange={(e) => onChange({ ...condition, compare_field_key: e.target.value })}>
            {compareFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
          </select>
        </>
      ) : isListOp ? (
        <input
          value={condition.value} onChange={(e) => onChange({ ...condition, value: e.target.value })}
          placeholder="Option A|Option B|Option C" required style={{ width: 200 }}
        />
      ) : isBuiltin ? (
        <BuiltinValueInput field={selectedBuiltin} value={condition.value} onChange={(v) => onChange({ ...condition, value: v })} />
      ) : (
        <input value={condition.value} onChange={(e) => onChange({ ...condition, value: e.target.value })} required style={{ width: 140 }} />
      ))}
      <button className="btn" type="button" onClick={onRemove}>Remove</button>
    </div>
  );
}

function ActionEditor({
  entityType,
  customFields,
  customObjects,
  users,
  relationshipDefs,
  action,
  onChange,
  onRemove,
}: {
  entityType: string;
  customFields: { key: string; label: string }[];
  customObjects: { key: string; plural_label: string }[];
  users: { id: string; display_name: string; is_active: boolean }[];
  relationshipDefs: { id: string; source_entity_type: string; target_entity_type: string; forward_label: string; reverse_label: string }[];
  action: WorkflowActionInput;
  onChange: (a: WorkflowActionInput) => void;
  onRemove: () => void;
}) {
  let params: Record<string, unknown>;
  try {
    params = JSON.parse(action.params_json) as Record<string, unknown>;
  } catch {
    params = defaultParamsFor(action.action_type, customFields[0]?.key ?? "");
  }
  const activeUsers = users.filter((u) => u.is_active);
  const applicableRelationships = relationshipDefs.filter((r) => r.source_entity_type === entityType || r.target_entity_type === entityType);
  // create_record can only construct Company (needs nothing but a name)
  // or an active custom object - see workflow_service::is_creatable_entity_type.
  const creatableEntityTypes = [{ key: "Company", label: entityTypeLabel("Company") }, ...customObjects.map((o) => ({ key: o.key, label: o.plural_label }))];

  const relatedDefForUpdate = relationshipDefs.find((r) => r.id === params.relationship_definition_id);
  const updateOtherType = relatedDefForUpdate
    ? (relatedDefForUpdate.source_entity_type === entityType ? relatedDefForUpdate.target_entity_type : relatedDefForUpdate.source_entity_type)
    : null;
  const otherTypeCustomFields = useQuery({
    queryKey: ["customFieldDefinitions", updateOtherType, "all"],
    queryFn: () => api.listCustomFieldDefinitions(updateOtherType as string, false),
    enabled: !!updateOtherType,
  });
  const otherTypeActiveCustomFields = (otherTypeCustomFields.data ?? []).filter((d) => d.is_active);
  const otherTypeBuiltinFields = updateOtherType ? builtinFieldsFor(updateOtherType).filter((f) => f.actionable) : [];
  const copyFromOptionsForUpdate = [...customFields, ...builtinFieldsFor(entityType).filter((f) => f.actionable)];

  function setParams(next: Record<string, unknown>) {
    onChange({ ...action, params_json: JSON.stringify(next) });
  }

  return (
    <div style={{ border: "1px solid var(--border, #ddd)", borderRadius: 6, padding: 8, marginBottom: 8 }}>
      <div style={{ display: "flex", gap: 6, alignItems: "center", marginBottom: 6 }}>
        <select
          value={action.action_type}
          onChange={(e) => {
            const type = e.target.value as WorkflowActionType;
            onChange({ action_type: type, params_json: JSON.stringify(defaultParamsFor(type, customFields[0]?.key ?? "")) });
          }}
        >
          {WORKFLOW_ACTION_TYPES.map((t) => <option key={t} value={t}>{ACTION_LABELS[t]}</option>)}
        </select>
        <button className="btn" type="button" onClick={onRemove}>Remove</button>
      </div>

      {(action.action_type === "create_task" || action.action_type === "create_reminder") && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          <input placeholder="Title" value={String(params.title ?? "")} onChange={(e) => setParams({ ...params, title: e.target.value })} required style={{ minWidth: 180 }} />
          <input
            type="number"
            placeholder={action.action_type === "create_task" ? "Due in days" : "Remind in days"}
            value={Number(params[action.action_type === "create_task" ? "due_in_days" : "remind_in_days"] ?? 0)}
            onChange={(e) => setParams({ ...params, [action.action_type === "create_task" ? "due_in_days" : "remind_in_days"]: Number(e.target.value) })}
            style={{ width: 130 }}
          />
          <select value={String(params.assignee_user_id ?? "")} onChange={(e) => setParams({ ...params, assignee_user_id: e.target.value || null })}>
            <option value="">The record's owner</option>
            {activeUsers.map((u) => <option key={u.id} value={u.id}>{u.display_name}</option>)}
          </select>
        </div>
      )}

      {action.action_type === "update_field" && (() => {
        const actionableBuiltinFields = builtinFieldsFor(entityType).filter((f) => f.actionable);
        const targetSource = (params.target_field_source as TriggerSource | undefined) ?? "custom";
        const isBuiltinTarget = targetSource === "builtin";
        const selectedBuiltin = isBuiltinTarget ? actionableBuiltinFields.find((f) => f.key === params.target_field_key) : undefined;
        const copyFromOptions = [...customFields, ...actionableBuiltinFields];
        return (
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            <select
              value={targetSource}
              onChange={(e) => {
                const source = e.target.value as TriggerSource;
                const first = source === "builtin" ? actionableBuiltinFields[0]?.key ?? "" : customFields[0]?.key ?? "";
                setParams({ ...params, target_field_source: source, target_field_key: first, value: "" });
              }}
            >
              <option value="custom">Custom field</option>
              <option value="builtin" disabled={actionableBuiltinFields.length === 0}>Built-in field</option>
            </select>
            <select value={String(params.target_field_key ?? "")} onChange={(e) => setParams({ ...params, target_field_key: e.target.value })} required>
              {(isBuiltinTarget ? actionableBuiltinFields : customFields).map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
            {isBuiltinTarget ? (
              <BuiltinValueInput field={selectedBuiltin} value={String(params.value ?? "")} onChange={(v) => setParams({ ...params, value: v, copy_from_field_key: null })} />
            ) : (
              <input placeholder="Set to value" value={String(params.value ?? "")} onChange={(e) => setParams({ ...params, value: e.target.value, copy_from_field_key: null })} style={{ width: 160 }} />
            )}
            <span style={{ alignSelf: "center", fontSize: 12, color: "var(--text-muted)" }}>or</span>
            <select value={String(params.copy_from_field_key ?? "")} onChange={(e) => setParams({ ...params, copy_from_field_key: e.target.value || null, value: null })}>
              <option value="">Copy from field...</option>
              {copyFromOptions.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
          </div>
        );
      })()}

      {action.action_type === "assign_owner" && (
        <select value={String(params.user_id ?? "")} onChange={(e) => setParams({ ...params, user_id: e.target.value || null })}>
          <option value="">— Unassigned —</option>
          {activeUsers.map((u) => <option key={u.id} value={u.id}>{u.display_name}</option>)}
        </select>
      )}

      {action.action_type === "create_record" && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          <select value={String(params.entity_type ?? "")} onChange={(e) => setParams({ ...params, entity_type: e.target.value })} required>
            <option value="">Create a...</option>
            {creatableEntityTypes.map((t) => <option key={t.key} value={t.key}>{t.label}</option>)}
          </select>
          <input placeholder="Name (optional, defaults to 'Related to ...')" value={String(params.name_template ?? "")} onChange={(e) => setParams({ ...params, name_template: e.target.value || null })} style={{ minWidth: 220 }} />
          <select
            value={String(params.relationship_definition_id ?? "")}
            onChange={(e) => setParams({ ...params, relationship_definition_id: e.target.value || null })}
          >
            <option value="">Don't link it to anything</option>
            {applicableRelationships.map((r) => (
              <option key={r.id} value={r.id}>Link as: {r.source_entity_type === entityType ? r.forward_label : r.reverse_label}</option>
            ))}
          </select>
        </div>
      )}

      {action.action_type === "update_related_record" && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          <select
            value={String(params.relationship_definition_id ?? "")}
            onChange={(e) => setParams({ ...params, relationship_definition_id: e.target.value, target_field_key: "", target_field_source: "custom", value: "", copy_from_field_key: null })}
            required
          >
            <option value="">Select relationship...</option>
            {applicableRelationships.map((r) => (
              <option key={r.id} value={r.id}>{r.source_entity_type === entityType ? r.forward_label : r.reverse_label}</option>
            ))}
          </select>
          {updateOtherType && (() => {
            const targetSource = (params.target_field_source as TriggerSource | undefined) ?? "custom";
            const isBuiltinTarget = targetSource === "builtin";
            return (
              <>
                <select
                  value={targetSource}
                  onChange={(e) => {
                    const source = e.target.value as TriggerSource;
                    const first = source === "builtin" ? otherTypeBuiltinFields[0]?.key ?? "" : otherTypeActiveCustomFields[0]?.key ?? "";
                    setParams({ ...params, target_field_source: source, target_field_key: first, value: "" });
                  }}
                >
                  <option value="custom" disabled={otherTypeActiveCustomFields.length === 0}>Custom field</option>
                  <option value="builtin" disabled={otherTypeBuiltinFields.length === 0}>Built-in field</option>
                </select>
                <select value={String(params.target_field_key ?? "")} onChange={(e) => setParams({ ...params, target_field_key: e.target.value })} required>
                  {(isBuiltinTarget ? otherTypeBuiltinFields : otherTypeActiveCustomFields).map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
                </select>
                <input placeholder="Set to value" value={String(params.value ?? "")} onChange={(e) => setParams({ ...params, value: e.target.value, copy_from_field_key: null })} style={{ width: 160 }} />
                <span style={{ alignSelf: "center", fontSize: 12, color: "var(--text-muted)" }}>or copy from this record's</span>
                <select value={String(params.copy_from_field_key ?? "")} onChange={(e) => setParams({ ...params, copy_from_field_key: e.target.value || null, value: null })}>
                  <option value="">Select field...</option>
                  {copyFromOptionsForUpdate.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
                </select>
              </>
            );
          })()}
        </div>
      )}

      {action.action_type === "add_notification" && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          <input placeholder="Message" value={String(params.message ?? "")} onChange={(e) => setParams({ ...params, message: e.target.value })} required style={{ minWidth: 220 }} />
          <select value={String(params.audience ?? "owner")} onChange={(e) => setParams({ ...params, audience: e.target.value as NotificationAudience })}>
            {NOTIFICATION_AUDIENCES.map((a) => <option key={a} value={a}>{a === "owner" ? "Record owner" : "All admins"}</option>)}
          </select>
        </div>
      )}
    </div>
  );
}

function WorkflowForm({
  entityType,
  customFields,
  customObjects,
  users,
  relationshipDefs,
  initial,
  submitLabel,
  onSubmit,
  onDone,
  onCancel,
  showActiveToggle,
}: {
  entityType: string;
  customFields: { key: string; label: string }[];
  customObjects: { key: string; plural_label: string }[];
  users: { id: string; display_name: string; is_active: boolean }[];
  relationshipDefs: { id: string; source_entity_type: string; target_entity_type: string; forward_label: string; reverse_label: string }[];
  initial: WorkflowDefinitionInput & { is_active?: boolean };
  submitLabel: string;
  onSubmit: (input: WorkflowDefinitionInput, isActive: boolean) => Promise<unknown>;
  onDone: () => void;
  onCancel: () => void;
  showActiveToggle?: boolean;
}) {
  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description ?? "");
  const [triggerType, setTriggerType] = useState<TriggerType>(initial.trigger_type);
  const [triggerStatus, setTriggerStatus] = useState(initial.trigger_status ?? transitionValuesForEntity(entityType)[0] ?? "");
  const [triggerFieldKey, setTriggerFieldKey] = useState(initial.trigger_field_key ?? "");
  const [triggerFieldSource, setTriggerFieldSource] = useState<TriggerSource>(initial.trigger_field_source ?? "custom");
  const [triggerOffsetDays, setTriggerOffsetDays] = useState(initial.trigger_offset_days);
  const [matchType, setMatchType] = useState<MatchType>(initial.match_type);
  const [priority, setPriority] = useState(initial.priority);
  const [conditions, setConditions] = useState<WorkflowConditionInput[]>(initial.conditions);
  const [actions, setActions] = useState<WorkflowActionInput[]>(initial.actions);
  const [isActive, setIsActive] = useState(initial.is_active ?? true);
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [editSection, setEditSection] = useState<"trigger" | "conditions" | "actions" | null>("trigger");
  const [zoom, setZoom] = useState(1);
  const [fullscreen, setFullscreen] = useState(false);

  const dateFields = dateFieldsFor(entityType);
  const labelByKey = new Map(customFields.map((f) => [f.key, f.label]));

  const save = useMutation({
    mutationFn: (nextActive: boolean) =>
      onSubmit(
        {
          entity_type: entityType, name, description: description || null, trigger_type: triggerType,
          trigger_status: triggerType === "status_changed" ? triggerStatus : null,
          trigger_field_key: triggerType === "field_changed" || triggerType === "date_reached" || triggerType === "due_overdue" ? triggerFieldKey : null,
          trigger_field_source: triggerType === "field_changed" ? triggerFieldSource : "custom",
          trigger_offset_days: triggerOffsetDays, match_type: matchType, priority, conditions, actions,
        },
        nextActive,
      ),
    onSuccess: (_, nextActive) => {
      setIsActive(nextActive);
      onDone();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this workflow"),
  });

  const triggerSummary = describeTrigger(entityType, triggerType, triggerStatus, triggerFieldKey, triggerOffsetDays);

  return (
    <div style={{ marginBottom: 16 }}>
      <div className="builder-header">
        <div>
          <div className="builder-breadcrumb">Workflow Automation / {name || "New workflow"}</div>
          <div className="builder-title-row">
            <h2>{name || "New workflow"}</h2>
            {showActiveToggle && <span className={`badge${isActive ? " badge-success" : ""}`}>{isActive ? "Active" : "Inactive"}</span>}
          </div>
          <p className="builder-subtitle">{description || triggerSummary}</p>
        </div>
        <div className="builder-header-actions">
          <button className="btn" type="button" onClick={() => setTesting((v) => !v)}>
            {testing ? "Hide test" : "Test run"}
          </button>
          {showActiveToggle && (
            <button className="btn" type="button" disabled={save.isPending} onClick={() => save.mutate(!isActive)}>
              {isActive ? "Deactivate" : "Activate"}
            </button>
          )}
          <button
            className="btn btn-primary"
            type="submit"
            form="workflow-form"
            disabled={save.isPending || actions.length === 0}
          >
            {submitLabel}
          </button>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}
      {testing && <TestWorkflowPanel entityType={entityType} customFields={customFields} />}

      <form
        id="workflow-form"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate(isActive);
        }}
      >
        <div className="workflow-layout">
          <div>
            <div className="form-grid" style={{ marginBottom: 16 }}>
              <div className="form-field full">
                <label>Workflow name</label>
                <input value={name} onChange={(e) => setName(e.target.value)} required />
              </div>
              <div className="form-field full">
                <label>Description (optional)</label>
                <input value={description} onChange={(e) => setDescription(e.target.value)} />
              </div>
              {showActiveToggle && (
                <div className="form-field">
                  <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
                    Active
                  </label>
                </div>
              )}
            </div>

            <div className="builder-section">
              <div className="builder-section-title" style={{ justifyContent: "space-between" }}>
                <span style={{ display: "flex", gap: 10, alignItems: "center" }}>
                  <span className="step-badge">1</span> Trigger
                </span>
                <button
                  className="link-button"
                  type="button"
                  onClick={() => setEditSection(editSection === "trigger" ? null : "trigger")}
                >
                  {editSection === "trigger" ? "Done" : "Edit"}
                </button>
              </div>
              {editSection !== "trigger" ? (
                <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>{triggerSummary}</p>
              ) : (
                <div className="form-grid">
                  <div className="form-field">
                    <label>Trigger</label>
                    <select value={triggerType} onChange={(e) => setTriggerType(e.target.value as TriggerType)}>
                      {TRIGGER_TYPES.map((t) => <option key={t} value={t}>{TRIGGER_LABELS[t]}</option>)}
                    </select>
                  </div>
                  {triggerType === "status_changed" && (
                    <div className="form-field">
                      <label>{transitionFieldFor(entityType)} reaches</label>
                      <select value={triggerStatus} onChange={(e) => setTriggerStatus(e.target.value)}>
                        {transitionValuesForEntity(entityType).map((s) => <option key={s} value={s}>{s}</option>)}
                      </select>
                    </div>
                  )}
                  {triggerType === "field_changed" && (() => {
                    const watchableBuiltinFields = builtinFieldsFor(entityType);
                    const isBuiltinWatch = triggerFieldSource === "builtin";
                    return (
                      <>
                        <div className="form-field">
                          <label>Field source</label>
                          <select
                            value={triggerFieldSource}
                            onChange={(e) => {
                              const source = e.target.value as TriggerSource;
                              setTriggerFieldSource(source);
                              setTriggerFieldKey((source === "builtin" ? watchableBuiltinFields[0]?.key : customFields[0]?.key) ?? "");
                            }}
                          >
                            <option value="custom">Custom field</option>
                            <option value="builtin">Built-in field</option>
                          </select>
                        </div>
                        <div className="form-field">
                          <label>Watch field</label>
                          <select value={triggerFieldKey} onChange={(e) => setTriggerFieldKey(e.target.value)}>
                            {(isBuiltinWatch ? watchableBuiltinFields : customFields).map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
                          </select>
                        </div>
                      </>
                    );
                  })()}
                  {(triggerType === "date_reached" || triggerType === "due_overdue") && (
                    <>
                      <div className="form-field">
                        <label>Watch date field</label>
                        <select value={triggerFieldKey} onChange={(e) => setTriggerFieldKey(e.target.value)}>
                          {dateFields.length === 0 && <option value="">No date field available for {entityTypeLabel(entityType)}</option>}
                          {dateFields.map((f) => <option key={f} value={f}>{f}</option>)}
                        </select>
                      </div>
                      <div className="form-field">
                        <label>Offset (days before/after)</label>
                        <input type="number" value={triggerOffsetDays} onChange={(e) => setTriggerOffsetDays(Number(e.target.value))} />
                      </div>
                    </>
                  )}
                  {triggerType === "scheduled" && (
                    <div className="form-field">
                      <label>Repeat every (days)</label>
                      <input type="number" min={1} value={triggerOffsetDays} onChange={(e) => setTriggerOffsetDays(Math.max(1, Number(e.target.value)))} />
                    </div>
                  )}
                  <div className="form-field">
                    <label>Priority (lower runs first)</label>
                    <input type="number" value={priority} onChange={(e) => setPriority(Number(e.target.value))} />
                  </div>
                </div>
              )}
            </div>

            <div className="builder-section">
              <div className="builder-section-title" style={{ justifyContent: "space-between" }}>
                <span style={{ display: "flex", gap: 10, alignItems: "center" }}>
                  <span className="step-badge">2</span> Conditions
                </span>
                <button
                  className="link-button"
                  type="button"
                  onClick={() => setEditSection(editSection === "conditions" ? null : "conditions")}
                >
                  {editSection === "conditions" ? "Done" : "Edit"}
                </button>
              </div>
              {editSection !== "conditions" ? (
                conditions.length === 0 ? (
                  <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>No extra conditions - fires on every trigger.</p>
                ) : (
                  <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>
                    {conditions.map((c) => describeCondition(entityType, c, labelByKey)).join(matchType === "any" ? " OR " : " AND ")}
                  </p>
                )
              ) : (
                <>
                  {conditions.map((c, i) => (
                    <ConditionRow
                      key={i}
                      entityType={entityType}
                      customFields={customFields}
                      condition={c}
                      onChange={(next) => setConditions(conditions.map((x, idx) => (idx === i ? next : x)))}
                      onRemove={() => setConditions(conditions.filter((_, idx) => idx !== i))}
                    />
                  ))}
                  <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <button className="btn" type="button" onClick={() => setConditions([...conditions, emptyCondition(entityType)])}>
                      + Add condition
                    </button>
                    {conditions.length > 1 && (
                      <select value={matchType} onChange={(e) => setMatchType(e.target.value as MatchType)}>
                        {MATCH_TYPES.map((m) => <option key={m} value={m}>{m === "all" ? "Match all (AND)" : "Match any (OR)"}</option>)}
                      </select>
                    )}
                  </div>
                </>
              )}
            </div>

            <div className="builder-section">
              <div className="builder-section-title" style={{ justifyContent: "space-between" }}>
                <span style={{ display: "flex", gap: 10, alignItems: "center" }}>
                  <span className="step-badge">3</span> Actions
                </span>
                <button
                  className="link-button"
                  type="button"
                  onClick={() => setEditSection(editSection === "actions" ? null : "actions")}
                >
                  {editSection === "actions" ? "Done" : "Edit"}
                </button>
              </div>
              {editSection !== "actions" ? (
                <p style={{ color: "var(--text-muted)", fontSize: 13, margin: 0 }}>
                  {actions.length === 0 ? "No actions yet." : actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}
                </p>
              ) : (
                <>
                  {actions.map((a, i) => (
                    <ActionEditor
                      key={i}
                      entityType={entityType}
                      customFields={customFields}
                      customObjects={customObjects}
                      users={users}
                      relationshipDefs={relationshipDefs}
                      action={a}
                      onChange={(next) => setActions(actions.map((x, idx) => (idx === i ? next : x)))}
                      onRemove={() => setActions(actions.filter((_, idx) => idx !== i))}
                    />
                  ))}
                  <button className="btn" type="button" onClick={() => setActions([...actions, emptyAction(customFields[0]?.key ?? "")])}>
                    + Add action
                  </button>
                </>
              )}
            </div>

            <button className="btn" type="button" onClick={onCancel}>
              Cancel
            </button>
          </div>

          <div>
            <div
              className="workflow-canvas-wrap"
              style={fullscreen ? { position: "fixed", inset: 12, zIndex: 1000 } : undefined}
            >
              <div className="workflow-canvas" style={{ transform: `scale(${zoom})` }}>
                <div className="workflow-node workflow-node-trigger">
                  <div className="workflow-node-head">Trigger</div>
                  <div className="workflow-node-body workflow-trigger-body">
                    <span className="workflow-trigger-icon">{TRIGGER_ICONS[triggerType]}</span>
                    <div>
                      <div className="workflow-trigger-title">{entityTypeLabel(entityType)}</div>
                      <div className="workflow-trigger-detail">{triggerSummary}</div>
                    </div>
                  </div>
                </div>

                <div className="workflow-connector">
                  <span className="workflow-connector-line" />
                  <span className="workflow-connector-arrow">▼</span>
                </div>

                {conditions.length > 0 && (
                  <>
                    <div className="workflow-node workflow-node-conditions">
                      <div className="workflow-node-head">Conditions</div>
                      <div className="workflow-node-body">
                        {conditions.map((c, i) => (
                          <div key={i}>
                            {i > 0 && <div className="workflow-match-chip">{matchType === "all" ? "AND" : "OR"}</div>}
                            <div className="workflow-condition-chip">{describeCondition(entityType, c, labelByKey)}</div>
                          </div>
                        ))}
                      </div>
                    </div>
                    <div className="workflow-connector">
                      <span className="workflow-connector-line" />
                      <span className="workflow-connector-arrow">▼</span>
                    </div>
                  </>
                )}

                <div className="workflow-node workflow-node-actions">
                  <div className="workflow-node-head">Actions</div>
                  <div className="workflow-node-body">
                    {actions.length === 0 && <p className="empty-state" style={{ padding: 8 }}>No actions yet</p>}
                    {actions.map((a, i) => (
                      <div className="workflow-action-card" key={i}>
                        <span className="workflow-action-icon">{ACTION_ICONS[a.action_type]}</span>
                        <div>
                          <div className="workflow-action-title">{ACTION_LABELS[a.action_type]}</div>
                          <div className="workflow-action-subtitle">{describeAction(entityType, a, labelByKey)}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="workflow-connector">
                  <span className="workflow-connector-line" />
                  <span className="workflow-connector-arrow">▼</span>
                </div>
                <div className="workflow-end-node">END</div>
              </div>

              <div className="workflow-zoom-controls">
                <button className="btn" type="button" title="Zoom in" onClick={() => setZoom((z) => Math.min(1.5, Math.round((z + 0.1) * 10) / 10))}>+</button>
                <button className="btn" type="button" title="Zoom out" onClick={() => setZoom((z) => Math.max(0.5, Math.round((z - 0.1) * 10) / 10))}>−</button>
                <button className="btn" type="button" title={fullscreen ? "Exit fullscreen" : "Fullscreen"} onClick={() => setFullscreen((f) => !f)}>
                  {fullscreen ? "⤡" : "⤢"}
                </button>
              </div>
            </div>
          </div>
        </div>
      </form>
    </div>
  );
}

/** Addendum Phase 3 (mirrors BusinessRulesAdmin's TestRulesPanel): lets an
 * admin try a hypothetical set of field values against every active
 * workflow for an entity type before relying on it. Unlike business
 * rules, workflow actions are side-effecting (create a task, write a
 * field, send a notification) so nothing here actually runs them - each
 * matching workflow's actions are described in plain language instead. */
function TestWorkflowPanel({ entityType, customFields }: { entityType: string; customFields: { key: string; label: string }[] }) {
  const builtinFields = builtinFieldsFor(entityType);
  const [values, setValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(builtinFields.filter((f) => f.field_type === "select").map((f) => [f.key, f.options?.[0] ?? ""])),
  );

  const test = useMutation({
    mutationFn: () => api.testWorkflows(entityType, values),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <h4 style={{ marginTop: 0 }}>Test workflows against a hypothetical record</h4>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Shows what each matching active workflow's conditions and actions would do - nothing is created, changed, or
        sent.
      </p>
      <div className="form-grid">
        {builtinFields.map((f) => (
          <div className="form-field" key={f.key}>
            <label>{f.label}</label>
            <BuiltinValueInput field={f} value={values[f.key] ?? ""} onChange={(v) => setValues({ ...values, [f.key]: v })} />
          </div>
        ))}
        {customFields.map((f) => (
          <div className="form-field" key={f.key}>
            <label>{f.label}</label>
            <input value={values[f.key] ?? ""} onChange={(e) => setValues({ ...values, [f.key]: e.target.value })} />
          </div>
        ))}
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="button" onClick={() => test.mutate()} disabled={test.isPending}>
            Run test
          </button>
        </div>
      </div>
      {test.data && (
        <div style={{ marginTop: 8, fontSize: 13 }}>
          {test.data.matches.length === 0 && <p className="empty-state">No active workflow matches this hypothetical record.</p>}
          {test.data.matches.map((m) => (
            <p key={m.workflow_id}>
              <strong>{m.workflow_name}</strong> - {m.action_descriptions.join("; ")}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
