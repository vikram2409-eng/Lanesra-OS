import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { entityTypeLabel, type EffectiveNumbering, type NumberingEntityType } from "../../lib/types";

/**
 * Admin screen for configurable ID/numbering formats - lets an
 * Administrator change the prefix and zero-padded digit width used for
 * each entity's auto-generated number (e.g. "CUS-000001" -> "ACC-000001"
 * or "ACC-ab0001" - the letters are just part of the chosen prefix text).
 * Changing the format never resets or renumbers already-issued numbers;
 * the underlying sequence just gets reformatted going forward.
 */
export function NumberingAdmin() {
  const [editingType, setEditingType] = useState<string | null>(null);
  const queryClient = useQueryClient();

  const formats = useQuery({ queryKey: ["numberingFormats"], queryFn: () => api.listNumberingFormats() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["numberingFormats"] });
  }

  const editing = formats.data?.find((f) => f.entity_type === editingType) ?? null;

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>ID / number format</h3>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        Customize the prefix and digit width used for each record type's auto-generated number. Changing the format
        doesn't reset or renumber anything already issued - the sequence just continues, reformatted.
      </p>

      {editing && (
        <FormatForm
          current={editing}
          onDone={() => {
            invalidate();
            setEditingType(null);
          }}
          onCancel={() => setEditingType(null)}
        />
      )}

      {formats.isLoading && <p>Loading...</p>}
      {formats.data && !editing && (
        <table>
          <thead>
            <tr>
              <th>Record type</th>
              <th>Prefix</th>
              <th>Digits</th>
              <th>Example</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {formats.data.map((f) => (
              <tr key={f.entity_type}>
                <td>{entityTypeLabel(f.entity_type)}</td>
                <td>{f.prefix}</td>
                <td>{f.digits}</td>
                <td>
                  <code>{f.example}</code>
                  {f.is_custom && <span className="badge" style={{ marginLeft: 6 }}>Custom</span>}
                </td>
                <td>
                  <button className="btn" onClick={() => setEditingType(f.entity_type)}>
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

function FormatForm({
  current,
  onDone,
  onCancel,
}: {
  current: EffectiveNumbering;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [prefix, setPrefix] = useState(current.prefix);
  const [digits, setDigits] = useState(current.digits);
  const [error, setError] = useState<string | null>(null);

  const save = useMutation({
    mutationFn: () =>
      api.setNumberingFormat({ entity_type: current.entity_type as NumberingEntityType, prefix, digits }),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save this format"),
  });

  const reset = useMutation({
    mutationFn: () => api.resetNumberingFormat(current.entity_type),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not reset this format"),
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
        <div className="form-field full">
          <label>{entityTypeLabel(current.entity_type)} prefix</label>
          <input value={prefix} onChange={(e) => setPrefix(e.target.value)} maxLength={20} required />
        </div>
        <div className="form-field">
          <label>Digit width</label>
          <input
            type="number"
            min={1}
            max={10}
            value={digits}
            onChange={(e) => setDigits(Math.min(10, Math.max(1, Number(e.target.value) || 1)))}
          />
        </div>
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          {current.is_custom && (
            <button className="btn" type="button" onClick={() => reset.mutate()} disabled={reset.isPending}>
              Reset to default
            </button>
          )}
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
