import { invoke } from "@tauri-apps/api/core";
import type {
  AppErrorPayload,
  BackupManifest,
  BackupPackage,
  ChangeOwnPassword,
  Company,
  CompanyInput,
  Contact,
  ContactInput,
  Contract,
  ContractInput,
  Credentials,
  DashboardSummary,
  Invoice,
  InvoiceInput,
  InvoiceWithLines,
  Opportunity,
  OpportunityInput,
  OpportunityProduct,
  OpportunityProductInput,
  Order,
  OrderInput,
  OrderWithLines,
  Product,
  ProductInput,
  Quote,
  QuoteInput,
  QuoteWithLines,
  PaymentInput,
  NewUser,
  PasswordChange,
  Task,
  TaskInput,
  User,
  UserUpdate,
  ArAgingBucket,
  BusinessRule,
  BusinessRuleInput,
  BusinessRuleUpdate,
  CustomFieldDefinition,
  CustomFieldDefinitionInput,
  CustomFieldDefinitionUpdate,
  CustomFieldValues,
  CustomObjectDefinition,
  CustomObjectDefinitionInput,
  CustomObjectDefinitionUpdate,
  CustomRecord,
  CustomRecordInput,
  CustomRecordUpdate,
  CustomReport,
  CustomReportInput,
  CustomReportRow,
  CustomReportUpdate,
  DashboardKpiPrefs,
  EffectiveNumbering,
  LostReasonBreakdown,
  Notification,
  NumberingOverrideInput,
  RelatedRecord,
  RelationshipDefinition,
  RelationshipDefinitionInput,
  RelationshipDefinitionUpdate,
  RelationshipInstance,
  ReportRange,
  RuleEvaluation,
  RevenueByMonth,
  SalesByOwner,
  StatusTransition,
  StatusTransitionInput,
  WinRateByOwner,
  WorkflowDefinition,
  WorkflowDefinitionInput,
  WorkflowDefinitionUpdate,
  WorkflowRun,
  Workspace,
  WorkspaceLogo,
  WorkspaceSetup,
  WorkspaceUpdate,
} from "./types";

export class ApiError extends Error {
  kind: AppErrorPayload["kind"];
  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.kind = payload.kind;
  }
}

// The desktop app talks to its Rust backend over Tauri's IPC bridge, which
// only exists inside a Tauri webview. When this same frontend is served by
// the Team Workspace HTTP server to a plain browser, there is no such
// bridge - so we fall back to fetching the equivalent HTTP endpoint, which
// mirrors every Tauri command 1:1 (see server/src/dispatch.rs).
function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function httpInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const response = await fetch(`/api/invoke/${command}`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args ?? {}),
  });
  const body = await response.json();
  if (!body.ok) {
    throw new ApiError(body.error as AppErrorPayload);
  }
  return body.data as T;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return isTauriRuntime() ? await invoke<T>(command, args) : await httpInvoke<T>(command, args);
  } catch (err) {
    if (err instanceof ApiError) throw err;
    if (err && typeof err === "object" && "kind" in err && "message" in err) {
      throw new ApiError(err as AppErrorPayload);
    }
    throw err;
  }
}

export const api = {
  workspaceStatus: () => call<Workspace | null>("workspace_status"),
  firstRunSetup: (setup: WorkspaceSetup) => call<[Workspace, User]>("first_run_setup", { setup }),
  updateWorkspace: (input: WorkspaceUpdate) => call<Workspace>("update_workspace", { input }),
  setWorkspaceLogo: (input: WorkspaceLogo) => call<Workspace>("set_workspace_logo", { input }),
  clearWorkspaceLogo: () => call<Workspace>("clear_workspace_logo"),
  setDashboardKpis: (prefs: DashboardKpiPrefs) => call<Workspace>("set_dashboard_kpis", { prefs }),

  login: (credentials: Credentials) => call<User>("login", { credentials }),
  logout: () => call<void>("logout"),
  currentUser: () => call<User | null>("current_user"),
  changeMyPassword: (input: ChangeOwnPassword) => call<void>("change_my_password", { input }),

  listCompanies: () => call<Company[]>("list_companies"),
  getCompany: (id: string) => call<Company>("get_company", { id }),
  createCompany: (input: CompanyInput) => call<Company>("create_company", { input }),
  updateCompany: (id: string, input: CompanyInput) => call<Company>("update_company", { id, input }),
  archiveCompany: (id: string) => call<void>("archive_company", { id }),
  checkCompanyDuplicates: (name: string, excludeId?: string) =>
    call<Company[]>("check_company_duplicates", { name, excludeId: excludeId ?? null }),

  listContacts: () => call<Contact[]>("list_contacts"),
  listContactsByCompany: (companyId: string) =>
    call<Contact[]>("list_contacts_by_company", { companyId }),
  getContact: (id: string) => call<Contact>("get_contact", { id }),
  createContact: (input: ContactInput) => call<Contact>("create_contact", { input }),
  updateContact: (id: string, input: ContactInput) => call<Contact>("update_contact", { id, input }),
  archiveContact: (id: string) => call<void>("archive_contact", { id }),
  checkContactDuplicates: (companyId: string, email: string, excludeId?: string) =>
    call<Contact[]>("check_contact_duplicates", { companyId, email, excludeId: excludeId ?? null }),

  listProducts: () => call<Product[]>("list_products"),
  getProduct: (id: string) => call<Product>("get_product", { id }),
  createProduct: (input: ProductInput) => call<Product>("create_product", { input }),
  updateProduct: (id: string, input: ProductInput) => call<Product>("update_product", { id, input }),
  archiveProduct: (id: string) => call<void>("archive_product", { id }),

  listOpportunities: () => call<Opportunity[]>("list_opportunities"),
  listOpportunitiesByCompany: (companyId: string) =>
    call<Opportunity[]>("list_opportunities_by_company", { companyId }),
  getOpportunity: (id: string) => call<Opportunity>("get_opportunity", { id }),
  createOpportunity: (input: OpportunityInput) => call<Opportunity>("create_opportunity", { input }),
  updateOpportunity: (id: string, input: OpportunityInput) =>
    call<Opportunity>("update_opportunity", { id, input }),
  archiveOpportunity: (id: string) => call<void>("archive_opportunity", { id }),
  setOpportunityProducts: (opportunityId: string, products: OpportunityProductInput[]) =>
    call<OpportunityProduct[]>("set_opportunity_products", { opportunityId, products }),
  listOpportunityProducts: (opportunityId: string) =>
    call<OpportunityProduct[]>("list_opportunity_products", { opportunityId }),

  listQuotes: () => call<Quote[]>("list_quotes"),
  getQuote: (id: string) => call<QuoteWithLines>("get_quote", { id }),
  createQuote: (input: QuoteInput) => call<QuoteWithLines>("create_quote", { input }),
  setQuoteStatus: (id: string, status: string) =>
    call<QuoteWithLines>("set_quote_status", { id, status }),
  convertQuoteToOrder: (quoteId: string) =>
    call<OrderWithLines>("convert_quote_to_order", { quoteId }),

  listOrders: () => call<Order[]>("list_orders"),
  getOrder: (id: string) => call<OrderWithLines>("get_order", { id }),
  createOrder: (input: OrderInput) => call<OrderWithLines>("create_order", { input }),
  setOrderStatus: (id: string, status: string) =>
    call<OrderWithLines>("set_order_status", { id, status }),
  convertOrderToInvoice: (orderId: string) =>
    call<InvoiceWithLines>("convert_order_to_invoice", { orderId }),

  listInvoices: () => call<Invoice[]>("list_invoices"),
  getInvoice: (id: string) => call<InvoiceWithLines>("get_invoice", { id }),
  createInvoice: (input: InvoiceInput) => call<InvoiceWithLines>("create_invoice", { input }),
  issueInvoice: (id: string) => call<InvoiceWithLines>("issue_invoice", { id }),
  voidInvoice: (id: string) => call<InvoiceWithLines>("void_invoice", { id }),
  recordInvoicePayment: (id: string, payment: PaymentInput) =>
    call<InvoiceWithLines>("record_invoice_payment", { id, payment }),
  refreshOverdueInvoices: () => call<number>("refresh_overdue_invoices"),

  dashboardSummary: () => call<DashboardSummary>("dashboard_summary"),

  listContracts: () => call<Contract[]>("list_contracts"),
  listContractsByCompany: (companyId: string) =>
    call<Contract[]>("list_contracts_by_company", { companyId }),
  getContract: (id: string) => call<Contract>("get_contract", { id }),
  createContract: (input: ContractInput) => call<Contract>("create_contract", { input }),
  updateContract: (id: string, input: ContractInput) => call<Contract>("update_contract", { id, input }),
  archiveContract: (id: string) => call<void>("archive_contract", { id }),

  listTasks: () => call<Task[]>("list_tasks"),
  listTasksByRelated: (relatedType: string, relatedId: string) =>
    call<Task[]>("list_tasks_by_related", { relatedType, relatedId }),
  getTask: (id: string) => call<Task>("get_task", { id }),
  createTask: (input: TaskInput) => call<Task>("create_task", { input }),
  updateTask: (id: string, input: TaskInput) => call<Task>("update_task", { id, input }),
  archiveTask: (id: string) => call<void>("archive_task", { id }),

  listUsers: () => call<User[]>("list_users"),
  createUser: (input: NewUser) => call<User>("create_user", { input }),
  updateUser: (id: string, input: UserUpdate) => call<User>("update_user", { id, input }),
  setUserPassword: (id: string, input: PasswordChange) => call<void>("set_user_password", { id, input }),

  reportRevenueByMonth: (range: ReportRange) => call<RevenueByMonth[]>("report_revenue_by_month", { range }),
  reportWinRateByOwner: (range: ReportRange) => call<WinRateByOwner[]>("report_win_rate_by_owner", { range }),
  reportLostReasons: (range: ReportRange) => call<LostReasonBreakdown[]>("report_lost_reasons", { range }),
  reportArAging: (asOfDate: string | null) => call<ArAgingBucket[]>("report_ar_aging", { asOfDate }),
  reportSalesByOwner: (range: ReportRange) => call<SalesByOwner[]>("report_sales_by_owner", { range }),

  listCustomFieldDefinitions: (entityType: string, activeOnly: boolean) =>
    call<CustomFieldDefinition[]>("list_custom_field_definitions", { entityType, activeOnly }),
  createCustomFieldDefinition: (input: CustomFieldDefinitionInput) =>
    call<CustomFieldDefinition>("create_custom_field_definition", { input }),
  updateCustomFieldDefinition: (id: string, input: CustomFieldDefinitionUpdate) =>
    call<CustomFieldDefinition>("update_custom_field_definition", { id, input }),
  deactivateCustomFieldDefinition: (id: string) =>
    call<CustomFieldDefinition>("deactivate_custom_field_definition", { id }),
  // Returns any non-blocking `show_message` texts that fired (empty
  // array normally) - see custom_field_service::set_entity_values.
  setCustomFieldValues: (entityType: string, entityId: string, values: CustomFieldValues) =>
    call<string[]>("set_custom_field_values", { entityType, entityId, values }),
  getCustomFieldValues: (entityId: string) => call<CustomFieldValues>("get_custom_field_values", { entityId }),

  listBusinessRules: (entityType: string, activeOnly: boolean) =>
    call<BusinessRule[]>("list_business_rules", { entityType, activeOnly }),
  createBusinessRule: (input: BusinessRuleInput) => call<BusinessRule>("create_business_rule", { input }),
  updateBusinessRule: (id: string, input: BusinessRuleUpdate) => call<BusinessRule>("update_business_rule", { id, input }),
  testBusinessRules: (entityType: string, context: Record<string, string>) =>
    call<RuleEvaluation>("test_business_rules", { entityType, context }),

  listStatusTransitions: (entityType: string) => call<StatusTransition[]>("list_status_transitions", { entityType }),
  createStatusTransition: (input: StatusTransitionInput) => call<StatusTransition>("create_status_transition", { input }),
  setStatusTransitionActive: (id: string, isActive: boolean) => call<void>("set_status_transition_active", { id, isActive }),
  deleteStatusTransition: (id: string) => call<void>("delete_status_transition", { id }),

  listWorkflowRules: (entityType: string) => call<WorkflowDefinition[]>("list_workflow_rules", { entityType }),
  createWorkflowRule: (input: WorkflowDefinitionInput) => call<WorkflowDefinition>("create_workflow_rule", { input }),
  updateWorkflowRule: (id: string, input: WorkflowDefinitionUpdate) =>
    call<WorkflowDefinition>("update_workflow_rule", { id, input }),
  listWorkflowRuns: (workflowId: string) => call<WorkflowRun[]>("list_workflow_runs", { workflowId }),
  runScheduledWorkflows: () => call<number>("run_scheduled_workflows"),

  listNotifications: (unreadOnly: boolean) => call<Notification[]>("list_notifications", { unreadOnly }),
  markNotificationRead: (id: string) => call<void>("mark_notification_read", { id }),
  markAllNotificationsRead: () => call<void>("mark_all_notifications_read"),

  listNumberingFormats: () => call<EffectiveNumbering[]>("list_numbering_formats"),
  setNumberingFormat: (input: NumberingOverrideInput) => call<EffectiveNumbering>("set_numbering_format", { input }),
  resetNumberingFormat: (entityType: string) =>
    call<EffectiveNumbering>("reset_numbering_format", { entityType }),

  listCustomReports: () => call<CustomReport[]>("list_custom_reports"),
  createCustomReport: (input: CustomReportInput) => call<CustomReport>("create_custom_report", { input }),
  updateCustomReport: (id: string, input: CustomReportUpdate) =>
    call<CustomReport>("update_custom_report", { id, input }),
  deleteCustomReport: (id: string) => call<void>("delete_custom_report", { id }),
  runCustomReport: (id: string) => call<CustomReportRow[]>("run_custom_report", { id }),

  listCustomObjects: (activeOnly: boolean) => call<CustomObjectDefinition[]>("list_custom_objects", { activeOnly }),
  createCustomObject: (input: CustomObjectDefinitionInput) =>
    call<CustomObjectDefinition>("create_custom_object", { input }),
  updateCustomObject: (id: string, input: CustomObjectDefinitionUpdate) =>
    call<CustomObjectDefinition>("update_custom_object", { id, input }),
  deactivateCustomObject: (id: string) => call<CustomObjectDefinition>("deactivate_custom_object", { id }),
  deleteCustomObject: (id: string) => call<void>("delete_custom_object", { id }),

  listCustomRecords: (objectKey: string) => call<CustomRecord[]>("list_custom_records", { objectKey }),
  getCustomRecord: (id: string) => call<CustomRecord>("get_custom_record", { id }),
  createCustomRecord: (input: CustomRecordInput) => call<CustomRecord>("create_custom_record", { input }),
  updateCustomRecord: (id: string, input: CustomRecordUpdate) =>
    call<CustomRecord>("update_custom_record", { id, input }),
  archiveCustomRecord: (id: string) => call<CustomRecord>("archive_custom_record", { id }),

  listRelationshipDefinitions: (activeOnly: boolean) =>
    call<RelationshipDefinition[]>("list_relationship_definitions", { activeOnly }),
  createRelationshipDefinition: (input: RelationshipDefinitionInput) =>
    call<RelationshipDefinition>("create_relationship_definition", { input }),
  updateRelationshipDefinition: (id: string, input: RelationshipDefinitionUpdate) =>
    call<RelationshipDefinition>("update_relationship_definition", { id, input }),
  deleteRelationshipDefinition: (id: string) => call<void>("delete_relationship_definition", { id }),
  linkRecords: (
    definitionId: string,
    sourceEntityType: string,
    sourceId: string,
    targetEntityType: string,
    targetId: string,
  ) =>
    call<RelationshipInstance>("link_records", {
      definitionId, sourceEntityType, sourceId, targetEntityType, targetId,
    }),
  unlinkRecords: (instanceId: string) => call<void>("unlink_records", { instanceId }),
  listRelatedRecords: (entityType: string, entityId: string) =>
    call<RelatedRecord[]>("list_related_records", { entityType, entityId }),

  createBackup: () => call<BackupPackage>("create_backup"),
  restoreBackup: (packageBase64: string) =>
    call<BackupManifest>("restore_backup", { packageBase64 }),
};
