import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { BuiltinValueInput } from "../../components/BuiltinValueInput";
import {
  ACTION_TYPES,
  builtinFieldsFor,
  builtinTriggerFieldFor,
  CONDITION_OPERATORS,
  CUSTOM_FIELD_ENTITY_TYPES,
  entityTypeLabel,
  FIELD_TARGETED_ACTIONS,
  MATCH_TYPES,
  MESSAGE_ACTIONS,
  TRIGGER_SOURCES,
  VALUELESS_OPERATORS,
  type ActionType,
  type BusinessRuleActionInput,
  type BusinessRuleConditionInput,
  type BusinessRuleInput,
  type ConditionOperator,
  type MatchType,
  type TriggerSource,
} from "../../lib/types";

const OPERATOR_LABELS: Record<ConditionOperator, string> = {
  equals: "equals", not_equals: "does not equal", contains: "contains", not_contains: "does not contain",
  starts_with: "starts with", ends_with: "ends with", in_list: "is one of", not_in_list: "is not one of",
  is_empty: "is empty", is_not_empty: "is not empty", greater_than: "is greater than", less_than: "is less than",
  on_or_after: "is on or after", on_or_before: "is on or before",
};

const ACTION_LABELS: Record<ActionType, string> = {
  require: "Require", hide: "Hide", lock: "Lock (read-only)", set_default: "Set default value",
  set_value: "Force value", block_save: "Block save", show_message: "Show message",
};

function emptyCondition(entityType: string): BusinessRuleConditionInput {
  return {
    field_source: "builtin", field_key: builtinTriggerFieldFor(entityType), operator: "equals", value: "",
    compare_field_source: null, compare_field_key: null,
  };
}

/** Defaults to a custom field target when one exists (matches this admin
 * screen's original behavior); falls back to the first actionable
 * built-in field so "+ Add action" still works on an entity with no
 * custom fields defined at all. */
function emptyAction(entityType: string, customFields: { key: string; label: string }[]): BusinessRuleActionInput {
  if (customFields.length > 0) {
    return { action_type: "require", target_field_key: customFields[0].key, target_field_source: "custom", action_value: null, message: null };
  }
  const builtin = builtinFieldsFor(entityType).find((f) => f.actionable);
  if (builtin) {
    return { action_type: "require", target_field_key: builtin.key, target_field_source: "builtin", action_value: null, message: null };
  }
  // No actionable target at all (e.g. Quote/Order/Invoice with no custom
  // fields defined) - default to a message action, the only kind that
  // doesn't need one.
  return { action_type: "show_message", target_field_key: null, target_field_source: "custom", action_value: null, message: "" };
}

/** Label for a condition/action field regardless of source - builtin
 * fields come from the static registry (`builtinFieldsFor`), custom
 * fields from the definitions the caller already fetched. */
function fieldLabel(entityType: string, source: TriggerSource, key: string, labelByKey: Map<string, string>): string {
  if (source === "builtin") {
    return builtinFieldsFor(entityType).find((f) => f.key === key)?.label ?? key;
  }
  return labelByKey.get(key) ?? key;
}

function describeCondition(entityType: string, c: BusinessRuleConditionInput, labelByKey: Map<string, string>): string {
  const label = fieldLabel(entityType, c.field_source, c.field_key, labelByKey);
  const needsValue = !VALUELESS_OPERATORS.includes(c.operator);
  const comparand = c.compare_field_key && c.compare_field_source
    ? fieldLabel(entityType, c.compare_field_source, c.compare_field_key, labelByKey)
    : `"${c.value}"`;
  return `${label} ${OPERATOR_LABELS[c.operator]}${needsValue ? ` ${comparand}` : ""}`;
}

function describeAction(entityType: string, a: BusinessRuleActionInput, labelByKey: Map<string, string>): string {
  const target = a.target_field_key ? fieldLabel(entityType, a.target_field_source, a.target_field_key, labelByKey) : "";
  switch (a.action_type) {
    case "require": return `require ${target}`;
    case "hide": return `hide ${target}`;
    case "lock": return `lock ${target}`;
    case "set_default": return `default ${target} to "${a.action_value ?? ""}"`;
    case "set_value": return `force ${target} to "${a.action_value ?? ""}"`;
    case "block_save": return `block save: "${a.message ?? ""}"`;
    case "show_message": return `show message: "${a.message ?? ""}"`;
    default: return a.action_type;
  }
}

/**
 * Admin extensibility Phase C (spec §22/ADM-BR): a no-code IF (AND/OR) /
 * THEN rule builder - any number of conditions, and actions beyond
 * require/hide (lock, set a default/forced value, block the save with a
 * custom message, or show a non-blocking message). Enforced server-side
 * (custom_field_service::set_entity_values) and mirrored client-side for
 * live form feedback for the three cosmetic effects (see
 * lib/businessRules.ts).
 */
export function BusinessRulesAdmin() {
  const [entityType, setEntityType] = useState<string>(CUSTOM_FIELD_ENTITY_TYPES[0]);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const queryClient = useQueryClient();

  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];
  const currentLabel = entityTabs.find((t) => t.key === entityType)?.label ?? entityType;

  const rules = useQuery({ queryKey: ["businessRules", entityType, "all"], queryFn: () => api.listBusinessRules(entityType, false) });
  const defs = useQuery({ queryKey: ["customFieldDefinitions", entityType, "all"], queryFn: () => api.listCustomFieldDefinitions(entityType, false) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["businessRules"] });
  }

  const activeDefs = (defs.data ?? []).filter((d) => d.is_active);
  const labelByKey = new Map((defs.data ?? []).map((d) => [d.key, d.label]));
  const editing = rules.data?.find((r) => r.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Business rules</h3>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn" onClick={() => setTesting((v) => !v)}>
            {testing ? "Hide test mode" : "Test rules"}
          </button>
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
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Build an IF (AND/OR conditions) / THEN (actions) rule against any built-in or custom field on {currentLabel}.
        The higher-priority rule wins when two rules disagree about the same target field.
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
              setTesting(false);
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {testing && <TestRulesPanel entityType={entityType} customFields={activeDefs} />}

      {creating && (
        <RuleForm
          entityType={entityType}
          customFields={activeDefs}
          initial={{
            entity_type: entityType, name: "", description: null, match_type: "all", priority: 0,
            effective_start_date: null, effective_end_date: null,
            conditions: [emptyCondition(entityType)], actions: [emptyAction(entityType, activeDefs)],
          }}
          submitLabel="Add rule"
          onSubmit={(input) => api.createBusinessRule(input)}
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
          customFields={activeDefs}
          initial={{
            entity_type: entityType, name: editing.name, description: editing.description, match_type: editing.match_type,
            priority: editing.priority, effective_start_date: editing.effective_start_date, effective_end_date: editing.effective_end_date,
            conditions: editing.conditions.map((c) => ({
              field_source: c.field_source, field_key: c.field_key, operator: c.operator, value: c.value,
              compare_field_source: c.compare_field_source, compare_field_key: c.compare_field_key,
            })),
            actions: editing.actions.map((a) => ({ action_type: a.action_type, target_field_key: a.target_field_key, target_field_source: a.target_field_source, action_value: a.action_value, message: a.message })),
            is_active: editing.is_active,
          }}
          submitLabel="Save"
          onSubmit={(input, isActive) => api.updateBusinessRule(editing.id, { ...input, is_active: isActive })}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
          showActiveToggle
        />
      )}

      {rules.isLoading && <p>Loading...</p>}
      {rules.data && rules.data.length === 0 && <p className="empty-state">No business rules defined for {currentLabel} yet.</p>}
      {rules.data && rules.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Rule</th>
              <th>If</th>
              <th>Then</th>
              <th>Priority</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rules.data.map((r) => (
              <tr key={r.id}>
                <td>{r.name}</td>
                <td>{r.conditions.map((c) => describeCondition(entityType, c, labelByKey)).join(r.match_type === "any" ? " OR " : " AND ")}</td>
                <td>{r.actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}</td>
                <td>{r.priority}</td>
                <td>
                  <span className={`badge${r.is_active ? " badge-success" : ""}`}>{r.is_active ? "Active" : "Inactive"}</span>
                  {r.is_protected && <span className="badge" style={{ marginLeft: 4 }}>System</span>}
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(r.id)} disabled={r.is_protected}>
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
  condition: BusinessRuleConditionInput;
  onChange: (c: BusinessRuleConditionInput) => void;
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
        {TRIGGER_SOURCES.map((s) => (
          <option key={s} value={s}>{s === "builtin" ? "Built-in field" : "Custom field"}</option>
        ))}
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

function ActionRow({
  entityType,
  customFields,
  action,
  onChange,
  onRemove,
}: {
  entityType: string;
  customFields: { key: string; label: string }[];
  action: BusinessRuleActionInput;
  onChange: (a: BusinessRuleActionInput) => void;
  onRemove: () => void;
}) {
  const actionableBuiltinFields = builtinFieldsFor(entityType).filter((f) => f.actionable);
  const isFieldTargeted = FIELD_TARGETED_ACTIONS.includes(action.action_type);
  const isMessageAction = MESSAGE_ACTIONS.includes(action.action_type);
  const needsValue = action.action_type === "set_default" || action.action_type === "set_value";
  const isBuiltinTarget = action.target_field_source === "builtin";
  const selectedBuiltin = isBuiltinTarget ? actionableBuiltinFields.find((f) => f.key === action.target_field_key) : undefined;

  return (
    <div style={{ display: "flex", gap: 6, alignItems: "center", flexWrap: "wrap", marginBottom: 6 }}>
      <select
        value={action.action_type}
        onChange={(e) => {
          const type = e.target.value as ActionType;
          onChange({
            action_type: type,
            target_field_key: FIELD_TARGETED_ACTIONS.includes(type) ? (action.target_field_key || customFields[0]?.key || "") : null,
            target_field_source: action.target_field_source,
            action_value: type === "set_default" || type === "set_value" ? action.action_value ?? "" : null,
            message: MESSAGE_ACTIONS.includes(type) ? action.message ?? "" : null,
          });
        }}
      >
        {ACTION_TYPES.map((t) => <option key={t} value={t}>{ACTION_LABELS[t]}</option>)}
      </select>
      {isFieldTargeted && (
        <>
          <select
            value={action.target_field_source}
            onChange={(e) => {
              const source = e.target.value as TriggerSource;
              const first = source === "builtin" ? actionableBuiltinFields[0]?.key ?? "" : customFields[0]?.key ?? "";
              onChange({ ...action, target_field_source: source, target_field_key: first, action_value: null });
            }}
          >
            <option value="custom">Custom field</option>
            <option value="builtin" disabled={actionableBuiltinFields.length === 0}>Built-in field</option>
          </select>
          {isBuiltinTarget ? (
            <select value={action.target_field_key ?? ""} onChange={(e) => onChange({ ...action, target_field_key: e.target.value, action_value: null })} required>
              {actionableBuiltinFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
          ) : (
            <select value={action.target_field_key ?? ""} onChange={(e) => onChange({ ...action, target_field_key: e.target.value })} required>
              {customFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
          )}
        </>
      )}
      {needsValue && (isBuiltinTarget ? (
        <BuiltinValueInput field={selectedBuiltin} value={action.action_value ?? ""} onChange={(v) => onChange({ ...action, action_value: v })} />
      ) : (
        <input value={action.action_value ?? ""} onChange={(e) => onChange({ ...action, action_value: e.target.value })} placeholder="Value" required style={{ width: 140 }} />
      ))}
      {isMessageAction && (
        <input value={action.message ?? ""} onChange={(e) => onChange({ ...action, message: e.target.value })} placeholder="Message shown to the user" required style={{ width: 260 }} />
      )}
      <button className="btn" type="button" onClick={onRemove}>Remove</button>
    </div>
  );
}

function RuleForm({
  entityType,
  customFields,
  initial,
  submitLabel,
  onSubmit,
  onDone,
  onCancel,
  showActiveToggle,
}: {
  entityType: string;
  customFields: { key: string; label: string }[];
  initial: BusinessRuleInput & { is_active?: boolean };
  submitLabel: string;
  onSubmit: (input: BusinessRuleInput, isActive: boolean) => Promise<unknown>;
  onDone: () => void;
  onCancel: () => void;
  showActiveToggle?: boolean;
}) {
  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description ?? "");
  const [matchType, setMatchType] = useState<MatchType>(initial.match_type);
  const [priority, setPriority] = useState(initial.priority);
  const [startDate, setStartDate] = useState(initial.effective_start_date ?? "");
  const [endDate, setEndDate] = useState(initial.effective_end_date ?? "");
  const [conditions, setConditions] = useState<BusinessRuleConditionInput[]>(initial.conditions);
  const [actions, setActions] = useState<BusinessRuleActionInput[]>(initial.actions);
  const [isActive, setIsActive] = useState(initial.is_active ?? true);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      onSubmit(
        {
          entity_type: entityType, name, description: description || null, match_type: matchType, priority,
          effective_start_date: startDate || null, effective_end_date: endDate || null, conditions, actions,
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
        <div className="form-field full">
          <label>Rule name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} required />
        </div>
        <div className="form-field full">
          <label>Description (optional)</label>
          <input value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
        <div className="form-field">
          <label>Match</label>
          <select value={matchType} onChange={(e) => setMatchType(e.target.value as MatchType)}>
            {MATCH_TYPES.map((m) => <option key={m} value={m}>{m === "all" ? "All conditions (AND)" : "Any condition (OR)"}</option>)}
          </select>
        </div>
        <div className="form-field">
          <label>Priority (lower runs first)</label>
          <input type="number" value={priority} onChange={(e) => setPriority(Number(e.target.value))} />
        </div>
        <div className="form-field">
          <label>Effective from (optional)</label>
          <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
        </div>
        <div className="form-field">
          <label>Effective until (optional)</label>
          <input type="date" value={endDate} onChange={(e) => setEndDate(e.target.value)} />
        </div>

        <div className="form-field full">
          <label>If</label>
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
          <button className="btn" type="button" onClick={() => setConditions([...conditions, emptyCondition(entityType)])}>
            + Add condition
          </button>
        </div>

        <div className="form-field full">
          <label>Then</label>
          {actions.map((a, i) => (
            <ActionRow
              key={i}
              entityType={entityType}
              customFields={customFields}
              action={a}
              onChange={(next) => setActions(actions.map((x, idx) => (idx === i ? next : x)))}
              onRemove={() => setActions(actions.filter((_, idx) => idx !== i))}
            />
          ))}
          <button className="btn" type="button" onClick={() => setActions([...actions, emptyAction(entityType, customFields)])}>
            + Add action
          </button>
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
          <button className="btn btn-primary" type="submit" disabled={save.isPending || conditions.length === 0 || actions.length === 0}>
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

/** ADM-BR-09 (Should): lets an admin try a hypothetical set of field
 * values - every built-in field, not just the status/stage one, plus
 * every custom field - against every active rule before relying on it,
 * without persisting anything. */
function TestRulesPanel({ entityType, customFields }: { entityType: string; customFields: { key: string; label: string }[] }) {
  const builtinFields = builtinFieldsFor(entityType);
  const [values, setValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(builtinFields.filter((f) => f.field_type === "select").map((f) => [f.key, f.options?.[0] ?? ""])),
  );

  const test = useMutation({
    mutationFn: () => api.testBusinessRules(entityType, values),
  });

  const builtinLabel = (key: string) => builtinFields.find((f) => f.key === key)?.label ?? key;
  const anyEffects = test.data && (
    test.data.blocked || test.data.messages.length > 0 ||
    Object.keys(test.data.field_effects).length > 0 || Object.keys(test.data.set_values).length > 0 ||
    Object.keys(test.data.builtin_field_effects).length > 0 || Object.keys(test.data.builtin_set_values).length > 0
  );

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <h4 style={{ marginTop: 0 }}>Test rules against a hypothetical record</h4>
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
          {test.data.blocked && <div className="error-banner">Save would be blocked: {test.data.blocked}</div>}
          {test.data.messages.length > 0 && (
            <p>Messages: {test.data.messages.join("; ")}</p>
          )}
          {Object.keys(test.data.field_effects).length > 0 && (
            <p>Custom field effects: {Object.entries(test.data.field_effects).map(([k, v]) => `${customFields.find((f) => f.key === k)?.label ?? k}: ${v}`).join(", ")}</p>
          )}
          {Object.keys(test.data.builtin_field_effects).length > 0 && (
            <p>Built-in field effects: {Object.entries(test.data.builtin_field_effects).map(([k, v]) => `${builtinLabel(k)}: ${v}`).join(", ")}</p>
          )}
          {Object.keys(test.data.set_values).length > 0 && (
            <p>Custom values that would be set: {Object.entries(test.data.set_values).map(([k, v]) => `${customFields.find((f) => f.key === k)?.label ?? k} = "${v}"`).join(", ")}</p>
          )}
          {Object.keys(test.data.builtin_set_values).length > 0 && (
            <p>Built-in values that would be set: {Object.entries(test.data.builtin_set_values).map(([k, v]) => `${builtinLabel(k)} = "${v}"`).join(", ")}</p>
          )}
          {!anyEffects && <p className="empty-state">No rule matches this hypothetical record.</p>}
        </div>
      )}
    </div>
  );
}
