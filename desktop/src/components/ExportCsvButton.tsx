import { toCsv, downloadCsv, type CsvColumn } from "../lib/csv";

export function ExportCsvButton<T>({
  rows,
  columns,
  filename,
}: {
  rows: T[];
  columns: CsvColumn<T>[];
  filename: string;
}) {
  return (
    <button className="btn" disabled={rows.length === 0} onClick={() => downloadCsv(filename, toCsv(rows, columns))}>
      Export CSV
    </button>
  );
}
