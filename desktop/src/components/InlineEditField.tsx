import { useState } from "react";
import type { ReactNode } from "react";

/**
 * Runtime UX Modernization (issue #198): click-to-edit-in-place on a
 * record detail page's read-only field, as a faster alternative to
 * opening the full edit form for a single small change. `onSave` is
 * always wired by the caller to the exact same entity service a full
 * edit already calls (e.g. `api.updateCompany`) - this is a different
 * entry point into that one write path, never a second one, so Business
 * Rules/Workflow/Access Control all fire identically either way.
 *
 * Read-only (no click handler at all) when `canEdit` is false - the same
 * `useCanWriteObject` capability check every Edit button already uses,
 * not a new check of its own.
 */
export function InlineEditField({
  value,
  displayValue,
  onSave,
  canEdit,
  type = "text",
  options,
  placeholder,
}: {
  /** The raw value edited - a plain string the input works with. Pass
   * `""` for null/empty. */
  value: string;
  /** What's shown when not editing - usually formatted (currency, a
   * fallback "—" for empty, etc.), unlike `value`. */
  displayValue: ReactNode;
  /** Persists `next` (the input's raw string) back to the record. Throws
   * or rejects on failure - the field stays in edit mode with the error
   * shown inline so nothing is silently lost. */
  onSave: (next: string) => Promise<void>;
  canEdit: boolean;
  type?: "text" | "number" | "select" | "textarea";
  options?: readonly string[];
  placeholder?: string;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!canEdit) {
    return <div>{displayValue}</div>;
  }

  if (!editing) {
    return (
      <div
        className="inline-edit-value"
        tabIndex={0}
        onClick={() => {
          setDraft(value);
          setError(null);
          setEditing(true);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            setDraft(value);
            setError(null);
            setEditing(true);
          }
        }}
        title="Click to edit"
      >
        {displayValue}
      </div>
    );
  }

  async function commit() {
    setSaving(true);
    setError(null);
    try {
      await onSave(draft);
      setEditing(false);
    } catch {
      setError("Could not save - try again.");
    } finally {
      setSaving(false);
    }
  }

  function cancel() {
    setEditing(false);
    setError(null);
  }

  return (
    <div className="inline-edit-active" onClick={(e) => e.stopPropagation()}>
      {error && <div className="inline-edit-error">{error}</div>}
      <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
        {type === "select" ? (
          <select autoFocus value={draft} onChange={(e) => setDraft(e.target.value)} disabled={saving}>
            {(options ?? []).map((o) => (
              <option key={o} value={o}>
                {o}
              </option>
            ))}
          </select>
        ) : type === "textarea" ? (
          <textarea autoFocus value={draft} onChange={(e) => setDraft(e.target.value)} disabled={saving} placeholder={placeholder} rows={3} />
        ) : (
          <input
            autoFocus
            type={type === "number" ? "number" : "text"}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            disabled={saving}
            placeholder={placeholder}
            onKeyDown={(e) => {
              if (e.key === "Enter") commit();
              if (e.key === "Escape") cancel();
            }}
          />
        )}
        <button type="button" className="btn btn-primary" disabled={saving} onClick={commit}>
          {saving ? "Saving…" : "Save"}
        </button>
        <button type="button" className="btn" disabled={saving} onClick={cancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
