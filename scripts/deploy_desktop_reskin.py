#!/usr/bin/env python3
"""Deploy Desktop Reskin (PR 4)."""

import os

REPO = "."

def write_file(path, content):
    full = os.path.join(REPO, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, "w", encoding="utf-8").write(content)
    print(f"Wrote {path}")

# File: desktop/src/styles/modern-reskin.css
write_file("desktop/src/styles/modern-reskin.css", """/* Modern UI Reskin — applies the demo's Modern UI styling to existing desktop
   components. Uses design tokens only, no hardcoded values.
   This layer modernizes legacy markup without changing component structure. */

/* ---- Cards ---- */
.card,
.panel,
.widget-card,
.detail-card,
.list-card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}

/* ---- Buttons (legacy .btn classes) ---- */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 0.875rem;
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--foreground);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn:hover {
  background: var(--secondary);
  border-color: var(--border);
}

.btn-primary {
  background: var(--primary);
  border-color: var(--primary);
  color: var(--primary-foreground);
}

.btn-primary:hover {
  background: var(--primary);
  opacity: 0.9;
}

.btn-destructive {
  background: var(--destructive);
  border-color: var(--destructive);
  color: var(--destructive-foreground);
}

/* ---- Tables ---- */
.data-table,
.list-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.data-table th,
.list-table th {
  background: var(--muted);
  color: var(--muted-foreground);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.75rem 1rem;
  text-align: left;
  border-bottom: 1px solid var(--border);
}

.data-table td,
.list-table td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}

.data-table tr:last-child td,
.list-table tr:last-child td {
  border-bottom: none;
}

.data-table tbody tr:hover,
.list-table tbody tr:hover {
  background: var(--muted);
}

/* ---- Forms ---- */
.form-group label,
.field label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 0.375rem;
  color: var(--foreground);
}

.form-group input,
.form-group select,
.form-group textarea,
.field input,
.field select,
.field textarea {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--input);
  border-radius: var(--radius-md);
  background: var(--card);
  color: var(--foreground);
  font-size: 0.875rem;
}

.form-group input:focus,
.field input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--ring);
}

/* ---- Badges (legacy .badge, .status-badge) ---- */
.badge,
.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.625rem;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 500;
  background: var(--secondary);
  color: var(--secondary-foreground);
  border: 1px solid var(--border);
}

.badge-success,
.status-active {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
  border-color: color-mix(in srgb, var(--success) 30%, transparent);
}

.badge-warning {
  background: color-mix(in srgb, var(--warning) 15%, transparent);
  color: var(--warning);
  border-color: color-mix(in srgb, var(--warning) 30%, transparent);
}

.badge-destructive,
.status-inactive {
  background: color-mix(in srgb, var(--destructive) 15%, transparent);
  color: var(--destructive);
  border-color: color-mix(in srgb, var(--destructive) 30%, transparent);
}

/* ---- Page headers ---- */
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border);
}

.page-header h1,
.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: -0.025em;
  margin: 0;
}

/* ---- Sidebar (AppShell) ---- */
.app-shell {
  display: flex;
  min-height: 100vh;
  background: var(--background);
}

.sidebar {
  width: 260px;
  background: var(--sidebar);
  color: var(--sidebar-foreground);
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  border-right: 1px solid var(--border);
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem;
  margin-bottom: 1rem;
  font-weight: 700;
  font-size: 1.125rem;
}

.sidebar-brand .brand-logo {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
}

.nav-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--sidebar-foreground);
  opacity: 0.7;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.nav-item:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.08);
}

.nav-item.active {
  opacity: 1;
  background: var(--primary);
  color: var(--primary-foreground);
}

/* ---- Main content ---- */
.main-content {
  flex: 1;
  padding: 1.5rem 2rem;
  max-width: 1400px;
  margin: 0 auto;
  width: 100%;
}

/* ---- KPI cards (Dashboard) ---- */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.kpi-card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.25rem;
  box-shadow: var(--shadow-sm);
}

.kpi-card .kpi-label {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--muted-foreground);
  margin-bottom: 0.5rem;
}

.kpi-card .kpi-value {
  font-size: 1.875rem;
  font-weight: 700;
  letter-spacing: -0.025em;
}

.kpi-card .kpi-trend {
  font-size: 0.75rem;
  margin-top: 0.25rem;
}

/* ---- Error banners ---- */
.error-banner {
  background: color-mix(in srgb, var(--destructive) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--destructive) 30%, transparent);
  color: var(--destructive);
  padding: 1rem;
  border-radius: var(--radius-lg);
  margin-bottom: 1rem;
}
""")

# File: desktop/src/components/ui/README.md
write_file("desktop/src/components/ui/README.md", """# LanesraOS Desktop UI Component Library

Pre-built React components for building LanesraOS desktop screens. All components use design tokens — no hardcoded colors.

## Quick Start: New Screen in 5 Minutes

Copy `../templates/ListScreen.tsx`, rename it, and customize:

```tsx
import { ListScreen } from "../templates/ListScreen";

export function MyFeature() {
  return (
    <ListScreen
      title="My Feature"
      description="Manage my feature records"
      columns={[
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ]}
      fetchData={() => api.listMyFeatures()}
      onCreate={() => {/* open create form */}}
    />
  );
}
```

## Components

### Button

```tsx
import { Button } from "../ui/button";

<Button>Default</Button>
<Button variant="secondary">Secondary</Button>
<Button variant="outline">Outline</Button>
<Button variant="ghost">Ghost</Button>
<Button variant="destructive">Delete</Button>
<Button size="sm">Small</Button>
<Button size="lg">Large</Button>
<Button size="icon"><PlusIcon /></Button>
```

Props: `variant`, `size`, `disabled`, `onClick`, `type`, `className`

### Card

```tsx
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "../ui/card";

<Card>
  <CardHeader>
    <CardTitle>Total Revenue</CardTitle>
    <CardDescription>Last 30 days</CardDescription>
  </CardHeader>
  <CardContent>
    <p className="text-3xl font-bold">$12,345</p>
  </CardContent>
  <CardFooter>
    <Button size="sm">View Details</Button>
  </CardFooter>
</Card>
```

### Badge

```tsx
import { Badge } from "../ui/badge";

<Badge>Default</Badge>
<Badge variant="secondary">Secondary</Badge>
<Badge variant="outline">Outline</Badge>
<Badge variant="success">Active</Badge>
<Badge variant="warning">Pending</Badge>
<Badge variant="destructive">Failed</Badge>
```

### Input

```tsx
import { Input } from "../ui/input";
import { Label } from "../ui/label";

<div>
  <Label htmlFor="name">Name</Label>
  <Input id="name" placeholder="Enter name" />
</div>
```

### Select

```tsx
import { Select } from "../ui/select";

<Select
  value={status}
  onValueChange={setStatus}
  options={[
    { value: "active", label: "Active" },
    { value: "inactive", label: "Inactive" },
  ]}
/>
```

### Checkbox

```tsx
import { Checkbox } from "../ui/checkbox";

<Checkbox
  checked={enabled}
  onCheckedChange={setEnabled}
  label="Enable feature"
/>
```

### Dialog (Modal)

```tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "../ui/dialog";

const [open, setOpen] = useState(false);

<Button onClick={() => setOpen(true)}>Open</Button>
<Dialog open={open} onOpenChange={setOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Confirm Action</DialogTitle>
      <DialogDescription>Are you sure?</DialogDescription>
    </DialogHeader>
    <p>Dialog content here</p>
    <DialogFooter>
      <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
      <Button onClick={handleConfirm}>Confirm</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

### Tabs

```tsx
import { Tabs, TabsList, TabsTrigger, TabsContent } from "../ui/tabs";

<Tabs defaultValue="overview">
  <TabsList>
    <TabsTrigger value="overview">Overview</TabsTrigger>
    <TabsTrigger value="details">Details</TabsTrigger>
  </TabsList>
  <TabsContent value="overview">Overview content</TabsContent>
  <TabsContent value="details">Details content</TabsContent>
</Tabs>
```

### Tooltip

```tsx
import { Tooltip } from "../ui/tooltip";

<Tooltip content="This is helpful">
  <Button variant="ghost" size="icon">?</Button>
</Tooltip>
```

### Magic UI Components

```tsx
import { ShimmerButton } from "../ui/shimmer-button";
import { AnimatedGradientText } from "../ui/animated-gradient-text";
import { NumberTicker } from "../ui/number-ticker";

<ShimmerButton>Get Started</ShimmerButton>
<AnimatedGradientText>Build apps. Create agents.</AnimatedGradientText>
<NumberTicker value={12345} />
```

## Patterns

### Page Layout

```tsx
export function MyPage() {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">My Page</h1>
          <p className="text-muted-foreground">Description here</p>
        </div>
        <Button>Create New</Button>
      </div>
      {/* content */}
    </div>
  );
}
```

### Data Table

```tsx
import { Card } from "../ui/card";
import { Badge } from "../ui/badge";

<Card>
  <table className="data-table">
    <thead>
      <tr><th>Name</th><th>Status</th><th>Actions</th></tr>
    </thead>
    <tbody>
      {items.map(item => (
        <tr key={item.id}>
          <td>{item.name}</td>
          <td><Badge variant="success">{item.status}</Badge></td>
          <td><Button size="sm" variant="ghost">Edit</Button></td>
        </tr>
      ))}
    </tbody>
  </table>
</Card>
```

### Form

```tsx
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from "../ui/card";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Button } from "../ui/button";

<Card>
  <CardHeader>
    <CardTitle>Create Record</CardTitle>
  </CardHeader>
  <CardContent className="space-y-4">
    <div>
      <Label htmlFor="name">Name</Label>
      <Input id="name" value={name} onChange={e => setName(e.target.value)} />
    </div>
  </CardContent>
  <CardFooter className="flex justify-end gap-2">
    <Button variant="outline" onClick={onCancel}>Cancel</Button>
    <Button onClick={onSave}>Save</Button>
  </CardFooter>
</Card>
```

## Design Tokens

Use CSS variables, never hardcode:

- `--background`, `--foreground` — page bg/text
- `--card`, `--card-foreground` — card bg/text
- `--primary`, `--primary-foreground` — brand color
- `--secondary`, `--secondary-foreground` — muted actions
- `--muted`, `--muted-foreground` — subtle bg/text
- `--border`, `--input`, `--ring` — borders and focus
- `--destructive` — errors/delete
- `--radius`, `--radius-lg` — corner rounding

Dark mode: tokens automatically switch via `html[data-theme="dark"]`.
""")

# File: desktop/src/templates/ListScreen.tsx
write_file("desktop/src/templates/ListScreen.tsx", """/**
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
""")

# File: desktop/src/templates/DetailScreen.tsx
write_file("desktop/src/templates/DetailScreen.tsx", """/**
 * DetailScreen Template — copy this to create a new detail/view screen.
 *
 * Usage:
 * 1. Copy this file to `../features/myfeature/MyFeatureDetail.tsx`
 * 2. Replace `Item` with your type, update sections
 * 3. Wire up your API calls
 */
import { useQuery } from "@tanstack/react-query";

import { Button } from "../components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/card";
import { Badge } from "../components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "../components/ui/tabs";

export interface DetailField {
  label: string;
  value: React.ReactNode;
}

export interface DetailSection {
  title: string;
  fields: DetailField[];
}

export interface DetailScreenTemplateProps<T> {
  title: string;
  subtitle?: string;
  status?: string;
  fetchItem: () => Promise<T>;
  sections: DetailSection[];
  onEdit?: () => void;
  onDelete?: () => void;
  onBack?: () => void;
  actions?: React.ReactNode;
  tabs?: { value: string; label: string; content: React.ReactNode }[];
}

export function DetailScreenTemplate<T>({
  title,
  subtitle,
  status,
  fetchItem,
  sections,
  onEdit,
  onDelete,
  onBack,
  actions,
  tabs,
}: DetailScreenTemplateProps<T>) {
  const { data, isLoading } = useQuery({
    queryKey: ["detail", title],
    queryFn: fetchItem,
  });

  if (isLoading) return <p>Loading...</p>;
  if (!data) return <div className="error-banner">Not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          {onBack && (
            <Button variant="ghost" size="sm" onClick={onBack} className="mb-2">
              ← Back
            </Button>
          )}
          <div className="flex items-center gap-3">
            <h1 className="page-title">{title}</h1>
            {status && <Badge>{status}</Badge>}
          </div>
          {subtitle && <p className="text-muted-foreground">{subtitle}</p>}
        </div>
        <div className="flex gap-2">
          {actions}
          {onEdit && <Button variant="outline" onClick={onEdit}>Edit</Button>}
          {onDelete && <Button variant="destructive" onClick={onDelete}>Delete</Button>}
        </div>
      </div>

      {tabs ? (
        <Tabs defaultValue={tabs[0].value}>
          <TabsList>
            {tabs.map((tab) => (
              <TabsTrigger key={tab.value} value={tab.value}>{tab.label}</TabsTrigger>
            ))}
          </TabsList>
          {tabs.map((tab) => (
            <TabsContent key={tab.value} value={tab.value}>{tab.content}</TabsContent>
          ))}
        </Tabs>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {sections.map((section) => (
            <Card key={section.title}>
              <CardHeader>
                <CardTitle>{section.title}</CardTitle>
              </CardHeader>
              <CardContent>
                <dl className="space-y-3">
                  {section.fields.map((field) => (
                    <div key={field.label}>
                      <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                      <dd className="text-sm mt-1">{field.value}</dd>
                    </div>
                  ))}
                </dl>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
""")

# File: desktop/src/templates/FormScreen.tsx
write_file("desktop/src/templates/FormScreen.tsx", """/**
 * FormScreen Template — copy this to create a new create/edit form.
 *
 * Usage:
 * 1. Copy this file to `../features/myfeature/MyFeatureForm.tsx`
 * 2. Replace `FormData` with your type, update fields
 * 3. Wire up your API calls
 */
import { useState } from "react";

import { Button } from "../components/ui/button";
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from "../components/ui/card";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Select } from "../components/ui/select";
import { Checkbox } from "../components/ui/checkbox";

export interface FormField {
  key: string;
  label: string;
  type: "text" | "number" | "email" | "select" | "checkbox" | "textarea";
  placeholder?: string;
  options?: { value: string; label: string }[];
  required?: boolean;
}

export interface FormScreenTemplateProps<T> {
  title: string;
  fields: FormField[];
  initialData: Partial<T>;
  onSave: (data: T) => Promise<void>;
  onCancel: () => void;
  saveLabel?: string;
}

export function FormScreenTemplate<T extends Record<string, any>>({
  title,
  fields,
  initialData,
  onSave,
  onCancel,
  saveLabel = "Save",
}: FormScreenTemplateProps<T>) {
  const [data, setData] = useState<Partial<T>>(initialData);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const set = (key: string, value: any) => {
    setData((prev) => ({ ...prev, [key]: value }));
    setError(null);
  };

  const handleSave = async () => {
    // Validate required fields
    for (const field of fields) {
      if (field.required && !data[field.key]) {
        setError(`${field.label} is required`);
        return;
      }
    }
    setSaving(true);
    try {
      await onSave(data as T);
    } catch (e: any) {
      setError(e.message || "Failed to save");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="page max-w-2xl mx-auto">
      <div className="page-header">
        <h1 className="page-title">{title}</h1>
      </div>

      {error && <div className="error-banner">{error}</div>}

      <Card>
        <CardHeader>
          <CardTitle>{title}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {fields.map((field) => (
            <div key={field.key}>
              <Label htmlFor={field.key}>
                {field.label}
                {field.required && <span className="text-destructive"> *</span>}
              </Label>
              {field.type === "select" ? (
                <Select
                  value={String(data[field.key] ?? "")}
                  onValueChange={(v) => set(field.key, v)}
                  options={field.options ?? []}
                  placeholder={field.placeholder}
                />
              ) : field.type === "checkbox" ? (
                <Checkbox
                  checked={!!data[field.key]}
                  onCheckedChange={(v) => set(field.key, v)}
                  label={field.label}
                />
              ) : field.type === "textarea" ? (
                <textarea
                  id={field.key}
                  className="w-full p-2 border rounded"
                  placeholder={field.placeholder}
                  value={String(data[field.key] ?? "")}
                  onChange={(e) => set(field.key, e.target.value)}
                  rows={4}
                />
              ) : (
                <Input
                  id={field.key}
                  type={field.type}
                  placeholder={field.placeholder}
                  value={String(data[field.key] ?? "")}
                  onChange={(e) => set(field.key, e.target.value)}
                />
              )}
            </div>
          ))}
        </CardContent>
        <CardFooter className="flex justify-end gap-2">
          <Button variant="outline" onClick={onCancel} disabled={saving}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? "Saving..." : saveLabel}
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
""")

# File: desktop/src/styles.css
write_file("desktop/src/styles.css", """@import "./styles/tokens.css";
@import "./components/ui/ui.css";
@import "./styles/modern-reskin.css";

/* Desktop UI Foundation (PR 1): the app now defaults to the Light theme
   (user decision). An explicit [data-theme] on <html> always wins;
   otherwise the OS prefers-color-scheme preference applies when no choice
   is set. Theme Studio's applyThemeTokens() keeps working — it sets inline
   custom properties, which outrank every rule below. */
:root {
  color-scheme: light dark;
  --bg: #f5f7fb;
  --bg-elevated: #ffffff;
  --surface-sidebar: var(--bg-elevated);
  --border: #dde3ef;
  --text: #131826;
  --text-muted: #5b6572;
  --accent: #4f7cff;
  --accent-secondary: #4f7cff;
  --accent-contrast: #ffffff;
  --danger: #ef4444;
  --success: #22c55e;
  --warning: #f59e0b;
  --info: #3b82f6;
  /* Runtime UX Modernization (issue #198): the default categorical chart
     palette, applied before any workspace publishes a theme - see
     applyTheme.ts for how a Published theme's own palette overrides this. */
  --chart-1: #635bff;
  --chart-2: #0f9d76;
  --chart-3: #f59e0b;
  --chart-4: #ef4444;
  --chart-5: #3b82f6;
  --chart-6: #8b5cf6;
  --font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --base-font-size: 14px;
  font-family: var(--font-family);
  font-size: var(--base-font-size);
}

@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --bg: #0f1420;
    --bg-elevated: #161d2e;
    --surface-sidebar: var(--bg-elevated);
    --border: #2a3550;
    --text: #e6ebf5;
    --text-muted: #94a3b8;
  }
}

/* Explicit theme overrides — html-qualified to outrank :root. */
html[data-theme="dark"] {
  --bg: #0f1420;
  --bg-elevated: #161d2e;
  --surface-sidebar: var(--bg-elevated);
  --border: #2a3550;
  --text: #e6ebf5;
  --text-muted: #94a3b8;
}

html[data-theme="light"] {
  --bg: #f5f7fb;
  --bg-elevated: #ffffff;
  --surface-sidebar: var(--bg-elevated);
  --border: #dde3ef;
  --text: #131826;
  --text-muted: #5b6572;
}

/* Ontology Explorer visual language (issue #339, extended for Object 360
   issue #352): a warm, organic canvas tone distinct from the cooler
   dot-grid `.graph-canvas-wrap` uses for every other builder (Agent Team
   Builder/Workflow Studio/Business Rule Board) - its own tokens so it
   stays theme-adaptive (light by default, dark override, same as every
   other token above) instead of a hardcoded cream that would look wrong
   in dark mode. */
:root {
  --ontology-canvas-bg: #faf6ee;
  --ontology-canvas-grid: #ece0c4;
}
html[data-theme="dark"] {
  --ontology-canvas-bg: #1c170f;
  --ontology-canvas-grid: #342b1b;
}
@media (prefers-color-scheme: dark) {
  html:not([data-theme]) {
    --ontology-canvas-bg: #1c170f;
    --ontology-canvas-grid: #342b1b;
  }
}

/* UX/UI Modernization, Phase A (issue #191): Theme Studio overrides every
   var above except --accent-contrast (no modeled token for it yet) via
   inline styles on :root - see lib/applyTheme.ts. A workspace with no
   Published theme never has those inline styles set, so it keeps exactly
   this static palette. Shape (radius_scale) and density are stored and
   previewed in Theme Studio but not yet retrofitted across this
   stylesheet's ~15 hardcoded border-radius/spacing rules - a named
   follow-up, not silently claimed here. */

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  min-width: 1024px;
}

button {
  font-family: inherit;
  cursor: pointer;
}

input,
select,
textarea {
  font-family: inherit;
  background: var(--bg-elevated);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
}

.app-shell {
  display: grid;
  grid-template-columns: 220px 1fr;
  min-height: 100vh;
}

.sidebar {
  background: var(--surface-sidebar);
  border-right: 1px solid var(--border);
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  font-size: 18px;
  padding: 8px;
  margin-bottom: 12px;
}

/* Desktop UI Foundation (PR 1): brand logo. */
.brand-logo {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  flex: none;
  display: block;
}
.auth-logo {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: block;
  margin: 0 auto 16px;
}

.nav-item {
  padding: 8px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text);
  text-align: left;
  font-size: 14px;
}

.nav-item:hover {
  background: rgba(127, 127, 127, 0.1);
}

.nav-item.active {
  background: var(--accent);
  color: var(--accent-contrast);
}

.sidebar-divider {
  height: 1px;
  background: var(--border);
  margin: 8px 4px;
}

.main {
  padding: 24px 32px;
  overflow-y: auto;
}

.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.btn {
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--text);
  padding: 7px 14px;
  border-radius: 6px;
  font-size: 14px;
}

.btn-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-contrast);
}

.link-button {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  cursor: pointer;
}

.link-button:hover {
  text-decoration: underline;
}

/* Record-detail-page round: every list's ID column is a visible hyperlink
   into that record's detail page, not just an invisible whole-row click
   target - the same "click the ID to open the record" affordance every
   list already gave you for the row itself, made explicit. */
.id-link {
  color: var(--accent);
  font-weight: 600;
}

.id-link:hover {
  text-decoration: underline;
}

/* Admin landing page redesign round: a categorized home in place of the
   old flat tab row, plus the breadcrumb shown once inside a tool. */
.admin-landing-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.admin-cat-card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 18px;
}

.admin-cat-head {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}

.admin-cat-icon {
  font-size: 20px;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: var(--bg);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.admin-cat-head h3 {
  margin: 0 0 2px;
  font-size: 15px;
}

.admin-cat-head p {
  margin: 0;
  font-size: 12.5px;
  color: var(--text-muted);
}

.admin-cat-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.admin-cat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  text-align: left;
  background: none;
  border: 0;
  border-radius: 8px;
  padding: 8px 9px;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
  cursor: pointer;
}

.admin-cat-item:hover {
  background: var(--bg);
  color: var(--accent);
}

.admin-cat-item .admin-cat-arrow {
  color: var(--text-muted);
  font-weight: 400;
}

.admin-cat-item:hover .admin-cat-arrow {
  color: var(--accent);
}

.admin-breadcrumb {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.btn-danger {
  border-color: var(--danger);
  color: var(--danger);
  background: transparent;
}

.card {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
}

/* A generic centered dialog - first used by Deployment Management's
   update-with-diff review step. Deliberately minimal (no other screen in
   the app has needed a true modal before this - version history/duplicate
   review etc. all use an inline panel instead). */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.modal {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 20px;
  width: 100%;
  max-height: 85vh;
  overflow-y: auto;
}

.kpi-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.kpi-tile {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 14px 16px;
  cursor: pointer;
}

.kpi-tile:hover {
  border-color: var(--accent);
}

.kpi-tile .value {
  font-size: 22px;
  font-weight: 700;
}

.kpi-tile .label {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

th,
td {
  text-align: left;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

th {
  color: var(--text-muted);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
}

tr:hover td {
  background: rgba(127, 127, 127, 0.06);
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  background: rgba(148, 163, 184, 0.2);
}

.badge-success {
  background: rgba(34, 197, 94, 0.18);
  color: var(--success);
}

.badge-warning {
  background: rgba(245, 158, 11, 0.18);
  color: var(--warning);
}

.badge-danger {
  background: rgba(239, 68, 68, 0.18);
  color: var(--danger);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  max-width: 640px;
}

.form-grid.full,
.form-field.full {
  grid-column: 1 / -1;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}

.form-field label {
  color: var(--text-muted);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid var(--danger);
  color: var(--danger);
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 14px;
}

.success-banner {
  background: rgba(34, 197, 94, 0.12);
  border: 1px solid var(--success);
  color: var(--success);
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 14px;
}

.empty-state {
  color: var(--text-muted);
  padding: 24px;
  text-align: center;
}

.centered-screen {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.auth-card {
  width: 380px;
}

.auth-card h1 {
  font-size: 20px;
  margin-bottom: 4px;
}

.auth-card p {
  color: var(--text-muted);
  font-size: 13px;
  margin-top: 0;
}

.line-items-table input {
  width: 100%;
}

.tab-row {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.tab {
  padding: 8px 4px;
  border: none;
  background: none;
  color: var(--text-muted);
  border-bottom: 2px solid transparent;
  font-size: 14px;
}

.tab.active {
  color: var(--text);
  border-bottom-color: var(--accent);
}

/* Printable documents (quotes/orders/invoices) - see PrintOverlay and
   PrintableDocument. On screen this is a full-page preview overlay; the
   @media print block below hides everything else so window.print() /
   "Save as PDF" captures only the document itself, in both the Tauri
   webview and a Team Workspace browser tab. Colors here are deliberately
   plain black-on-white rather than the app's --text/--bg tokens, since a
   printed business document shouldn't follow dark mode. */
.print-overlay {
  position: fixed;
  inset: 0;
  background: #fff;
  color: #111;
  z-index: 1000;
  overflow: auto;
}

.print-overlay-bar {
  position: sticky;
  top: 0;
  display: flex;
  gap: 8px;
  padding: 12px 24px;
  background: #f2f2f2;
  border-bottom: 1px solid #ddd;
}

.print-overlay-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 32px;
}

.print-doc-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 28px;
}

.print-doc-business {
  font-weight: 700;
  font-size: 18px;
}

.print-doc-logo {
  max-width: 64px;
  max-height: 64px;
  object-fit: contain;
}

.print-doc-muted {
  color: #666;
  font-size: 13px;
}

.print-doc-heading {
  text-align: right;
}

.print-doc-heading h1 {
  margin: 0 0 4px 0;
  font-size: 20px;
}

.print-doc-parties {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 28px;
}

.print-doc-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #888;
}

.print-doc-strong {
  font-weight: 600;
}

.print-doc-dates {
  text-align: right;
  font-size: 13px;
}

.print-doc-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 16px;
}

.print-doc-table th,
.print-doc-table td {
  text-align: left;
  padding: 6px 8px;
  border-bottom: 1px solid #ddd;
  font-size: 13px;
}

.print-doc-table th {
  font-size: 11px;
  text-transform: uppercase;
  color: #888;
}

.print-doc-totals {
  margin-left: auto;
  max-width: 280px;
  font-size: 13px;
}

.print-doc-totals > div {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
}

.print-doc-total-line {
  font-weight: 700;
  border-top: 1px solid #333;
  margin-top: 4px;
  padding-top: 4px !important;
}

.print-doc-notes {
  margin-top: 28px;
  font-size: 13px;
}

/* Business Rules / Workflow Automation visual redesign - shared chrome
   both builders use: a breadcrumb+title header with action buttons, and
   numbered sections. Kept generic (not rule- or workflow- specific) so
   both builders share one set of classes. */
.builder-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
  gap: 16px;
}

.builder-breadcrumb {
  color: var(--text-muted);
  font-size: 12px;
  margin-bottom: 2px;
}

.builder-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.builder-title-row h2 {
  margin: 0;
}

.builder-subtitle {
  color: var(--text-muted);
  font-size: 13px;
  margin: 4px 0 0 0;
  max-width: 640px;
}

.builder-header-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.builder-layout {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 16px;
  align-items: start;
}

.builder-section {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 16px;
}

.builder-section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  margin-bottom: 12px;
}

.step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 999px;
  background: var(--accent);
  color: var(--accent-contrast);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.builder-row-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  background: var(--bg);
}

.builder-row-remove {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 16px;
  line-height: 1;
  padding: 4px;
  cursor: pointer;
}

.builder-row-remove:hover {
  color: var(--danger);
}

.builder-and-divider {
  text-align: center;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  margin: -2px 0 6px 0;
}

/* One level of nested OR-grouping (migration 0020) - a dashed box around
   the conditions that get OR'd together into one sub-unit before that
   unit participates in the rule/workflow's top-level match_type. */
.builder-or-group {
  border: 1px dashed var(--accent);
  border-radius: 8px;
  padding: 10px 12px 4px 12px;
  margin-bottom: 8px;
  background: var(--bg-elevated);
}

.builder-or-group-label {
  color: var(--accent);
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 8px;
}

.builder-or-group .builder-row-card {
  background: var(--bg);
}

.builder-summary-panel {
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px;
  position: sticky;
  top: 0;
}

.builder-summary-panel h4 {
  margin: 0 0 12px 0;
  font-size: 13px;
}

.builder-summary-panel h4:not(:first-child) {
  margin-top: 18px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
  padding: 5px 0;
  border-bottom: 1px solid var(--border);
}

.summary-row:last-child {
  border-bottom: none;
}

.summary-row .label {
  color: var(--text-muted);
}

.summary-row .value {
  text-align: right;
  font-weight: 600;
}

.legend-group {
  margin-bottom: 10px;
}

.legend-group-title {
  color: var(--text-muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

/* Business Rule Board 2.0 (issue #194): the IF -> THEN lane board that
   replaces the old numbered-steps form. One .rule-lane-set per rule in a
   chain (the "if" rule, plus any linked "else_if"/"else" siblings), laid
   out left to right so branching reads the same way a flowchart would.
   Drag-and-drop reorders/groups cards within a lane - there's no existing
   list-drag primitive in this codebase (the Shared Visual Builder
   Framework's own drag hook is for free-form x/y canvas nodes, a different
   shape entirely), so this is a small, self-contained native HTML5
   drag-and-drop implementation scoped to this one screen. */
.rule-board {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rule-chain {
  display: flex;
  gap: 16px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.rule-lane-set {
  flex: 0 0 auto;
  width: 420px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rule-lane-set-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.rule-branch-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.rule-branch-badge.if {
  background: var(--accent);
  color: var(--accent-contrast);
}

.rule-branch-badge.else_if {
  background: var(--warning, #b8860b);
  color: #fff;
}

.rule-branch-badge.else {
  background: var(--text-muted);
  color: var(--bg);
}

.rule-lane {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  padding: 12px;
  min-height: 80px;
}

.rule-lane-title {
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.rule-lane-card {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin-bottom: 8px;
}

.rule-lane-card.dragging {
  opacity: 0.4;
}

.rule-lane-card .rule-drag-handle {
  cursor: grab;
  color: var(--text-muted);
  padding: 10px 2px;
  user-select: none;
  flex-shrink: 0;
}

.rule-lane-card .builder-row-card,
.rule-lane-card .builder-or-group {
  flex: 1;
  margin-bottom: 0;
}

.rule-lane-dropzone {
  height: 10px;
  margin: -4px 0;
  border-radius: 4px;
}

.rule-lane-dropzone.drop-over {
  background: var(--accent);
  opacity: 0.35;
}

.rule-conflict-banner {
  border: 1px solid var(--danger);
  border-radius: 10px;
  padding: 12px 14px;
  background: color-mix(in srgb, var(--danger) 10%, transparent);
  font-size: 13px;
}

.rule-conflict-banner ul {
  margin: 6px 0 0 0;
  padding-left: 18px;
}

.rule-template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 12px;
  margin: 12px 0;
}

.rule-template-card {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 14px;
  background: var(--bg-elevated);
  cursor: pointer;
  text-align: left;
}

.rule-template-card:hover {
  border-color: var(--accent);
}

.rule-template-card h4 {
  margin: 0 0 6px 0;
  font-size: 14px;
}

.rule-template-card p {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
}

.rule-ifthen-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  align-items: start;
}

.rule-ifthen-row > .rule-lane {
  flex: 1 1 280px;
}

/* Screen Builder 2.0 (issue #195, 5a): palette | canvas | inspector/layers
   three-pane layout, same drag/drop visual language as the business rule
   lane board above (a drag handle, a dashed drop-zone that lights up on
   hover) applied to a nested 12-column grid instead of a flat lane list. */
.page-builder-layout {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  flex-wrap: wrap;
}

.page-builder-palette {
  flex: 0 0 200px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-height: 640px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  background: var(--bg-elevated);
}

.page-builder-palette-group-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.page-builder-palette-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  cursor: grab;
  font-size: 13px;
  margin-bottom: 6px;
  user-select: none;
}

.page-builder-palette-item.dragging {
  opacity: 0.4;
}

.page-builder-palette-icon {
  flex-shrink: 0;
}

.page-builder-canvas-wrap {
  flex: 1 1 420px;
  min-width: 320px;
  overflow-x: auto;
  padding: 8px 0;
}

.page-builder-canvas-frame {
  border: 1px dashed var(--border);
  border-radius: 12px;
  padding: 16px;
  margin: 0 auto;
  background: var(--bg);
  transition: width 0.15s ease;
}

.page-builder-dropzone {
  height: 10px;
  margin: 4px 0;
  border-radius: 4px;
  border: 1px dashed transparent;
}

.page-builder-dropzone.drop-over {
  background: var(--accent);
  opacity: 0.35;
  border-color: var(--accent);
}

.page-builder-node {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elevated);
  margin-bottom: 4px;
}

.page-builder-node.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.page-builder-node.dragging {
  opacity: 0.4;
}

.page-builder-node-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  font-size: 12px;
  cursor: pointer;
}

.page-builder-drag-handle {
  cursor: grab;
  color: var(--text-muted);
  flex-shrink: 0;
}

.page-builder-node-icon {
  flex-shrink: 0;
}

.page-builder-node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.page-builder-node-delete {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
}

.page-builder-node-delete:hover {
  color: var(--danger);
}

.page-builder-node-body {
  padding: 8px 8px 2px;
  border-top: 1px dashed var(--border);
}

.page-builder-empty-hint {
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
  padding: 32px 0;
}

.page-builder-side {
  flex: 0 0 260px;
  min-width: 220px;
}

.page-builder-inspector,
.page-builder-layers {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  background: var(--bg-elevated);
  margin-bottom: 12px;
}

.page-builder-inspector-head {
  font-weight: 600;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.page-builder-inspector-divider {
  border-top: 1px solid var(--border);
  margin: 14px 0;
}

.page-builder-layer-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
}

.page-builder-layer-row:hover {
  background: var(--bg);
}

.page-builder-layer-row.selected {
  background: var(--accent);
  color: var(--accent-contrast);
}

.page-builder-layer-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 900px) {
  .page-builder-palette,
  .page-builder-side {
    flex: 1 1 100%;
  }
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 3px 0;
}

.char-counter {
  text-align: right;
  color: var(--text-muted);
  font-size: 11px;
  margin-top: 2px;
}

/* Workflow Automation's visual flow canvas - a static, live rendering of
   the trigger/conditions/actions state (editing happens in the left
   panel's numbered sections, not by dragging nodes here). */
.workflow-layout {
  display: grid;
  grid-template-columns: 380px 1fr;
  gap: 16px;
  align-items: start;
}

.workflow-canvas-wrap {
  position: relative;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: auto;
  min-height: 480px;
  background-image: radial-gradient(var(--border) 1px, transparent 1px);
  background-size: 20px 20px;
}

.workflow-canvas {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;
  transform-origin: top center;
  width: fit-content;
  margin: 0 auto;
}

.workflow-node {
  width: 320px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  overflow: hidden;
}

.workflow-node-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.workflow-node-trigger .workflow-node-head {
  background: rgba(34, 197, 94, 0.15);
  color: var(--success);
}

.workflow-node-conditions .workflow-node-head {
  background: rgba(79, 124, 255, 0.15);
  color: var(--accent);
}

.workflow-node-actions .workflow-node-head {
  background: rgba(245, 158, 11, 0.15);
  color: var(--warning);
}

.workflow-node-body {
  padding: 12px;
}

.workflow-trigger-body {
  display: flex;
  align-items: center;
  gap: 10px;
}

.workflow-trigger-icon {
  font-size: 22px;
}

.workflow-trigger-title {
  font-weight: 600;
  font-size: 14px;
}

.workflow-trigger-detail {
  color: var(--text-muted);
  font-size: 12px;
}

.workflow-condition-chip {
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
  font-size: 12px;
  margin-bottom: 6px;
}

.workflow-condition-chip:last-child {
  margin-bottom: 0;
}

.workflow-match-chip {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  text-align: center;
  width: 100%;
  margin: -2px 0 6px 0;
}

/* Canvas-node counterpart to .builder-or-group - the same one level of
   nested OR-grouping, drawn small inside the Conditions node. */
.workflow-or-group {
  border: 1px dashed var(--accent);
  border-radius: 6px;
  padding: 6px 6px 0 6px;
  margin-bottom: 6px;
}

.workflow-or-group-label {
  color: var(--accent);
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.workflow-action-card {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  margin-bottom: 8px;
  background: var(--bg);
}

.workflow-action-card:last-child {
  margin-bottom: 0;
}

.workflow-action-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.workflow-action-title {
  font-weight: 600;
  font-size: 13px;
}

.workflow-action-subtitle {
  color: var(--text-muted);
  font-size: 12px;
}

.workflow-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 28px;
  color: var(--text-muted);
}

.workflow-connector-line {
  width: 2px;
  flex: 1;
  background: var(--border);
}

.workflow-connector-arrow {
  font-size: 12px;
  line-height: 1;
  margin-top: -2px;
}

.workflow-end-node {
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 6px 20px;
  font-size: 12px;
  font-weight: 700;
  color: var(--text-muted);
  background: var(--bg-elevated);
}

.workflow-zoom-controls {
  position: sticky;
  float: right;
  bottom: 12px;
  margin-right: 12px;
  margin-top: -48px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: fit-content;
}

.workflow-zoom-controls .btn {
  width: 30px;
  height: 30px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* AI Agent Platform v2, Phase 5b: the Agent Team Builder's free-form
   canvas - a genuine node-drag/SVG-connector graph editor, unlike
   Workflow Automation's own fixed linear canvas above. Reuses that
   canvas's dotted-grid background and sticky zoom-controls placement,
   the only two things worth reusing from a fixed layout. */
.graph-editor-layout {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: 16px;
  align-items: start;
}

.graph-palette {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.graph-connect-banner {
  background: rgba(79, 124, 255, 0.12);
  border: 1px solid var(--accent);
  color: var(--accent);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  margin-bottom: 8px;
}

/* Shown until a freshly-created graph has its first edge - the only other
   cue that nodes need to be manually connected is the small per-node "→"
   button itself, easy to miss on first use (confirmed by a real user
   confused that a canvas full of just-added nodes looked "all unrelated"). */
.graph-hint-banner {
  background: var(--bg-elevated);
  border: 1px dashed var(--border);
  color: var(--text-muted);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  margin-bottom: 8px;
}

.graph-canvas-wrap {
  position: relative;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: auto;
  max-height: 640px;
  background-image: radial-gradient(var(--border) 1px, transparent 1px);
  background-size: 20px 20px;
}

.graph-canvas {
  position: relative;
  transform-origin: top left;
}

.graph-edges-svg {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}

.graph-edges-svg g {
  pointer-events: all;
}

.graph-node {
  position: absolute;
  width: 220px;
  border: 2px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  overflow: hidden;
  cursor: grab;
  user-select: none;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.08);
}

.graph-node:active {
  cursor: grabbing;
}

.graph-node-selected {
  box-shadow: 0 0 0 2px var(--accent);
}

.graph-node-head {
  padding: 6px 10px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.graph-node-body {
  padding: 8px 10px;
}

.graph-node-body strong {
  display: block;
  font-size: 13px;
}

.graph-node-body small {
  display: block;
  color: var(--text-muted);
  font-size: 11px;
  margin-top: 2px;
}

.graph-node-connect {
  position: absolute;
  right: -12px;
  bottom: -12px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--accent);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
}

.graph-zoom-controls {
  position: sticky;
  float: right;
  bottom: 12px;
  margin-right: 12px;
  margin-top: -48px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: fit-content;
}

.graph-zoom-controls .btn {
  width: 30px;
  height: 30px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Ontology Explorer visual language (issue #339, extended for Object 360
   issue #352): the Dependency Explorer's and Object 360's own node shape
   (a small colored dot, not a boxed card), so these are scoped new
   classes rather than edits to .graph-node/.graph-canvas above - those
   stay exactly as every other VisualBuilderCanvas consumer (Agent Team
   Builder, Workflow Studio, Business Rule Board) already depends on. */
.ontology-canvas-wrap {
  background-color: var(--ontology-canvas-bg);
  background-image: radial-gradient(var(--ontology-canvas-grid) 1.5px, transparent 1.5px);
  background-size: 24px 24px;
}

.ontology-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 14px;
  padding: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 999px;
  width: fit-content;
  max-width: 100%;
}

.ontology-legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  white-space: nowrap;
  border: 0;
  background: transparent;
  border-radius: 999px;
  padding: 5px 11px;
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease, opacity 0.15s ease;
}

.ontology-legend-item:hover {
  background: rgba(127, 127, 127, 0.1);
  color: var(--text);
}

.ontology-legend-item-off {
  opacity: 0.4;
}

@keyframes ontology-dot-in {
  from {
    opacity: 0;
    transform: scale(0.3);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.ontology-node-dot {
  position: absolute;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
  animation: ontology-dot-in 0.3s cubic-bezier(0.34, 1.3, 0.64, 1) both;
  transition: left 0.4s cubic-bezier(0.34, 1.2, 0.64, 1), top 0.4s cubic-bezier(0.34, 1.2, 0.64, 1), transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.15s ease,
    opacity 0.2s ease;
}

.ontology-node-dot::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: radial-gradient(circle at 32% 28%, rgba(255, 255, 255, 0.55), rgba(255, 255, 255, 0) 65%);
  pointer-events: none;
}

.ontology-node-dot:hover {
  transform: scale(1.35);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
  z-index: 3;
}

.ontology-node-dimmed {
  opacity: 0.18;
}

.ontology-node-dimmed:hover {
  opacity: 0.5;
}

@keyframes ontology-root-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 3px var(--accent), 0 2px 6px rgba(0, 0, 0, 0.3);
  }
  50% {
    box-shadow: 0 0 0 5px var(--accent), 0 2px 10px rgba(0, 0, 0, 0.3);
  }
}

.ontology-node-root {
  cursor: default;
  animation: ontology-dot-in 0.3s cubic-bezier(0.34, 1.3, 0.64, 1) both, ontology-root-pulse 2.4s ease-in-out 0.3s infinite;
}

.ontology-node-root:hover {
  transform: none;
}

.ontology-node-inspecting:not(.ontology-node-root) {
  box-shadow: 0 0 0 2px var(--text), 0 2px 8px rgba(0, 0, 0, 0.25);
  transform: scale(1.15);
}

@keyframes ontology-card-in {
  from {
    opacity: 0;
    transform: translateY(6px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.ontology-inspector-card {
  position: absolute;
  top: 16px;
  right: 16px;
  width: 290px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 16px;
  box-shadow: 0 20px 44px rgba(0, 0, 0, 0.22), 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 5;
  animation: ontology-card-in 0.2s cubic-bezier(0.2, 0.8, 0.2, 1) both;
}

.ontology-inspector-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 10px;
}

.ontology-inspector-head strong {
  font-size: 15px;
}

.ontology-inspector-subtitle {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-top: 2px;
}

.ontology-inspector-definition {
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--text-muted);
  margin: 0 0 12px;
}

.ontology-inspector-actions {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
}

.ontology-inspector-actions .btn {
  flex: 1;
  padding: 7px 8px;
  font-size: 12px;
  border-radius: 8px;
}

.ontology-inspector-groups {
  max-height: 260px;
  overflow-y: auto;
}

.ontology-inspector-group {
  padding-top: 8px;
  margin-top: 8px;
  border-top: 1px solid var(--border);
}

.ontology-inspector-group:first-child {
  padding-top: 0;
  margin-top: 0;
  border-top: 0;
}

.ontology-inspector-group-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 5px;
}

.ontology-inspector-bullet {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  border: 0;
  border-radius: 6px;
  background: none;
  padding: 4px 6px;
  margin: 0 -6px;
  font-size: 13px;
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.ontology-inspector-bullet:hover {
  background: rgba(127, 127, 127, 0.1);
  color: var(--accent);
}

.graph-property-panel {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elevated);
  padding: 12px;
  min-height: 200px;
}

.graph-condition-row {
  display: flex;
  gap: 6px;
  margin-bottom: 6px;
  align-items: center;
}

.graph-condition-row input,
.graph-condition-row select {
  flex: 1;
  min-width: 0;
}

.graph-router-branch {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px;
  margin-bottom: 8px;
}

.graph-run-row {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px;
  margin-bottom: 8px;
}

@media print {
  body > *:not(#print-root) {
    display: none !important;
  }
  .no-print {
    display: none !important;
  }
  .print-overlay {
    position: static;
    inset: auto;
  }
  .print-overlay-page {
    max-width: none;
    padding: 0;
  }
}

/* Help tab - a category/topic list on the left, article content on the
   right, matching the search-then-drill-in shape of every other admin
   list+detail screen rather than inventing a new layout pattern. */
.help-admin-layout {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 20px;
  align-items: start;
}
.help-admin-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.help-admin-cat-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin: 14px 0 6px;
}
.help-admin-cat-label:first-child {
  margin-top: 0;
}
.help-admin-topic-btn {
  display: block;
  width: 100%;
  text-align: left;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  padding: 9px 10px;
  font-size: 13.5px;
  color: var(--text);
  cursor: pointer;
}
.help-admin-topic-btn:hover {
  background: var(--bg-elevated);
}
.help-admin-topic-btn.active {
  background: var(--bg-elevated);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}
.help-admin-toc {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 18px 0 0;
  padding: 14px 16px;
}
.help-admin-toc div {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-muted);
  margin-bottom: 2px;
}
.help-admin-toc a {
  font-size: 13px;
  color: var(--text-muted);
}
.help-admin-toc a:hover {
  color: var(--accent);
}
.help-admin-article h4 {
  font-size: 16px;
  margin: 26px 0 10px;
  scroll-margin-top: 16px;
}
.help-admin-article h4:first-child {
  margin-top: 0;
}
.help-admin-article p,
.help-admin-article ul,
.help-admin-article ol {
  font-size: 14px;
  line-height: 1.75;
  color: var(--text);
  margin: 0 0 14px;
}
.help-admin-article ul,
.help-admin-article ol {
  padding-left: 20px;
}
.help-admin-article li {
  margin-bottom: 5px;
}
.help-admin-article code {
  font-family: var(--font-mono, ui-monospace, monospace);
  background: rgba(127, 127, 127, 0.15);
  border-radius: 4px;
  padding: 1px 5px;
  font-size: 12.5px;
}
.help-admin-article pre {
  background: #0d0f13;
  color: #d7f7ff;
  border-radius: 10px;
  padding: 14px 16px;
  overflow-x: auto;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12.5px;
  line-height: 1.7;
  margin: 0 0 14px;
}
.help-admin-article pre code {
  background: transparent;
  padding: 0;
  color: inherit;
}
.help-admin-article .help-note {
  background: rgba(99, 102, 241, 0.12);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 13px;
  margin: 0 0 14px;
}
.help-admin-related {
  margin-top: 32px;
  padding-top: 18px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
@media (max-width: 800px) {
  .help-admin-layout {
    grid-template-columns: 1fr;
  }
}

/* ListTable (Runtime UX Modernization, issue #198): the shared
   dynamic-column table - density, column resize/reorder/picker chrome,
   and the table<->card mobile collapse. See components/ListTable.tsx. */
.list-table-wrap {
  width: 100%;
}

.list-table-toolbar {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.segmented {
  display: inline-flex;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}
.segmented-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
  padding: 5px 10px;
  cursor: pointer;
}
.segmented-btn + .segmented-btn {
  border-left: 1px solid var(--border);
}
.segmented-btn.active {
  background: var(--accent);
  color: #fff;
}

.list-table-column-picker {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 10;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px;
  min-width: 180px;
  max-height: 320px;
  overflow-y: auto;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}
.list-table-column-picker-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}
.list-table-column-picker-row:hover {
  background: rgba(127, 127, 127, 0.08);
  border-radius: 4px;
}

.list-table-table th {
  position: relative;
  user-select: none;
}
.list-table-resize-handle {
  position: absolute;
  /* Stays fully inside its own <th> box (not straddling the boundary)
     so it never sits under the next column's header, which would
     otherwise win hit-testing since later DOM siblings paint on top. */
  right: 0;
  top: 0;
  bottom: 0;
  width: 6px;
  cursor: col-resize;
  touch-action: none;
}
.list-table-resize-handle:hover {
  background: var(--accent);
  opacity: 0.4;
}

/* Density: default row padding already matches "comfortable" (see the
   base th/td rule); compact/dense tighten it. */
.list-table-compact .list-table-table th,
.list-table-compact .list-table-table td {
  padding: 5px 10px;
}
.list-table-dense .list-table-table th,
.list-table-dense .list-table-table td {
  padding: 2px 8px;
  font-size: 13px;
}

/* Mobile card collapse: both markups stay in the DOM; only one shows at
   a time, so no resize listener is needed. */
.list-table-cards {
  display: none;
}
@media (max-width: 720px) {
  .list-table-table {
    display: none;
  }
  .list-table-cards {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
}
.list-table-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  cursor: pointer;
  background: var(--bg-elevated);
}
.list-table-card-field {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
}
.list-table-card-label {
  color: var(--text-muted);
}
.list-table-card-value {
  text-align: right;
}

/* QuickPreviewDrawer: a right-side slide-in, not a centered .modal - it
   sits alongside the list rather than blocking it. */
.quick-preview-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}
.quick-preview-drawer {
  width: 380px;
  max-width: 90vw;
  height: 100%;
  background: var(--bg-elevated);
  border-left: 1px solid var(--border);
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: quick-preview-slide-in 0.15s ease-out;
}
@keyframes quick-preview-slide-in {
  from {
    transform: translateX(16px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
.quick-preview-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}
.quick-preview-fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}
.quick-preview-field {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}
.quick-preview-label {
  color: var(--text-muted);
}
.quick-preview-value {
  text-align: right;
}
.quick-preview-footer {
  display: flex;
  justify-content: flex-end;
}

/* RecordHeader (Runtime UX Modernization, issue #198): the one
   title/subtitle/status/owner/record-number/avatar + up to 4 key
   attributes header every detail/edit view uses now. */
.record-header {
  margin-bottom: 16px;
}
.record-header-top {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}
.record-header-avatar {
  width: 40px;
  height: 40px;
  border-radius: 999px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-weight: 700;
}
.record-header-attributes {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
  margin-top: 10px;
}
.record-header-attribute {
  display: flex;
  flex-direction: column;
  font-size: 13px;
}
.record-header-attribute-label {
  color: var(--text-muted);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

/* Sticky command bar: the Back/Edit toolbar atop a long record detail
   page - a variant of .toolbar that stays visible while scrolling,
   without changing every other .toolbar in the app. */
.record-command-bar {
  position: sticky;
  top: 0;
  z-index: 3;
  background: var(--bg);
  padding: 8px 0;
  margin-bottom: 8px;
}

/* InlineEditField (Runtime UX Modernization, issue #198): a read-only
   detail field that's click-to-edit when the viewer has write access. */
.inline-edit-value {
  cursor: pointer;
  border-radius: 4px;
  padding: 2px 4px;
  margin: -2px -4px;
}
.inline-edit-value:hover,
.inline-edit-value:focus-visible {
  background: rgba(127, 127, 127, 0.1);
  outline: 1px dashed var(--border);
}
.inline-edit-active select,
.inline-edit-active input,
.inline-edit-active textarea {
  max-width: 220px;
}
.inline-edit-error {
  color: var(--danger);
  font-size: 12px;
  margin-bottom: 4px;
}

/* DashboardGrid (Runtime UX Modernization, issue #198): the shared
   widget grid - one widget-positioning implementation for both the live
   Dashboard (view mode) and the dashboard-layout editor's canvas (edit
   mode, with drag/resize handles). See components/DashboardGrid.tsx. */
.dashboard-grid-item {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}
.dashboard-grid-item.editable {
  border: 1px solid transparent;
  border-radius: 8px;
}
.dashboard-grid-item.editable:hover {
  border-color: var(--border);
}
.dashboard-grid-item.editable.selected {
  border-color: var(--accent);
}
.dashboard-grid-item-handle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--bg-elevated);
  border-radius: 8px 8px 0 0;
  cursor: move;
  touch-action: none;
  user-select: none;
}
.dashboard-grid-item-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.dashboard-grid-item-resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 14px;
  height: 14px;
  cursor: nwse-resize;
  touch-action: none;
}
.dashboard-grid-item-resize::after {
  content: "";
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 6px;
  height: 6px;
  border-right: 2px solid var(--border);
  border-bottom: 2px solid var(--border);
}

/* Lookup Custom Field Type: search-as-you-type combobox, used wherever a
   field_type === "lookup" field renders on a create/edit form (see
   CustomFieldsSection.tsx's useCustomFieldElements). */
.lookup-picker {
  position: relative;
}
.lookup-picker-input-row {
  display: flex;
  align-items: center;
  gap: 4px;
}
.lookup-picker-input-row input {
  flex: 1;
}
.lookup-picker-clear {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 2px 6px;
}
.lookup-picker-clear:hover {
  color: var(--text);
}
.lookup-picker-menu {
  position: absolute;
  z-index: 20;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  max-height: 240px;
  overflow-y: auto;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
  list-style: none;
  margin: 0;
  padding: 4px 0;
}
.lookup-picker-option {
  padding: 6px 10px;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.lookup-picker-option.active {
  background: var(--accent);
  color: var(--accent-contrast);
}
.lookup-picker-option-empty {
  padding: 6px 10px;
  color: var(--text-muted);
  cursor: default;
}
""")

# File: desktop/src/features/dashboard/DashboardWidgetCard.tsx
write_file("desktop/src/features/dashboard/DashboardWidgetCard.tsx", """import { useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import { Bar } from "../../components/Bar";
import { ListTable, type ListTableColumn } from "../../components/ListTable";
import type { Section } from "../../components/AppShell";
import { sectionFor } from "../../components/GlobalSearch";
import { useCurrentUser } from "../../lib/useCurrentUser";
import { Card, CardContent } from "../../components/ui/card";
import {
  entityTypeLabel,
  type CustomReport,
  type DashboardSummary,
  type DashboardWidget,
  type RecordListMode,
  type RecordListRow,
  type Task,
  type TaskQueueMode,
} from "../../lib/types";
import { KPI_DEFS } from "./kpis";
import { TASK_QUEUE_MODE_LABELS } from "./widgetMeta";

/**
 * Runtime UX Modernization (issue #198): the one renderer every widget kind
 * goes through, used identically by the live Dashboard (view mode) and the
 * dashboard-layout editor's canvas (edit mode) - "one shared card system"
 * rather than Dashboard.tsx special-casing each kind in its own markup the
 * way it did before this grid existed. `interactive=false` (the editor)
 * suppresses row-click navigation, since a click there selects the widget
 * for the Inspector instead of leaving the admin screen.
 */
export function DashboardWidgetCard({
  widget,
  summary,
  reports,
  interactive = true,
  onNavigate,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  summary: DashboardSummary | null;
  reports: CustomReport[];
  interactive?: boolean;
  onNavigate: (section: Section) => void;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  switch (widget.kind) {
    case "kpi":
      return <KpiWidgetCard widget={widget} summary={summary} interactive={interactive} onNavigate={onNavigate} />;
    case "chart":
      return <ChartWidgetCard widget={widget} reports={reports} />;
    case "record_list":
      return <RecordListWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "table":
      return <TableWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "saved_view":
      return <SavedViewWidgetCard widget={widget} interactive={interactive} onOpenRecord={onOpenRecord} />;
    case "task_queue":
      return <TaskQueueWidgetCard widget={widget} interactive={interactive} onNavigate={onNavigate} />;
    case "agent_insight":
      return <AgentInsightWidgetCard widget={widget} />;
    default:
      return (
        <div className="card" style={{ height: "100%", margin: 0 }}>
          <p className="empty-state">Unknown widget "{widget.kind}"</p>
        </div>
      );
  }
}

function KpiWidgetCard({
  widget,
  summary,
  interactive,
  onNavigate,
}: {
  widget: DashboardWidget;
  summary: DashboardSummary | null;
  interactive: boolean;
  onNavigate: (section: Section) => void;
}) {
  const kpi = KPI_DEFS.find((k) => k.key === widget.config.kpi_key);
  if (!kpi) return null;
  return (
    <Card
      className="kpi-tile"
      style={{ height: "100%", margin: 0, cursor: interactive ? "pointer" : "default" }}
      onClick={interactive ? () => onNavigate(kpi.section) : undefined}
    >
      <CardContent className="pt-6">
        <div className="text-3xl font-bold tracking-tight">{summary ? kpi.value(summary) : "—"}</div>
        <div className="text-sm text-muted-foreground mt-1">{summary ? kpi.label(summary) : "Loading..."}</div>
      </CardContent>
    </Card>
  );
}

/** Runs its report fresh (the same `run_custom_report` command the Reports
 * screen's own runner uses) and draws it with the same dependency-free
 * `Bar` the Reports screen uses, so a chart looks identical whether it's
 * viewed there or here. */
function ChartWidgetCard({ widget, reports }: { widget: DashboardWidget; reports: CustomReport[] }) {
  const report = reports.find((r) => r.id === widget.config.report_id);
  const q = useQuery({
    queryKey: ["runCustomReport", report?.id],
    queryFn: () => api.runCustomReport(report!.id),
    enabled: !!report,
  });
  const rows = q.data ?? [];
  const max = Math.max(0, ...rows.map((r) => r.value));

  if (!report) {
    return (
      <div className="card" style={{ height: "100%", margin: 0 }}>
        <p className="empty-state">(report deleted)</p>
      </div>
    );
  }

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>{report.name}</h3>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">No data yet.</p>}
      {rows.length > 0 && (
        <table>
          <thead>
            <tr>
              <th>Group</th>
              <th></th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r, i) => (
              <tr key={r.group}>
                <td>{r.group}</td>
                <td>
                  <Bar value={r.value} max={max} index={i} />
                </td>
                <td>{report.aggregate === "sum" ? r.value.toLocaleString() : r.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

function useDashboardRecordListRows(widget: DashboardWidget) {
  const entityType = widget.config.entity_type as string;
  const mode = widget.config.mode as RecordListMode;
  const limit = (widget.config.limit as number) ?? 5;
  const savedViewId = (widget.config.saved_view_id as string | undefined) ?? null;
  const q = useQuery({
    queryKey: ["dashboardRecordList", entityType, mode, limit, savedViewId],
    queryFn: () => api.runDashboardRecordList(entityType, mode, limit, savedViewId),
  });
  return { entityType, mode, rows: q.data ?? [], isLoading: q.isLoading };
}

/** A short list of records for `widget.config.entity_type`, run fresh via
 * `run_dashboard_record_list` (see `dashboard_widget_service` in core for
 * what "recent" vs "due_soon" mean). Clicking a row jumps straight to that
 * record, the same one-shot navigation an ID hyperlink or a Global search
 * result already uses. */
function RecordListWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const { entityType, mode, rows, isLoading } = useDashboardRecordListRows(widget);

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>
        {entityTypeLabel(entityType)} - {mode === "due_soon" ? "due soon" : "recent"}
      </h3>
      {isLoading && <p>Loading...</p>}
      {rows.length === 0 && !isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((r) => (
            <li
              key={r.entity_id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
            >
              <span>{r.title}</span>
              {r.subtitle && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{r.subtitle}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

const TABLE_WIDGET_COLUMNS: ListTableColumn<RecordListRow>[] = [
  { key: "title", label: "Name", getValue: (r) => r.title },
  { key: "subtitle", label: "Detail", getValue: (r) => r.subtitle },
];

/** Same data as a record-list widget, rendered as a full `ListTable` (the
 * same sortable/resizable grid a list screen uses) instead of a compact
 * bullet list - for a dashboard tile that wants to read more like a real
 * data grid. No new backend: identical `run_dashboard_record_list` call. */
function TableWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const { entityType, mode, rows, isLoading } = useDashboardRecordListRows(widget);

  return (
    <div className="card" style={{ height: "100%", margin: 0, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <h3 style={{ marginTop: 0, flexShrink: 0 }}>
        {entityTypeLabel(entityType)} - {mode === "due_soon" ? "due soon" : "recent"}
      </h3>
      {isLoading && <p>Loading...</p>}
      <div style={{ flex: 1, overflow: "auto" }}>
        <ListTable
          storageKey={`dashboardTable:${widget.id}`}
          columns={TABLE_WIDGET_COLUMNS}
          visibleKeys={null}
          onVisibleKeysChange={() => {}}
          groups={[{ label: "", rows }]}
          getRowId={(r) => r.entity_id}
          onRowClick={interactive ? (r) => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
          density="compact"
          mobileCollapse={false}
          emptyMessage="Nothing here yet."
        />
      </div>
    </div>
  );
}

/** Embeds one specific Saved View's exact filters/sort (see `useSavedViews`)
 * as a dashboard tile - the data resolution is identical to a record-list
 * widget narrowed by `saved_view_id`, just always scoped to that one view
 * rather than letting the widget's own `mode` pick "recent"/"due soon". */
function SavedViewWidgetCard({
  widget,
  interactive,
  onOpenRecord,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onOpenRecord: (section: Section, id: string) => void;
}) {
  const entityType = widget.config.entity_type as string;
  const limit = (widget.config.limit as number) ?? 5;
  const savedViewId = widget.config.saved_view_id as string;
  const views = useQuery({ queryKey: ["savedViewsForWidget", entityType], queryFn: () => api.listSavedViews(entityType) });
  const view = views.data?.find((v) => v.id === savedViewId);
  const q = useQuery({
    queryKey: ["dashboardRecordList", entityType, "recent", limit, savedViewId],
    queryFn: () => api.runDashboardRecordList(entityType, "recent", limit, savedViewId),
  });
  const rows = q.data ?? [];

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>{view ? view.name : `${entityTypeLabel(entityType)} saved view`}</h3>
      {q.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !q.isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((r) => (
            <li
              key={r.entity_id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onOpenRecord(sectionFor(r.entity_type), r.entity_id) : undefined}
            >
              <span>{r.title}</span>
              {r.subtitle && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{r.subtitle}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** A queue of open Tasks filtered the same way Tasks.tsx's own tabs filter
 * them, plus "mine" (owner_user_id === the signed-in user, via
 * `useCurrentUser`) since a dashboard tile is personal in a way a shared
 * list screen tab isn't. No new backend - filters the same
 * `api.listTasks()` rows the Tasks screen already fetches. */
function TaskQueueWidgetCard({
  widget,
  interactive,
  onNavigate,
}: {
  widget: DashboardWidget;
  interactive: boolean;
  onNavigate: (section: Section) => void;
}) {
  const mode = widget.config.mode as TaskQueueMode;
  const limit = (widget.config.limit as number) ?? 5;
  const tasks = useQuery({ queryKey: ["tasks"], queryFn: () => api.listTasks() });
  const currentUser = useCurrentUser();
  const today = new Date().toISOString().slice(0, 10);

  const open = (tasks.data ?? []).filter((t) => t.status !== "Completed" && !t.archived_at);
  let rows: Task[];
  if (mode === "today") rows = open.filter((t) => t.due_date === today);
  else if (mode === "overdue") rows = open.filter((t) => !!t.due_date && t.due_date < today);
  else if (mode === "upcoming") rows = open.filter((t) => !!t.due_date && t.due_date > today);
  else rows = open.filter((t) => t.owner_user_id === currentUser.data?.id);
  rows = rows.slice(0, limit);

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>Tasks - {TASK_QUEUE_MODE_LABELS[mode] ?? mode}</h3>
      {tasks.isLoading && <p>Loading...</p>}
      {rows.length === 0 && !tasks.isLoading && <p className="empty-state">Nothing here yet.</p>}
      {rows.length > 0 && (
        <ul style={{ margin: 0, paddingLeft: 0, listStyle: "none", fontSize: 14 }}>
          {rows.map((t) => (
            <li
              key={t.id}
              style={{
                display: "flex",
                justifyContent: "space-between",
                gap: 12,
                padding: "6px 0",
                borderBottom: "1px solid var(--border, #e5e7eb)",
                cursor: interactive ? "pointer" : "default",
              }}
              onClick={interactive ? () => onNavigate("tasks") : undefined}
            >
              <span>{t.title}</span>
              {t.due_date && <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>{t.due_date}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** One active AI agent's today's token usage and recent run outcomes -
 * resolved from `api.getAiAgentTokenUsage`/`api.listAiAgentRuns`, both
 * already exposed to the frontend (see the AI Gateway's existing
 * per-(workspace, agent, user, day) accounting), so this adds no new Rust
 * surface either. */
function AgentInsightWidgetCard({ widget }: { widget: DashboardWidget }) {
  const agentId = widget.config.agent_id as string;
  const agents = useQuery({ queryKey: ["aiAgentsForWidget"], queryFn: () => api.listAiAgents(false) });
  const usage = useQuery({ queryKey: ["aiAgentTokenUsage", agentId], queryFn: () => api.getAiAgentTokenUsage(agentId) });
  const runs = useQuery({ queryKey: ["aiAgentRunsForWidget", agentId], queryFn: () => api.listAiAgentRuns("agent", agentId, 5) });
  const agent = agents.data?.find((a) => a.id === agentId);
  const recentRuns = runs.data ?? [];
  const succeeded = recentRuns.filter((r) => r.status === "succeeded").length;

  if (!agent) {
    return (
      <div className="card" style={{ height: "100%", margin: 0 }}>
        <p className="empty-state">(agent deleted)</p>
      </div>
    );
  }

  return (
    <div className="card" style={{ height: "100%", margin: 0, overflow: "auto" }}>
      <h3 style={{ marginTop: 0 }}>
        {agent.name} <span className={`badge${agent.is_active ? " badge-success" : ""}`}>{agent.is_active ? "Active" : "Inactive"}</span>
      </h3>
      {usage.data && (
        <p style={{ fontSize: 13, color: "var(--text-muted)", margin: "4px 0" }}>
          Today: {usage.data.today_input_tokens + usage.data.today_output_tokens} tokens
          {usage.data.daily_token_budget ? ` of ${usage.data.daily_token_budget} budget` : ""}
        </p>
      )}
      {recentRuns.length > 0 ? (
        <p style={{ fontSize: 13, margin: "4px 0" }}>
          Last {recentRuns.length} runs: {succeeded} succeeded, {recentRuns.length - succeeded} other
        </p>
      ) : (
        <p className="empty-state">No runs yet.</p>
      )}
    </div>
  );
}
""")

print("Done")
