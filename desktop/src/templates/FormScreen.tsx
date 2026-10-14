/**
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
