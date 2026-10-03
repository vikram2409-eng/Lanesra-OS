import type { PageDefinition, PageNode } from "../../lib/types";

/**
 * Screen Builder 2.0 (issue #195, 5b): the 4 built-in page templates
 * ("Executive 360," "Operations Workspace," "Clean Detail," "Data &
 * Insights") offered in PageBuilderAdmin's "Start from a template" picker,
 * alongside any Organization Templates an admin has saved (those come
 * from the backend - see page_template_service.rs). A built-in template
 * is pure client-side data, the same as `workflowTemplates.ts`'s and
 * `businessRuleTemplates.ts`'s starter templates - "applying" one just
 * seeds a page's own draft via the ordinary `updatePageLayout` call, so
 * the result is an immediately independent, editable page like any other
 * ("never permanently template-bound," per the issue's own wording) and
 * no backend concept of "built-in template" is needed at all.
 *
 * Each template's `build` is object-aware: it's handed this entity type's
 * real fields and relationship list and places real field keys / a real
 * relationship, picking the first few available rather than leaving every
 * picker blank - the honest middle ground between a fully turnkey page
 * (would require guessing which fields matter) and a content-free shape
 * (wouldn't be "object-aware" at all). An object with fewer fields or no
 * relationships yet still gets a valid page - sections/components that
 * have nothing to bind to are simply left out, not rendered broken.
 *
 * `thumbnail` is a tiny schematic block list the picker renders as a
 * miniature grid - the same "shows the shape, not real data" scoping this
 * app's node cards and dashboards already use elsewhere, not a real
 * screenshot.
 */

export type ThumbnailBlock = { span: number; kind: "header" | "kpi" | "field" | "list" | "action" };

export type PageTemplateDef = {
  key: string;
  label: string;
  description: string;
  thumbnail: ThumbnailBlock[];
  build: (fields: { key: string; label: string }[], relatedLists: { key: string; label: string }[]) => PageDefinition;
};

function newId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto ? crypto.randomUUID() : `id-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function node(componentType: string, config: Record<string, unknown>, columnSpan: number, children: PageNode[] = []): PageNode {
  return { id: newId(), component_type: componentType, config, children, layout: { column_span: columnSpan, tablet_column_span: null, mobile_column_span: null, order: 0 } };
}

export const PAGE_TEMPLATES: PageTemplateDef[] = [
  {
    key: "executive_360",
    label: "Executive 360",
    description: "A high-level, glanceable summary - status, owner, a few key numbers and the most important related records. Best for a leader who wants the headline, not every field.",
    thumbnail: [{ span: 12, kind: "header" }, { span: 4, kind: "kpi" }, { span: 4, kind: "kpi" }, { span: 4, kind: "kpi" }, { span: 12, kind: "list" }],
    build: (_fields, relatedLists) => {
      const root: PageNode[] = [node("record_header", {}, 12)];
      root.push(node("grid", { columns: "3" }, 12, [
        node("status_badge", {}, 4),
        node("owner", {}, 4),
        node("record_number", {}, 4),
      ]));
      const kpis = relatedLists.slice(0, 3).map((r) => node("kpi", { metric_label: r.label, source: "related_count", relationship_key: r.key }, 4));
      if (kpis.length > 0) root.push(node("grid", { columns: String(Math.max(1, kpis.length)) }, 12, kpis));
      if (relatedLists[0]) root.push(node("related_list", { relationship_key: relatedLists[0].key }, 12));
      return { root };
    },
  },
  {
    key: "operations_workspace",
    label: "Operations Workspace",
    description: "Denser and tab-organized for day-to-day work: an Overview tab with the core fields, a Related tab for everything linked to this record, plus a command bar of quick actions.",
    thumbnail: [{ span: 12, kind: "header" }, { span: 12, kind: "action" }, { span: 12, kind: "field" }, { span: 12, kind: "list" }],
    build: (fields, relatedLists) => {
      const coreFields = fields.slice(0, 6).map((f) => node("field", { field_key: f.key }, 6));
      const overviewTab = node("field_group", { title: "Overview" }, 12, coreFields);
      const relatedTab = node("section", { title: "Related" }, 12, relatedLists.slice(0, 3).map((r) => node("related_list", { relationship_key: r.key }, 12)));
      const root: PageNode[] = [
        node("record_header", {}, 12),
        node("command_bar", { buttonLabels: "Edit, Clone" }, 12),
        node("tabs", { tabLabels: "Overview, Related" }, 12, [overviewTab, relatedTab]),
      ];
      return { root };
    },
  },
  {
    key: "clean_detail",
    label: "Clean Detail",
    description: "The simplest possible page: a header, one tidy group of the core fields, and one related list at the bottom. For an object where a plain, uncluttered form is all anyone needs.",
    thumbnail: [{ span: 12, kind: "header" }, { span: 12, kind: "field" }, { span: 12, kind: "list" }],
    build: (fields, relatedLists) => {
      const root: PageNode[] = [
        node("record_header", {}, 12),
        node("field_group", { title: "Details" }, 12, fields.slice(0, 8).map((f) => node("field", { field_key: f.key }, 6))),
      ];
      if (relatedLists[0]) root.push(node("related_list", { relationship_key: relatedLists[0].key }, 12));
      return { root };
    },
  },
  {
    key: "data_insights",
    label: "Data & Insights",
    description: "Analytics-leaning: a row of KPI tiles computed from this record's relationships, a chart, and a dense table of related records - for an object people mainly want to measure.",
    thumbnail: [{ span: 12, kind: "header" }, { span: 3, kind: "kpi" }, { span: 3, kind: "kpi" }, { span: 3, kind: "kpi" }, { span: 3, kind: "kpi" }, { span: 6, kind: "kpi" }, { span: 12, kind: "list" }],
    build: (_fields, relatedLists) => {
      const root: PageNode[] = [node("record_header", {}, 12)];
      const kpis = relatedLists.slice(0, 4).map((r) => node("kpi", { metric_label: r.label, source: "related_count", relationship_key: r.key }, 3));
      if (kpis.length > 0) root.push(node("grid", { columns: String(Math.min(4, Math.max(1, kpis.length))) }, 12, kpis));
      if (relatedLists[0]) root.push(node("chart", { chart_type: "bar", metric_label: relatedLists[0].label }, 6));
      if (relatedLists[0]) root.push(node("table", { relationship_key: relatedLists[0].key }, 12));
      return { root };
    },
  },
];

export function pageTemplate(key: string): PageTemplateDef | undefined {
  return PAGE_TEMPLATES.find((t) => t.key === key);
}
