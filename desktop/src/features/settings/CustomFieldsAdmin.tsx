import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import {
  CUSTOM_FIELD_ENTITY_TYPES,
  CUSTOM_FIELD_TYPES,
  entityTypeLabel,
  type CustomFieldDefinition,
  type CustomFieldDefinitionInput,
} from "../../lib/types";

function emptyInput(entityType: string): CustomFieldDefinitionInput {
  return {
    entity_type: entityType, label: "", field_type: "text", options: [], required: false, show_in_list: false, sort_order: 0,
    min_value: null, max_value: null, max_length: null, regex_pattern: null,
    is_searchable: false, is_filterable: false, is_reportable: true,
    default_value: null, is_unique: false, help_text: null, placeholder: null,
  };
}

/** "Retail, Manufacturing, Services" <-> ["Retail", "Manufacturing", "Services"] */
function optionsToText(options: string[]): string {
  return options.join(", ");
}
function textToOptions(text: string): string[] {
  return text
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);
}

export function CustomFieldsAdmin() {
  const [entityType, setEntityType] = useState<string>("Company");
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const queryClient = useQueryClient();

  // Active custom objects appear as extra tabs alongside the nine built-in
  // entity types - a custom object's fields go through this exact same
  // admin screen, not a separate one.
  const customObjects = useQuery({ queryKey: ["customObjects", "active"], queryFn: () => api.listCustomObjects(true) });
  const entityTabs: { key: string; label: string }[] = [
    ...CUSTOM_FIELD_ENTITY_TYPES.map((t) => ({ key: t as string, label: entityTypeLabel(t) })),
    ...(customObjects.data ?? []).map((o) => ({ key: o.key, label: o.plural_label })),
  ];

  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType, "all"],
    queryFn: () => api.listCustomFieldDefinitions(entityType, false),
  });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["customFieldDefinitions"] });
  }

  const editing = defs.data?.find((d) => d.id === editingId) ?? null;

  return (
    <div className="card">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Custom fields</h3>
        <button
          className="btn btn-primary"
          onClick={() => {
            setCreating((v) => !v);
            setEditingId(null);
          }}
        >
          + New field
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Extra fields your business needs that aren't built in - shown on that record's create/edit form. Type and
        key can't change once created; everything else can.
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
        <DefinitionForm
          entityType={entityType}
          onDone={() => {
            invalidate();
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}

      {editing && (
        <DefinitionEditForm
          definition={editing}
          onDone={() => {
            invalidate();
            setEditingId(null);
          }}
          onCancel={() => setEditingId(null)}
        />
      )}

      {defs.isLoading && <p>Loading...</p>}
      {defs.data && defs.data.length === 0 && (
        <p className="empty-state">
          No custom fields defined for {entityTabs.find((t) => t.key === entityType)?.label ?? entityType} yet.
        </p>
      )}
      {defs.data && defs.data.length > 0 && !creating && !editing && (
        <table>
          <thead>
            <tr>
              <th>Label</th>
              <th>Type</th>
              <th>Required</th>
              <th>Flags</th>
              <th>Status</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {defs.data.map((d) => (
              <tr key={d.id}>
                <td>{d.label}</td>
                <td>{d.field_type}</td>
                <td>{d.required ? "Yes" : "No"}</td>
                <td style={{ fontSize: 12, color: "var(--text-muted)" }}>
                  {[d.is_searchable && "Searchable", d.is_filterable && "Filterable", d.is_reportable && "Reportable"].filter(Boolean).join(", ") || "—"}
                </td>
                <td>
                  <span className={`badge${d.is_active ? " badge-success" : ""}`}>{d.is_active ? "Active" : "Inactive"}</span>
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingId(d.id)}>
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

function ValidationAndFlagFields({
  fieldType,
  minValue, maxValue, maxLength, regexPattern, isSearchable, isFilterable, isReportable,
  defaultValue, isUnique, helpText, placeholder,
  onChange,
}: {
  fieldType: string;
  minValue: string | null; maxValue: string | null; maxLength: number | null; regexPattern: string | null;
  isSearchable: boolean; isFilterable: boolean; isReportable: boolean;
  defaultValue: string | null; isUnique: boolean; helpText: string | null; placeholder: string | null;
  onChange: (
    patch: Partial<{
      min_value: string | null; max_value: string | null; max_length: number | null; regex_pattern: string | null;
      is_searchable: boolean; is_filterable: boolean; is_reportable: boolean;
      default_value: string | null; is_unique: boolean; help_text: string | null; placeholder: string | null;
    }>,
  ) => void;
}) {
  return (
    <>
      {fieldType === "number" && (
        <>
          <div className="form-field">
            <label>Minimum (optional)</label>
            <input value={minValue ?? ""} onChange={(e) => onChange({ min_value: e.target.value || null })} placeholder="No minimum" />
          </div>
          <div className="form-field">
            <label>Maximum (optional)</label>
            <input value={maxValue ?? ""} onChange={(e) => onChange({ max_value: e.target.value || null })} placeholder="No maximum" />
          </div>
        </>
      )}
      {fieldType === "text" && (
        <>
          <div className="form-field">
            <label>Maximum length (optional)</label>
            <input
              type="number"
              min={1}
              value={maxLength ?? ""}
              onChange={(e) => onChange({ max_length: e.target.value ? Number(e.target.value) : null })}
              placeholder="No limit"
            />
          </div>
          <div className="form-field">
            <label>Pattern (regex, optional)</label>
            <input value={regexPattern ?? ""} onChange={(e) => onChange({ regex_pattern: e.target.value || null })} placeholder="e.g. ^[A-Z]{2}-\d{3}$" />
          </div>
        </>
      )}
      <div className="form-field">
        <label>Default value (optional)</label>
        <input
          value={defaultValue ?? ""}
          onChange={(e) => onChange({ default_value: e.target.value || null })}
          placeholder="Used when a record is saved with this field left blank"
        />
      </div>
      <div className="form-field">
        <label>Placeholder text (optional)</label>
        <input value={placeholder ?? ""} onChange={(e) => onChange({ placeholder: e.target.value || null })} placeholder="Shown inside the empty input" />
      </div>
      <div className="form-field full">
        <label>Help text (optional)</label>
        <input value={helpText ?? ""} onChange={(e) => onChange({ help_text: e.target.value || null })} placeholder="Shown under the field on the form" />
      </div>
      <div className="form-field full" style={{ flexDirection: "row", gap: 16 }}>
        <label style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input type="checkbox" checked={isSearchable} onChange={(e) => onChange({ is_searchable: e.target.checked })} />
          Searchable
        </label>
        <label style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input type="checkbox" checked={isFilterable} onChange={(e) => onChange({ is_filterable: e.target.checked })} />
          Filterable
        </label>
        <label style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input type="checkbox" checked={isReportable} onChange={(e) => onChange({ is_reportable: e.target.checked })} />
          Reportable
        </label>
        <label style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input
            type="checkbox"
            checked={isUnique}
            disabled={fieldType === "boolean"}
            onChange={(e) => onChange({ is_unique: e.target.checked })}
          />
          Unique
        </label>
      </div>
    </>
  );
}

function DefinitionForm({
  entityType,
  onDone,
  onCancel,
}: {
  entityType: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [input, setInput] = useState<CustomFieldDefinitionInput>(emptyInput(entityType));
  const [optionsText, setOptionsText] = useState("");
  const [error, setError] = useState<string | null>(null);

  const create = useMutation({
    mutationFn: () => api.createCustomFieldDefinition({ ...input, options: textToOptions(optionsText) }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not create this field"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate();
        }}
      >
        <div className="form-field full">
          <label>Label</label>
          <input value={input.label} onChange={(e) => setInput({ ...input, label: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={input.field_type} onChange={(e) => setInput({ ...input, field_type: e.target.value as typeof input.field_type })}>
            {CUSTOM_FIELD_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={input.required} onChange={(e) => setInput({ ...input, required: e.target.checked })} />
            Required
          </label>
        </div>
        {input.field_type === "select" && (
          <div className="form-field full">
            <label>Options (comma-separated)</label>
            <input value={optionsText} onChange={(e) => setOptionsText(e.target.value)} placeholder="Retail, Manufacturing, Services" required />
          </div>
        )}
        <ValidationAndFlagFields
          fieldType={input.field_type}
          minValue={input.min_value} maxValue={input.max_value} maxLength={input.max_length} regexPattern={input.regex_pattern}
          isSearchable={input.is_searchable} isFilterable={input.is_filterable} isReportable={input.is_reportable}
          defaultValue={input.default_value} isUnique={input.is_unique} helpText={input.help_text} placeholder={input.placeholder}
          onChange={(patch) => setInput({ ...input, ...patch })}
        />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={create.isPending}>
            Add field
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}

function DefinitionEditForm({
  definition,
  onDone,
  onCancel,
}: {
  definition: CustomFieldDefinition;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [label, setLabel] = useState(definition.label);
  const [optionsText, setOptionsText] = useState(optionsToText(definition.options));
  const [required, setRequired] = useState(definition.required);
  const [isActive, setIsActive] = useState(definition.is_active);
  const [minValue, setMinValue] = useState(definition.min_value);
  const [maxValue, setMaxValue] = useState(definition.max_value);
  const [maxLength, setMaxLength] = useState(definition.max_length);
  const [regexPattern, setRegexPattern] = useState(definition.regex_pattern);
  const [isSearchable, setIsSearchable] = useState(definition.is_searchable);
  const [isFilterable, setIsFilterable] = useState(definition.is_filterable);
  const [isReportable, setIsReportable] = useState(definition.is_reportable);
  const [defaultValue, setDefaultValue] = useState(definition.default_value);
  const [isUnique, setIsUnique] = useState(definition.is_unique);
  const [helpText, setHelpText] = useState(definition.help_text);
  const [placeholder, setPlaceholder] = useState(definition.placeholder);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.updateCustomFieldDefinition(definition.id, {
        label,
        options: textToOptions(optionsText),
        required,
        show_in_list: definition.show_in_list,
        sort_order: definition.sort_order,
        is_active: isActive,
        min_value: minValue,
        max_value: maxValue,
        max_length: maxLength,
        regex_pattern: regexPattern,
        is_searchable: isSearchable,
        is_filterable: isFilterable,
        is_reportable: isReportable,
        default_value: defaultValue,
        is_unique: isUnique,
        help_text: helpText,
        placeholder: placeholder,
      }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this field"),
  });

  return (
    <div className="card" style={{ marginBottom: 16, background: "var(--surface-2, transparent)" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 13, marginTop: 0 }}>
        Type: <code>{definition.field_type}</code> · Key: <code>{definition.key}</code> (fixed)
      </p>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <div className="form-field full">
          <label>Label</label>
          <input value={label} onChange={(e) => setLabel(e.target.value)} required />
        </div>
        {definition.field_type === "select" && (
          <div className="form-field full">
            <label>Options (comma-separated)</label>
            <input value={optionsText} onChange={(e) => setOptionsText(e.target.value)} required />
          </div>
        )}
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={required} onChange={(e) => setRequired(e.target.checked)} />
            Required
          </label>
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input type="checkbox" checked={isActive} onChange={(e) => setIsActive(e.target.checked)} />
            Active
          </label>
        </div>
        <ValidationAndFlagFields
          fieldType={definition.field_type}
          minValue={minValue} maxValue={maxValue} maxLength={maxLength} regexPattern={regexPattern}
          isSearchable={isSearchable} isFilterable={isFilterable} isReportable={isReportable}
          defaultValue={defaultValue} isUnique={isUnique} helpText={helpText} placeholder={placeholder}
          onChange={(patch) => {
            if ("min_value" in patch) setMinValue(patch.min_value ?? null);
            if ("max_value" in patch) setMaxValue(patch.max_value ?? null);
            if ("max_length" in patch) setMaxLength(patch.max_length ?? null);
            if ("regex_pattern" in patch) setRegexPattern(patch.regex_pattern ?? null);
            if ("is_searchable" in patch) setIsSearchable(!!patch.is_searchable);
            if ("is_filterable" in patch) setIsFilterable(!!patch.is_filterable);
            if ("is_reportable" in patch) setIsReportable(!!patch.is_reportable);
            if ("default_value" in patch) setDefaultValue(patch.default_value ?? null);
            if ("is_unique" in patch) setIsUnique(!!patch.is_unique);
            if ("help_text" in patch) setHelpText(patch.help_text ?? null);
            if ("placeholder" in patch) setPlaceholder(patch.placeholder ?? null);
          }}
        />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
