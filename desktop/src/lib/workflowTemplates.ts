import type { WorkflowActionInput, WorkflowConditionInput } from "./types";

/** Workflow Studio 2.0 (issue #193): 5 starter templates an admin can
 * begin a new workflow from, instead of a fully blank Trigger/Conditions/
 * Actions form. Each only pre-fills what's genuinely entity-type-agnostic
 * (trigger shape, action type, a short placeholder) - a relationship,
 * specific field key or agent/pipeline id still needs picking per
 * workspace, the same honest "this saves clicks, it isn't a turnkey
 * automation" scope every reference package's own manifest already keeps
 * to. Only uses action types `WorkflowAutomationAdmin.tsx`'s own
 * `ActionEditor` can actually author (`WORKFLOW_ACTION_TYPES`) -
 * `call_connector_action` has no UI here yet, so no template uses it. */
export type WorkflowTemplate = {
  key: string;
  label: string;
  description: string;
  build: (firstTransitionValue: string | null) => {
    trigger_type: "status_changed" | "scheduled" | "record_created" | "record_updated";
    trigger_status: string | null;
    trigger_offset_days: number;
    match_type: "all" | "any";
    conditions: WorkflowConditionInput[];
    actions: WorkflowActionInput[];
  };
};

export const WORKFLOW_TEMPLATES: WorkflowTemplate[] = [
  {
    key: "approval_flow",
    label: "Approval Flow",
    description: "When a record reaches a status you pick, notify every Administrator that it needs approval.",
    build: (firstTransitionValue) => ({
      trigger_type: "status_changed",
      trigger_status: firstTransitionValue,
      trigger_offset_days: 0,
      match_type: "all",
      conditions: [],
      actions: [{ action_type: "add_notification", params_json: JSON.stringify({ audience: "all_admins", message: "This record needs your approval" }) }],
    }),
  },
  {
    key: "status_automation",
    label: "Status Automation",
    description: "When a record reaches a status you pick, automatically create a follow-up task.",
    build: (firstTransitionValue) => ({
      trigger_type: "status_changed",
      trigger_status: firstTransitionValue,
      trigger_offset_days: 0,
      match_type: "all",
      conditions: [],
      actions: [{ action_type: "create_task", params_json: JSON.stringify({ title: "Follow up", description: null, due_in_days: 1, assignee_user_id: null }) }],
    }),
  },
  {
    key: "scheduled_follow_up",
    label: "Scheduled Follow-up",
    description: "On a recurring schedule, create a reminder for every active record of this type.",
    build: () => ({
      trigger_type: "scheduled",
      trigger_status: null,
      trigger_offset_days: 7,
      match_type: "all",
      conditions: [],
      actions: [{ action_type: "create_reminder", params_json: JSON.stringify({ title: "Weekly follow-up", description: null, remind_in_days: 0, assignee_user_id: null }) }],
    }),
  },
  {
    key: "integration_sync",
    label: "Integration Sync",
    description: "When a record updates, keep a field on a linked record in sync - pick the relationship and fields after creating this.",
    build: () => ({
      trigger_type: "record_updated",
      trigger_status: null,
      trigger_offset_days: 0,
      match_type: "all",
      conditions: [],
      actions: [{ action_type: "update_related_record", params_json: JSON.stringify({ relationship_definition_id: "", target_field_key: "", target_field_source: "custom", value: "", copy_from_field_key: null }) }],
    }),
  },
  {
    key: "agent_assisted_process",
    label: "Agent-Assisted Process",
    description: "When a record is created, hand it to an AI Agent or Agent Team - pick which one after creating this.",
    build: () => ({
      trigger_type: "record_created",
      trigger_status: null,
      trigger_offset_days: 0,
      match_type: "all",
      conditions: [],
      actions: [{ action_type: "run_ai_agent", params_json: JSON.stringify({ target_type: "agent", target_id: "", input_template: "" }) }],
    }),
  },
];
