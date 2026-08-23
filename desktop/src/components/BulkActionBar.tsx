import { useState } from "react";

import type { BulkSelectionState } from "../lib/useBulkSelection";
import type { BulkActionResult } from "../lib/types";

/**
 * One bulk operation offered in the bar. `run` gets the selected ids (and
 * `value`, when this action collects one) and returns a `BulkActionResult`
 * per id - bulk operations are independent per record (see
 * `bulk_action_service`'s own doc comment), so a partial failure is normal,
 * not an error, and is shown as a per-record summary rather than a single
 * toast.
 */
export type BulkAction = {
  key: string;
  label: string;
  valueOptions?: { key: string; label: string }[];
  valuePlaceholder?: string; // renders a free-text input instead of a select when set
  confirmMessage?: string; // shown via window.confirm before running (e.g. archive)
  run: (ids: string[], value: string) => Promise<BulkActionResult[]>;
};

export function BulkActionBar({ selection, actions, onDone }: { selection: BulkSelectionState; actions: BulkAction[]; onDone?: () => void }) {
  const [pendingKey, setPendingKey] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [results, setResults] = useState<{ action: string; results: BulkActionResult[] } | null>(null);
  const [running, setRunning] = useState(false);

  if (selection.count === 0 && !results) return null;

  const pending = actions.find((a) => a.key === pendingKey) ?? null;

  async function runAction(action: BulkAction, chosenValue: string) {
    if (action.confirmMessage && !confirm(action.confirmMessage.replace("{n}", String(selection.count)))) return;
    setRunning(true);
    try {
      const outcome = await action.run(selection.selectedIds, chosenValue);
      setResults({ action: action.label, results: outcome });
      setPendingKey(null);
      setValue("");
      selection.clear();
      onDone?.();
    } finally {
      setRunning(false);
    }
  }

  return (
    <div
      style={{
        position: "sticky",
        top: 0,
        zIndex: 5,
        display: "flex",
        flexDirection: "column",
        gap: 8,
        background: "var(--bg-elevated)",
        border: "1px solid var(--border)",
        borderRadius: 10,
        padding: 12,
        margin: "0 0 14px",
      }}
    >
      {selection.count > 0 && (
        <div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 10 }}>
          <strong>{selection.count} selected</strong>
          <button type="button" className="link-button" onClick={selection.clear}>
            Clear
          </button>
          {!pending &&
            actions.map((action) => (
              <button
                key={action.key}
                type="button"
                onClick={() => (action.valueOptions || action.valuePlaceholder ? setPendingKey(action.key) : runAction(action, ""))}
                disabled={running}
              >
                {action.label}
              </button>
            ))}
          {pending && (
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span>{pending.label}:</span>
              {pending.valueOptions ? (
                <select value={value} onChange={(e) => setValue(e.target.value)} autoFocus>
                  <option value="">Choose…</option>
                  {pending.valueOptions.map((o) => (
                    <option key={o.key} value={o.key}>
                      {o.label}
                    </option>
                  ))}
                </select>
              ) : (
                <input value={value} onChange={(e) => setValue(e.target.value)} placeholder={pending.valuePlaceholder} autoFocus />
              )}
              <button type="button" onClick={() => runAction(pending, value)} disabled={!value.trim() || running}>
                Apply
              </button>
              <button
                type="button"
                className="link-button"
                onClick={() => {
                  setPendingKey(null);
                  setValue("");
                }}
              >
                Cancel
              </button>
            </div>
          )}
        </div>
      )}

      {results && (
        <div style={{ fontSize: 13 }}>
          <div>
            <strong>{results.action}:</strong> {results.results.filter((r) => r.ok).length} of {results.results.length} succeeded.
            <button type="button" className="link-button" style={{ marginLeft: 8 }} onClick={() => setResults(null)}>
              Dismiss
            </button>
          </div>
          {results.results.some((r) => !r.ok) && (
            <ul style={{ margin: "6px 0 0", paddingLeft: 18, color: "var(--danger)" }}>
              {results.results
                .filter((r) => !r.ok)
                .map((r) => (
                  <li key={r.id}>{r.error}</li>
                ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
