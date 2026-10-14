/**
 * ListScreen Template — copy this to create a new list/table screen.
 *
 * Usage:
 * 1. Copy this file to `../features/myfeature/MyFeature.tsx`
 * 2. Replace `Item` with your type, update columns
 * 3. Wire up your API calls
 *
 * Example:
 * ```tsx
 * import { ListScreenTemplate } from "../templates/ListScreen";
 * export function Products() {
 *   return <ListScreenTemplate
 *     title="Products"
 *     columns={[{ key: "name", label: "Name" }]}
 *     fetchItems={() => api.listProducts()}
 *   />;
 * }
 * ```
 */
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { Button } from "../components/ui/button";
import { Card } from "../components/ui/card";
import { Badge } from "../components/ui/badge";
import { Input } from "../components/ui/input";

export interface Column<T> {
  key: string;
  label: string;
  render?: (item: T) => React.ReactNode;
}

export interface ListScreenTemplateProps<T> {
  title: string;
  description?: string;
  columns: Column<T>[];
  fetchItems: () => Promise<T[]>;
  getItemId: (item: T) => string;
  onCreate?: () => void;
  onEdit?: (item: T) => void;
  onDelete?: (item: T) => void;
  onRowClick?: (item: T) => void;
  createLabel?: string;
  searchPlaceholder?: string;
  searchKeys?: (keyof T)[];
}

export function ListScreenTemplate<T>({
  title,
  description,
  columns,
  fetchItems,
  getItemId,
  onCreate,
  onEdit,
  onDelete,
  onRowClick,
  createLabel = "Create",
  searchPlaceholder = "Search...",
  searchKeys = [],
}: ListScreenTemplateProps<T>) {
  const [search, setSearch] = useState("");
  const { data: items = [], isLoading } = useQuery({
    queryKey: [title, "list"],
    queryFn: fetchItems,
  });

  const filtered = search && searchKeys.length > 0
    ? items.filter((item) =>
        searchKeys.some((key) =>
          String(item[key] ?? "").toLowerCase().includes(search.toLowerCase())
        )
      )
    : items;

  if (isLoading) return <p>Loading...</p>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{title}</h1>
          {description && <p className="text-muted-foreground">{description}</p>}
        </div>
        <div className="flex gap-2">
          <Input
            placeholder={searchPlaceholder}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-64"
          />
          {onCreate && <Button onClick={onCreate}>{createLabel}</Button>}
        </div>
      </div>

      <Card>
        <table className="data-table">
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col.key}>{col.label}</th>
              ))}
              {(onEdit || onDelete) && <th>Actions</th>}
            </tr>
          </thead>
          <tbody>
            {filtered.map((item) => (
              <tr
                key={getItemId(item)}
                onClick={onRowClick ? () => onRowClick(item) : undefined}
                style={onRowClick ? { cursor: "pointer" } : undefined}
              >
                {columns.map((col) => (
                  <td key={col.key}>
                    {col.render ? col.render(item) : String((item as any)[col.key] ?? "")}
                  </td>
                ))}
                {(onEdit || onDelete) && (
                  <td>
                    <div className="flex gap-1">
                      {onEdit && (
                        <Button size="sm" variant="ghost" onClick={(e) => { e.stopPropagation(); onEdit(item); }}>
                          Edit
                        </Button>
                      )}
                      {onDelete && (
                        <Button size="sm" variant="ghost" onClick={(e) => { e.stopPropagation(); onDelete(item); }}>
                          Delete
                        </Button>
                      )}
                    </div>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-center text-muted-foreground py-8">No items found.</p>
        )}
      </Card>
    </div>
  );
}

// Helper for status badges
export function StatusBadge({ status }: { status: string }) {
  const variant = status === "active" ? "success" : status === "pending" ? "warning" : "secondary";
  return <Badge variant={variant as any}>{status}</Badge>;
}
