/**
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
