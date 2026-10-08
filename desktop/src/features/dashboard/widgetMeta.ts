import { entityTypeLabel, type CustomFieldEntityType, type CustomReport, type DashboardWidget, type TaskQueueMode } from "../../lib/types";
import { kpiLabel } from "./kpis";

/** Entity types whose "due_soon" mode actually sorts by a real due date -
 * see `dashboard_widget_service::run` in core. Every other entity type
 * only offers "Recently created". Shared by the record-list/table widget
 * pickers in DashboardLayoutsAdmin.tsx and by WidgetInspector's in-place
 * editor for the same two widget kinds. */
export const DUE_SOON_ENTITY_TYPES: CustomFieldEntityType[] = ["Task", "Invoice"];

export const TASK_QUEUE_MODE_LABELS: Record<TaskQueueMode, string> = {
  today: "Due today",
  overdue: "Overdue",
  upcoming: "Upcoming",
  mine: "Assigned to me",
};

/** A short human label for one dashboard widget, used both by the admin
 * chip/grid chrome and the live Dashboard's widget headers. */
export function widgetLabel(w: DashboardWidget, reports: CustomReport[]): string {
  if (w.kind === "kpi") return kpiLabel(w.config.kpi_key as string);
  if (w.kind === "chart") {
    const report = reports.find((r) => r.id === w.config.report_id);
    return report ? `📊 ${report.name}` : "📊 (report deleted)";
  }
  if (w.kind === "record_list" || w.kind === "table") {
    const entityType = w.config.entity_type as string;
    const mode = w.config.mode as string;
    const icon = w.kind === "table" ? "🗂️" : "📋";
    return `${icon} ${entityTypeLabel(entityType)} - ${mode === "due_soon" ? "due soon" : "recent"}`;
  }
  if (w.kind === "saved_view") {
    return `🔖 ${entityTypeLabel(w.config.entity_type as string)} saved view`;
  }
  if (w.kind === "task_queue") {
    return `✅ Tasks - ${TASK_QUEUE_MODE_LABELS[w.config.mode as TaskQueueMode] ?? (w.config.mode as string)}`;
  }
  if (w.kind === "agent_insight") {
    return `🤖 Agent insight`;
  }
  return w.kind;
}
