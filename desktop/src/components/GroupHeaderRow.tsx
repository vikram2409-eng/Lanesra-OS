/** A group-by header row for a list table using Saved Views' `group_by_field`
 * - a plain `<tr>` with one full-width `<td>`, so it drops into any
 * existing `<tbody>` without changing the table's column structure. */
export function GroupHeaderRow({ label, colSpan }: { label: string; colSpan: number }) {
  return (
    <tr>
      <td colSpan={colSpan} style={{ background: "var(--bg-elevated)", fontWeight: 700, fontSize: 13, padding: "8px 12px" }}>
        {label}
      </td>
    </tr>
  );
}
