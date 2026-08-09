import { useQuery } from "@tanstack/react-query";

import { api } from "../lib/api";
import type { CustomFieldEntityType, CustomFieldValues } from "../lib/types";

/**
 * Renders the active custom fields for an entity type inside whatever
 * form is using it - expects to sit inside a `.form-grid` container, since
 * it only renders `.form-field` children, not a wrapper div, so it lays
 * out identically to the form's built-in fields (FR-CFG-04).
 */
export function CustomFieldsSection({
  entityType,
  values,
  onChange,
}: {
  entityType: CustomFieldEntityType;
  values: CustomFieldValues;
  onChange: (values: CustomFieldValues) => void;
}) {
  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType],
    queryFn: () => api.listCustomFieldDefinitions(entityType, true),
  });

  if (!defs.data || defs.data.length === 0) return null;

  function setValue(key: string, value: string) {
    onChange({ ...values, [key]: value });
  }

  return (
    <>
      {defs.data.map((def) => (
        <div className="form-field" key={def.id}>
          <label>
            {def.label}
            {def.required ? " *" : ""}
          </label>
          {def.field_type === "select" && (
            <select value={values[def.key] ?? ""} onChange={(e) => setValue(def.key, e.target.value)} required={def.required}>
              <option value="">— Select —</option>
              {def.options.map((o) => (
                <option key={o} value={o}>
                  {o}
                </option>
              ))}
            </select>
          )}
          {def.field_type === "boolean" && (
            <select value={values[def.key] ?? ""} onChange={(e) => setValue(def.key, e.target.value)}>
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
              required={def.required}
            />
          )}
        </div>
      ))}
    </>
  );
}
