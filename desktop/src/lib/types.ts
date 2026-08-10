// Mirrors the serde-serialized shapes returned by the Rust commands in
// src-tauri/src/models. Money is always integer cents; quantities are
// integers scaled by 1000 (milli); rates/discounts are basis points
// (10000 = 100%). Never use these as floats in the UI - format them with
// lib/money.ts.

export interface Workspace {
  id: string;
  business_name: string;
  legal_name: string | null;
  currency_code: string;
  locale: string;
  timezone: string;
  default_tax_rate_bp: number;
  operating_mode: string;
  business_address: string | null;
  phone: string | null;
  logo_base64: string | null;
  logo_mime: string | null;
  dashboard_kpi_prefs: string | null;
  created_at: string;
  updated_at: string;
}

export interface WorkspaceSetup {
  business_name: string;
  legal_name: string | null;
  currency_code: string;
  locale: string;
  timezone: string;
  default_tax_rate_bp: number;
  admin_username: string;
  admin_display_name: string;
  admin_password: string;
  load_sample_data: boolean;
}

// FR-BRD-01: editing the workspace profile after first-run. Logo is a
// separate command (WorkspaceLogo) so a routine text edit never has to
// re-transmit the image payload.
export interface WorkspaceUpdate {
  business_name: string;
  legal_name: string | null;
  business_address: string | null;
  phone: string | null;
  currency_code: string;
  locale: string;
  timezone: string;
  default_tax_rate_bp: number;
}

export interface WorkspaceLogo {
  logo_base64: string;
  logo_mime: string;
}

/** FR-KPI: admin-chosen Dashboard KPI tiles and order; empty resets to default. */
export interface DashboardKpiPrefs {
  keys: string[];
}

export interface User {
  id: string;
  workspace_id: string;
  username: string;
  display_name: string;
  is_active: boolean;
  roles: string[];
  created_at: string;
  updated_at: string;
}

export interface Credentials {
  username: string;
  password: string;
}

export const ROLES = ["Administrator", "Manager", "Sales", "Finance", "ReadOnly"] as const;

export interface NewUser {
  username: string;
  display_name: string;
  password: string;
  roles: string[];
}

export interface UserUpdate {
  display_name: string;
  roles: string[];
  is_active: boolean;
}

export interface PasswordChange {
  new_password: string;
}

export interface ChangeOwnPassword {
  current_password: string;
  new_password: string;
}

export interface BackupManifest {
  format_version: number;
  schema_version: number;
  workspace_name: string;
  created_at: string;
  app_version: string;
}

export interface BackupPackage {
  file_name: string;
  package_base64: string;
  manifest: BackupManifest;
}

export const COMPANY_STATUSES = ["Prospect", "Active Customer", "Inactive", "Archived"] as const;
export type CompanyStatus = (typeof COMPANY_STATUSES)[number];

export interface Company {
  id: string;
  workspace_id: string;
  customer_number: string;
  name: string;
  status: string;
  owner_user_id: string | null;
  tax_number: string | null;
  billing_address: string | null;
  shipping_address: string | null;
  tags: string | null;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface CompanyInput {
  name: string;
  status: string;
  owner_user_id: string | null;
  tax_number: string | null;
  billing_address: string | null;
  shipping_address: string | null;
  tags: string | null;
  notes: string | null;
}

export const CONTACT_STATUSES = ["Active", "Inactive", "Archived"] as const;

export interface Contact {
  id: string;
  workspace_id: string;
  contact_number: string;
  company_id: string;
  first_name: string;
  last_name: string;
  job_title: string | null;
  email: string | null;
  phone: string | null;
  mobile: string | null;
  is_primary: boolean;
  status: string;
  tags: string | null;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface ContactInput {
  company_id: string;
  first_name: string;
  last_name: string;
  job_title: string | null;
  email: string | null;
  phone: string | null;
  mobile: string | null;
  is_primary: boolean;
  status: string;
  tags: string | null;
  notes: string | null;
}

export const PRODUCT_TYPES = ["Product", "Service"] as const;

export interface Product {
  id: string;
  workspace_id: string;
  product_number: string;
  sku: string | null;
  type: string;
  name: string;
  category: string | null;
  description: string | null;
  unit_price_cents: number;
  cost_cents: number;
  tax_rate_bp: number;
  default_quantity_milli: number;
  is_active: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface ProductInput {
  sku: string | null;
  type: string;
  name: string;
  category: string | null;
  description: string | null;
  unit_price_cents: number;
  cost_cents: number;
  tax_rate_bp: number;
  default_quantity_milli: number;
  is_active: boolean;
}

export const OPPORTUNITY_STAGES = [
  "New",
  "Qualified",
  "Discovery",
  "Proposal",
  "Negotiation",
  "Won",
  "Lost",
] as const;
export const OPPORTUNITY_STATUSES = ["Open", "Won", "Lost", "Archived"] as const;

export interface Opportunity {
  id: string;
  workspace_id: string;
  opportunity_number: string;
  company_id: string;
  primary_contact_id: string | null;
  name: string;
  stage: string;
  status: string;
  value_cents: number;
  currency_code: string;
  probability_bp: number;
  expected_close_date: string | null;
  owner_user_id: string | null;
  lost_reason: string | null;
  next_step: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface OpportunityInput {
  company_id: string;
  primary_contact_id: string | null;
  name: string;
  stage: string;
  status: string;
  value_cents: number;
  currency_code: string;
  probability_bp: number;
  expected_close_date: string | null;
  owner_user_id: string | null;
  lost_reason: string | null;
  next_step: string | null;
}

export interface OpportunityProduct {
  id: string;
  opportunity_id: string;
  product_id: string;
  quantity_milli: number;
  unit_price_cents: number;
}

export interface OpportunityProductInput {
  product_id: string;
  quantity_milli: number;
  unit_price_cents: number;
}

export const QUOTE_STATUSES = [
  "Draft",
  "Sent",
  "Viewed",
  "Accepted",
  "Rejected",
  "Expired",
  "Cancelled",
] as const;

export interface DocumentLine {
  id: string;
  product_id: string | null;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  discount_bp: number;
  tax_rate_bp: number;
  line_total_cents: number;
  sort_order: number;
}

export interface QuoteLine extends DocumentLine {
  quote_id: string;
}

export interface Quote {
  id: string;
  workspace_id: string;
  quote_number: string;
  company_id: string;
  contact_id: string | null;
  opportunity_id: string | null;
  status: string;
  currency_code: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  total_cents: number;
  issue_date: string | null;
  expiry_date: string | null;
  notes: string | null;
  terms: string | null;
  version: number;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface QuoteLineInput {
  product_id: string | null;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  discount_bp: number;
  tax_rate_bp: number;
}

export interface QuoteInput {
  company_id: string;
  contact_id: string | null;
  opportunity_id: string | null;
  currency_code: string;
  issue_date: string | null;
  expiry_date: string | null;
  notes: string | null;
  terms: string | null;
  lines: QuoteLineInput[];
}

export interface QuoteWithLines {
  quote: Quote;
  lines: QuoteLine[];
}

export const ORDER_STATUSES = [
  "Draft",
  "Confirmed",
  "Processing",
  "Partially Fulfilled",
  "Fulfilled",
  "Cancelled",
] as const;

export interface OrderLine extends DocumentLine {
  order_id: string;
}

export interface Order {
  id: string;
  workspace_id: string;
  order_number: string;
  company_id: string;
  contact_id: string | null;
  source_quote_id: string | null;
  status: string;
  currency_code: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  total_cents: number;
  order_date: string | null;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface OrderLineInput {
  product_id: string | null;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  discount_bp: number;
  tax_rate_bp: number;
}

export interface OrderInput {
  company_id: string;
  contact_id: string | null;
  currency_code: string;
  order_date: string | null;
  notes: string | null;
  lines: OrderLineInput[];
}

export interface OrderWithLines {
  order: Order;
  lines: OrderLine[];
}

export const INVOICE_STATUSES = [
  "Draft",
  "Issued",
  "Partially Paid",
  "Paid",
  "Overdue",
  "Void",
  "Cancelled",
] as const;

export interface InvoiceLine extends DocumentLine {
  invoice_id: string;
}

export interface Invoice {
  id: string;
  workspace_id: string;
  invoice_number: string;
  company_id: string;
  contact_id: string | null;
  source_order_id: string | null;
  status: string;
  currency_code: string;
  subtotal_cents: number;
  discount_cents: number;
  tax_cents: number;
  total_cents: number;
  amount_paid_cents: number;
  balance_cents: number;
  issue_date: string | null;
  due_date: string | null;
  payment_terms: string | null;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

export interface InvoiceLineInput {
  product_id: string | null;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  discount_bp: number;
  tax_rate_bp: number;
}

export interface InvoiceInput {
  company_id: string;
  contact_id: string | null;
  currency_code: string;
  issue_date: string | null;
  due_date: string | null;
  payment_terms: string | null;
  notes: string | null;
  lines: InvoiceLineInput[];
}

export interface Payment {
  id: string;
  invoice_id: string;
  amount_cents: number;
  paid_at: string;
  method: string | null;
  reference: string | null;
  created_at: string;
  created_by: string | null;
}

export interface PaymentInput {
  amount_cents: number;
  paid_at: string;
  method: string | null;
  reference: string | null;
}

export interface InvoiceWithLines {
  invoice: Invoice;
  lines: InvoiceLine[];
  payments: Payment[];
}

export interface StageCount {
  stage: string;
  count: number;
  value_cents: number;
}

export interface RecentActivity {
  occurred_at: string;
  event_type: string;
  summary: string;
}

export interface DashboardSummary {
  open_pipeline_value_cents: number;
  open_pipeline_count: number;
  won_revenue_cents: number;
  outstanding_invoices_cents: number;
  overdue_invoices_cents: number;
  overdue_invoices_count: number;
  quotes_awaiting_response: number;
  contracts_renewing_30_days: number;
  contracts_renewing_60_days: number;
  contracts_renewing_90_days: number;
  open_tasks: number;
  overdue_tasks: number;
  pipeline_by_stage: StageCount[];
  recent_activity: RecentActivity[];
}

export const CONTRACT_STATUSES = [
  "Draft",
  "Under Review",
  "Active",
  "Expiring",
  "Renewed",
  "Expired",
  "Terminated",
] as const;

export interface Contract {
  id: string;
  workspace_id: string;
  contract_number: string;
  company_id: string;
  contact_id: string | null;
  source_quote_id: string | null;
  title: string;
  type: string | null;
  value_cents: number;
  currency_code: string;
  owner_user_id: string | null;
  start_date: string | null;
  end_date: string | null;
  renewal_date: string | null;
  notice_period_days: number | null;
  status: string;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

// Deliberately has no opportunity_id field - a contract must never
// reference an opportunity (FR-CTR-03 / BR-009).
export interface ContractInput {
  company_id: string;
  contact_id: string | null;
  source_quote_id: string | null;
  title: string;
  type: string | null;
  value_cents: number;
  currency_code: string;
  owner_user_id: string | null;
  start_date: string | null;
  end_date: string | null;
  renewal_date: string | null;
  notice_period_days: number | null;
  status: string;
  notes: string | null;
}

export const TASK_PRIORITIES = ["Low", "Normal", "High", "Urgent"] as const;
export const TASK_STATUSES = ["Not Started", "In Progress", "Waiting", "Completed", "Cancelled"] as const;
export const TASK_RELATED_TYPES = [
  "Company",
  "Contact",
  "Opportunity",
  "Quote",
  "Order",
  "Invoice",
  "Contract",
] as const;
export type TaskRelatedType = (typeof TASK_RELATED_TYPES)[number];

export interface Task {
  id: string;
  workspace_id: string;
  task_number: string;
  title: string;
  description: string | null;
  owner_user_id: string | null;
  priority: string;
  status: string;
  due_date: string | null;
  reminder_at: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
  related_type: string | null;
  related_id: string | null;
}

export interface TaskInput {
  title: string;
  description: string | null;
  owner_user_id: string | null;
  priority: string;
  status: string;
  due_date: string | null;
  reminder_at: string | null;
  related_type: string | null;
  related_id: string | null;
}

export interface AppErrorPayload {
  kind: "database" | "not_found" | "validation" | "conflict";
  message: string;
}

// FR-RPT: reports beyond the dashboard's KPI tiles. `from`/`to` are ISO
// dates ("YYYY-MM-DD"); omit either (or both) for an all-time range.
export interface ReportRange {
  from: string | null;
  to: string | null;
}

export interface RevenueByMonth {
  month: string;
  invoice_count: number;
  total_cents: number;
}

export interface WinRateByOwner {
  owner_user_id: string | null;
  owner_name: string;
  won_count: number;
  lost_count: number;
  won_value_cents: number;
}

export interface LostReasonBreakdown {
  reason: string;
  count: number;
  value_cents: number;
}

export interface ArAgingBucket {
  bucket: string;
  invoice_count: number;
  balance_cents: number;
}

export interface SalesByOwner {
  owner_user_id: string | null;
  owner_name: string;
  invoice_count: number;
  total_cents: number;
}

// FR-CFG: custom fields on every major entity, defined by an
// Administrator via an attribute side-table rather than a schema change.
export const CUSTOM_FIELD_TYPES = ["text", "number", "date", "boolean", "select"] as const;
export type CustomFieldType = (typeof CUSTOM_FIELD_TYPES)[number];
export const CUSTOM_FIELD_ENTITY_TYPES = [
  "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task", "Product",
] as const;
export type CustomFieldEntityType = (typeof CUSTOM_FIELD_ENTITY_TYPES)[number];

/** Plural label for an entity type, used in admin screen tabs/headings. */
export function entityTypeLabel(entityType: string): string {
  const labels: Record<string, string> = {
    Company: "Companies",
    Contact: "Contacts",
    Opportunity: "Opportunities",
    Quote: "Quotes",
    Order: "Orders",
    Invoice: "Invoices",
    Contract: "Contracts",
    Task: "Tasks",
    Product: "Products",
  };
  return labels[entityType] ?? `${entityType}s`;
}

/** The one built-in, enum-like field each entity exposes as a rule
 * trigger - mirrors field_rule::builtin_trigger_field_for in the Rust
 * core. "status" for every entity except Product, which only has
 * `is_active` (compared as the strings "true"/"false"). */
export function builtinTriggerFieldFor(entityType: string): string {
  return entityType === "Product" ? "is_active" : "status";
}

export interface CustomFieldDefinition {
  id: string;
  workspace_id: string;
  entity_type: string;
  key: string;
  label: string;
  field_type: string;
  options: string[];
  required: boolean;
  show_in_list: boolean;
  sort_order: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CustomFieldDefinitionInput {
  entity_type: CustomFieldEntityType;
  label: string;
  field_type: CustomFieldType;
  options: string[];
  required: boolean;
  show_in_list: boolean;
  sort_order: number;
}

export interface CustomFieldDefinitionUpdate {
  label: string;
  options: string[];
  required: boolean;
  show_in_list: boolean;
  sort_order: number;
  is_active: boolean;
}

/** Values keyed by field key, e.g. { industry: "Retail" }. */
export type CustomFieldValues = Record<string, string>;

// FR-RUL: admin-defined conditional rules over custom fields, e.g.
// "require Lead Source when Status = Prospect". Scoped to custom fields
// as the target (and, for the trigger, either the entity's one built-in
// field from `builtinTriggerFieldFor` or another custom field) - see
// field_rule_service's doc comment for why built-in fields in general are
// out of scope.
export const RULE_OPERATORS = ["equals", "not_equals"] as const;
export type RuleOperator = (typeof RULE_OPERATORS)[number];
export const RULE_EFFECTS = ["require", "hide"] as const;
export type RuleEffect = (typeof RULE_EFFECTS)[number];
export const TRIGGER_SOURCES = ["builtin", "custom"] as const;
export type TriggerSource = (typeof TRIGGER_SOURCES)[number];

/** The valid values for an entity's built-in trigger field
 * (`builtinTriggerFieldFor`) - its status/stage enum, or ["true","false"]
 * for Product's is_active. Used to populate the value dropdown when
 * building a business rule or workflow rule against that field. */
export function statusesForEntity(entityType: string): readonly string[] {
  switch (entityType) {
    case "Company": return COMPANY_STATUSES;
    case "Contact": return CONTACT_STATUSES;
    case "Opportunity": return OPPORTUNITY_STATUSES;
    case "Quote": return QUOTE_STATUSES;
    case "Order": return ORDER_STATUSES;
    case "Invoice": return INVOICE_STATUSES;
    case "Contract": return CONTRACT_STATUSES;
    case "Task": return TASK_STATUSES;
    case "Product": return ["true", "false"];
    default: return [];
  }
}

/** Same as statusesForEntity, but for the field a *workflow* rule
 * transitions on (`transitionFieldFor`) - only Opportunity differs, using
 * its stage list instead of its status list. */
export function transitionValuesForEntity(entityType: string): readonly string[] {
  return entityType === "Opportunity" ? OPPORTUNITY_STAGES : statusesForEntity(entityType);
}

export interface FieldRule {
  id: string;
  workspace_id: string;
  entity_type: string;
  trigger_field_source: string;
  trigger_field_key: string;
  operator: string;
  trigger_value: string;
  target_field_key: string;
  effect: string;
  is_active: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface FieldRuleInput {
  entity_type: CustomFieldEntityType;
  trigger_field_source: TriggerSource;
  trigger_field_key: string;
  operator: RuleOperator;
  trigger_value: string;
  target_field_key: string;
  effect: RuleEffect;
  sort_order: number;
}

export interface FieldRuleUpdate {
  trigger_field_source: TriggerSource;
  trigger_field_key: string;
  operator: RuleOperator;
  trigger_value: string;
  target_field_key: string;
  effect: RuleEffect;
  sort_order: number;
  is_active: boolean;
}

// FR-WFL: admin-defined workflow automation - "when an Opportunity's stage
// (or another entity's status) transitions to X, create a follow-up Task
// automatically." Unlike FR-RUL, every matching active rule fires and
// creates its own task - there's no "highest wins" conflict to resolve,
// since the effect is additive (creating a task), not a value. Every
// entity with a status-like field except Product, which only has a
// boolean is_active and no meaningful "transition" to automate on.
export const WORKFLOW_ENTITY_TYPES = [
  "Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task",
] as const;
export type WorkflowEntityType = (typeof WORKFLOW_ENTITY_TYPES)[number];

/** "stage" for Opportunity (the field that actually flows through the
 * sales pipeline), "status" for everything else - mirrors
 * workflow_rule::transition_field_for in the Rust core. */
export function transitionFieldFor(entityType: string): string {
  return entityType === "Opportunity" ? "stage" : "status";
}

export interface WorkflowRule {
  id: string;
  workspace_id: string;
  entity_type: string;
  trigger_status: string;
  task_title: string;
  task_description: string | null;
  due_in_days: number;
  assignee_user_id: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface WorkflowRuleInput {
  entity_type: WorkflowEntityType;
  trigger_status: string;
  task_title: string;
  task_description: string | null;
  due_in_days: number;
  assignee_user_id: string | null;
}

export interface WorkflowRuleUpdate {
  trigger_status: string;
  task_title: string;
  task_description: string | null;
  due_in_days: number;
  assignee_user_id: string | null;
  is_active: boolean;
}

// Admin flexibility: configurable ID/numbering format per entity type.
// The letters in an example like "ACC-ab0001" are just part of the
// chosen prefix text - there is no separate alpha-segment syntax.
export const NUMBERING_ENTITY_TYPES = [
  "Company", "Contact", "Opportunity", "Product", "Quote", "Order", "Invoice", "Contract", "Task",
] as const;
export type NumberingEntityType = (typeof NUMBERING_ENTITY_TYPES)[number];

export interface EffectiveNumbering {
  entity_type: string;
  prefix: string;
  digits: number;
  example: string;
  is_custom: boolean;
}

export interface NumberingOverrideInput {
  entity_type: NumberingEntityType;
  prefix: string;
  digits: number;
}

// Admin flexibility: a simple report builder - pick an entity, a
// group-by field (built-in status/stage, or an active custom field), and
// an aggregate (count, or sum of a numeric custom field).
export const REPORT_AGGREGATES = ["count", "sum"] as const;
export type ReportAggregate = (typeof REPORT_AGGREGATES)[number];
export const REPORT_GROUP_BY_SOURCES = ["builtin", "custom"] as const;
export type ReportGroupBySource = (typeof REPORT_GROUP_BY_SOURCES)[number];

export interface CustomReport {
  id: string;
  workspace_id: string;
  name: string;
  entity_type: string;
  group_by_source: string;
  group_by_field: string;
  aggregate: string;
  sum_field_key: string | null;
  created_at: string;
  updated_at: string;
}

export interface CustomReportInput {
  name: string;
  entity_type: CustomFieldEntityType;
  group_by_source: ReportGroupBySource;
  group_by_field: string;
  aggregate: ReportAggregate;
  sum_field_key: string | null;
}

export type CustomReportUpdate = CustomReportInput;

export interface CustomReportRow {
  group: string;
  value: number;
}
