import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  builtinTriggerFieldFor,
  CUSTOM_FIELD_ENTITY_TYPES,
  entityTypeLabel,
  RULE_EFFECTS,
  RULE_OPERATORS,
  statusesForEntity,
  TRIGGER_SOURCES,
  type CustomFieldEntityType,
  type FieldRule,
  type FieldRuleInput,
  type RuleEffect,
  type RuleOperator,
  type TriggerSource,
} from "../../lib/types";

function emptyInput(entityType: CustomFieldEntityType): FieldRuleInput {
  return {
    entity_type: entityType,
    trigger_field_source: "builtin",
    trigger_field_key: builtinTriggerFieldFor(entityType),
    operator: "equals",
    trigger_value: "",
    target_field_key: "",
    effect: "require",
    sort_order: 0,
  };
}

/** Plain-English summary of a rule, e.g. "When Status equals Prospect, require Lead Source." */
function describeRule(rule: FieldRule, labelByKey: Map<string, string>): string {
  const triggerLabel =
    rule.trigger_field_source === "builtin"
      ? rule.trigger_field_key === "is_active" ? "Active" : "Status"
      : labelByKey.get(rule.trigger_field_key) ?? rule.trigger_field_key;
  const op = rule.operator === "equals" ? "is" : "is not";
  const effect = rule.effect === "require" ? "require" : "hide";
  const targetLabel = labelByKey.get(rule.target_field_key) ?? rule.target_field_key;
  return `When ${triggerLabel} ${op} "${rule.trigger_value}", ${effect} ${targetLabel}.`;
}

/**
 * Admin screen for FR-RUL conditional business rules - lets an
 * Administrator make a custom field required, or hide it, based on the
 * entity's built-in status or another custom field's value. Rules are
 * enforced server-side (custom_field_service::set_entity_values) and
 * mirrored client-side for live form feedback (see lib/fieldRules.ts).
 */
export function FieldRulesAdmin() {
  const [entityType, setEntityType] = useState<CustomFieldEntityType>(CUSTOM_FIELD_ENTITY_TYPES[0]);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const rules = useQuery({
    queryKey: ["fieldRules", entityType, "all"],
    queryFn: () => api.listFieldRules(entityType, false),
  });
  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType, "all"],
    queryFn: () => api.listCustomFieldDefinitions(entityType, false),
  });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["fieldRules"] });
  }

  const activeDefs = (defs.data ?? []).filter((d) => d.is_active);
  const labelByKey = new Map((defs.data ?? []).map((d) => [d.key, d.label]));
  const editing = rules.data?.find((r) => r.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Business rules</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
          disabled={activeDefs.length === 0}
        >
          + New rule
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Make a custom field required, or hide it, depending on the record's status. The higher sort order wins when two
        rules target the same field.
      </p>
      {activeDefs.length === 0 && (
        <p className="empty-state">Add an active custom field for {entityTypeLabel(entityType)} first - rules always target a custom field.</p>
      )}

      <div className="tab-row">
        {CUSTOM_FIELD_ENTITY_TYPES.map((t) => (
          <button
            key={t}
            className={`tab${entityType === t ? " active" : ""}`}
            onClick={() => {
              setEntityType(t);
              setCreating(false);
              setEditingId(null);
            }}
          >
            {entityTypeLabel(t)}
          </button>
        ))}
      </div>

      {creating && (
        <RuleForm
          entityType={entityType}
          customFields={activeDefs}
          initial={emptyInput(entityType)}
          submitLabel="Add rule"
          onSubmit={(input) => api.createFieldRule(input)}
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
            entity_type: entityType,
            trigger_field_source: editing.trigger_field_source as TriggerSource,
            trigger_field_key: editing.trigger_field_key,
            operator: editing.operator as RuleOperator,
            trigger_value: editing.trigger_value,
            target_field_key: editing.target_field_key,
            effect: editing.effect as RuleEffect,
            sort_order: editing.sort_order,
            is_active: editing.is_active,
          }}
          submitLabel="Save"
          onSubmit={(input, isActive) => api.updateFieldRule(editing.id, { ...input, is_active: isActive })}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
          showActiveToggle
        />
      )}

      {rules.isLoading && <p>Loading...</p>}
      {rules.data && rules.data.length === 0 && <p className="empty-state">No business rules defined for {entityTypeLabel(entityType)} yet.</p>}
      {rules.data && rules.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Rule</th>
              <th>Sort order</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rules.data.map((r) => (
              <tr key={r.id}>
                <td>{describeRule(r, labelByKey)}</td>
                <td>{r.sort_order}</td>
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
  customFields,
  initial,
  submitLabel,
  onSubmit,
  onDone,
  onCancel,
  showActiveToggle,
}: {
  entityType: CustomFieldEntityType;
  customFields: { key: string; label: string }[];
  initial: FieldRuleInput & { is_active?: boolean };
  submitLabel: string;
  onSubmit: (input: FieldRuleInput, isActive: boolean) => Promise<unknown>;
  onDone: () => void;
  onCancel: () => void;
  showActiveToggle?: boolean;
}) {
  const [source, setSource] = useState<TriggerSource>(initial.trigger_field_source);
  const [triggerKey, setTriggerKey] = useState(initial.trigger_field_key);
  const [operator, setOperator] = useState<RuleOperator>(initial.operator);
  const [triggerValue, setTriggerValue] = useState(initial.trigger_value);
  const [targetKey, setTargetKey] = useState(initial.target_field_key || customFields[0]?.key || "");
  const [effect, setEffect] = useState<RuleEffect>(initial.effect);
  const [sortOrder, setSortOrder] = useState(initial.sort_order);
  const [isActive, setIsActive] = useState(initial.is_active ?? true);
  const [error, setError] = useState<string | null>(null);

  const statuses = statusesForEntity(entityType);
  const builtinField = builtinTriggerFieldFor(entityType);
  const isStatusTrigger = source === "builtin";

  const save = useMutation({
    mutationFn: () =>
      onSubmit(
        {
          entity_type: entityType,
          trigger_field_source: source,
          trigger_field_key: triggerKey,
          operator,
          trigger_value: triggerValue,
          target_field_key: targetKey,
          effect,
          sort_order: sortOrder,
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
          <label>Trigger source</label>
          <select
            value={source}
            onChange={(e) => {
              const next = e.target.value as TriggerSource;
              setSource(next);
              setTriggerKey(next === "builtin" ? builtinField : customFields[0]?.key ?? "");
              setTriggerValue("");
            }}
          >
            {TRIGGER_SOURCES.map((s) => (
              <option key={s} value={s}>
                {s === "builtin" ? "Built-in field" : "Custom field"}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Trigger field</label>
          {isStatusTrigger ? (
            <select value={triggerKey} onChange={(e) => setTriggerKey(e.target.value)}>
              <option value={builtinField}>{builtinField === "is_active" ? "Active" : "Status"}</option>
            </select>
          ) : (
            <select value={triggerKey} onChange={(e) => setTriggerKey(e.target.value)}>
              {customFields.map((f) => (
                <option key={f.key} value={f.key}>
                  {f.label}
                </option>
              ))}
            </select>
          )}
        </div>
        <div className="form-field">
          <label>Operator</label>
          <select value={operator} onChange={(e) => setOperator(e.target.value as RuleOperator)}>
            {RULE_OPERATORS.map((o) => (
              <option key={o} value={o}>
                {o === "equals" ? "equals" : "does not equal"}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Trigger value</label>
          {isStatusTrigger ? (
            <select value={triggerValue} onChange={(e) => setTriggerValue(e.target.value)} required>
              <option value="">— Select —</option>
              {statuses.map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          ) : (
            <input value={triggerValue} onChange={(e) => setTriggerValue(e.target.value)} required />
          )}
        </div>
        <div className="form-field">
          <label>Target field</label>
          <select value={targetKey} onChange={(e) => setTargetKey(e.target.value)} required>
            {customFields.map((f) => (
              <option key={f.key} value={f.key}>
                {f.label}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Effect</label>
          <select value={effect} onChange={(e) => setEffect(e.target.value as RuleEffect)}>
            {RULE_EFFECTS.map((e) => (
              <option key={e} value={e}>
                {e === "require" ? "Require" : "Hide"}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>Sort order</label>
          <input
            type="number"
            value={sortOrder}
            onChange={(e) => setSortOrder(Number(e.target.value))}
          />
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
          <button className="btn btn-primary" type="submit" disabled={save.isPending || !targetKey}>
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
