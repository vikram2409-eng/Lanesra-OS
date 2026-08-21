import { invoke } from "@tauri-apps/api/core";
import type {
  AccessibleApp,
  AppDefinition,
  AppDefinitionInput,
  AppDefinitionUpdate,
  AppErrorPayload,
  AppInstallRun,
  AppPackage,
  AppPermission,
  AppPermissionInput,
  AuditEvent,
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
  DashboardLayout,
  DashboardLayoutInput,
  DashboardLayoutUpdate,
  DashboardSummary,
  EffectiveDashboard,
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
  BusinessRuleVersion,
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
  ImportPackageInput,
  InstalledApp,
  InstalledAppDetail,
  WorkspaceArtifact,
  WorkspaceDependency,
  Publisher,
  PublisherInput,
  WorkspaceComponent,
  LocalWorkspaceSummary,
  PackageUpdateDiff,
  Solution,
  SolutionDetail,
  SolutionInput,
  SolutionUpdate,
  SolutionMemberInput,
  Connection,
  ConnectionInput,
  ConnectionUpdate,
  ConnectionTestResult,
  ConnectionRef,
  ConnectionRefInput,
  ApiClient,
  ApiClientInput,
  IssuedApiClient,
  Webhook,
  WebhookInput,
  WebhookDelivery,
  Mapping,
  MappingInput,
  CsvImportInput,
  CsvImportResult,
  ApiListQuery,
  ApiObjectMetadata,
  IntegrationExecution,
  IntegrationExecutionQuery,
  IntegrationOverview,
  IntegrationSettings,
  IntegrationSettingsUpdate,
  Connector,
  ConnectorImportInput,
  ConnectorExecutionResult,
  OpenApiImportPreview,
  ExternalObject,
  ExternalObjectInput,
  IntegrationJob,
  IntegrationJobInput,
  IntegrationJobRun,
  LostReasonBreakdown,
  Notification,
  NumberingOverrideInput,
  RecordListRow,
  RelatedRecord,
  RelationshipDefinition,
  RelationshipDefinitionInput,
  RelationshipDefinitionUpdate,
  RelationshipInstance,
  ReportRange,
  RuleEvaluation,
  RevenueByMonth,
  SalesByOwner,
  SaveNotices,
  ScreenLayout,
  ScreenLayoutInput,
  ScreenLayoutUpdate,
  EffectiveLayout,
  SearchResult,
  StatusTransition,
  StatusTransitionInput,
  WinRateByOwner,
  WorkflowDefinition,
  WorkflowDefinitionInput,
  WorkflowDefinitionUpdate,
  WorkflowRuleVersion,
  WorkflowRun,
  WorkflowTestResult,
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

function normalizeError(err: unknown): never {
  if (err instanceof ApiError) throw err;
  if (err && typeof err === "object" && "kind" in err && "message" in err) {
    throw new ApiError(err as AppErrorPayload);
  }
  throw err;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return isTauriRuntime() ? await invoke<T>(command, args) : await httpInvoke<T>(command, args);
  } catch (err) {
    normalizeError(err);
  }
}

// Integration Hub's five genuinely-async admin actions (Test Connection,
// Test Action, Test Delivery, Run Now, External Object preview) don't fit
// the generic `/api/invoke/:command` dispatcher on the server - see
// `server/src/dispatch.rs`'s own comment on why - so on the server they're
// their own small route group under `/api/admin/...` instead
// (`server/src/admin_actions.rs`). Inside Tauri they're an ordinary async
// command like any other. This is the one place the dual-transport
// abstraction needs a per-command HTTP path rather than the uniform
// `/api/invoke/<command>` every other operation uses.
async function httpAdminAction<T>(method: "GET" | "POST", path: string, body?: unknown): Promise<T> {
  const response = await fetch(path, {
    method,
    credentials: "include",
    headers: body !== undefined ? { "Content-Type": "application/json" } : undefined,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  const responseBody = await response.json();
  if (!responseBody.ok) {
    throw new ApiError(responseBody.error as AppErrorPayload);
  }
  return responseBody.data as T;
}

async function callAdminAction<T>(
  tauriCommand: string,
  tauriArgs: Record<string, unknown>,
  httpMethod: "GET" | "POST",
  httpPath: string,
  httpBody?: unknown,
): Promise<T> {
  try {
    return isTauriRuntime() ? await invoke<T>(tauriCommand, tauriArgs) : await httpAdminAction<T>(httpMethod, httpPath, httpBody);
  } catch (err) {
    normalizeError(err);
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
  globalSearch: (query: string) => call<SearchResult[]>("global_search", { query }),
  listAuditEvents: (entityType: string, entityId: string) =>
    call<AuditEvent[]>("list_audit_events", { entityType, entityId }),

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
  // Admin UX polish (spec §10): call before either deactivation path above
  // and show a confirmation when the result isn't empty - see
  // custom_field_service::describe_active_dependents's doc comment.
  describeCustomFieldDependents: (id: string) => call<string[]>("describe_custom_field_dependents", { id }),
  // Returns any non-blocking show_error/show_warning notices that fired
  // (both empty normally) - see custom_field_service::set_entity_values.
  setCustomFieldValues: (entityType: string, entityId: string, values: CustomFieldValues) =>
    call<SaveNotices>("set_custom_field_values", { entityType, entityId, values }),
  getCustomFieldValues: (entityId: string) => call<CustomFieldValues>("get_custom_field_values", { entityId }),
  // List-view filtering: every is_filterable value for one entity type,
  // keyed by entity id then field key - one call per list screen load.
  listFilterableCustomFieldValues: (entityType: string) =>
    call<Record<string, CustomFieldValues>>("list_filterable_custom_field_values", { entityType }),

  listBusinessRules: (entityType: string, activeOnly: boolean) =>
    call<BusinessRule[]>("list_business_rules", { entityType, activeOnly }),
  createBusinessRule: (input: BusinessRuleInput) => call<BusinessRule>("create_business_rule", { input }),
  updateBusinessRule: (id: string, input: BusinessRuleUpdate) => call<BusinessRule>("update_business_rule", { id, input }),
  testBusinessRules: (entityType: string, context: Record<string, string>) =>
    call<RuleEvaluation>("test_business_rules", { entityType, context }),
  duplicateBusinessRule: (id: string) => call<BusinessRule>("duplicate_business_rule", { id }),
  listBusinessRuleVersions: (ruleId: string) => call<BusinessRuleVersion[]>("list_business_rule_versions", { ruleId }),
  restoreBusinessRuleVersion: (ruleId: string, versionId: string) =>
    call<BusinessRule>("restore_business_rule_version", { ruleId, versionId }),

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
  testWorkflows: (entityType: string, context: Record<string, string>) =>
    call<WorkflowTestResult>("test_workflows", { entityType, context }),
  duplicateWorkflowRule: (id: string) => call<WorkflowDefinition>("duplicate_workflow_rule", { id }),
  listWorkflowRuleVersions: (workflowId: string) => call<WorkflowRuleVersion[]>("list_workflow_rule_versions", { workflowId }),
  restoreWorkflowRuleVersion: (workflowId: string, versionId: string) =>
    call<WorkflowDefinition>("restore_workflow_rule_version", { workflowId, versionId }),

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

  listScreenLayouts: (entityType: string) => call<ScreenLayout[]>("list_screen_layouts", { entityType }),
  createScreenLayout: (input: ScreenLayoutInput) => call<ScreenLayout>("create_screen_layout", { input }),
  updateScreenLayout: (id: string, update: ScreenLayoutUpdate) => call<ScreenLayout>("update_screen_layout", { id, update }),
  publishScreenLayout: (id: string) => call<ScreenLayout>("publish_screen_layout", { id }),
  unpublishScreenLayout: (id: string) => call<ScreenLayout>("unpublish_screen_layout", { id }),
  revertScreenLayoutDraft: (id: string) => call<ScreenLayout>("revert_screen_layout_draft", { id }),
  makeScreenLayoutDefault: (id: string) => call<ScreenLayout>("make_screen_layout_default", { id }),
  deleteScreenLayout: (id: string) => call<void>("delete_screen_layout", { id }),
  effectiveScreenLayout: (entityType: string) => call<EffectiveLayout>("effective_screen_layout", { entityType }),

  listDashboardLayouts: () => call<DashboardLayout[]>("list_dashboard_layouts"),
  createDashboardLayout: (input: DashboardLayoutInput) => call<DashboardLayout>("create_dashboard_layout", { input }),
  updateDashboardLayout: (id: string, update: DashboardLayoutUpdate) => call<DashboardLayout>("update_dashboard_layout", { id, update }),
  publishDashboardLayout: (id: string) => call<DashboardLayout>("publish_dashboard_layout", { id }),
  unpublishDashboardLayout: (id: string) => call<DashboardLayout>("unpublish_dashboard_layout", { id }),
  revertDashboardLayoutDraft: (id: string) => call<DashboardLayout>("revert_dashboard_layout_draft", { id }),
  makeDashboardLayoutDefault: (id: string) => call<DashboardLayout>("make_dashboard_layout_default", { id }),
  deleteDashboardLayout: (id: string) => call<void>("delete_dashboard_layout", { id }),
  effectiveDashboardLayout: () => call<EffectiveDashboard>("effective_dashboard_layout"),
  runDashboardRecordList: (entityType: string, mode: string, limit: number) =>
    call<RecordListRow[]>("run_dashboard_record_list", { entityType, mode, limit }),

  listApps: () => call<AppDefinition[]>("list_apps"),
  createApp: (input: AppDefinitionInput) => call<AppDefinition>("create_app", { input }),
  updateApp: (id: string, update: AppDefinitionUpdate) => call<AppDefinition>("update_app", { id, update }),
  publishApp: (id: string) => call<AppDefinition>("publish_app", { id }),
  unpublishApp: (id: string) => call<AppDefinition>("unpublish_app", { id }),
  deleteApp: (id: string) => call<void>("delete_app", { id }),
  listAppPermissions: (appId: string) => call<AppPermission[]>("list_app_permissions", { appId }),
  grantAppPermission: (appId: string, input: AppPermissionInput) => call<AppPermission>("grant_app_permission", { appId, input }),
  revokeAppPermission: (id: string) => call<void>("revoke_app_permission", { id }),
  listAccessibleApps: () => call<AccessibleApp[]>("list_accessible_apps"),
  canWriteObject: (entityType: string) => call<boolean>("can_write_object", { entityType }),

  importIndustryPackage: (input: ImportPackageInput) => call<AppPackage>("import_industry_package", { input }),
  listIndustryPackages: () => call<AppPackage[]>("list_industry_packages"),
  validateIndustryPackage: (appPackageId: string) => call<void>("validate_industry_package", { appPackageId }),
  installIndustryPackage: (appPackageId: string) => call<InstalledApp>("install_industry_package", { appPackageId }),
  listInstalledApps: () => call<InstalledApp[]>("list_installed_apps"),
  getInstalledAppDetail: (id: string) => call<InstalledAppDetail>("get_installed_app_detail", { id }),
  listIndustryInstallRuns: () => call<AppInstallRun[]>("list_industry_install_runs"),
  deactivateInstalledApp: (id: string) => call<InstalledApp>("deactivate_installed_app", { id }),
  reactivateInstalledApp: (id: string) => call<InstalledApp>("reactivate_installed_app", { id }),
  getReferencePackageManifest: (key: string) => call<string>("get_reference_package_manifest", { key }),
  listPackageDependencies: () => call<WorkspaceDependency[]>("list_package_dependencies"),
  listPackageArtifactsForWorkspace: () => call<WorkspaceArtifact[]>("list_package_artifacts_for_workspace"),
  listPublishers: () => call<Publisher[]>("list_publishers"),
  createPublisher: (input: PublisherInput) => call<Publisher>("create_publisher", { input }),
  listSolutionComponents: () => call<WorkspaceComponent[]>("list_solution_components"),
  getLocalWorkspaceSummary: () => call<LocalWorkspaceSummary>("get_local_workspace_summary"),
  listPackageVersions: (packageId: string) => call<AppPackage[]>("list_package_versions", { packageId }),
  exportLocalWorkspace: () => call<string>("export_local_workspace"),
  planPackageUpdate: (newAppPackageId: string) => call<PackageUpdateDiff>("plan_package_update", { newAppPackageId }),
  applyPackageUpdate: (newAppPackageId: string) => call<InstalledApp>("apply_package_update", { newAppPackageId }),
  listSolutions: () => call<Solution[]>("list_solutions"),
  getSolutionDetail: (id: string) => call<SolutionDetail>("get_solution_detail", { id }),
  createSolution: (input: SolutionInput) => call<Solution>("create_solution", { input }),
  updateSolution: (id: string, input: SolutionUpdate) => call<Solution>("update_solution", { id, input }),
  deleteSolution: (id: string) => call<void>("delete_solution", { id }),
  addSolutionComponent: (solutionId: string, input: SolutionMemberInput) => call<void>("add_solution_component", { solutionId, input }),
  removeSolutionComponent: (solutionId: string, artifactType: string, metadataId: string) =>
    call<void>("remove_solution_component", { solutionId, artifactType, metadataId }),
  exportSolution: (solutionId: string) => call<string>("export_solution", { solutionId }),

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

  // --- Integration Hub -------------------------------------------------------
  listConnections: () => call<Connection[]>("list_connections"),
  createConnection: (input: ConnectionInput) => call<Connection>("create_connection", { input }),
  updateConnection: (id: string, input: ConnectionUpdate) => call<Connection>("update_connection", { id, input }),
  deleteConnection: (id: string) => call<void>("delete_connection", { id }),
  testConnection: (id: string) => callAdminAction<ConnectionTestResult>("test_connection", { id }, "POST", `/api/admin/connections/${encodeURIComponent(id)}/test`),

  listConnectionRefs: () => call<ConnectionRef[]>("list_connection_refs"),
  createConnectionRef: (input: ConnectionRefInput) => call<ConnectionRef>("create_connection_ref", { input }),
  bindConnectionRef: (id: string, connectionId: string | null) => call<ConnectionRef>("bind_connection_ref", { id, connectionId }),
  deleteConnectionRef: (id: string) => call<void>("delete_connection_ref", { id }),

  previewConnectorImport: (specText: string, specFormat: string) =>
    call<OpenApiImportPreview>("preview_connector_import", { specText, specFormat }),
  importConnector: (input: ConnectorImportInput) => call<Connector>("import_connector", { input }),
  listConnectors: () => call<Connector[]>("list_connectors"),
  getConnector: (id: string) => call<Connector>("get_connector", { id }),
  deleteConnector: (id: string) => call<void>("delete_connector", { id }),
  testConnectorAction: (connectorId: string, actionKey: string, referenceKey: string, params: unknown) =>
    callAdminAction<ConnectorExecutionResult>(
      "test_connector_action",
      { connectorId, actionKey, referenceKey, params },
      "POST",
      `/api/admin/connectors/${encodeURIComponent(connectorId)}/actions/${encodeURIComponent(actionKey)}/test`,
      { reference_key: referenceKey, params },
    ),

  listApiClients: () => call<ApiClient[]>("list_api_clients"),
  createApiClient: (input: ApiClientInput) => call<IssuedApiClient>("create_api_client", { input }),
  rotateApiClientSecret: (id: string) => call<IssuedApiClient>("rotate_api_client_secret", { id }),
  revokeApiClient: (id: string) => call<void>("revoke_api_client", { id }),
  reactivateApiClient: (id: string) => call<void>("reactivate_api_client", { id }),
  deleteApiClient: (id: string) => call<void>("delete_api_client", { id }),

  listWebhooks: () => call<Webhook[]>("list_webhooks"),
  createWebhook: (input: WebhookInput) => call<Webhook>("create_webhook", { input }),
  listWebhookDeliveries: (webhookId: string) => call<WebhookDelivery[]>("list_webhook_deliveries", { webhookId }),
  pauseWebhook: (id: string) => call<void>("pause_webhook", { id }),
  reactivateWebhook: (id: string) => call<void>("reactivate_webhook", { id }),
  deleteWebhook: (id: string) => call<void>("delete_webhook", { id }),
  testWebhookDelivery: (webhookId: string) =>
    callAdminAction<void>("test_webhook_delivery", { webhookId }, "POST", `/api/admin/webhooks/${encodeURIComponent(webhookId)}/test`),

  listMappings: () => call<Mapping[]>("list_mappings"),
  createMapping: (input: MappingInput) => call<Mapping>("create_mapping", { input }),
  deleteMapping: (id: string) => call<void>("delete_mapping", { id }),

  importCsv: (input: CsvImportInput) => call<CsvImportResult>("import_csv", { input }),
  exportCsv: (objectKey: string, query: ApiListQuery) => call<string>("export_csv", { objectKey, query }),
  listIntegrationObjectKeys: () => call<ApiObjectMetadata[]>("list_integration_object_keys"),

  listExternalObjects: () => call<ExternalObject[]>("list_external_objects"),
  createExternalObject: (input: ExternalObjectInput) => call<ExternalObject>("create_external_object", { input }),
  deleteExternalObject: (id: string) => call<void>("delete_external_object", { id }),
  previewExternalObjectRecords: (objectKey: string) =>
    callAdminAction<unknown[]>("preview_external_object_records", { objectKey }, "GET", `/api/admin/external-objects/${encodeURIComponent(objectKey)}/preview`),

  listIntegrationJobs: () => call<IntegrationJob[]>("list_integration_jobs"),
  createIntegrationJob: (input: IntegrationJobInput) => call<IntegrationJob>("create_integration_job", { input }),
  updateIntegrationJob: (id: string, input: IntegrationJobInput, status: string) =>
    call<IntegrationJob>("update_integration_job", { id, input, status }),
  deleteIntegrationJob: (id: string) => call<void>("delete_integration_job", { id }),
  listIntegrationJobRuns: (jobId: string, limit: number) => call<IntegrationJobRun[]>("list_integration_job_runs", { jobId, limit }),
  runIntegrationJobNow: (id: string) =>
    callAdminAction<IntegrationJobRun>("run_integration_job_now", { id }, "POST", `/api/admin/jobs/${encodeURIComponent(id)}/run`),

  getIntegrationOverview: () => call<IntegrationOverview>("get_integration_overview"),
  listIntegrationExecutions: (query: IntegrationExecutionQuery) =>
    call<IntegrationExecution[]>("list_integration_executions", { query }),
  getIntegrationSettings: () => call<IntegrationSettings>("get_integration_settings"),
  updateIntegrationSettings: (input: IntegrationSettingsUpdate) =>
    call<IntegrationSettings>("update_integration_settings", { input }),
  purgeExpiredIntegrationLogs: () => call<number>("purge_expired_integration_logs"),
};
