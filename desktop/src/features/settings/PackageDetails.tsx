import { builtinFieldsFor, CUSTOM_FIELD_ENTITY_TYPES, entityTypeLabel, LIST_SEPARATOR } from "../../lib/types";

/**
 * App Catalog "package details before install" (roadmap: "a package
 * detail screen before install... showing what a reference package
 * builds, how it connects to existing data, its automation in plain
 * language, and guidance on fitting it to a real org"). Parses the raw
 * manifest_json an AppPackage already carries once imported - the same
 * JSON `industry_package_service::import_package` validated - entirely
 * client-side, no new backend endpoint needed: every field this screen
 * shows was already sitting in AppPackage.manifest_json.
 *
 * The condition/action vocabulary and plain-language phrasing below
 * deliberately mirrors BusinessRulesAdmin.tsx's and
 * WorkflowAutomationAdmin.tsx's own describeCondition/describeAction/
 * describeTrigger - same operator words, same action words - so a rule
 * an admin reads here reads identically once they open the real Business
 * Rules/Workflow Automation admin screens after installing. Kept as its
 * own copy rather than a shared import since this file works from raw
 * manifest JSON (snake_case, no live field-definition query) while those
 * two work from the *Input types React Query already fetched - same
 * shape, different data source.
 */

interface ManifestObject {
  key: string;
  singular_label: string;
  plural_label: string;
  icon: string;
  prefix: string;
  digits: number;
}

interface ManifestField {
  key: string;
  entity_type: string;
  label: string;
  field_type: string;
  options: string[];
  required: boolean;
}

interface ManifestRelationship {
  source_entity_type: string;
  target_entity_type: string;
  relationship_type: string;
  forward_label: string;
  reverse_label: string;
  is_required: boolean;
}

interface ManifestCondition {
  field_source: string;
  field_key: string;
  operator: string;
  value: string;
  compare_field_source: string | null;
  compare_field_key: string | null;
}

interface ManifestRuleAction {
  action_type: string;
  target_field_key: string | null;
  target_field_source: string;
  action_value: string | null;
  message: string | null;
}

interface ManifestBusinessRule {
  entity_type: string;
  name: string;
  description: string | null;
  conditions: ManifestCondition[];
  actions: ManifestRuleAction[];
}

interface ManifestWorkflowAction {
  action_type: string;
  params_json: string;
}

interface ManifestWorkflow {
  entity_type: string;
  name: string;
  description: string | null;
  trigger_type: string;
  trigger_status: string | null;
  trigger_field_key: string | null;
  trigger_offset_days: number;
  conditions: ManifestCondition[];
  actions: ManifestWorkflowAction[];
}

interface ManifestRecommendedPermission {
  role: string;
  level: string;
}

interface Manifest {
  package_id: string;
  name: string;
  industry: string;
  version: string;
  objects: ManifestObject[];
  fields: ManifestField[];
  relationships: ManifestRelationship[];
  business_rules: ManifestBusinessRule[];
  workflows: ManifestWorkflow[];
  seed_data: unknown[];
  app: {
    name: string;
    icon: string;
    description: string | null;
    recommended_permissions: ManifestRecommendedPermission[];
  };
}

const OPERATOR_LABELS: Record<string, string> = {
  equals: "equals", not_equals: "does not equal", contains: "contains", not_contains: "does not contain",
  starts_with: "starts with", ends_with: "ends with", in_list: "is one of", not_in_list: "is not one of",
  is_empty: "is empty", is_not_empty: "is not empty", greater_than: "is greater than", less_than: "is less than",
  on_or_after: "is on or after", on_or_before: "is on or before",
};
const VALUELESS_OPERATORS = new Set(["is_empty", "is_not_empty"]);

const RULE_ACTION_LABELS: Record<string, (target: string, a: ManifestRuleAction) => string> = {
  require: (target) => `require ${target}`,
  hide: (target) => `hide ${target}`,
  show: (target) => `show ${target}`,
  lock: (target) => `lock ${target}`,
  editable: (target) => `unlock ${target}`,
  set_default: (target, a) => `default ${target} to "${a.action_value ?? ""}"`,
  set_value: (target, a) => `force ${target} to "${a.action_value ?? ""}"`,
  clear_value: (target) => `clear ${target}`,
  restrict_choices: (target, a) => `restrict ${target} to ${(a.action_value ?? "").split(LIST_SEPARATOR).filter(Boolean).join(", ") || "no options"}`,
  block_save: (_t, a) => `block save: "${a.message ?? ""}"`,
  show_error: (_t, a) => `show error: "${a.message ?? ""}"`,
  show_warning: (_t, a) => `show warning: "${a.message ?? ""}"`,
};

const WORKFLOW_TRIGGER_LABELS: Record<string, string> = {
  record_created: "created",
  record_updated: "updated",
  status_changed: "status/stage reaches a value",
  field_changed: "a custom field changes",
  date_reached: "a date is reached",
  due_overdue: "a date is overdue",
  scheduled: "on a recurring schedule",
};

/** True for one of the nine built-in entity types this workspace ships
 * with regardless of any package (plus Task, already in that list) - the
 * "existing data" a package's own relationships can connect into. */
function isBuiltinEntity(entityType: string): boolean {
  return (CUSTOM_FIELD_ENTITY_TYPES as readonly string[]).includes(entityType);
}

function objectLabel(entityType: string, objectsByKey: Map<string, ManifestObject>): string {
  if (isBuiltinEntity(entityType)) return entityTypeLabel(entityType);
  return objectsByKey.get(entityType)?.plural_label ?? entityType;
}

function fieldLabel(entityType: string, source: string, key: string, labelByKey: Map<string, string>): string {
  if (source === "builtin") {
    return builtinFieldsFor(entityType).find((f) => f.key === key)?.label ?? key;
  }
  return labelByKey.get(key) ?? key;
}

function describeCondition(entityType: string, c: ManifestCondition, labelByKey: Map<string, string>): string {
  const label = fieldLabel(entityType, c.field_source, c.field_key, labelByKey);
  const needsValue = !VALUELESS_OPERATORS.has(c.operator);
  const comparand = c.compare_field_key && c.compare_field_source
    ? fieldLabel(entityType, c.compare_field_source, c.compare_field_key, labelByKey)
    : `"${c.value}"`;
  return `${label} ${OPERATOR_LABELS[c.operator] ?? c.operator}${needsValue ? ` ${comparand}` : ""}`;
}

function describeConditions(entityType: string, conditions: ManifestCondition[], labelByKey: Map<string, string>): string {
  if (conditions.length === 0) return "always";
  return conditions.map((c) => describeCondition(entityType, c, labelByKey)).join(" AND ");
}

function describeRuleAction(entityType: string, a: ManifestRuleAction, labelByKey: Map<string, string>): string {
  const target = a.target_field_key ? fieldLabel(entityType, a.target_field_source, a.target_field_key, labelByKey) : "";
  const fn = RULE_ACTION_LABELS[a.action_type];
  return fn ? fn(target, a) : a.action_type;
}

function describeWorkflowTrigger(entityType: string, w: ManifestWorkflow, objectsByKey: Map<string, ManifestObject>): string {
  const subject = objectLabel(entityType, objectsByKey);
  switch (w.trigger_type) {
    case "record_created": return `When a new ${subject} record is created`;
    case "status_changed": return `When ${subject}'s status reaches "${w.trigger_status ?? ""}"`;
    case "field_changed": return `When ${subject}'s ${w.trigger_field_key ?? "a field"} changes`;
    case "date_reached": return `When ${subject}'s ${w.trigger_field_key ?? "a date field"} is reached`;
    case "due_overdue": return `When ${subject}'s ${w.trigger_field_key ?? "a date field"} is overdue`;
    default: return `When a ${subject} record is ${WORKFLOW_TRIGGER_LABELS[w.trigger_type] ?? w.trigger_type}`;
  }
}

function describeWorkflowAction(entityType: string, a: ManifestWorkflowAction, labelByKey: Map<string, string>): string {
  try {
    const p = JSON.parse(a.params_json) as Record<string, unknown>;
    switch (a.action_type) {
      case "create_task": return `create a task ("${p.title}")`;
      case "add_notification": return `notify ${p.audience === "all_admins" ? "admins" : "the owner"}`;
      case "update_field": return `set ${p.target_field_key ? fieldLabel(entityType, String(p.target_field_source), String(p.target_field_key), labelByKey) : "a field"}`;
      case "update_related_record": return "update a linked record";
      case "create_record": return `create a new ${String(p.entity_type ?? "record")} and link it`;
      default: return a.action_type.replace(/_/g, " ");
    }
  } catch {
    return a.action_type.replace(/_/g, " ");
  }
}

/** Guidance is derived from what's actually in the manifest, not
 * templated boilerplate - each bullet only appears when the condition it
 * describes is actually true of this package. */
function orgFitGuidance(m: Manifest, objectsByKey: Map<string, ManifestObject>): string[] {
  const bullets: string[] = [];
  const linkedBuiltins = [...new Set(
    m.relationships
      .flatMap((r) => [r.source_entity_type, r.target_entity_type])
      .filter(isBuiltinEntity)
  )].map(entityTypeLabel);
  if (linkedBuiltins.length > 0) {
    bullets.push(
      `This package links its new objects into your existing ${linkedBuiltins.join(", ")} - install it once you have real records there, not before, so the first ${objectLabel(m.objects[0]?.key ?? "", objectsByKey)} you create has something real to connect to.`,
    );
  }
  if (m.seed_data.length === 0) {
    bullets.push("It ships with no starter records - your team creates the first real ones after installing.");
  }
  bullets.push(
    "Object names, ID prefixes and picklist options match this package's own defaults - rename fields, adjust numbering, or add/remove select options from Custom Objects after installing to match how your team actually talks about the work.",
  );
  if (m.app.recommended_permissions.length > 0) {
    bullets.push(
      `${m.app.recommended_permissions.length} recommended permission grant${m.app.recommended_permissions.length === 1 ? "" : "s"} are listed for review after install - installing never changes anyone's access on its own.`,
    );
  }
  bullets.push(
    "Every rule and workflow here only reads and writes fields on the record that actually changed - a process that needs to check other linked records first (an overlap check, a running total) still needs a person to enforce it for now.",
  );
  return bullets;
}

export function PackageDetailsPanel({ manifestJson }: { manifestJson: string }) {
  let manifest: Manifest;
  try {
    manifest = JSON.parse(manifestJson) as Manifest;
  } catch {
    return <p className="error-banner">This manifest isn't valid JSON - can't build a preview.</p>;
  }

  const objectsByKey = new Map(manifest.objects.map((o) => [o.key, o]));
  const fieldsByEntity = new Map<string, ManifestField[]>();
  for (const f of manifest.fields) {
    fieldsByEntity.set(f.entity_type, [...(fieldsByEntity.get(f.entity_type) ?? []), f]);
  }
  const labelByKeyFor = (entityType: string) =>
    new Map((fieldsByEntity.get(entityType) ?? []).map((f) => [f.key, f.label]));

  const builtinRelationships = manifest.relationships.filter(
    (r) => isBuiltinEntity(r.source_entity_type) || isBuiltinEntity(r.target_entity_type),
  );
  const internalRelationships = manifest.relationships.filter(
    (r) => !isBuiltinEntity(r.source_entity_type) && !isBuiltinEntity(r.target_entity_type),
  );

  return (
    <div style={{ padding: "8px 0", display: "grid", gap: 16 }}>
      {manifest.app.description && <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>{manifest.app.description}</p>}

      <section>
        <h4 style={{ marginBottom: 6 }}>What this builds</h4>
        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, display: "grid", gap: 4 }}>
          {manifest.objects.map((o) => (
            <li key={o.key}>
              {o.icon} <strong>{o.plural_label}</strong> ({(fieldsByEntity.get(o.key) ?? []).length} custom field
              {(fieldsByEntity.get(o.key) ?? []).length === 1 ? "" : "s"}, IDs like {o.prefix}-{"0".repeat(o.digits)})
            </li>
          ))}
        </ul>
        {internalRelationships.length > 0 && (
          <ul style={{ margin: "8px 0 0", paddingLeft: 18, fontSize: 13, color: "var(--text-muted)", display: "grid", gap: 2 }}>
            {internalRelationships.map((r, i) => (
              <li key={i}>
                {objectLabel(r.source_entity_type, objectsByKey)} → {objectLabel(r.target_entity_type, objectsByKey)}
                {" "}({r.forward_label}{r.is_required ? ", required" : ""})
              </li>
            ))}
          </ul>
        )}
      </section>

      {builtinRelationships.length > 0 && (
        <section>
          <h4 style={{ marginBottom: 6 }}>How it connects to your existing data</h4>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, display: "grid", gap: 4 }}>
            {builtinRelationships.map((r, i) => {
              const sourceIsBuiltin = isBuiltinEntity(r.source_entity_type);
              const newObj = sourceIsBuiltin ? r.target_entity_type : r.source_entity_type;
              const existing = sourceIsBuiltin ? r.source_entity_type : r.target_entity_type;
              return (
                <li key={i}>
                  {objectLabel(newObj, objectsByKey)} link{sourceIsBuiltin ? "s from" : "s to"} your existing{" "}
                  <strong>{entityTypeLabel(existing)}</strong> ({r.forward_label}{r.is_required ? ", required" : ""})
                </li>
              );
            })}
          </ul>
        </section>
      )}

      {(manifest.business_rules.length > 0 || manifest.workflows.length > 0) && (
        <section>
          <h4 style={{ marginBottom: 6 }}>Automation, in plain language</h4>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, display: "grid", gap: 6 }}>
            {manifest.business_rules.map((r, i) => {
              const labelByKey = labelByKeyFor(r.entity_type);
              return (
                <li key={`r${i}`}>
                  <strong>{objectLabel(r.entity_type, objectsByKey)}:</strong> when {describeConditions(r.entity_type, r.conditions, labelByKey)},{" "}
                  {r.actions.map((a) => describeRuleAction(r.entity_type, a, labelByKey)).join("; ")}.
                </li>
              );
            })}
            {manifest.workflows.map((w, i) => {
              const labelByKey = labelByKeyFor(w.entity_type);
              const extraConditions = w.conditions.length > 0 ? ` and ${describeConditions(w.entity_type, w.conditions, labelByKey)}` : "";
              return (
                <li key={`w${i}`}>
                  {describeWorkflowTrigger(w.entity_type, w, objectsByKey)}
                  {extraConditions}, {w.actions.map((a) => describeWorkflowAction(w.entity_type, a, labelByKey)).join(" and ")}.
                </li>
              );
            })}
          </ul>
        </section>
      )}

      <section>
        <h4 style={{ marginBottom: 6 }}>Fitting this to your organization</h4>
        <ul style={{ margin: 0, paddingLeft: 18, fontSize: 13, color: "var(--text-muted)", display: "grid", gap: 4 }}>
          {orgFitGuidance(manifest, objectsByKey).map((g, i) => (
            <li key={i}>{g}</li>
          ))}
        </ul>
      </section>
    </div>
  );
}
