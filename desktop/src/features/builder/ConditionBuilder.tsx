/**
 * Nested any/all condition builder (WP-0718 §2, REQ-LANE-024).
 *
 * A condition group is a row with an AND/OR switch and nested rows; a
 * condition is a row with field / operator / value. Rows add, remove
 * and nest by button, never by typing JSON; the group structure is
 * visible as indentation. Operators come from the schema list
 * (CONDITION_OPERATORS). Each `$run_data` reference is a chip: clicking
 * it jumps to the step that sets that field, and a reference no step
 * sets — or that is not declared — is flagged at once (P-11, live).
 */
import { CONDITION_OPERATORS, collectRunDataRefs, findStepSettingField } from "./builderUtils";
import type { LaneDefinition } from "../../lib/types";

type Condition = Record<string, unknown>;

function isGroup(c: Condition | null | undefined): c is { match: string; conditions: Condition[] } {
  return !!c && typeof c.match === "string" && Array.isArray(c.conditions);
}

export function ConditionBuilder({
  condition,
  definition,
  onChange,
  onJumpToStep,
}: {
  condition: unknown;
  definition: LaneDefinition;
  onChange: (condition: unknown) => void;
  onJumpToStep: (stepId: string) => void;
}) {
  const declared = (definition.run_data?.fields ?? []).map((f) => f.key);

  function updateAt(path: number[], next: Condition | null): void {
    function rec(current: Condition, depth: number): Condition | null {
      if (depth === path.length) return next;
      if (!isGroup(current)) return current;
      const conditions = current.conditions
        .map((child, i) => (i === path[depth] ? rec(child, depth + 1) : child))
        .filter((c): c is Condition => c !== null);
      return { ...current, conditions };
    }
    if (path.length === 0) {
      onChange(next);
      return;
    }
    if (isGroup(condition as Condition)) onChange(rec(condition as Condition, 0));
  }

  function addChild(path: number[], child: Condition): void {
    function rec(current: Condition, depth: number): Condition {
      if (depth === path.length) {
        return isGroup(current) ? { ...current, conditions: [...current.conditions, child] } : current;
      }
      if (!isGroup(current)) return current;
      return {
        ...current,
        conditions: current.conditions.map((c, i) => (i === path[depth] ? rec(c, depth + 1) : c)),
      };
    }
    if (path.length === 0 && isGroup(condition as Condition)) {
      onChange({
        ...(condition as Condition),
        conditions: [...((condition as Condition).conditions as Condition[]), child],
      });
      return;
    }
    if (isGroup(condition as Condition)) onChange(rec(condition as Condition, 0));
  }

  function renderNode(node: Condition, path: number[], depth: number): React.ReactNode {
    if (isGroup(node)) {
      return (
        <div
          key={path.join("-") || "root"}
          style={{ marginLeft: depth * 16, borderLeft: "2px solid var(--border, #ccc)", paddingLeft: 8, marginBottom: 8 }}
        >
          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <label>
              Match{" "}
              <select
                aria-label={depth === 0 ? "Match" : `Match (group level ${depth})`}
                value={node.match}
                onChange={(e) => {
                  const next = { ...node, match: e.target.value };
                  if (path.length === 0) onChange(next);
                  else updateAt(path, next);
                }}
              >
                <option value="all">all (AND)</option>
                <option value="any">any (OR)</option>
              </select>
            </label>
            <button type="button" className="btn" onClick={() => addChild(path, { field: "", operator: "equals", value: "" })}>
              Add condition
            </button>
            <button type="button" className="btn" onClick={() => addChild(path, { match: "all", conditions: [] })}>
              Add group
            </button>
            {path.length > 0 && (
              <button type="button" className="btn" onClick={() => updateAt(path, null)}>
                Remove group
              </button>
            )}
          </div>
          {node.conditions.length === 0 && (
            <p style={{ color: "var(--text-muted)" }}>Empty group — add a condition.</p>
          )}
          {node.conditions.map((child, i) => renderNode(child, [...path, i], depth + 1))}
        </div>
      );
    }
    const field = typeof node.field === "string" ? node.field : "";
    const refs = collectRunDataRefs(field);
    return (
      <div key={path.join("-")} style={{ marginLeft: depth * 16, marginBottom: 8, display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <label>
          Field{" "}
          <input
            type="text"
            aria-label="Condition field"
            value={field}
            placeholder="$run_data.field or $item.field"
            list="builder-condition-fields"
            onChange={(e) => updateAt(path, { ...node, field: e.target.value })}
          />
        </label>
        <datalist id="builder-condition-fields">
          {declared.map((k) => (
            <option key={k} value={`$run_data.${k}`} />
          ))}
          <option value="$record.status" />
          <option value="$item.qty_ordered" />
        </datalist>
        <label>
          Operator{" "}
          <select
            aria-label="Condition operator"
            value={typeof node.operator === "string" ? node.operator : "equals"}
            onChange={(e) => updateAt(path, { ...node, operator: e.target.value })}
          >
            {CONDITION_OPERATORS.map((op) => (
              <option key={op} value={op}>
                {op}
              </option>
            ))}
          </select>
        </label>
        <label>
          Value{" "}
          <input
            type="text"
            aria-label="Condition value"
            value={node.value === undefined ? "" : String(node.value)}
            onChange={(e) => updateAt(path, { ...node, value: e.target.value })}
          />
        </label>
        <label>
          Compare field{" "}
          <input
            type="text"
            aria-label="Condition compare field"
            value={typeof node.compare_field === "string" ? node.compare_field : ""}
            placeholder="optional $field"
            onChange={(e) => updateAt(path, { ...node, compare_field: e.target.value || undefined })}
          />
        </label>
        <button type="button" className="btn" onClick={() => updateAt(path, null)}>
          Remove condition
        </button>
        {refs.map((ref) => {
          const setter = findStepSettingField(definition, ref);
          const declaredRef = declared.includes(ref);
          return (
            <span key={ref} style={{ display: "inline-flex", gap: 4, alignItems: "center" }}>
              {setter ? (
                <button
                  type="button"
                  className="btn"
                  aria-label={`Go to step that sets ${ref}`}
                  onClick={() => onJumpToStep(setter)}
                >
                  $run_data.{ref}
                </button>
              ) : (
                <span role="alert" style={{ color: "var(--danger)" }}>
                  $run_data.{ref} is not declared in lane.run_data.fields and is not set by any step
                  {!declaredRef ? " (P-11)" : ""}
                </span>
              )}
            </span>
          );
        })}
      </div>
    );
  }

  if (!condition || typeof condition !== "object") {
    return (
      <div>
        <p style={{ color: "var(--text-muted)" }}>No condition — the step runs unconditionally.</p>
        <button type="button" className="btn" onClick={() => onChange({ match: "all", conditions: [] })}>
          Add condition group
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => onChange({ field: "", operator: "equals", value: "" })}
        >
          Add single condition
        </button>
      </div>
    );
  }
  return <div>{renderNode(condition as Condition, [], 0)}</div>;
}
