/** Addendum Phase 5 (Customer 360 / Contact 360): shared read-only list
 * card for a record-detail tab - a title, an optional "+ New" button that
 * hands off to the target section pre-filled with this record's
 * relationship (see `Prefill`'s doc comment in AppShell.tsx), and a plain
 * table of rows. Used by both CompanyDetail and ContactDetail so the two
 * 360 views render their related-record tabs identically. */
export function TabListCard<T>({
  title,
  newLabel,
  onNew,
  rows,
  columns,
  render,
}: {
  title: string;
  newLabel: string;
  onNew?: () => void;
  rows: T[];
  columns: string[];
  render: (row: T) => (string | number)[];
}) {
  return (
    <div className="card" style={{ marginTop: 16 }}>
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>{title}</h3>
        {onNew && (
          <button className="btn btn-primary" onClick={onNew}>
            {newLabel}
          </button>
        )}
      </div>
      {rows.length === 0 ? (
        <p className="empty-state">None yet</p>
      ) : (
        <table>
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col}>{col}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, idx) => (
              <tr key={idx}>
                {render(row).map((cell, cidx) => (
                  <td key={cidx}>{cell}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
