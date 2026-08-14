import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import { fieldEffectsFor, restrictedChoicesFor } from "../lib/businessRules";
import type { CustomFieldDefinition, CustomFieldValues } from "../lib/types";

const LIST_SEPARATOR = "|";

/** Mirrors custom_field_service::field_is_hidden: a field flagged
 * is_hidden_by_default is left off the form unless a matching rule's
 * "show" action currently targets it; an ordinary field can still be
 * hidden by a matching "hide" action. */
function fieldIsHidden(def: CustomFieldDefinition, effect: string | undefined): boolean {
  if (effect === "hide") return true;
  if (effect === "show") return false;
  return def.is_hidden_by_default;
}

/**
 * Renders the active custom fields for an entity type inside whatever
 * form is using it - expects to sit inside a `.form-grid` container, since
 * it only renders `.form-field` children, not a wrapper div, so it lays
 * out identically to the form's built-in fields (FR-CFG-04).
 *
 * `status` is the entity's current built-in status value, used as the
 * trigger context for any conditional business rules (ADM-BR) - a rule
 * can require/hide/show/lock/editable/restrict_choices a field based on
 * it. This mirrors the server's own evaluation in
 * `custom_field_service::set_entity_values`, but is UX-only: the server
 * re-validates `require` and the is_hidden_by_default skip independently
 * on save (the other cosmetic effects have nothing for the server to
 * validate - see that function's comment).
 */
export function CustomFieldsSection({
  entityType,
  status,
  values,
  onChange,
}: {
  entityType: string;
  status: string;
  values: CustomFieldValues;
  onChange: (values: CustomFieldValues) => void;
}) {
  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType],
    queryFn: () => api.listCustomFieldDefinitions(entityType, true),
  });
  const rules = useQuery({
    queryKey: ["businessRules", entityType],
    queryFn: () => api.listBusinessRules(entityType, true),
  });

  if (!defs.data || defs.data.length === 0) return null;

  const context = { ...values, status };
  const effects = fieldEffectsFor(rules.data ?? [], context);
  const restrictedChoices = restrictedChoicesFor(rules.data ?? [], context);

  function setValue(key: string, value: string) {
    onChange({ ...values, [key]: value });
  }

  return (
    <>
      {defs.data
        .filter((def) => !fieldIsHidden(def, effects[def.key]))
        .map((def) => {
          const required = def.required || effects[def.key] === "require";
          const locked = effects[def.key] === "lock";
          const allowedOptions = restrictedChoices[def.key]
            ? def.options.filter((o) => restrictedChoices[def.key].split(LIST_SEPARATOR).includes(o))
            : def.options;
          return (
            <div className="form-field" key={def.id}>
              <label>
                {def.label}
                {required ? " *" : ""}
                {locked ? " 🔒" : ""}
              </label>
              {def.field_type === "select" && (
                <select value={values[def.key] ?? ""} onChange={(e) => setValue(def.key, e.target.value)} required={required} disabled={locked}>
                  <option value="">{def.placeholder || "— Select —"}</option>
                  {allowedOptions.map((o) => (
                    <option key={o} value={o}>
                      {o}
                    </option>
                  ))}
                </select>
              )}
              {def.field_type === "boolean" && (
                <select value={values[def.key] ?? ""} onChange={(e) => setValue(def.key, e.target.value)} disabled={locked}>
                  <option value="">—</option>
                  <option value="true">Yes</option>
                  <option value="false">No</option>
                </select>
              )}
              {(def.field_type === "text" || def.field_type === "number" || def.field_type === "date") && (
                <input
                  type={def.field_type === "date" ? "date" : def.field_type === "number" ? "number" : "text"}
                  value={values[def.key] ?? ""}
                  onChange={(e) => setValue(def.key, e.target.value)}
                  required={required}
                  disabled={locked}
                  placeholder={def.placeholder ?? undefined}
                />
              )}
              {def.help_text && (
                <span style={{ fontSize: 12, color: "var(--text-muted)" }}>{def.help_text}</span>
              )}
            </div>
          );
        })}
    </>
  );
}
