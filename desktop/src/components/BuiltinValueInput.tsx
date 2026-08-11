import type { BuiltinFieldDef } from "../lib/types";

/** The options list to render as a <select> for a builtin field's value -
 * its own `options` for select-type fields, Yes/No for boolean, or null
 * (meaning: render a plain typed input instead) for everything else.
 * Shared by BusinessRulesAdmin and WorkflowAutomationAdmin so a condition/
 * trigger/action value input always matches the field's real type,
 * whichever admin screen is building it. */
export function builtinFieldOptions(field: BuiltinFieldDef | undefined): readonly string[] | null {
  if (!field) return null;
  if (field.field_type === "select") return field.options ?? [];
  if (field.field_type === "boolean") return ["true", "false"];
  return null;
}

/** A value input matching a builtin field's type - select for
 * select/boolean fields, a typed <input> (date/number/text) otherwise. */
export function BuiltinValueInput({ field, value, onChange }: { field: BuiltinFieldDef | undefined; value: string; onChange: (v: string) => void }) {
  const options = builtinFieldOptions(field);
  if (options) {
    return (
      <select value={value} onChange={(e) => onChange(e.target.value)} required>
        <option value="">— Select —</option>
        {options.map((o) => <option key={o} value={o}>{field?.field_type === "boolean" ? (o === "true" ? "Yes" : "No") : o}</option>)}
      </select>
    );
  }
  const isDecimal = field?.field_type === "money" || field?.field_type === "percent";
  const inputType = field?.field_type === "date" ? "date" : field?.field_type === "number" || isDecimal ? "number" : "text";
  return (
    <input
      type={inputType} step={isDecimal ? "0.01" : undefined}
      value={value} onChange={(e) => onChange(e.target.value)} required style={{ width: 140 }}
    />
  );
}
