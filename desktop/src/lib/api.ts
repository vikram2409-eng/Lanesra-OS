import { invoke } from "@tauri-apps/api/core";
import type {
  AppErrorPayload,
  Company,
  CompanyInput,
  Contact,
  ContactInput,
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
  User,
  Workspace,
  WorkspaceSetup,
} from "./types";

export class ApiError extends Error {
  kind: AppErrorPayload["kind"];
  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.kind = payload.kind;
  }
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    if (err && typeof err === "object" && "kind" in err && "message" in err) {
      throw new ApiError(err as AppErrorPayload);
    }
    throw err;
  }
}

export const api = {
  workspaceStatus: () => call<Workspace | null>("workspace_status"),
  firstRunSetup: (setup: WorkspaceSetup) => call<[Workspace, User]>("first_run_setup", { setup }),

  login: (credentials: Credentials) => call<User>("login", { credentials }),
  logout: () => call<void>("logout"),
  currentUser: () => call<User | null>("current_user"),

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
};
