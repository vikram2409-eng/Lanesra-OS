import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { AppScopeFilter, AppScopeSelect, matchesAppFilter, useApps } from "../../components/AppScope";
import { BuiltinValueInput } from "../../components/BuiltinValueInput";
import { describeGroupedConditions, groupConditionIndices, newGroupId } from "../../lib/conditionGroups";
import { describeConflict, findRuleConflicts } from "../../lib/businessRuleConflicts";
import { BUSINESS_RULE_TEMPLATES, type BusinessRuleTemplate } from "../../lib/businessRuleTemplates";
import {
  builtinFieldsFor,
  builtinTriggerFieldFor,
  CONDITION_OPERATORS,
  CURRENT_ACTION_TYPES,
  CUSTOM_FIELD_ENTITY_TYPES,
  entityTypeLabel,
  FIELD_TARGETED_ACTIONS,
  LIST_SEPARATOR,
  MATCH_TYPES,
  MESSAGE_ACTIONS,
  TRIGGER_SOURCES,
  VALUE_REQUIRED_ACTIONS,
  VALUELESS_OPERATORS,
  type ActionType,
  type AppDefinition,
  type BusinessRule,
  type BusinessRuleActionInput,
  type BusinessRuleBranchRole,
  type BusinessRuleConditionInput,
  type BusinessRuleInput,
  type ConditionOperator,
  type CustomFieldDefinition,
  type MatchType,
  type RelationshipDefinition,
  type TriggerSource,
} from "../../lib/types";

/** Only key/label/field_type/options are used here - every caller passes a
 * full CustomFieldDefinition (or CustomFieldDefinition[]), this just names
 * the slice this file actually reads. */
type CustomFieldLite = Pick<CustomFieldDefinition, "key" | "label" | "field_type" | "options">;

const OPERATOR_LABELS: Record<ConditionOperator, string> = {
  equals: "equals", not_equals: "does not equal", contains: "contains", not_contains: "does not contain",
  starts_with: "starts with", ends_with: "ends with", in_list: "is one of", not_in_list: "is not one of",
  is_empty: "is empty", is_not_empty: "is not empty", greater_than: "is greater than", less_than: "is less than",
  on_or_after: "is on or after", on_or_before: "is on or before",
};

const ACTION_LABELS: Record<ActionType, string> = {
  require: "Require field", hide: "Hide field", show: "Show field", lock: "Make read-only", editable: "Make editable",
  set_default: "Set default value", set_value: "Set field value", clear_value: "Clear field value",
  restrict_choices: "Restrict choices", block_save: "Block save", show_error: "Show error", show_warning: "Show warning",
  show_message: "Show message (legacy)",
};

const ACTION_ICONS: Record<ActionType, string> = {
  require: "✅", hide: "🙈", show: "👁️", lock: "🔒", editable: "🔓",
  set_default: "🔧", set_value: "✏️", clear_value: "🧹", restrict_choices: "🎯",
  block_save: "🚫", show_error: "❗", show_warning: "⚠️", show_message: "💬",
};

/** Action-type legend the summary panel shows - grouped exactly like the
 * "Effect types you can use" design mockup: Validation (require/block/
 * error/warning - govern whether the save proceeds), Field behavior
 * (show/hide/lock/editable/set/clear a value - govern how a field
 * renders), Other (restrict_choices). "Trigger approval" from the mockup
 * is deliberately not offered yet. Built from CURRENT_ACTION_TYPES/
 * ACTION_LABELS so it can never list an action the engine doesn't
 * actually support. */
const ACTION_LEGEND: { title: string; types: ActionType[] }[] = [
  { title: "Validation", types: ["require", "block_save", "show_error", "show_warning"] },
  { title: "Field behavior", types: ["show", "hide", "lock", "editable", "set_value", "clear_value", "set_default"] },
  { title: "Other", types: ["restrict_choices"] },
];

/** Plain array move, used by the lane board's native HTML5 drag-and-drop to
 * reorder conditions/actions within a lane. */
function reorder<T>(list: T[], from: number, to: number): T[] {
  if (from === to) return list;
  const next = list.slice();
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved);
  return next;
}

function emptyCondition(entityType: string): BusinessRuleConditionInput {
  return {
    field_source: "builtin", field_key: builtinTriggerFieldFor(entityType), operator: "equals", value: "",
    compare_field_source: null, compare_field_key: null, group_id: null, relationship_definition_id: null,
  };
}

/** Defaults to a custom field target when one exists (matches this admin
 * screen's original behavior); falls back to the first actionable
 * built-in field so "+ Add action" still works on an entity with no
 * custom fields defined at all. */
function emptyAction(entityType: string, customFields: CustomFieldLite[]): BusinessRuleActionInput {
  if (customFields.length > 0) {
    return { action_type: "require", target_field_key: customFields[0].key, target_field_source: "custom", action_value: null, message: null };
  }
  const builtin = builtinFieldsFor(entityType).find((f) => f.actionable);
  if (builtin) {
    return { action_type: "require", target_field_key: builtin.key, target_field_source: "builtin", action_value: null, message: null };
  }
  // No actionable target at all (e.g. Quote/Order/Invoice with no custom
  // fields defined) - default to a warning action, the only kind that
  // doesn't need one.
  return { action_type: "show_warning", target_field_key: null, target_field_source: "custom", action_value: null, message: "" };
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
    case "show": return `show ${target}`;
    case "lock": return `lock ${target}`;
    case "editable": return `unlock ${target}`;
    case "set_default": return `default ${target} to "${a.action_value ?? ""}"`;
    case "set_value": return `force ${target} to "${a.action_value ?? ""}"`;
    case "clear_value": return `clear ${target}`;
    case "restrict_choices": return `restrict ${target} to ${(a.action_value ?? "").split(LIST_SEPARATOR).filter(Boolean).join(", ") || "no options"}`;
    case "block_save": return `block save: "${a.message ?? ""}"`;
    case "show_error": return `show error: "${a.message ?? ""}"`;
    case "show_warning": return `show warning: "${a.message ?? ""}"`;
    case "show_message": return `show message: "${a.message ?? ""}"`;
    default: return a.action_type;
  }
}

/**
 * Admin extensibility Phase C (spec §22/ADM-BR), extended by the second
 * Admin Automation & Customization addendum round: a no-code IF (AND/OR,
 * plus one level of nested OR-groups) / THEN (any number of actions) rule
 * builder - require/hide/show/lock/editable a field, set/clear its value,
 * restrict a select field's choices, block the save with a custom
 * message, or show a non-blocking error/warning. Enforced server-side
 * (custom_field_service::set_entity_values) and mirrored client-side for
 * live form feedback for the cosmetic field-behavior actions (see
 * lib/businessRules.ts).
 */
/** Chain ordering: the "if" head first, then every "else_if" in priority
 * order, the terminal "else" (if any) last - the same left-to-right reading
 * order the board lays the lane-sets out in. */
function branchRank(role: BusinessRuleBranchRole): number {
  return role === "if" ? 0 : role === "else_if" ? 1 : 2;
}

export function BusinessRulesAdmin() {
  const [entityType, setEntityType] = useState<string>(CUSTOM_FIELD_ENTITY_TYPES[0]);
  const [creating, setCreating] = useState(false);
  const [pendingTemplate, setPendingTemplate] = useState<BusinessRuleTemplate | null>(null);
  const [showTemplatePicker, setShowTemplatePicker] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [historyForId, setHistoryForId] = useState<string | null>(null);
  const [appFilter, setAppFilter] = useState<"all" | "none" | string>("all");
  const queryClient = useQueryClient();

  const apps = useApps();
  const appList = apps.data ?? [];

  const duplicate = useMutation({
    mutationFn: (id: string) => api.duplicateBusinessRule(id),
    onSuccess: (copy) => {
      queryClient.invalidateQueries({ queryKey: ["businessRules"] });
      setEditingId(copy.id);
    },
  });

  const addBranch = useMutation({
    mutationFn: (vars: { parentId: string; role: "else_if" | "else" }) => api.createBusinessRuleBranch(vars.parentId, vars.role),
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ["businessRules"] });
      setEditingId(created.id);
    },
  });

  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];
  const currentLabel = entityTabs.find((t) => t.key === entityType)?.label ?? entityType;

  const rules = useQuery({ queryKey: ["businessRules", entityType, "all"], queryFn: () => api.listBusinessRules(entityType, false) });
  const defs = useQuery({ queryKey: ["customFieldDefinitions", entityType, "all"], queryFn: () => api.listCustomFieldDefinitions(entityType, false) });
  const relationshipDefs = useQuery({ queryKey: ["relationshipDefinitions", "active"], queryFn: () => api.listRelationshipDefinitions(true) });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["businessRules"] });
  }

  const activeDefs = (defs.data ?? []).filter((d) => d.is_active);
  const labelByKey = new Map((defs.data ?? []).map((d) => [d.key, d.label]));
  const editing = rules.data?.find((r) => r.id === editingId) ?? null;
  const historyRule = rules.data?.find((r) => r.id === historyForId) ?? null;
  const visibleRules = (rules.data ?? []).filter((r) => matchesAppFilter(r.app_id, appFilter));
  const newRuleAppId = appFilter !== "all" && appFilter !== "none" ? appFilter : null;
  const conflicts = findRuleConflicts(rules.data ?? []);

  const chainId = editing ? editing.branch_group_id ?? editing.id : null;
  const chain = chainId
    ? (rules.data ?? [])
        .filter((r) => r.id === chainId || r.branch_group_id === chainId)
        .sort((a, b) => branchRank(a.branch_role) - branchRank(b.branch_role) || a.priority - b.priority)
    : [];
  const chainHasElse = chain.some((r) => r.branch_role === "else");
  const lastChainMember = chain[chain.length - 1] ?? null;

  function startCreating(template: BusinessRuleTemplate | null) {
    setPendingTemplate(template);
    setCreating(true);
    setShowTemplatePicker(false);
    setEditingId(null);
    setHistoryForId(null);
  }

  const newRuleSeed = pendingTemplate ? pendingTemplate.build(entityType, activeDefs) : null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Business rules</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            if (creating) {
              setCreating(false);
            } else {
              setShowTemplatePicker(true);
            }
            setEditingId(null);
            setHistoryForId(null);
          }}
        >
          {creating ? "Cancel" : "+ New rule"}
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Build an IF (AND/OR conditions) / THEN (actions) rule against any built-in or custom field on {currentLabel} - drag
        conditions into the IF lane and effects into the THEN lane, then branch to ELSE IF/ELSE lanes for more complex rules.
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
              setHistoryForId(null);
            }}
          >
            {t.label}
          </button>
        ))}
      </div>

      {showTemplatePicker && (
        <div className="card" style={{ marginBottom: 16 }}>
          <div className="toolbar">
            <h4 style={{ margin: 0 }}>Start a new rule</h4>
            <button className="btn" onClick={() => setShowTemplatePicker(false)}>Close</button>
          </div>
          <div className="rule-template-grid">
            <button className="rule-template-card" onClick={() => startCreating(null)}>
              <h4>Start from scratch</h4>
              <p>A blank IF/THEN rule - pick every condition and action yourself.</p>
            </button>
            {BUSINESS_RULE_TEMPLATES.map((t) => (
              <button key={t.key} className="rule-template-card" onClick={() => startCreating(t)}>
                <h4>{t.label}</h4>
                <p>{t.description}</p>
              </button>
            ))}
          </div>
        </div>
      )}

      {conflicts.length > 0 && !creating && !editing && !historyRule && (
        <div className="rule-conflict-banner" style={{ marginBottom: 16 }}>
          <strong>{conflicts.length} possible rule conflict{conflicts.length > 1 ? "s" : ""} on {currentLabel}</strong>
          <ul>
            {conflicts.map((c, i) => (
              <li key={i}>{describeConflict(c, fieldLabel(entityType, c.fieldSource, c.fieldKey, labelByKey))}</li>
            ))}
          </ul>
        </div>
      )}

      {historyRule && (
        <RuleHistoryPanel
          entityType={entityType}
          customFields={activeDefs}
          ruleId={historyRule.id}
          ruleName={historyRule.name}
          onDone={() => {
            invalidate();
          }}
          onClose={() => setHistoryForId(null)}
        />
      )}

      {creating && !historyRule && (
        <RuleForm
          entityType={entityType}
          customFields={activeDefs}
          relationshipDefs={relationshipDefs.data ?? []}
          apps={appList}
          branchRole="if"
          initial={{
            entity_type: entityType, name: newRuleSeed?.name ?? "", description: null,
            match_type: newRuleSeed?.match_type ?? "all", priority: 0,
            effective_start_date: null, effective_end_date: null, app_id: newRuleAppId,
            conditions: newRuleSeed?.conditions ?? [emptyCondition(entityType)],
            actions: newRuleSeed?.actions ?? [emptyAction(entityType, activeDefs)],
          }}
          submitLabel="Add rule"
          onSubmit={(input) => api.createBusinessRule(input)}
          onDone={() => {
            invalidate();
            setCreating(false);
            setPendingTemplate(null);
          }}
          onCancel={() => {
            setCreating(false);
            setPendingTemplate(null);
          }}
        />
      )}

      {editing && !historyRule && (
        <div className="rule-board">
          <div className="rule-chain">
            {chain.map((member) =>
              member.id === editing.id ? (
                <div className="rule-lane-set" key={member.id}>
                  <RuleForm
                    entityType={entityType}
                    customFields={activeDefs}
                    relationshipDefs={relationshipDefs.data ?? []}
                    apps={appList}
                    branchRole={member.branch_role}
                    initial={{
                      entity_type: entityType, name: editing.name, description: editing.description, match_type: editing.match_type,
                      priority: editing.priority, effective_start_date: editing.effective_start_date, effective_end_date: editing.effective_end_date,
                      app_id: editing.app_id,
                      conditions: editing.conditions.map((c) => ({
                        field_source: c.field_source, field_key: c.field_key, operator: c.operator, value: c.value,
                        compare_field_source: c.compare_field_source, compare_field_key: c.compare_field_key, group_id: c.group_id,
                        relationship_definition_id: c.relationship_definition_id,
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
                    allowEmptyConditions={member.branch_role === "else"}
                  />
                </div>
              ) : (
                <RuleBranchSummaryCard
                  key={member.id}
                  entityType={entityType}
                  rule={member}
                  labelByKey={labelByKey}
                  onEdit={() => setEditingId(member.id)}
                />
              ),
            )}
            {lastChainMember && (
              <div className="rule-lane-set" style={{ width: 220, justifyContent: "center" }}>
                <button className="btn" disabled={addBranch.isPending} onClick={() => addBranch.mutate({ parentId: lastChainMember.id, role: "else_if" })}>
                  + Add ELSE IF
                </button>
                {!chainHasElse && (
                  <button className="btn" disabled={addBranch.isPending} onClick={() => addBranch.mutate({ parentId: lastChainMember.id, role: "else" })}>
                    + Add ELSE
                  </button>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {!creating && !editing && !historyRule && <AppScopeFilter apps={appList} value={appFilter} onChange={setAppFilter} />}

      {rules.isLoading && !creating && !editing && !historyRule && <p>Loading...</p>}
      {rules.data && visibleRules.length === 0 && !creating && !editing && !historyRule && (
        <p className="empty-state">
          {rules.data.length === 0 ? `No business rules defined for ${currentLabel} yet.` : "No rules match this app filter."}
        </p>
      )}
      {visibleRules.length > 0 && !creating && !editing && !historyRule && (
        <table>
          <thead>
            <tr>
              <th>Rule</th>
              <th>Branch</th>
              <th>If</th>
              <th>Then</th>
              <th>Priority</th>
              <th>App</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {visibleRules.map((r) => (
              <tr key={r.id}>
                <td>{r.name}</td>
                <td>{r.branch_group_id ? <span className={`rule-branch-badge ${r.branch_role}`}>{r.branch_role.replace("_", " ")}</span> : "—"}</td>
                <td>{r.conditions.length === 0 ? <em>always</em> : describeGroupedConditions(r.conditions, r.match_type, (c) => describeCondition(entityType, c, labelByKey))}</td>
                <td>{r.actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}</td>
                <td>{r.priority}</td>
                <td>{r.app_id ? appList.find((a) => a.id === r.app_id)?.name ?? "—" : <span style={{ color: "var(--text-muted)" }}>Workspace-wide</span>}</td>
                <td>
                  <span className={`badge${r.is_active ? " badge-success" : ""}`}>{r.is_active ? "Active" : "Inactive"}</span>
                  {r.is_protected && <span className="badge" style={{ marginLeft: 4 }}>System</span>}
                </td>
                <td>
                  <div style={{ display: "flex", gap: 6 }}>
                    <button className="btn" onClick={() => setEditingId(r.id)} disabled={r.is_protected}>
                      Edit
                    </button>
                    <button className="btn" onClick={() => duplicate.mutate(r.id)} disabled={duplicate.isPending} title="Duplicate as an inactive draft">
                      Duplicate
                    </button>
                    <button className="btn" onClick={() => setHistoryForId(r.id)} title="Version history">
                      History
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

/** A collapsed, read-only lane-set for a chain member that isn't the one
 * currently being edited - click "Edit this branch" to swap into it (one
 * rule is edited at a time; the whole chain stays visible either way). */
function RuleBranchSummaryCard({
  entityType,
  rule,
  labelByKey,
  onEdit,
}: {
  entityType: string;
  rule: BusinessRule;
  labelByKey: Map<string, string>;
  onEdit: () => void;
}) {
  return (
    <div className="rule-lane-set">
      <div className="rule-lane-set-header">
        <span className={`rule-branch-badge ${rule.branch_role}`}>{rule.branch_role.replace("_", " ")}</span>
        <strong style={{ fontSize: 13 }}>{rule.name}</strong>
      </div>
      <div className="rule-lane">
        <div className="rule-lane-title">If</div>
        <p style={{ fontSize: 13, margin: 0 }}>
          {rule.conditions.length === 0 ? "Always (no conditions)" : describeGroupedConditions(rule.conditions, rule.match_type, (c) => describeCondition(entityType, c, labelByKey))}
        </p>
      </div>
      <div className="rule-lane">
        <div className="rule-lane-title">Then</div>
        <p style={{ fontSize: 13, margin: 0 }}>{rule.actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}</p>
      </div>
      <button className="btn" type="button" onClick={onEdit}>
        Edit this branch
      </button>
    </div>
  );
}

/** Relationships where `entityType` has at most one linked record - the
 * only shape "cross-record validation" can read a determinate value
 * from (the "many" side of a many_to_one, or either side of a
 * one_to_one). See the Rust engine's `resolve_related_field` doc
 * comment for the identical restriction. */
function eligibleRelatedRecordSources(entityType: string, relationshipDefs: RelationshipDefinition[]): RelationshipDefinition[] {
  return relationshipDefs.filter(
    (r) =>
      (r.source_entity_type === entityType && (r.relationship_type === "many_to_one" || r.relationship_type === "one_to_one")) ||
      (r.target_entity_type === entityType && r.relationship_type === "one_to_one"),
  );
}

function ConditionRow({
  entityType,
  customFields,
  relationshipDefs,
  condition,
  onChange,
  onRemove,
}: {
  entityType: string;
  customFields: CustomFieldLite[];
  relationshipDefs: RelationshipDefinition[];
  condition: BusinessRuleConditionInput;
  onChange: (c: BusinessRuleConditionInput) => void;
  onRemove: () => void;
}) {
  const eligibleRelationships = eligibleRelatedRecordSources(entityType, relationshipDefs);
  const relatedDef = relationshipDefs.find((r) => r.id === condition.relationship_definition_id) ?? null;
  const relatedType = relatedDef ? (relatedDef.source_entity_type === entityType ? relatedDef.target_entity_type : relatedDef.source_entity_type) : null;
  // Same pattern WorkflowAutomationAdmin's update_related_record action
  // editor already uses to populate the *other* object's fields once a
  // relationship is chosen.
  const relatedTypeCustomFields = useQuery({
    queryKey: ["customFieldDefinitions", relatedType, "all"],
    queryFn: () => api.listCustomFieldDefinitions(relatedType as string, false),
    enabled: !!relatedType,
  });
  const activeRelatedCustomFields = (relatedTypeCustomFields.data ?? []).filter((d) => d.is_active);

  const builtinFields = relatedType ? builtinFieldsFor(relatedType) : builtinFieldsFor(entityType);
  const activeCustomFields = relatedDef ? activeRelatedCustomFields : customFields;
  const isBuiltin = condition.field_source === "builtin";
  const needsValue = !VALUELESS_OPERATORS.includes(condition.operator);
  const isListOp = condition.operator === "in_list" || condition.operator === "not_in_list";
  const comparesToField = condition.compare_field_key !== null;
  const selectedBuiltin = isBuiltin ? builtinFields.find((f) => f.key === condition.field_key) : undefined;
  const compareFields = condition.compare_field_source === "builtin" ? builtinFieldsFor(entityType) : customFields;

  return (
    <div className="builder-row-card">
      {eligibleRelationships.length > 0 && (
        <select
          value={condition.relationship_definition_id ?? ""}
          onChange={(e) => {
            const defId = e.target.value || null;
            const def = relationshipDefs.find((r) => r.id === defId) ?? null;
            const otherType = def ? (def.source_entity_type === entityType ? def.target_entity_type : def.source_entity_type) : entityType;
            onChange({
              ...condition,
              relationship_definition_id: defId,
              field_source: "builtin",
              field_key: builtinFieldsFor(otherType)[0]?.key ?? "",
              value: "",
            });
          }}
          title="Read this condition's field from this record, or from a related record"
        >
          <option value="">This record</option>
          {eligibleRelationships.map((r) => (
            <option key={r.id} value={r.id}>{r.source_entity_type === entityType ? r.forward_label : r.reverse_label} (related record)</option>
          ))}
        </select>
      )}
      <select
        value={condition.field_source}
        onChange={(e) => {
          const source = e.target.value as TriggerSource;
          onChange({ ...condition, field_source: source, field_key: source === "builtin" ? builtinFields[0]?.key ?? "" : activeCustomFields[0]?.key ?? "", value: "" });
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
          {activeCustomFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
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
      <button className="builder-row-remove" type="button" onClick={onRemove} title="Remove condition">✕</button>
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
  customFields: CustomFieldLite[];
  action: BusinessRuleActionInput;
  onChange: (a: BusinessRuleActionInput) => void;
  onRemove: () => void;
}) {
  const isRestrictChoices = action.action_type === "restrict_choices";
  // restrict_choices only makes sense on a select-typed field - narrow
  // both target pickers to those when it's the chosen action, same
  // constraint business_rule_service::validate_actions enforces.
  const actionableBuiltinFields = builtinFieldsFor(entityType).filter((f) => f.actionable && (!isRestrictChoices || f.field_type === "select"));
  const targetableCustomFields = customFields.filter((f) => !isRestrictChoices || f.field_type === "select");
  const isFieldTargeted = FIELD_TARGETED_ACTIONS.includes(action.action_type);
  const isMessageAction = MESSAGE_ACTIONS.includes(action.action_type);
  const needsValue = VALUE_REQUIRED_ACTIONS.includes(action.action_type) && !isRestrictChoices;
  const isBuiltinTarget = action.target_field_source === "builtin";
  const selectedBuiltin = isBuiltinTarget ? actionableBuiltinFields.find((f) => f.key === action.target_field_key) : undefined;
  const selectedCustom = !isBuiltinTarget ? targetableCustomFields.find((f) => f.key === action.target_field_key) : undefined;
  const restrictableOptions = isBuiltinTarget ? selectedBuiltin?.options ?? [] : selectedCustom?.options ?? [];
  const selectedChoices = (action.action_value ?? "").split(LIST_SEPARATOR).filter(Boolean);
  // The dropdown only offers CURRENT_ACTION_TYPES for a *new* pick, but
  // must still be able to display a legacy show_message action loaded
  // from an existing rule without silently changing its value.
  const dropdownOptions = CURRENT_ACTION_TYPES.includes(action.action_type) ? CURRENT_ACTION_TYPES : [...CURRENT_ACTION_TYPES, action.action_type];

  return (
    <div className="builder-row-card">
      <span style={{ fontSize: 15 }}>{ACTION_ICONS[action.action_type]}</span>
      <select
        value={action.action_type}
        onChange={(e) => {
          const type = e.target.value as ActionType;
          const nowRestrict = type === "restrict_choices";
          const fields = nowRestrict
            ? builtinFieldsFor(entityType).filter((f) => f.actionable && f.field_type === "select")
            : builtinFieldsFor(entityType).filter((f) => f.actionable);
          const customChoices = nowRestrict ? customFields.filter((f) => f.field_type === "select") : customFields;
          const stillValid = FIELD_TARGETED_ACTIONS.includes(type) && action.target_field_key &&
            (action.target_field_source === "builtin" ? fields : customChoices).some((f) => f.key === action.target_field_key);
          const fallback = customChoices[0]?.key ?? fields[0]?.key ?? "";
          onChange({
            action_type: type,
            target_field_key: FIELD_TARGETED_ACTIONS.includes(type) ? (stillValid ? action.target_field_key : fallback) : null,
            target_field_source: action.target_field_source,
            action_value: VALUE_REQUIRED_ACTIONS.includes(type) ? (nowRestrict ? "" : action.action_value ?? "") : null,
            message: MESSAGE_ACTIONS.includes(type) ? action.message ?? "" : null,
          });
        }}
      >
        {dropdownOptions.map((t) => <option key={t} value={t}>{ACTION_LABELS[t]}</option>)}
      </select>
      {isFieldTargeted && (
        <>
          <select
            value={action.target_field_source}
            onChange={(e) => {
              const source = e.target.value as TriggerSource;
              const first = source === "builtin" ? actionableBuiltinFields[0]?.key ?? "" : targetableCustomFields[0]?.key ?? "";
              onChange({ ...action, target_field_source: source, target_field_key: first, action_value: isRestrictChoices ? "" : null });
            }}
          >
            <option value="custom" disabled={targetableCustomFields.length === 0}>Custom field</option>
            <option value="builtin" disabled={actionableBuiltinFields.length === 0}>Built-in field</option>
          </select>
          {isBuiltinTarget ? (
            <select value={action.target_field_key ?? ""} onChange={(e) => onChange({ ...action, target_field_key: e.target.value, action_value: isRestrictChoices ? "" : null })} required>
              {actionableBuiltinFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
          ) : (
            <select value={action.target_field_key ?? ""} onChange={(e) => onChange({ ...action, target_field_key: e.target.value, action_value: isRestrictChoices ? "" : action.action_value })} required>
              {targetableCustomFields.map((f) => <option key={f.key} value={f.key}>{f.label}</option>)}
            </select>
          )}
        </>
      )}
      {needsValue && (isBuiltinTarget ? (
        <BuiltinValueInput field={selectedBuiltin} value={action.action_value ?? ""} onChange={(v) => onChange({ ...action, action_value: v })} />
      ) : (
        <input value={action.action_value ?? ""} onChange={(e) => onChange({ ...action, action_value: e.target.value })} placeholder="Value" required style={{ width: 140 }} />
      ))}
      {isRestrictChoices && (
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap", alignItems: "center" }}>
          {restrictableOptions.length === 0 && <span style={{ fontSize: 12, color: "var(--text-muted)" }}>Pick a select field first</span>}
          {restrictableOptions.map((o) => (
            <label key={o} style={{ display: "flex", gap: 4, alignItems: "center", fontSize: 12 }}>
              <input
                type="checkbox"
                checked={selectedChoices.includes(o)}
                onChange={(e) => {
                  const next = e.target.checked ? [...selectedChoices, o] : selectedChoices.filter((c) => c !== o);
                  onChange({ ...action, action_value: next.join(LIST_SEPARATOR) });
                }}
              />
              {o}
            </label>
          ))}
        </div>
      )}
      {isMessageAction && (
        <input value={action.message ?? ""} onChange={(e) => onChange({ ...action, message: e.target.value })} placeholder="Message shown to the user" required style={{ width: 260 }} />
      )}
      <button className="builder-row-remove" type="button" onClick={onRemove} title="Remove action">✕</button>
    </div>
  );
}

function RuleForm({
  entityType,
  customFields,
  relationshipDefs,
  apps,
  initial,
  submitLabel,
  onSubmit,
  onDone,
  onCancel,
  showActiveToggle,
  branchRole = "if",
  allowEmptyConditions = false,
}: {
  entityType: string;
  customFields: CustomFieldLite[];
  relationshipDefs: RelationshipDefinition[];
  apps: AppDefinition[];
  initial: BusinessRuleInput & { is_active?: boolean };
  submitLabel: string;
  onSubmit: (input: BusinessRuleInput, isActive: boolean) => Promise<unknown>;
  onDone: () => void;
  onCancel: () => void;
  showActiveToggle?: boolean;
  /** Business Rule Board 2.0: which lane-set this rule is in its chain -
   * purely a badge/label, "if" (the default) when this rule isn't part of
   * a chain at all. */
  branchRole?: BusinessRuleBranchRole;
  /** True only for an "else" branch - lets the IF lane sit empty ("always
   * matches") instead of requiring at least one condition. */
  allowEmptyConditions?: boolean;
}) {
  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description ?? "");
  const [matchType, setMatchType] = useState<MatchType>(initial.match_type);
  const [priority, setPriority] = useState(initial.priority);
  const [startDate, setStartDate] = useState(initial.effective_start_date ?? "");
  const [endDate, setEndDate] = useState(initial.effective_end_date ?? "");
  const [appId, setAppId] = useState<string | null>(initial.app_id);
  const [conditions, setConditions] = useState<BusinessRuleConditionInput[]>(initial.conditions);
  const [actions, setActions] = useState<BusinessRuleActionInput[]>(initial.actions);
  const [isActive, setIsActive] = useState(initial.is_active ?? true);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [draggedUnit, setDraggedUnit] = useState<number | null>(null);
  const [unitDropIndex, setUnitDropIndex] = useState<number | null>(null);
  const [draggedAction, setDraggedAction] = useState<number | null>(null);
  const [actionDropIndex, setActionDropIndex] = useState<number | null>(null);

  const save = useMutation({
    mutationFn: (nextActive: boolean) =>
      onSubmit(
        {
          entity_type: entityType, name, description: description || null, match_type: matchType, priority,
          effective_start_date: startDate || null, effective_end_date: endDate || null, app_id: appId, conditions, actions,
        },
        nextActive,
      ),
    onSuccess: (_, nextActive) => {
      setIsActive(nextActive);
      onDone();
    },
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this rule"),
  });

  function updateConditionAt(i: number, next: BusinessRuleConditionInput) {
    setConditions(conditions.map((c, idx) => (idx === i ? next : c)));
  }
  /** Removing a condition that's the last remaining member of its OR
   * group auto-ungroups it - a one-member "group" is meaningless, and
   * this keeps the array from accumulating orphaned group_ids. */
  function removeConditionAt(i: number) {
    const groupId = conditions[i].group_id;
    const remaining = conditions.filter((_, idx) => idx !== i);
    if (groupId && remaining.filter((c) => c.group_id === groupId).length === 1) {
      setConditions(remaining.map((c) => (c.group_id === groupId ? { ...c, group_id: null } : c)));
    } else {
      setConditions(remaining);
    }
  }
  const conditionUnits = groupConditionIndices(conditions);

  /** Drops the unit being dragged into gap `to` (0..conditionUnits.length,
   * the position *between* rendered units/dropzones) - reordering whole
   * units (a lone condition, or a whole OR group moving together) rather
   * than individual conditions, matching the issue's "drag conditions/
   * groups into the IF lane" wording. Flattens the reordered units back
   * into a flat conditions array, preserving each group's internal order. */
  function dropConditionUnit(to: number) {
    if (draggedUnit === null) return;
    const adjusted = to > draggedUnit ? to - 1 : to;
    const reordered = reorder(conditionUnits, draggedUnit, adjusted);
    setConditions(reordered.flatMap((u) => (u.kind === "single" ? [conditions[u.index]] : u.indices.map((i) => conditions[i]))));
    setDraggedUnit(null);
    setUnitDropIndex(null);
  }

  function dropAction(to: number) {
    if (draggedAction === null) return;
    const adjusted = to > draggedAction ? to - 1 : to;
    setActions(reorder(actions, draggedAction, adjusted));
    setDraggedAction(null);
    setActionDropIndex(null);
  }

  function ConditionDropZone({ index }: { index: number }) {
    return (
      <div
        className={`rule-lane-dropzone${unitDropIndex === index ? " drop-over" : ""}`}
        onDragOver={(e) => {
          if (draggedUnit === null) return;
          e.preventDefault();
          setUnitDropIndex(index);
        }}
        onDragLeave={() => setUnitDropIndex((v) => (v === index ? null : v))}
        onDrop={(e) => {
          e.preventDefault();
          dropConditionUnit(index);
        }}
      />
    );
  }

  function ActionDropZone({ index }: { index: number }) {
    return (
      <div
        className={`rule-lane-dropzone${actionDropIndex === index ? " drop-over" : ""}`}
        onDragOver={(e) => {
          if (draggedAction === null) return;
          e.preventDefault();
          setActionDropIndex(index);
        }}
        onDragLeave={() => setActionDropIndex((v) => (v === index ? null : v))}
        onDrop={(e) => {
          e.preventDefault();
          dropAction(index);
        }}
      />
    );
  }

  // Rule summary panel: computed live from the form state below, not
  // stored separately - "what will this rule actually do" at a glance.
  const fieldLabels = Array.from(
    new Set(
      actions
        .filter((a) => a.target_field_key)
        .map((a) => fieldLabel(entityType, a.target_field_source, a.target_field_key as string, new Map(customFields.map((f) => [f.key, f.label])))),
    ),
  );
  const blocksSave = actions.some((a) => a.action_type === "block_save");

  return (
    <div style={{ marginBottom: 16 }}>
      <div className="builder-header">
        <div>
          <div className="builder-breadcrumb">Business Rules / {name || "New rule"}</div>
          <div className="builder-title-row">
            {branchRole !== "if" && <span className={`rule-branch-badge ${branchRole}`}>{branchRole.replace("_", " ")}</span>}
            <h2>{name || "New rule"}</h2>
            {showActiveToggle && <span className={`badge${isActive ? " badge-success" : ""}`}>{isActive ? "Active" : "Inactive"}</span>}
          </div>
          <p className="builder-subtitle">{description || `Applies to ${entityTypeLabel(entityType)}.`}</p>
        </div>
        <div className="builder-header-actions">
          <button className="btn" type="button" onClick={() => setTesting((v) => !v)}>
            {testing ? "Hide test" : "Test rule"}
          </button>
          {showActiveToggle && (
            <button className="btn" type="button" disabled={save.isPending} onClick={() => save.mutate(!isActive)}>
              {isActive ? "Deactivate" : "Activate"}
            </button>
          )}
          <button
            className="btn btn-primary"
            type="submit"
            form="business-rule-form"
            disabled={save.isPending || (conditions.length === 0 && !allowEmptyConditions) || actions.length === 0}
          >
            {submitLabel}
          </button>
        </div>
      </div>

      {error && <div className="error-banner">{error}</div>}
      {testing && <TestRulesPanel entityType={entityType} customFields={customFields} />}

      <form
        id="business-rule-form"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate(isActive);
        }}
      >
        <div className="builder-section">
          <div className="builder-section-title">Details</div>
          <div className="form-grid">
            <div className="form-field full">
              <label>Rule name</label>
              <input value={name} onChange={(e) => setName(e.target.value)} required />
            </div>
            <div className="form-field full">
              <label>Description (optional)</label>
              <input value={description} onChange={(e) => setDescription(e.target.value)} />
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
            <div className="form-field">
              <AppScopeSelect apps={apps} value={appId} onChange={setAppId} />
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
        </div>

        <div className="rule-ifthen-row">
          <div className="rule-lane">
            <div className="rule-lane-title">If</div>
            <div className="form-field" style={{ maxWidth: 260, marginBottom: 10 }}>
              <label>Match</label>
              <select value={matchType} onChange={(e) => setMatchType(e.target.value as MatchType)}>
                {MATCH_TYPES.map((m) => (
                  <option key={m} value={m}>{m === "all" ? "All conditions (AND)" : "Any condition (OR)"}</option>
                ))}
              </select>
            </div>
            {conditionUnits.length === 0 && allowEmptyConditions && (
              <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
                Always matches - this branch runs whenever nothing earlier in its chain did. Add a condition below if you
                want this branch to need one.
              </p>
            )}
            <ConditionDropZone index={0} />
            {conditionUnits.map((u, ui) => (
              <div key={u.kind === "single" ? `s${u.index}` : `g${u.groupId}`}>
                {ui > 0 && <div className="builder-and-divider">{matchType === "all" ? "AND" : "OR"}</div>}
                <div
                  className={`rule-lane-card${draggedUnit === ui ? " dragging" : ""}`}
                  draggable
                  onDragStart={() => setDraggedUnit(ui)}
                  onDragEnd={() => {
                    setDraggedUnit(null);
                    setUnitDropIndex(null);
                  }}
                >
                  <span className="rule-drag-handle" title="Drag to reorder">⠿</span>
                  {u.kind === "single" ? (
                    <ConditionRow
                      entityType={entityType}
                      customFields={customFields}
                      relationshipDefs={relationshipDefs}
                      condition={conditions[u.index]}
                      onChange={(next) => updateConditionAt(u.index, next)}
                      onRemove={() => removeConditionAt(u.index)}
                    />
                  ) : (
                    <div className="builder-or-group">
                      <div className="builder-or-group-label">OR group - any one condition below satisfies this unit</div>
                      {u.indices.map((idx, mi) => (
                        <div key={idx}>
                          {mi > 0 && <div className="builder-and-divider">OR</div>}
                          <ConditionRow
                            entityType={entityType}
                            customFields={customFields}
                            relationshipDefs={relationshipDefs}
                            condition={conditions[idx]}
                            onChange={(next) => updateConditionAt(idx, next)}
                            onRemove={() => removeConditionAt(idx)}
                          />
                        </div>
                      ))}
                      <button
                        className="btn"
                        type="button"
                        style={{ marginBottom: 8 }}
                        onClick={() => setConditions([...conditions, { ...emptyCondition(entityType), group_id: u.groupId }])}
                      >
                        + Add to OR group
                      </button>
                    </div>
                  )}
                </div>
                <ConditionDropZone index={ui + 1} />
              </div>
            ))}
            <div style={{ display: "flex", gap: 8 }}>
              <button className="btn" type="button" onClick={() => setConditions([...conditions, emptyCondition(entityType)])}>
                + Add condition
              </button>
              <button
                className="btn"
                type="button"
                title="Add two conditions that are OR'd together into one unit before the Match setting above applies"
                onClick={() => {
                  const gid = newGroupId();
                  setConditions([...conditions, { ...emptyCondition(entityType), group_id: gid }, { ...emptyCondition(entityType), group_id: gid }]);
                }}
              >
                + OR group
              </button>
            </div>
          </div>

          <div className="rule-lane">
            <div className="rule-lane-title">Then</div>
            <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: -2 }}>
              What happens when the conditions on the left are met - one rule can have several effects.
            </p>
            <ActionDropZone index={0} />
            {actions.map((a, i) => (
              <div key={i}>
                <div
                  className={`rule-lane-card${draggedAction === i ? " dragging" : ""}`}
                  draggable
                  onDragStart={() => setDraggedAction(i)}
                  onDragEnd={() => {
                    setDraggedAction(null);
                    setActionDropIndex(null);
                  }}
                >
                  <span className="rule-drag-handle" title="Drag to reorder">⠿</span>
                  <ActionRow
                    entityType={entityType}
                    customFields={customFields}
                    action={a}
                    onChange={(next) => setActions(actions.map((x, idx) => (idx === i ? next : x)))}
                    onRemove={() => setActions(actions.filter((_, idx) => idx !== i))}
                  />
                </div>
                <ActionDropZone index={i + 1} />
              </div>
            ))}
            <button className="btn" type="button" onClick={() => setActions([...actions, emptyAction(entityType, customFields)])}>
              + Add action
            </button>
          </div>
        </div>

        <div className="builder-summary-panel" style={{ marginTop: 16 }}>
          <h4>Rule summary</h4>
          <p style={{ fontSize: 13, color: "var(--text-muted)", marginTop: -6 }}>
            {conditions.length === 0
              ? "Always"
              : `If ${describeGroupedConditions(conditions, matchType, (c) => describeCondition(entityType, c, new Map(customFields.map((f) => [f.key, f.label]))))}`}
            {" → "}
            {actions.map((a) => describeAction(entityType, a, new Map(customFields.map((f) => [f.key, f.label])))).join("; ") || "no actions yet"}
          </p>
          <div className="summary-row"><span className="label">Applies to</span><span className="value">{entityTypeLabel(entityType)}</span></div>
          <div className="summary-row"><span className="label">Execute on</span><span className="value">Create and edit</span></div>
          <div className="summary-row"><span className="label">Field dependency</span><span className="value">{fieldLabels.length > 0 ? fieldLabels.join(", ") : "None"}</span></div>
          <div className="summary-row"><span className="label">Priority</span><span className="value">{priority}</span></div>
          <div className="summary-row"><span className="label">Stop processing</span><span className="value">{blocksSave ? "Yes (block save)" : "No"}</span></div>

          <h4>Action types you can use</h4>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
            {ACTION_LEGEND.map((group) => (
              <div className="legend-group" key={group.title}>
                <div className="legend-group-title">{group.title}</div>
                {group.types.map((t) => (
                  <div className="legend-item" key={t}>
                    <span>{ACTION_ICONS[t]}</span>
                    <span>{ACTION_LABELS[t]}</span>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </div>

        <div style={{ display: "flex", gap: 8, marginTop: 16 }}>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

/**
 * Admin UX polish (spec §10): the last `VERSION_HISTORY_LIMIT` (10) saves
 * of a rule, newest first, each restorable in one click. Restoring is
 * itself a normal validated update (see business_rule_service::
 * restore_version), so it snapshots the state it's replacing too - a
 * restore is never a dead end.
 */
function RuleHistoryPanel({
  entityType,
  customFields,
  ruleId,
  ruleName,
  onDone,
  onClose,
}: {
  entityType: string;
  customFields: CustomFieldLite[];
  ruleId: string;
  ruleName: string;
  onDone: () => void;
  onClose: () => void;
}) {
  const queryClient = useQueryClient();
  const labelByKey = new Map(customFields.map((f) => [f.key, f.label]));
  const versions = useQuery({ queryKey: ["businessRuleVersions", ruleId], queryFn: () => api.listBusinessRuleVersions(ruleId) });

  const restore = useMutation({
    mutationFn: (versionId: string) => api.restoreBusinessRuleVersion(ruleId, versionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["businessRuleVersions", ruleId] });
      onDone();
    },
  });

  return (
    <div className="card" style={{ marginBottom: 16 }}>
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Version history — {ruleName}</h3>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      {versions.isLoading && <p>Loading...</p>}
      {versions.data && versions.data.length === 0 && (
        <p className="empty-state">No saved versions yet - history starts recording from the next edit.</p>
      )}
      {versions.data && versions.data.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {versions.data.map((v) => (
            <div key={v.id} className="builder-row-card" style={{ flexDirection: "column", alignItems: "flex-start", gap: 6 }}>
              <div style={{ display: "flex", justifyContent: "space-between", width: "100%" }}>
                <strong>{new Date(v.saved_at).toLocaleString()}</strong>
                <span className={`badge${v.snapshot.is_active ? " badge-success" : ""}`}>{v.snapshot.is_active ? "Active" : "Inactive"}</span>
              </div>
              <div style={{ fontSize: 13, color: "var(--text-muted)" }}>
                {describeGroupedConditions(v.snapshot.conditions, v.snapshot.match_type, (c) => describeCondition(entityType, c, labelByKey))}
                {" → "}
                {v.snapshot.actions.map((a) => describeAction(entityType, a, labelByKey)).join("; ")}
              </div>
              <button className="btn" type="button" disabled={restore.isPending} onClick={() => restore.mutate(v.id)}>
                Restore this version
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/** ADM-BR-09 (Should): lets an admin try a hypothetical set of field
 * values - every built-in field, not just the status/stage one, plus
 * every custom field - against every active rule before relying on it,
 * without persisting anything. */
function TestRulesPanel({ entityType, customFields }: { entityType: string; customFields: CustomFieldLite[] }) {
  const builtinFields = builtinFieldsFor(entityType);
  const [values, setValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(builtinFields.filter((f) => f.field_type === "select").map((f) => [f.key, f.options?.[0] ?? ""])),
  );

  const test = useMutation({
    mutationFn: () => api.testBusinessRules(entityType, values),
  });

  const builtinLabel = (key: string) => builtinFields.find((f) => f.key === key)?.label ?? key;
  const anyEffects = test.data && (
    test.data.blocked || test.data.errors.length > 0 || test.data.warnings.length > 0 ||
    Object.keys(test.data.field_effects).length > 0 || Object.keys(test.data.set_values).length > 0 ||
    Object.keys(test.data.builtin_field_effects).length > 0 || Object.keys(test.data.builtin_set_values).length > 0 ||
    Object.keys(test.data.restricted_choices).length > 0
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
          {test.data.errors.length > 0 && (
            <p>Errors: {test.data.errors.join("; ")}</p>
          )}
          {test.data.warnings.length > 0 && (
            <p>Warnings: {test.data.warnings.join("; ")}</p>
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
          {Object.keys(test.data.restricted_choices).length > 0 && (
            <p>Restricted choices: {Object.entries(test.data.restricted_choices).map(([k, v]) => `${customFields.find((f) => f.key === k)?.label ?? builtinLabel(k)}: ${v.split(LIST_SEPARATOR).join(", ")}`).join(", ")}</p>
          )}
          {!anyEffects && <p className="empty-state">No rule matches this hypothetical record.</p>}
        </div>
      )}
    </div>
  );
}
