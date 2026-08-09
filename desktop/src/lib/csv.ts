// CSV export/import helpers. No third-party CSV library - the app has no
// dependency on one anywhere else (see PrintableDocument's browser-native
// print for the same "keep it dependency-free" call), and RFC 4180 quoting
// is small enough to hand-roll correctly.

export interface CsvColumn<T> {
  label: string;
  get: (row: T) => string;
}

function escapeCsvField(value: string): string {
  if (/[",\r\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

export function toCsv<T>(rows: T[], columns: CsvColumn<T>[]): string {
  const header = columns.map((c) => escapeCsvField(c.label)).join(",");
  const lines = rows.map((row) => columns.map((c) => escapeCsvField(c.get(row))).join(","));
  return [header, ...lines].join("\r\n") + "\r\n";
}

export function downloadCsv(filename: string, content: string): void {
  // Prefix a UTF-8 BOM so Excel (which otherwise guesses the system codepage)
  // opens non-ASCII business names/addresses correctly.
  const blob = new Blob(["﻿", content], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Parses CSV text into rows of string cells. Handles quoted fields, embedded
 * commas/newlines inside quotes, and "" as an escaped quote (RFC 4180).
 * Trailing blank lines are dropped.
 */
export function parseCsv(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let inQuotes = false;
  let i = 0;
  const len = text.length;

  function pushField() {
    row.push(field);
    field = "";
  }
  function pushRow() {
    pushField();
    rows.push(row);
    row = [];
  }

  while (i < len) {
    const ch = text[i];
    if (inQuotes) {
      if (ch === '"') {
        if (text[i + 1] === '"') {
          field += '"';
          i += 2;
          continue;
        }
        inQuotes = false;
        i++;
        continue;
      }
      field += ch;
      i++;
      continue;
    }
    if (ch === '"') {
      inQuotes = true;
      i++;
      continue;
    }
    if (ch === ",") {
      pushField();
      i++;
      continue;
    }
    if (ch === "\r") {
      i++;
      continue;
    }
    if (ch === "\n") {
      pushRow();
      i++;
      continue;
    }
    field += ch;
    i++;
  }
  if (field.length > 0 || row.length > 0) {
    pushRow();
  }
  return rows.filter((r) => !(r.length === 1 && r[0] === ""));
}

/** Turns parsed CSV rows (first row = header) into header-keyed records. */
export function csvRowsToRecords(rows: string[][]): Record<string, string>[] {
  if (rows.length < 2) return [];
  const header = rows[0].map((h) => h.trim());
  return rows.slice(1).map((row) => {
    const record: Record<string, string> = {};
    header.forEach((h, idx) => {
      record[h] = (row[idx] ?? "").trim();
    });
    return record;
  });
}

/** Case-insensitive, whitespace-trimmed lookup across one or more accepted header names. */
export function field(record: Record<string, string>, ...names: string[]): string {
  for (const name of names) {
    for (const key of Object.keys(record)) {
      if (key.trim().toLowerCase() === name.trim().toLowerCase()) {
        const value = record[key];
        if (value) return value;
      }
    }
  }
  return "";
}
