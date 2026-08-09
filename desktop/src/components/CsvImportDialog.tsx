import { useState } from "react";

import { ApiError } from "../lib/api";
import { csvRowsToRecords, parseCsv } from "../lib/csv";

export interface CsvImportColumn {
  label: string;
  required?: boolean;
}

/** What parseRow returns for one CSV data row: either a ready-to-submit input, or a reason it can't be. */
export interface ParsedCsvRow<T> {
  preview: string;
  input?: T;
  error?: string;
}

type RowState<T> = ParsedCsvRow<T> & { status: "ready" | "invalid" | "pending" | "ok" | "error"; message?: string };

/**
 * Generic "pick a CSV, preview it, import row by row" panel. Each row is
 * created with its own createFn call (not a bulk/transactional import) so a
 * bad row fails on its own without blocking the rest - appropriate at the
 * small-to-medium row counts this app's workspaces deal in, and it reuses
 * the exact same create command (with the exact same validation) as the
 * manual "New ..." form, so there's no separate import code path to drift
 * out of sync with business rules.
 */
export function CsvImportDialog<T>({
  title,
  columns,
  parseRow,
  createFn,
  onImported,
  onClose,
}: {
  title: string;
  columns: CsvImportColumn[];
  parseRow: (record: Record<string, string>) => ParsedCsvRow<T>;
  createFn: (input: T) => Promise<unknown>;
  onImported: () => void;
  onClose: () => void;
}) {
  const [fileName, setFileName] = useState<string | null>(null);
  const [rows, setRows] = useState<RowState<T>[]>([]);
  const [importing, setImporting] = useState(false);
  const [imported, setImported] = useState(false);

  function handleFile(file: File) {
    setImported(false);
    setFileName(file.name);
    const reader = new FileReader();
    reader.onload = () => {
      const records = csvRowsToRecords(parseCsv(String(reader.result ?? "")));
      setRows(
        records.map((record) => {
          const parsed = parseRow(record);
          return { ...parsed, status: parsed.error ? "invalid" : "ready" };
        }),
      );
    };
    reader.readAsText(file);
  }

  async function runImport() {
    setImporting(true);
    for (let i = 0; i < rows.length; i++) {
      if (rows[i].status !== "ready" || !rows[i].input) continue;
      setRows((current) => current.map((r, idx) => (idx === i ? { ...r, status: "pending" } : r)));
      try {
        await createFn(rows[i].input as T);
        setRows((current) => current.map((r, idx) => (idx === i ? { ...r, status: "ok" } : r)));
      } catch (err) {
        const message = err instanceof ApiError ? err.message : "Could not create this row";
        setRows((current) => current.map((r, idx) => (idx === i ? { ...r, status: "error", message } : r)));
      }
    }
    setImporting(false);
    setImported(true);
    onImported();
  }

  const readyCount = rows.filter((r) => r.status === "ready").length;
  const invalidCount = rows.filter((r) => r.status === "invalid").length;
  const okCount = rows.filter((r) => r.status === "ok").length;
  const errorCount = rows.filter((r) => r.status === "error").length;

  return (
    <div className="card" style={{ marginBottom: 16 }}>
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>{title}</h3>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        CSV with a header row. Expected columns:{" "}
        {columns.map((c) => `${c.label}${c.required ? "" : " (optional)"}`).join(", ")}.
      </p>
      <input
        type="file"
        accept=".csv,text/csv"
        onChange={(e) => e.target.files?.[0] && handleFile(e.target.files[0])}
      />
      {fileName && rows.length === 0 && <p className="empty-state">No data rows found in {fileName}.</p>}
      {rows.length > 0 && !imported && (
        <>
          <p style={{ fontSize: 13, marginTop: 12 }}>
            {rows.length} row(s) found - {readyCount} ready to import
            {invalidCount > 0 ? `, ${invalidCount} with errors (won't be imported)` : ""}.
          </p>
          <table>
            <thead>
              <tr>
                <th>Row</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r, idx) => (
                <tr key={idx}>
                  <td>{r.preview}</td>
                  <td>{r.error ? <span style={{ color: "var(--danger)" }}>{r.error}</span> : "Ready"}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <button
            className="btn btn-primary"
            style={{ marginTop: 12 }}
            onClick={runImport}
            disabled={readyCount === 0 || importing}
          >
            {importing ? "Importing..." : `Import ${readyCount} row(s)`}
          </button>
        </>
      )}
      {imported && (
        <>
          <p style={{ fontSize: 13, marginTop: 12 }}>
            Done: {okCount} created{errorCount > 0 ? `, ${errorCount} failed` : ""}
            {invalidCount > 0 ? `, ${invalidCount} skipped (invalid)` : ""}.
          </p>
          <table>
            <thead>
              <tr>
                <th>Row</th>
                <th>Result</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r, idx) => (
                <tr key={idx}>
                  <td>{r.preview}</td>
                  <td>
                    {r.status === "ok" && <span style={{ color: "var(--success)" }}>Created</span>}
                    {r.status === "error" && <span style={{ color: "var(--danger)" }}>{r.message}</span>}
                    {r.status === "invalid" && <span style={{ color: "var(--danger)" }}>{r.error}</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </div>
  );
}
