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

export const PREFERRED_CONTACT_METHODS = ["Email", "Phone", "Text"] as const;

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
  phone: string | null;
  email: string | null;
  website: string | null;
  annual_revenue_cents: number | null;
  employee_count: number | null;
  preferred_contact_method: string | null;
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
  phone: string | null;
  email: string | null;
  website: string | null;
  annual_revenue_cents: number | null;
  employee_count: number | null;
  preferred_contact_method: string | null;
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
  department: string | null;
  preferred_contact_method: string | null;
  linkedin_url: string | null;
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
  department: string | null;
  preferred_contact_method: string | null;
  linkedin_url: string | null;
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

// Global search: entity_type is either a core entity name (Company,
// Contact, Opportunity, Quote, Order, Invoice, Contract, Task, Product) or
// a custom object's key - see search_service::global_search. subtitle is
// set when the match came from something other than the title itself (an
// email/phone, or a matched searchable custom field's "label: value").
export interface SearchResult {
  entity_type: string;
  entity_id: string;
  title: string;
  subtitle: string | null;
}

// A single logged create/update/archive, keyed to the record it happened
// to - see audit_repo::record, called from every built-in entity's service
// plus custom_record_service. user_id is null for a write with no
// authenticated actor (a system/scheduled job); resolve it against
// api.listUsers() the same way created_by/updated_by are resolved.
export interface AuditEvent {
  id: string;
  workspace_id: string;
  occurred_at: string;
  user_id: string | null;
  event_type: string;
  entity_type: string | null;
  entity_id: string | null;
  summary: string;
  details_json: string | null;
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

/** Mirrors `domain::builtin_fields::BuiltinField` in the Rust core - see
 * that module's doc comment for exactly what's excluded (foreign keys,
 * generated/computed columns, owner_user_id) and why. `field_type` follows
 * the same vocabulary custom fields use, plus "money" (a dollar amount
 * backed by an integer-cents column) and "percent" (backed by basis
 * points) for the couple of built-in fields that need one. */
export interface BuiltinFieldDef {
  key: string;
  label: string;
  field_type: "text" | "number" | "money" | "percent" | "date" | "boolean" | "select";
  options?: readonly string[];
  /** Whether require/hide/lock/set_default/set_value/update_field may
   * target this field, not just read it in a condition/trigger - false
   * for each entity's status-equivalent field (it has its own dedicated
   * mechanism instead) and every Quote/Order/Invoice field (those
   * documents have no general-purpose edit path once created - see the
   * Rust registry's comment). */
  actionable: boolean;
}

const CUSTOM_RECORD_BUILTIN_FIELDS: BuiltinFieldDef[] = [
  { key: "primary_name", label: "Name", field_type: "text", actionable: true },
  // Same fixed set as CUSTOM_RECORD_STATUSES below - inlined rather than
  // referenced since that const is declared later in this file and a
  // module-level const can't reference one not yet initialized.
  { key: "status", label: "Status", field_type: "select", options: ["Active", "Inactive", "Archived"], actionable: false },
  { key: "notes", label: "Notes", field_type: "text", actionable: true },
];

const BUILTIN_FIELDS_BY_ENTITY: Record<string, BuiltinFieldDef[]> = {
  Company: [
    { key: "name", label: "Company name", field_type: "text", actionable: true },
    { key: "status", label: "Status", field_type: "select", options: COMPANY_STATUSES, actionable: false },
    { key: "tax_number", label: "Tax number", field_type: "text", actionable: true },
    { key: "billing_address", label: "Billing address", field_type: "text", actionable: true },
    { key: "shipping_address", label: "Shipping address", field_type: "text", actionable: true },
    { key: "tags", label: "Tags", field_type: "text", actionable: true },
    { key: "notes", label: "Notes", field_type: "text", actionable: true },
  ],
  Contact: [
    { key: "first_name", label: "First name", field_type: "text", actionable: true },
    { key: "last_name", label: "Last name", field_type: "text", actionable: true },
    { key: "job_title", label: "Job title", field_type: "text", actionable: true },
    { key: "email", label: "Email", field_type: "text", actionable: true },
    { key: "phone", label: "Phone", field_type: "text", actionable: true },
    { key: "mobile", label: "Mobile", field_type: "text", actionable: true },
    { key: "is_primary", label: "Primary contact", field_type: "boolean", actionable: true },
    { key: "status", label: "Status", field_type: "select", options: CONTACT_STATUSES, actionable: false },
    { key: "tags", label: "Tags", field_type: "text", actionable: true },
    { key: "notes", label: "Notes", field_type: "text", actionable: true },
  ],
  Opportunity: [
    { key: "name", label: "Opportunity name", field_type: "text", actionable: true },
    { key: "stage", label: "Stage", field_type: "select", options: OPPORTUNITY_STAGES, actionable: false },
    { key: "status", label: "Status", field_type: "select", options: OPPORTUNITY_STATUSES, actionable: false },
    { key: "value", label: "Value", field_type: "money", actionable: true },
    { key: "probability", label: "Probability", field_type: "percent", actionable: true },
    { key: "expected_close_date", label: "Expected close date", field_type: "date", actionable: true },
    { key: "lost_reason", label: "Lost reason", field_type: "text", actionable: true },
    { key: "next_step", label: "Next step", field_type: "text", actionable: true },
  ],
  Product: [
    { key: "name", label: "Name", field_type: "text", actionable: true },
    { key: "sku", label: "SKU", field_type: "text", actionable: true },
    { key: "type", label: "Type", field_type: "select", options: PRODUCT_TYPES, actionable: true },
    { key: "category", label: "Category", field_type: "text", actionable: true },
    { key: "description", label: "Description", field_type: "text", actionable: true },
    { key: "unit_price", label: "Unit price", field_type: "money", actionable: true },
    { key: "cost", label: "Cost", field_type: "money", actionable: true },
    { key: "tax_rate", label: "Tax rate", field_type: "percent", actionable: true },
    { key: "is_active", label: "Active", field_type: "boolean", actionable: false },
  ],
  // Quote/Order/Invoice: conditionable only - these documents have no
  // general-purpose edit path once created (only status transitions and
  // conversion), so there's no safe write path for a generic action to
  // route through. See domain::builtin_fields' comment in the Rust core.
  Quote: [
    { key: "status", label: "Status", field_type: "select", options: QUOTE_STATUSES, actionable: false },
    { key: "issue_date", label: "Issue date", field_type: "date", actionable: false },
    { key: "expiry_date", label: "Valid until", field_type: "date", actionable: false },
    { key: "terms", label: "Terms", field_type: "text", actionable: false },
    { key: "notes", label: "Notes", field_type: "text", actionable: false },
  ],
  Order: [
    { key: "status", label: "Status", field_type: "select", options: ORDER_STATUSES, actionable: false },
    { key: "order_date", label: "Order date", field_type: "date", actionable: false },
    { key: "notes", label: "Notes", field_type: "text", actionable: false },
  ],
  Invoice: [
    { key: "status", label: "Status", field_type: "select", options: INVOICE_STATUSES, actionable: false },
    { key: "issue_date", label: "Issue date", field_type: "date", actionable: false },
    { key: "due_date", label: "Due date", field_type: "date", actionable: false },
    { key: "payment_terms", label: "Payment terms", field_type: "text", actionable: false },
    { key: "notes", label: "Notes", field_type: "text", actionable: false },
  ],
  Contract: [
    { key: "title", label: "Title", field_type: "text", actionable: true },
    { key: "type", label: "Type", field_type: "text", actionable: true },
    { key: "value", label: "Value", field_type: "money", actionable: true },
    { key: "start_date", label: "Start date", field_type: "date", actionable: true },
    { key: "end_date", label: "End date", field_type: "date", actionable: true },
    { key: "renewal_date", label: "Renewal date", field_type: "date", actionable: true },
    { key: "notice_period_days", label: "Notice period (days)", field_type: "number", actionable: true },
    { key: "status", label: "Status", field_type: "select", options: CONTRACT_STATUSES, actionable: false },
    { key: "notes", label: "Notes", field_type: "text", actionable: true },
  ],
  Task: [
    { key: "title", label: "Title", field_type: "text", actionable: true },
    { key: "description", label: "Description", field_type: "text", actionable: true },
    { key: "priority", label: "Priority", field_type: "select", options: TASK_PRIORITIES, actionable: true },
    { key: "status", label: "Status", field_type: "select", options: TASK_STATUSES, actionable: false },
    { key: "due_date", label: "Due date", field_type: "date", actionable: true },
    { key: "reminder_at", label: "Reminder", field_type: "date", actionable: true },
  ],
};

/** Every built-in field `entityType` exposes as a condition/trigger/action
 * target - the 9 core entities from the table above, or (for any active
 * custom object key) the fixed shape every custom record shares. */
export function builtinFieldsFor(entityType: string): BuiltinFieldDef[] {
  return BUILTIN_FIELDS_BY_ENTITY[entityType] ?? CUSTOM_RECORD_BUILTIN_FIELDS;
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
  // ADM-CF-04/05 (Phase E): optional validation (number min/max, text
  // max length/regex) and searchable/filterable/reportable capability
  // flags - see the migration's header comment for what's actually wired
  // up (is_reportable) versus forward-looking metadata.
  min_value: string | null;
  max_value: string | null;
  max_length: number | null;
  regex_pattern: string | null;
  is_searchable: boolean;
  is_filterable: boolean;
  is_reportable: boolean;
  // Addendum Phase 4 (spec §4): default_value/is_unique are enforced
  // server-side by custom_field_service::set_entity_values;
  // help_text/placeholder are presentation-only, rendered by the form.
  default_value: string | null;
  is_unique: boolean;
  help_text: string | null;
  placeholder: string | null;
  // Second addendum round (migration 0020): omitted from every create/edit
  // form unless a business rule's "show" action currently targets it -
  // enforced server-side in custom_field_service::set_entity_values, so a
  // hidden-by-default+required field can never block a save on its own.
  is_hidden_by_default: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface CustomFieldDefinitionInput {
  // A built-in entity type or an active custom object's key (see
  // CustomObjectDefinition) - custom objects widen this beyond the fixed
  // CustomFieldEntityType union, so it's plain `string` here.
  entity_type: string;
  label: string;
  field_type: CustomFieldType;
  options: string[];
  required: boolean;
  show_in_list: boolean;
  sort_order: number;
  min_value: string | null;
  max_value: string | null;
  max_length: number | null;
  regex_pattern: string | null;
  is_searchable: boolean;
  is_filterable: boolean;
  is_reportable: boolean;
  default_value: string | null;
  is_unique: boolean;
  help_text: string | null;
  placeholder: string | null;
  is_hidden_by_default: boolean;
}

export interface CustomFieldDefinitionUpdate {
  label: string;
  options: string[];
  required: boolean;
  show_in_list: boolean;
  sort_order: number;
  is_active: boolean;
  min_value: string | null;
  max_value: string | null;
  max_length: number | null;
  regex_pattern: string | null;
  is_searchable: boolean;
  is_filterable: boolean;
  is_reportable: boolean;
  default_value: string | null;
  is_unique: boolean;
  help_text: string | null;
  placeholder: string | null;
  is_hidden_by_default: boolean;
}

/** Values keyed by field key, e.g. { industry: "Retail" }. */
export type CustomFieldValues = Record<string, string>;

/** Return shape of `set_custom_field_values` - non-blocking notices from
 * any business rule that matched the save (show_error/show_warning, plus
 * legacy show_message folded into warnings). Save already succeeded by
 * the time these are returned; see custom_field_service::set_entity_values. */
export interface SaveNotices {
  errors: string[];
  warnings: string[];
}

// Admin extensibility Phase C (spec §22/ADM-BR): a richer IF (AND/OR) /
// THEN business rule engine - any number of conditions per rule (matched
// as AND or OR), and actions beyond require/hide: lock (read-only), set a
// default or forced value, block the whole save with a custom message, or
// show a non-blocking message. A condition/action's field can be any
// active custom field, or any field in `builtinFieldsFor(entityType)`
// (`field_source`/`target_field_source: "builtin"`) - see that function's
// doc comment for which built-in fields are excluded and why. Only
// `actionable` built-in fields can be an action's target; every built-in
// field can be a condition's.
export const MATCH_TYPES = ["all", "any"] as const;
export type MatchType = (typeof MATCH_TYPES)[number];
// Admin Automation & Customization addendum, Phase 1 (spec §2.2): four new
// operators (starts_with/ends_with/in_list/not_in_list), matching
// core::domain::conditions::CONDITION_OPERATORS exactly.
export const CONDITION_OPERATORS = [
  "equals", "not_equals", "contains", "not_contains", "starts_with", "ends_with",
  "in_list", "not_in_list", "is_empty", "is_not_empty",
  "greater_than", "less_than", "on_or_after", "on_or_before",
] as const;
export type ConditionOperator = (typeof CONDITION_OPERATORS)[number];
/** Operators that don't compare against a value at all - matches
 * core::domain::conditions::VALUELESS_OPERATORS. */
export const VALUELESS_OPERATORS: ConditionOperator[] = ["is_empty", "is_not_empty"];
/** `in_list`/`not_in_list` split on this - same convention select-field
 * options already use ("Option A|Option B|Option C"). */
export const LIST_SEPARATOR = "|";
export const TRIGGER_SOURCES = ["builtin", "custom"] as const;
export type TriggerSource = (typeof TRIGGER_SOURCES)[number];
// Second addendum round: the full action palette, mirroring
// core::models::business_rule::ACTION_TYPES exactly. `show`/`editable` are
// the explicit counterparts to `hide`/`lock` - most useful on a field
// flagged is_hidden_by_default, which otherwise never renders, or to
// override a lower-priority rule's hide/lock ("last matching rule wins"
// per target field). `clear_value` is `set_value` with an empty value
// written unconditionally. `restrict_choices` only makes sense on a
// select-typed field. `show_error`/`show_warning` replace the old
// `show_message` for new rules (severity split); `show_message` is kept
// only so a rule saved before this round keeps evaluating exactly as it
// always did - CURRENT_ACTION_TYPES (below) is what the builder offers.
export const ACTION_TYPES = [
  "require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value",
  "restrict_choices", "block_save", "show_error", "show_warning", "show_message",
] as const;
export type ActionType = (typeof ACTION_TYPES)[number];
/** Actions the rule builder offers when creating or editing a rule -
 * `show_message` is legacy-only (see ACTION_TYPES doc comment). */
export const CURRENT_ACTION_TYPES: ActionType[] = [
  "require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value",
  "restrict_choices", "block_save", "show_error", "show_warning",
];
export const FIELD_TARGETED_ACTIONS: ActionType[] = ["require", "hide", "show", "lock", "editable", "set_default", "set_value", "clear_value", "restrict_choices"];
export const MESSAGE_ACTIONS: ActionType[] = ["block_save", "show_error", "show_warning", "show_message"];
/** `set_default`/`set_value`/`restrict_choices` need a value; `clear_value`
 * deliberately doesn't (the whole point is writing empty). */
export const VALUE_REQUIRED_ACTIONS: ActionType[] = ["set_default", "set_value", "restrict_choices"];

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
    // Anything else reaching this point is a custom object - they all
    // share the same fixed status set (CUSTOM_RECORD_STATUSES).
    default: return CUSTOM_RECORD_STATUSES;
  }
}

/** Same as statusesForEntity, but for the field a *workflow* rule
 * transitions on (`transitionFieldFor`) - only Opportunity differs, using
 * its stage list instead of its status list. */
export function transitionValuesForEntity(entityType: string): readonly string[] {
  return entityType === "Opportunity" ? OPPORTUNITY_STAGES : statusesForEntity(entityType);
}

export interface BusinessRuleCondition {
  id: string;
  field_source: TriggerSource;
  field_key: string;
  operator: ConditionOperator;
  value: string;
  /** Addendum §2.2 field-to-field comparison: when both are set, the
   * condition compares against this field's live value instead of the
   * literal `value` above. */
  compare_field_source: TriggerSource | null;
  compare_field_key: string | null;
  /** See migration 0020 / core::domain::conditions::conditions_match -
   * `null` for an ungrouped, top-level condition; a shared string for
   * conditions OR'd together into one sub-unit before that unit
   * participates in the rule's top-level match_type. */
  group_id: string | null;
  sort_order: number;
}

export interface BusinessRuleConditionInput {
  field_source: TriggerSource;
  field_key: string;
  operator: ConditionOperator;
  value: string;
  compare_field_source: TriggerSource | null;
  compare_field_key: string | null;
  group_id: string | null;
}

export interface BusinessRuleAction {
  id: string;
  action_type: ActionType;
  target_field_key: string | null;
  target_field_source: TriggerSource;
  action_value: string | null;
  message: string | null;
  sort_order: number;
}

export interface BusinessRuleActionInput {
  action_type: ActionType;
  target_field_key: string | null;
  target_field_source: TriggerSource;
  action_value: string | null;
  message: string | null;
}

export interface BusinessRule {
  id: string;
  workspace_id: string;
  entity_type: string;
  name: string;
  description: string | null;
  match_type: MatchType;
  priority: number;
  is_active: boolean;
  effective_start_date: string | null;
  effective_end_date: string | null;
  is_protected: boolean;
  /** Per-app scoped automation: which App Builder app this rule belongs
   * to, or null for workspace-wide - see migration 0028's own doc comment
   * for the full rationale. Purely a "which app's Admin screen shows this
   * by default" tag; evaluation is unaffected either way. */
  app_id: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  conditions: BusinessRuleCondition[];
  actions: BusinessRuleAction[];
}

export interface BusinessRuleInput {
  entity_type: string;
  name: string;
  description: string | null;
  match_type: MatchType;
  priority: number;
  effective_start_date: string | null;
  effective_end_date: string | null;
  app_id: string | null;
  conditions: BusinessRuleConditionInput[];
  actions: BusinessRuleActionInput[];
}

export interface BusinessRuleUpdate {
  name: string;
  description: string | null;
  match_type: MatchType;
  priority: number;
  is_active: boolean;
  effective_start_date: string | null;
  effective_end_date: string | null;
  app_id: string | null;
  conditions: BusinessRuleConditionInput[];
  actions: BusinessRuleActionInput[];
}

/** Admin UX polish (spec §10): a read-only snapshot of a rule as it stood
 * right before an edit overwrote it - see business_rule_service::
 * BusinessRuleVersion's doc comment. `snapshot` is a full BusinessRule, so
 * restoring one is just re-submitting its shape as a normal update. */
export interface BusinessRuleVersion {
  id: string;
  business_rule_id: string;
  snapshot: BusinessRule;
  saved_at: string;
}

export interface RuleEvaluation {
  field_effects: Record<string, string>;
  set_values: Record<string, string>;
  /** Same shape as field_effects/set_values, but for built-in-field
   * targets - see business_rule_service's RuleEvaluation doc comment for
   * why they're kept separate (a custom field can share a key with a
   * built-in one). */
  builtin_field_effects: Record<string, string>;
  builtin_set_values: Record<string, string>;
  /** Pipe-delimited (LIST_SEPARATOR) allowed-options subset from a
   * matching `restrict_choices` action, keyed by target field key -
   * custom and built-in targets share this one map (no separate
   * builtin_restricted_choices - restrict_choices only ever targets a
   * select-typed field, and custom/built-in select fields don't collide
   * the way plain field keys can). */
  restricted_choices: Record<string, string>;
  blocked: string | null;
  /** show_error messages - non-blocking but flagged more urgently than
   * warnings. */
  errors: string[];
  /** show_warning (and legacy show_message) texts. */
  warnings: string[];
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

// Admin extensibility Phase D (spec §23/ADM-WF): a richer Trigger ->
// Conditions -> Actions engine - more trigger types, AND/OR conditions
// (the same domain::conditions matcher business rules use), and actions
// beyond task creation.
export const TRIGGER_TYPES = [
  "record_created", "record_updated", "status_changed", "field_changed", "date_reached", "due_overdue", "scheduled",
] as const;
export type TriggerType = (typeof TRIGGER_TYPES)[number];
export const WORKFLOW_ACTION_TYPES = [
  "create_task", "update_field", "assign_owner", "create_record", "update_related_record", "add_notification", "create_reminder",
  // Second addendum round: the trigger-time subset of business rules'
  // field-effect vocabulary that fits a "fires once" model - update_field
  // already covers "set field value", these two round it out. The
  // continuous, live-form-governance effects (require/hide/show/lock/
  // editable/restrict_choices/block_save/show_error/show_warning) stay
  // business-rule-only.
  "set_default_field", "clear_field",
] as const;
export type WorkflowActionType = (typeof WORKFLOW_ACTION_TYPES)[number];
export const NOTIFICATION_AUDIENCES = ["owner", "all_admins"] as const;
export type NotificationAudience = (typeof NOTIFICATION_AUDIENCES)[number];

/** Built-in date fields date_reached/due_overdue can watch, per entity
 * type - mirrors workflow::date_fields_for in the Rust core. */
export function dateFieldsFor(entityType: string): string[] {
  switch (entityType) {
    case "Task": return ["due_date"];
    case "Quote": return ["expiry_date"];
    case "Contract": return ["end_date", "renewal_date"];
    case "Invoice": return ["due_date"];
    default: return [];
  }
}

export interface WorkflowCondition {
  id: string;
  field_source: TriggerSource;
  field_key: string;
  operator: ConditionOperator;
  value: string;
  compare_field_source: TriggerSource | null;
  compare_field_key: string | null;
  group_id: string | null;
  sort_order: number;
}
export interface WorkflowConditionInput {
  field_source: TriggerSource;
  field_key: string;
  operator: ConditionOperator;
  value: string;
  compare_field_source: TriggerSource | null;
  compare_field_key: string | null;
  group_id: string | null;
}
export interface WorkflowAction {
  id: string;
  action_type: WorkflowActionType;
  params_json: string;
  sort_order: number;
}
export interface WorkflowActionInput {
  action_type: WorkflowActionType;
  params_json: string;
}

export interface WorkflowDefinition {
  id: string;
  workspace_id: string;
  entity_type: string;
  name: string;
  description: string | null;
  trigger_type: TriggerType;
  trigger_status: string | null;
  trigger_field_key: string | null;
  /** Only meaningful for the `field_changed` trigger - see `builtinFieldsFor`
   * doc comment. `date_reached`/`due_overdue` always name a curated
   * built-in date field regardless of this value. */
  trigger_field_source: TriggerSource;
  trigger_offset_days: number;
  match_type: MatchType;
  priority: number;
  is_active: boolean;
  is_protected: boolean;
  last_scheduled_run_at: string | null;
  /** Per-app scoped automation - see `BusinessRule.app_id`'s doc comment. */
  app_id: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  conditions: WorkflowCondition[];
  actions: WorkflowAction[];
}

export interface WorkflowDefinitionInput {
  entity_type: string;
  name: string;
  description: string | null;
  trigger_type: TriggerType;
  trigger_status: string | null;
  trigger_field_key: string | null;
  trigger_field_source: TriggerSource;
  trigger_offset_days: number;
  match_type: MatchType;
  priority: number;
  app_id: string | null;
  conditions: WorkflowConditionInput[];
  actions: WorkflowActionInput[];
}

export interface WorkflowDefinitionUpdate {
  name: string;
  description: string | null;
  trigger_status: string | null;
  trigger_field_key: string | null;
  trigger_field_source: TriggerSource;
  trigger_offset_days: number;
  match_type: MatchType;
  priority: number;
  is_active: boolean;
  app_id: string | null;
  conditions: WorkflowConditionInput[];
  actions: WorkflowActionInput[];
}

/** Admin UX polish (spec §10) - see BusinessRuleVersion's doc comment for
 * the full rationale; identical mechanism for workflows. */
export interface WorkflowRuleVersion {
  id: string;
  workflow_id: string;
  snapshot: WorkflowDefinition;
  saved_at: string;
}

export interface WorkflowRun {
  id: string;
  workspace_id: string;
  workflow_id: string;
  entity_type: string;
  entity_id: string | null;
  trigger_type: string;
  triggered_at: string;
  outcome: "success" | "error" | "skipped";
  actions_summary: string | null;
  error_message: string | null;
}

/** Addendum Phase 3: dry-run result from `test_workflows` - mirrors
 * `RuleEvaluation` in spirit (a hypothetical-context evaluation that writes
 * nothing), but since workflow actions are side-effecting rather than
 * value-computing, each match just carries plain-language descriptions of
 * what its actions would have done. */
export interface WorkflowTestMatch {
  workflow_id: string;
  workflow_name: string;
  trigger_type: string;
  action_descriptions: string[];
}
export interface WorkflowTestResult {
  matches: WorkflowTestMatch[];
}

export interface Notification {
  id: string;
  workspace_id: string;
  recipient_user_id: string | null;
  message: string;
  entity_type: string | null;
  entity_id: string | null;
  created_at: string;
  read_at: string | null;
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
  // Only set when is_custom is true - a fallback to the built-in default
  // has no real record behind it to attribute.
  created_by: string | null;
  updated_by: string | null;
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
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface CustomReportInput {
  name: string;
  entity_type: string;
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

// Admin extensibility (spec §20.2): an admin-defined custom object - a
// whole new business object type, not just a field on an existing one.
// A custom object's `key` becomes an entity_type value everywhere custom
// fields, business rules and the custom report builder already accept a
// string entity_type - see CustomFieldsAdmin/FieldRulesAdmin, which list
// active custom objects as extra tabs alongside the nine built-in types.
export const CUSTOM_RECORD_STATUSES = ["Active", "Inactive", "Archived"] as const;
export type CustomRecordStatus = (typeof CUSTOM_RECORD_STATUSES)[number];

export interface CustomObjectDefinition {
  id: string;
  workspace_id: string;
  key: string;
  singular_label: string;
  plural_label: string;
  icon: string;
  prefix: string;
  digits: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CustomObjectDefinitionInput {
  singular_label: string;
  plural_label: string;
  icon: string;
  prefix: string;
  digits: number;
}

export interface CustomObjectDefinitionUpdate extends CustomObjectDefinitionInput {
  is_active: boolean;
}

export interface CustomRecord {
  id: string;
  workspace_id: string;
  object_key: string;
  display_number: string;
  primary_name: string;
  status: string;
  owner_user_id: string | null;
  notes: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
  archived_at: string | null;
}

// Screen/App Builder Phase 1: an object (built-in entity_type or a custom
// object's key) can have several named layouts, each with its own tabs of
// admin-drag-ordered field sections. Fields are opaque key strings here -
// this layer round-trips whatever the caller places, the same way the
// backend does (see the migration's own comment).

// Phase 2: a section lays its fields out in a `columns`-wide grid (1-3);
// each field spans one column or the section's full width. Mirrors
// core::models::screen_layout::SectionField - see its doc comment for
// the backward-compatible wire format (a layout saved before Phase 2
// still loads as plain field-key strings, each implicitly one column
// wide, with `columns` defaulting to 2).
export interface SectionField {
  key: string;
  full_width: boolean;
}

export interface LayoutSection {
  id: string;
  title: string;
  columns: number;
  fields: SectionField[];
}

export interface LayoutTab {
  id: string;
  title: string;
  sections: LayoutSection[];
  // Phase 3: RelationshipDefinition.key values whose related-records list
  // renders on this tab. A key placed on no tab isn't hidden - it falls
  // back to an always-visible spot outside the tab strip.
  related: string[];
}

export interface LayoutTabs {
  tabs: LayoutTab[];
}

export interface ScreenLayout {
  id: string;
  workspace_id: string;
  entity_type: string;
  name: string;
  is_default: boolean;
  roles: string[];
  draft: LayoutTabs;
  published: LayoutTabs | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface ScreenLayoutInput {
  entity_type: string;
  name: string;
  initial_fields: string[];
}

export interface ScreenLayoutUpdate {
  name: string;
  roles: string[];
  draft: LayoutTabs;
}

export interface EffectiveLayout {
  tabs: LayoutTabs | null;
}

// Dashboard customization Phase 1: mirrors ScreenLayout/LayoutTabs above -
// a workspace can have several named dashboard layouts (widgets instead
// of field tabs, no entity_type since a dashboard isn't per-object),
// role-assigned with a required default fallback, draft/publish. See
// core::models::dashboard_layout's doc comment for the full rationale.

/** One dashboard tile. `kind` selects how `config` is shaped - Phase 1
 * ships `"kpi"`, whose config is `{kpi_key}` (one of KPI_DEFS's keys -
 * see kpis.tsx). Phase 2 adds `"chart"`, whose config is `{report_id}` -
 * an existing saved Custom Report, run fresh on every render (see
 * `DashboardChartCard` in Dashboard.tsx); a report deleted after being
 * added to a dashboard is simply skipped, not an error. Phase 3 adds
 * `"record_list"`, whose config is `{entity_type, mode, limit, saved_view_id?}`
 * - a short list of records for one entity type, run fresh via
 * `run_dashboard_record_list` (see `dashboard_widget_service` in core
 * for what "recent" vs "due_soon" mean). `saved_view_id`, when set, names
 * a Saved View (see `SavedView` below) whose filters narrow the widget's
 * rows - the same reuse a list screen's own `useSavedViews` gets, applied
 * here to a dashboard tile's data source. This layer (like the backend)
 * never inspects `config` beyond that per-kind shape. */
export interface DashboardWidget {
  id: string;
  kind: string;
  config: Record<string, unknown>;
}

export interface DashboardWidgets {
  widgets: DashboardWidget[];
}

export interface DashboardLayout {
  id: string;
  workspace_id: string;
  name: string;
  is_default: boolean;
  roles: string[];
  draft: DashboardWidgets;
  published: DashboardWidgets | null;
  /** Per-app scoped automation - see `BusinessRule.app_id`'s doc comment. */
  app_id: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface DashboardLayoutInput {
  name: string;
  initial_kpi_keys: string[];
  app_id: string | null;
}

export interface DashboardLayoutUpdate {
  name: string;
  roles: string[];
  draft: DashboardWidgets;
  app_id: string | null;
}

export interface EffectiveDashboard {
  widgets: DashboardWidgets | null;
}

// Dashboard customization Phase 3: record-list widget data - see
// `DashboardWidget`'s own doc comment above for the "record_list" kind's
// config shape, and `dashboard_widget_service` in core for what each mode
// actually does per entity type.
export const RECORD_LIST_MODES = ["recent", "due_soon"] as const;
export type RecordListMode = (typeof RECORD_LIST_MODES)[number];

export interface RecordListRow {
  entity_type: string;
  entity_id: string;
  title: string;
  subtitle: string | null;
}

export interface CustomRecordInput {
  object_key: string;
  primary_name: string;
  status: string;
  owner_user_id: string | null;
  notes: string | null;
}

export interface CustomRecordUpdate {
  primary_name: string;
  status: string;
  owner_user_id: string | null;
  notes: string | null;
}

// Admin extensibility Phase B (spec §20.3/§21): admin-defined relationships
// between any two object types - built-in or custom. `entity_type` here is
// the same free-form string custom fields/business rules/workflow already
// key off, so a relationship can connect any built-in entity to any active
// custom object, or two custom objects to each other.
export const RELATIONSHIP_TYPES = ["many_to_one", "one_to_one", "many_to_many"] as const;
export type RelationshipType = (typeof RELATIONSHIP_TYPES)[number];
export const DELETE_BEHAVIORS = ["restrict", "archive"] as const;
export type DeleteBehavior = (typeof DELETE_BEHAVIORS)[number];

export interface RelationshipDefinition {
  id: string;
  workspace_id: string;
  key: string;
  source_entity_type: string;
  target_entity_type: string;
  relationship_type: RelationshipType;
  forward_label: string;
  reverse_label: string;
  is_required: boolean;
  show_related_list: boolean;
  delete_behavior: DeleteBehavior;
  is_protected: boolean;
  is_active: boolean;
  sort_order: number;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface RelationshipDefinitionInput {
  source_entity_type: string;
  target_entity_type: string;
  relationship_type: RelationshipType;
  forward_label: string;
  reverse_label: string;
  is_required: boolean;
  show_related_list: boolean;
  delete_behavior: DeleteBehavior;
  sort_order: number;
}

export interface RelationshipDefinitionUpdate {
  forward_label: string;
  reverse_label: string;
  is_required: boolean;
  show_related_list: boolean;
  delete_behavior: DeleteBehavior;
  sort_order: number;
  is_active: boolean;
}

export interface RelationshipInstance {
  id: string;
  workspace_id: string;
  relationship_definition_id: string;
  source_entity_type: string;
  source_id: string;
  target_entity_type: string;
  target_id: string;
  created_at: string;
  created_by: string | null;
}

export interface RelatedRecord {
  instance_id: string;
  relationship_definition_id: string;
  relationship_key: string;
  label: string;
  entity_type: string;
  entity_id: string;
  display_name: string;
  status: string;
  archived: boolean;
}

// Admin Automation & Customization addendum, Phase 2 (spec §2.5): the
// Status Transition editor - an allow-list of From -> To pairs for an
// entity's status/stage field. With zero rules for an entity type,
// transitions stay fully unrestricted (today's behavior). Matches
// core::models::status_transition::TRANSITION_ENTITY_TYPES - the entities
// whose status/stage changes through one generic, caller-supplied entry
// point (excludes Invoice, whose status changes through several dedicated
// methods with their own hardcoded semantics, and Product, whose
// "status" is a boolean with no meaningful transition concept).
export const TRANSITION_ENTITY_TYPES = ["Company", "Contact", "Opportunity", "Quote", "Order", "Contract", "Task"] as const;
export type TransitionEntityType = (typeof TRANSITION_ENTITY_TYPES)[number];

export interface StatusTransition {
  id: string;
  workspace_id: string;
  entity_type: string;
  /** `null` = "from any status" (a wildcard rule). */
  from_status: string | null;
  to_status: string;
  is_active: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface StatusTransitionInput {
  entity_type: string;
  from_status: string | null;
  to_status: string;
}

// App Builder (spec §24): a named, publishable grouping of already-existing
// objects, screens and a dashboard - the packaging layer on top of Custom
// Objects/Screen Builder/Dashboards, plus a genuinely new per-app
// permission model (a grant to a role OR to one specific user, not just a
// role checkbox list). Phase 2 enforces "editor" server-side on the
// object's own create/update/archive commands - see the Rust core's
// app_service doc comment for the full rationale and exactly what's
// covered.
export interface AppDefinition {
  id: string;
  workspace_id: string;
  name: string;
  icon: string;
  description: string | null;
  /** Entity types (built-in, e.g. "Task", or a custom object's key) this
   * app groups into its own nav - opaque here, resolved by AppShell the
   * same way a custom object's key already is. */
  object_keys: string[];
  dashboard_id: string | null;
  is_published: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface AppDefinitionInput {
  name: string;
  icon: string;
  description: string | null;
}

export interface AppDefinitionUpdate {
  name: string;
  icon: string;
  description: string | null;
  object_keys: string[];
  dashboard_id: string | null;
}

export const APP_PERMISSION_PRINCIPAL_TYPES = ["role", "user"] as const;
export type AppPermissionPrincipalType = (typeof APP_PERMISSION_PRINCIPAL_TYPES)[number];
export const APP_PERMISSION_LEVELS = ["viewer", "editor"] as const;
export type AppPermissionLevel = (typeof APP_PERMISSION_LEVELS)[number];

export interface AppPermission {
  id: string;
  app_id: string;
  principal_type: string;
  principal_id: string;
  level: string;
  created_at: string;
  created_by: string | null;
}

export interface AppPermissionInput {
  principal_type: AppPermissionPrincipalType;
  principal_id: string;
  level: AppPermissionLevel;
}

/** One app the current signed-in user can see, with their resolved access
 * level on it - drives the sidebar's app switcher. */
export interface AccessibleApp {
  app: AppDefinition;
  level: string;
}

// Industry Data Model foundations (roadmap "Industry Data Model"): a
// declarative package - object/field/relationship/business rule/workflow/
// screen layout/dashboard/report/numbering definitions plus an optional
// App Builder grouping - that installs into an existing workspace. This
// mirrors the Rust core's `models::industry_package` shapes; see that
// module's doc comments for the full rationale (deterministic keys,
// index-based relationship/report references, transactional install).
// The Admin -> App Catalog screen is the only UI these types back for
// now - no per-industry package content exists yet, only the machinery
// to import and install whatever manifest is handed to it.

export interface RecommendedPermission {
  role: string;
  level: string;
}

/** A package imported into this workspace's local catalog - validated
 * and available to install, not yet necessarily installed. */
export interface AppPackage {
  id: string;
  workspace_id: string;
  package_id: string;
  name: string;
  industry: string;
  version: string;
  min_lanesra_version: string;
  manifest_json: string;
  checksum: string;
  source: string;
  imported_at: string;
  imported_by: string | null;
  publisher_id: string | null;
  is_managed: boolean;
}

export const INSTALLED_APP_STATUSES = ["active", "deactivated"] as const;
export type InstalledAppStatus = (typeof INSTALLED_APP_STATUSES)[number];

/** One package actually installed into this workspace. */
export interface InstalledApp {
  id: string;
  workspace_id: string;
  package_id: string;
  name: string;
  icon: string;
  industry: string;
  description: string | null;
  installed_version: string;
  status: string;
  app_definition_id: string | null;
  recommended_permissions: RecommendedPermission[];
  installed_at: string;
  installed_by: string | null;
  updated_at: string;
  updated_by: string | null;
  deactivated_at: string | null;
  deactivated_by: string | null;
}

/** One record an install created - lets the App Catalog detail view show
 * exactly what an install touched. */
export interface PackageArtifact {
  id: string;
  installed_app_id: string;
  artifact_type: string;
  metadata_id: string;
  origin_version: string;
  is_locally_customized: boolean;
  created_at: string;
}

export const APP_INSTALL_RUN_ACTIONS = ["install", "update", "deactivate", "reactivate"] as const;
export type AppInstallRunAction = (typeof APP_INSTALL_RUN_ACTIONS)[number];
export const APP_INSTALL_RUN_STATUSES = ["running", "succeeded", "failed"] as const;
export type AppInstallRunStatus = (typeof APP_INSTALL_RUN_STATUSES)[number];

/** One install/update/deactivate/reactivate attempt, success or failure -
 * kept even when it failed, as the readable "why did this fail" record. */
export interface AppInstallRun {
  id: string;
  workspace_id: string;
  package_id: string;
  package_version: string;
  action: string;
  status: string;
  started_at: string;
  completed_at: string | null;
  backup_snapshot_path: string | null;
  error_message: string | null;
  actor_user_id: string | null;
}

export interface InstalledAppDetail {
  app: InstalledApp;
  artifacts: PackageArtifact[];
}

export interface ImportPackageInput {
  manifest_json: string;
}

/** A package version's declared dependency on another package, as
 * recorded in the registry at import time. */
export interface AppDependency {
  id: string;
  app_package_id: string;
  dependency_package_id: string;
  version_constraint: string;
  is_required: boolean;
}

/** An AppDependency alongside its declaring package's identity and
 * whether it's currently satisfied in this workspace - Admin > Solution
 * Management's Dependencies tab. */
export interface WorkspaceDependency {
  dependency: AppDependency;
  package_id: string;
  package_name: string;
  package_version: string;
  is_satisfied: boolean;
}

/** A PackageArtifact alongside its owning InstalledApp's identity -
 * Admin > Deployment Management's Components tab, the workspace-wide
 * "what have I customized beyond what I installed" view. */
export interface WorkspaceArtifact {
  artifact: PackageArtifact;
  installed_app_name: string;
  package_id: string;
}

/** A registered namespace owner in this workspace - every package_id is
 * expected to be "<publisher.key>.<name>", enforced at import time. */
export interface Publisher {
  id: string;
  workspace_id: string;
  key: string;
  name: string;
  description: string | null;
  is_official: boolean;
  is_local: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface PublisherInput {
  key: string;
  name: string;
  description: string | null;
}

/** Saved Views & Bulk Actions (product backlog): a named, persisted
 * filter/sort/column/grouping combination for one object_key. `filters` is
 * the exact {custom_field_key: value} shape useCustomFieldFilters already
 * produces - a saved view remembers what you already had set, it doesn't
 * add a new query capability. */
export interface SavedView {
  id: string;
  workspace_id: string;
  object_key: string;
  name: string;
  owner_user_id: string;
  owner_name: string | null;
  visibility: "private" | "shared";
  filters: Record<string, string>;
  sort_field: string | null;
  sort_direction: "asc" | "desc";
  columns: string[] | null;
  group_by_field: string | null;
  is_object_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface SavedViewInput {
  object_key: string;
  name: string;
  visibility: "private" | "shared";
  filters: Record<string, string>;
  sort_field: string | null;
  sort_direction: "asc" | "desc";
  columns: string[] | null;
  group_by_field: string | null;
}

/** One record's outcome from a bulk action - bulk operations are
 * independent per record (see bulk_action_service's own doc comment), so
 * a call always returns one of these per selected id, never aborting the
 * rest of the batch on the first failure. */
export interface BulkActionResult {
  id: string;
  ok: boolean;
  error: string | null;
}

// Solution Packages & Admin IA design spec, Phase 3: component-tagging,
// the Local Workspace (Custom) grouping, .lanesra export, and
// update-with-diff. Mirrors core::models::solution_component and the new
// pieces of core::models::industry_package - see those modules' doc
// comments for the full design.

/** One component's current owner - the workspace-wide registry every
 * hand-built and package-installed custom object/field/relationship/
 * business rule/workflow/screen layout/report shares. */
export interface SolutionComponent {
  id: string;
  workspace_id: string;
  artifact_type: string;
  metadata_id: string;
  publisher_id: string;
  installed_app_id: string | null;
  created_at: string;
  created_by: string | null;
}

/** A SolutionComponent joined with its owning publisher's display fields
 * - Admin > Deployment Management's Components tab, now covering both
 * hand-built and package-installed components (superseding the narrower
 * WorkspaceArtifact view). */
export interface WorkspaceComponent {
  component: SolutionComponent;
  publisher_key: string;
  publisher_name: string;
  is_local: boolean;
  installed_app_name: string | null;
}

/** The Packaged/Custom distinction's Custom half: a count of
 * everything still owned by the `local` publisher, broken down by type -
 * rendered as a synthetic "Local Workspace" row in Solution Packages
 * without a real app_packages row ever existing for it. */
export interface LocalWorkspaceSummary {
  publisher_id: string;
  component_count: number;
  components_by_type: [string, number][];
}

/** One object/field key's change between the installed version and a
 * newly-imported one - see plan_package_update's own doc comment in the
 * Rust core for exactly what "modified" means per type. */
export interface PackageUpdateDiffEntry {
  key: string;
  kind: "added" | "modified" | "removed";
}

/** The update-with-diff review step's output. Relationships/business
 * rules/workflows/screen layouts/reports have no stable cross-version
 * identity, so they're summarized as a single added-count rather than a
 * full per-item diff - see the Rust core's plan_update doc comment. */
export interface PackageUpdateDiff {
  package_id: string;
  from_version: string;
  to_version: string;
  objects: PackageUpdateDiffEntry[];
  fields: PackageUpdateDiffEntry[];
  relationships_added: number;
  business_rules_added: number;
  workflows_added: number;
  screen_layouts_added: number;
  reports_added: number;
}

// --- Solution Packages & Admin IA design spec, Phase 4: named, scoped
// Solutions - the Dynamics-365-style "build a solution in test, export it,
// import it in prod" workflow. See the Rust core's migration 0031 and
// solution_service for the full design.

/** A named, versioned, admin-curated subset of this workspace's
 * components - a deliberate, exportable-on-its-own unit, unlike the
 * all-or-nothing "everything Local Workspace owns" Export button. */
export interface Solution {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  version: string;
  publisher_id: string | null;
  publisher_name: string | null;
  member_count: number;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface SolutionInput {
  name: string;
  description: string | null;
  version: string | null;
  publisher_id: string | null;
}

export interface SolutionUpdate {
  name: string;
  description: string | null;
  version: string;
  publisher_id: string | null;
}

export interface SolutionMemberInput {
  artifact_type: string;
  metadata_id: string;
}

/** A Solution plus its curated members, each resolved to the same display
 * shape the Components tab uses. */
export interface SolutionDetail {
  solution: Solution;
  members: WorkspaceComponent[];
}

// --- Integration Hub (Lanesra_OS_Integration_Hub_Admin_Design_Development_Spec_v1.0) -
// mirrors core::models::integration 1:1. See IntegrationHubAdmin.tsx for the
// admin screens built on these.

export interface Connection {
  id: string;
  workspace_id: string;
  name: string;
  connection_type: string;
  base_url: string | null;
  auth_mode: string;
  has_secret: boolean;
  config_json: string;
  owner_user_id: string | null;
  status: string;
  last_test_at: string | null;
  last_test_status: string | null;
  last_test_message: string | null;
  last_failure_at: string | null;
  credential_expires_at: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface ConnectionInput {
  name: string;
  connection_type: string;
  base_url: string | null;
  auth_mode: string;
  secret_value: string | null;
  config_json: string;
  owner_user_id: string | null;
}

export interface ConnectionUpdate {
  name: string;
  base_url: string | null;
  auth_mode: string;
  secret_value: string | null;
  config_json: string;
  owner_user_id: string | null;
  status: string;
}

export interface ConnectionTestResult {
  ok: boolean;
  latency_ms: number;
  status_code: number | null;
  message: string;
}

export interface ConnectionRef {
  id: string;
  workspace_id: string;
  reference_name: string;
  reference_key: string;
  expected_connection_type: string;
  connection_id: string | null;
  connection_name: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface ConnectionRefInput {
  reference_name: string;
  reference_key: string;
  expected_connection_type: string;
  connection_id: string | null;
}

export interface ApiClient {
  id: string;
  workspace_id: string;
  name: string;
  client_id: string;
  status: string;
  scopes: string[];
  allowed_cidr: string | null;
  owner_user_id: string | null;
  last_used_at: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface ApiClientInput {
  name: string;
  scopes: string[];
  allowed_cidr: string | null;
  owner_user_id: string | null;
}

/** Returned exactly once, at creation and at secret rotation - never
 * recoverable again after this response (spec §8.1). */
export interface IssuedApiClient {
  client: ApiClient;
  api_key: string;
}

export interface Webhook {
  id: string;
  workspace_id: string;
  name: string;
  connection_id: string;
  endpoint_url: string | null;
  event_types: string[];
  object_scope: string | null;
  filter_json: string | null;
  payload_version: string;
  has_secret: boolean;
  retry_policy_json: string;
  status: string;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface WebhookInput {
  name: string;
  connection_id: string;
  event_types: string[];
  object_scope: string | null;
  filter_json: string | null;
  payload_version: string | null;
  retry_policy_json: string | null;
}

export interface WebhookDelivery {
  id: string;
  webhook_id: string;
  event_id: string;
  event_type: string;
  attempt_number: number;
  status: string;
  http_status: number | null;
  duration_ms: number | null;
  response_snippet: string | null;
  created_at: string;
}

export interface FieldMapEntry {
  source_column: string;
  target_field: string;
  transform: string | null;
  default_value: string | null;
  constant: string | null;
}

export interface Mapping {
  id: string;
  workspace_id: string;
  name: string;
  target_object_key: string;
  operation: string;
  match_key: string | null;
  field_map: FieldMapEntry[];
  duplicate_policy: string;
  needs_review: boolean;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface MappingInput {
  name: string;
  target_object_key: string;
  operation: string;
  match_key: string | null;
  field_map: FieldMapEntry[];
  duplicate_policy: string;
}

export interface CsvImportInput {
  target_object_key: string;
  csv_text: string;
  operation: string;
  match_key: string | null;
  field_map: FieldMapEntry[];
  duplicate_policy: string;
  dry_run: boolean;
}

export interface CsvRowResult {
  row_index: number;
  status: string;
  record_id: string | null;
  error: string | null;
}

export interface CsvImportResult {
  total_rows: number;
  successful: number;
  failed: number;
  skipped_duplicates: number;
  row_results: CsvRowResult[];
  duration_ms: number;
}

export interface IntegrationExecution {
  id: string;
  workspace_id: string;
  execution_type: string;
  correlation_id: string | null;
  ref_id: string | null;
  direction: string;
  started_at: string;
  ended_at: string | null;
  duration_ms: number | null;
  status: string;
  http_status: number | null;
  records_read: number;
  records_written: number;
  records_skipped: number;
  records_failed: number;
  retry_count: number;
  error_category: string | null;
  error_message: string | null;
  actor_user_id: string | null;
}

export interface IntegrationExecutionQuery {
  execution_type?: string | null;
  status?: string | null;
  correlation_id?: string | null;
  limit?: number | null;
}

export interface IntegrationOverview {
  active_connections: number;
  failed_connections: number;
  api_calls_today: number;
  failed_webhooks_today: number;
  jobs_running: number;
  jobs_failed_today: number;
}

export interface IntegrationSettings {
  workspace_id: string;
  api_rate_limit_per_minute: number;
  global_rate_limit_per_minute: number;
  log_retention_days: number;
  file_retention_days: number;
  allow_insecure_connections: boolean;
  updated_at: string;
  updated_by: string | null;
}

export interface IntegrationSettingsUpdate {
  api_rate_limit_per_minute: number;
  global_rate_limit_per_minute: number;
  log_retention_days: number;
  file_retention_days: number;
  allow_insecure_connections: boolean;
}

export interface ConnectorActionParam {
  name: string;
  location: string;
  required: boolean;
  schema_type: string;
}

export interface ConnectorAction {
  id: string;
  connector_id: string;
  action_key: string;
  display_name: string;
  http_method: string;
  path_template: string;
  params: ConnectorActionParam[];
  request_schema_json: string | null;
  response_schema_json: string | null;
}

export interface Connector {
  id: string;
  workspace_id: string;
  name: string;
  description: string | null;
  connection_type: string;
  spec_source: string;
  publisher_id: string | null;
  actions: ConnectorAction[];
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface DiscoveredOperation {
  operation_id: string;
  http_method: string;
  path_template: string;
  summary: string | null;
  params: ConnectorActionParam[];
}

export interface OpenApiImportPreview {
  title: string;
  version: string;
  operations: DiscoveredOperation[];
  warnings: string[];
}

export interface ConnectorExecutionResult {
  ok: boolean;
  status_code: number | null;
  duration_ms: number;
  response_body: unknown;
  message: string;
}

export interface ConnectorImportInput {
  name: string;
  description: string | null;
  spec_text: string;
  spec_format: string;
  selected_operation_ids: string[];
}

export interface ExternalObject {
  id: string;
  workspace_id: string;
  object_key: string;
  display_name: string;
  connection_id: string;
  resource_path: string;
  field_map: FieldMapEntry[];
  cache_ttl_seconds: number | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface ExternalObjectInput {
  object_key: string;
  display_name: string;
  connection_id: string;
  resource_path: string;
  field_map: FieldMapEntry[];
  cache_ttl_seconds: number | null;
}

export interface ApiFieldMetadata {
  key: string;
  label: string;
  field_type: string;
  required: boolean;
  is_custom: boolean;
}

export interface ApiObjectMetadata {
  object_key: string;
  label: string;
  is_custom: boolean;
  fields: ApiFieldMetadata[];
}

export interface ApiListQuery {
  select?: string[] | null;
  filter?: unknown;
  sort?: string[] | null;
  page?: number | null;
  page_size?: number | null;
}

export interface IntegrationJob {
  id: string;
  workspace_id: string;
  name: string;
  external_object_id: string;
  target_object_key: string;
  match_key: string;
  cursor_field: string | null;
  cursor_value: string | null;
  interval_minutes: number;
  status: string;
  last_run_at: string | null;
  last_run_status: string | null;
  created_at: string;
  created_by: string | null;
  updated_at: string;
  updated_by: string | null;
}

export interface IntegrationJobInput {
  name: string;
  external_object_id: string;
  target_object_key: string;
  match_key: string;
  cursor_field: string | null;
  interval_minutes: number;
}

export interface IntegrationJobRun {
  id: string;
  job_id: string;
  workspace_id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
  records_processed: number;
  records_failed: number;
  error_message: string | null;
  cursor_before: string | null;
  cursor_after: string | null;
}
