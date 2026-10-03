// Screen Builder 2.0 (issue #195, 5a): the client-side mirror of
// page_layout_service::COMPONENT_TYPES/CONTAINER_COMPONENT_TYPES - the
// single source of truth the palette, canvas and inspector all read from,
// so adding a new component type later only ever touches this file plus
// the matching server-side allowlist, never three separate UI pieces.
//
// `configSchema` drives one generic inspector form (see
// PageBuilderInspector in PageBuilderAdmin.tsx) instead of ~24 bespoke
// property panels - each entry is a plain {key, label, type, ...} the
// inspector renders as the matching input and writes back into the
// node's `config` object under that key.

export type PageComponentCategory = "layout" | "record" | "data" | "actions" | "content" | "navigation" | "utility";

export type ConfigFieldType = "text" | "textarea" | "select" | "number" | "boolean" | "field_picker";

export interface ConfigFieldSchema {
  key: string;
  label: string;
  type: ConfigFieldType;
  options?: string[]; // for "select"
  placeholder?: string;
  defaultValue?: string | number | boolean;
  help?: string;
}

export interface PageComponentDef {
  type: string;
  category: PageComponentCategory;
  label: string;
  icon: string;
  isContainer: boolean;
  description: string;
  configSchema: ConfigFieldSchema[];
  defaultColumnSpan: number; // 1-12, seeded onto a new node's NodeLayout.column_span
}

export const PAGE_COMPONENT_CATEGORIES: { key: PageComponentCategory; label: string }[] = [
  { key: "layout", label: "Layout" },
  { key: "record", label: "Record" },
  { key: "data", label: "Data" },
  { key: "actions", label: "Actions" },
  { key: "content", label: "Content" },
  { key: "navigation", label: "Navigation" },
  { key: "utility", label: "Utility" },
];

export const PAGE_COMPONENT_LIBRARY: PageComponentDef[] = [
  // Layout
  { type: "section", category: "layout", label: "Section", icon: "▭", isContainer: true, description: "A titled, full-width group that other components stack or arrange inside.", configSchema: [{ key: "title", label: "Title", type: "text", placeholder: "Section title" }], defaultColumnSpan: 12 },
  { type: "grid", category: "layout", label: "Grid", icon: "▦", isContainer: true, description: "An even grid of sub-columns for cards, tiles or small components.", configSchema: [{ key: "columns", label: "Columns", type: "select", options: ["2", "3", "4"], defaultValue: "2" }], defaultColumnSpan: 12 },
  { type: "columns", category: "layout", label: "Columns", icon: "⫼", isContainer: true, description: "Two or three unevenly-weighted columns side by side.", configSchema: [{ key: "ratio", label: "Split", type: "select", options: ["50/50", "33/33/33", "25/75", "75/25", "33/67"], defaultValue: "50/50" }], defaultColumnSpan: 12 },
  { type: "tabs", category: "layout", label: "Tabs", icon: "⬒", isContainer: true, description: "A tabbed container - each direct child becomes one tab's content.", configSchema: [{ key: "tabLabels", label: "Tab labels (comma-separated)", type: "text", placeholder: "Overview, Activity, Related" }], defaultColumnSpan: 12 },
  { type: "divider", category: "layout", label: "Divider", icon: "—", isContainer: false, description: "A plain horizontal rule.", configSchema: [], defaultColumnSpan: 12 },
  { type: "spacer", category: "layout", label: "Spacer", icon: "␣", isContainer: false, description: "Blank vertical space.", configSchema: [{ key: "height", label: "Height", type: "select", options: ["small", "medium", "large"], defaultValue: "medium" }], defaultColumnSpan: 12 },
  { type: "sticky_panel", category: "layout", label: "Sticky Panel", icon: "📌", isContainer: true, description: "Stays pinned to the top or bottom of the page while the rest scrolls.", configSchema: [{ key: "position", label: "Position", type: "select", options: ["top", "bottom"], defaultValue: "top" }], defaultColumnSpan: 12 },

  // Record
  { type: "field", category: "record", label: "Field", icon: "🏷️", isContainer: false, description: "One field's label and value.", configSchema: [{ key: "field_key", label: "Field", type: "field_picker" }], defaultColumnSpan: 6 },
  { type: "field_group", category: "record", label: "Field Group", icon: "🏷️🏷️", isContainer: true, description: "A compact cluster of Field components under one shared heading.", configSchema: [{ key: "title", label: "Heading", type: "text", placeholder: "Optional heading" }], defaultColumnSpan: 6 },
  { type: "record_header", category: "record", label: "Record Header", icon: "🪪", isContainer: false, description: "The record's name/title plus its primary identifying detail, styled as a page header.", configSchema: [], defaultColumnSpan: 12 },
  { type: "status_badge", category: "record", label: "Status Badge", icon: "🔖", isContainer: false, description: "A colored pill showing the record's status/stage field.", configSchema: [{ key: "field_key", label: "Status field", type: "field_picker", help: "Defaults to the object's built-in status/stage field if left blank." }], defaultColumnSpan: 3 },
  { type: "owner", category: "record", label: "Owner", icon: "👤", isContainer: false, description: "The record's assigned owner (user or team).", configSchema: [], defaultColumnSpan: 3 },
  { type: "record_number", category: "record", label: "Record Number", icon: "#", isContainer: false, description: "The record's auto-numbered identifier (e.g. Q-2026-041).", configSchema: [], defaultColumnSpan: 3 },

  // Data
  { type: "related_list", category: "data", label: "Related List", icon: "🔗", isContainer: false, description: "A card-style list of records from one relationship - the same related-records card every detail page already shows.", configSchema: [{ key: "relationship_key", label: "Relationship", type: "text", placeholder: "e.g. opportunities" }], defaultColumnSpan: 12 },
  { type: "table", category: "data", label: "Table", icon: "▤", isContainer: false, description: "The same related records as a dense, sortable table instead of cards.", configSchema: [{ key: "relationship_key", label: "Relationship", type: "text", placeholder: "e.g. quotes" }], defaultColumnSpan: 12 },
  { type: "kpi", category: "data", label: "KPI", icon: "📊", isContainer: false, description: "A single computed number - a sum, a count, or a related-record count.", configSchema: [{ key: "metric_label", label: "Label", type: "text", placeholder: "Open balance" }, { key: "source", label: "Computed from", type: "select", options: ["field_sum", "field_count", "related_count"], defaultValue: "field_sum" }, { key: "field_key", label: "Field", type: "field_picker" }], defaultColumnSpan: 3 },
  { type: "chart", category: "data", label: "Chart", icon: "📈", isContainer: false, description: "A small bar/line/pie chart over related data.", configSchema: [{ key: "chart_type", label: "Chart type", type: "select", options: ["bar", "line", "pie"], defaultValue: "bar" }, { key: "metric_label", label: "Label", type: "text" }], defaultColumnSpan: 6 },

  // Actions
  { type: "button", category: "actions", label: "Button", icon: "🔘", isContainer: false, description: "A single labeled action button.", configSchema: [{ key: "label", label: "Label", type: "text", placeholder: "Edit" }, { key: "style", label: "Style", type: "select", options: ["primary", "secondary"], defaultValue: "secondary" }, { key: "action", label: "Action", type: "select", options: ["none", "edit", "navigate"], defaultValue: "none" }], defaultColumnSpan: 3 },
  { type: "command_bar", category: "actions", label: "Command Bar", icon: "⌘", isContainer: false, description: "A row of several buttons together, like a page's own toolbar.", configSchema: [{ key: "buttonLabels", label: "Buttons (comma-separated)", type: "text", placeholder: "Edit, Clone, Delete" }], defaultColumnSpan: 12 },
  { type: "quick_action", category: "actions", label: "Quick Action", icon: "⚡", isContainer: false, description: "A one-click action that changes a status or field without opening the full edit form.", configSchema: [{ key: "label", label: "Label", type: "text", placeholder: "Mark Won" }, { key: "target_status", label: "Sets status to", type: "text" }], defaultColumnSpan: 3 },
  { type: "agent_action", category: "actions", label: "Agent Action", icon: "🤖", isContainer: false, description: "Runs a named AI Agent or Pipeline against this record.", configSchema: [{ key: "label", label: "Label", type: "text", placeholder: "Summarize with AI" }, { key: "agent_name", label: "Agent or Pipeline name", type: "text", help: "Matched by name at render time - a picker bound to the real Agent Foundry list is 5b scope." }], defaultColumnSpan: 3 },

  // Content / Navigation / Utility - one representative type each; the
  // issue's own spec didn't itemize these three categories the way it
  // did Layout/Record/Data/Actions, so this is a deliberately small,
  // honest starting set rather than a guessed-at longer list.
  { type: "rich_text", category: "content", label: "Rich Text", icon: "📝", isContainer: false, description: "A block of admin-authored formatted text.", configSchema: [{ key: "text", label: "Text", type: "textarea", placeholder: "Write something..." }], defaultColumnSpan: 12 },
  { type: "anchor_link", category: "navigation", label: "Anchor Link", icon: "⚓", isContainer: false, description: "A jump-to-section link, useful on a long detail page.", configSchema: [{ key: "label", label: "Link text", type: "text" }, { key: "target_section_id", label: "Jumps to (Section id)", type: "text" }], defaultColumnSpan: 3 },
  { type: "note", category: "utility", label: "Note", icon: "💬", isContainer: false, description: "A static callout banner for admin-authored guidance.", configSchema: [{ key: "text", label: "Text", type: "textarea" }, { key: "tone", label: "Tone", type: "select", options: ["info", "warning"], defaultValue: "info" }], defaultColumnSpan: 12 },
];

export function componentDef(componentType: string): PageComponentDef | undefined {
  return PAGE_COMPONENT_LIBRARY.find((c) => c.type === componentType);
}

export function isContainerType(componentType: string): boolean {
  return componentDef(componentType)?.isContainer ?? false;
}
