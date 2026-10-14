# LanesraOS Desktop UI Component Library

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
