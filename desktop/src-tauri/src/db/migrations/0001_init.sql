-- Lanesra OS Desktop - initial schema
-- Money stored as integer minor units (cents). Quantities/rates stored as
-- integers scaled by 1000 (milli) or 10000 (basis points) to avoid floating
-- point in persisted totals (BR-014). Foreign keys enforced at the
-- connection level (BR-016), not just in this file.

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    business_name TEXT NOT NULL,
    legal_name TEXT,
    currency_code TEXT NOT NULL DEFAULT 'USD',
    locale TEXT NOT NULL DEFAULT 'en-US',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    default_tax_rate_bp INTEGER NOT NULL DEFAULT 0,
    operating_mode TEXT NOT NULL DEFAULT 'Personal' CHECK (operating_mode IN ('Personal', 'Team')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    username TEXT NOT NULL,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (workspace_id, username)
);

CREATE TABLE roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE CHECK (name IN ('Administrator', 'Manager', 'Sales', 'Finance', 'ReadOnly'))
);

CREATE TABLE user_roles (
    user_id TEXT NOT NULL REFERENCES users(id),
    role_id TEXT NOT NULL REFERENCES roles(id),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE number_sequences (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    prefix TEXT NOT NULL,
    period_key TEXT NOT NULL DEFAULT '',
    next_value INTEGER NOT NULL DEFAULT 1,
    UNIQUE (workspace_id, entity_type, period_key)
);

CREATE TABLE companies (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    customer_number TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('Prospect', 'Active Customer', 'Inactive', 'Archived')),
    owner_user_id TEXT REFERENCES users(id),
    tax_number TEXT,
    billing_address TEXT,
    shipping_address TEXT,
    tags TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_companies_workspace ON companies(workspace_id);
CREATE INDEX idx_companies_status ON companies(status);
CREATE INDEX idx_companies_name ON companies(name);

CREATE TABLE contacts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    contact_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    job_title TEXT,
    email TEXT,
    phone TEXT,
    mobile TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('Active', 'Inactive', 'Archived')),
    tags TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_contacts_company ON contacts(company_id);
CREATE INDEX idx_contacts_email ON contacts(email);

CREATE TABLE products (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    product_number TEXT NOT NULL UNIQUE,
    sku TEXT,
    type TEXT NOT NULL CHECK (type IN ('Product', 'Service')),
    name TEXT NOT NULL,
    category TEXT,
    description TEXT,
    unit_price_cents INTEGER NOT NULL DEFAULT 0,
    cost_cents INTEGER NOT NULL DEFAULT 0,
    tax_rate_bp INTEGER NOT NULL DEFAULT 0,
    default_quantity_milli INTEGER NOT NULL DEFAULT 1000,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_products_workspace ON products(workspace_id);

CREATE TABLE opportunities (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    opportunity_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    primary_contact_id TEXT REFERENCES contacts(id),
    name TEXT NOT NULL,
    stage TEXT NOT NULL CHECK (stage IN ('New', 'Qualified', 'Discovery', 'Proposal', 'Negotiation', 'Won', 'Lost')),
    status TEXT NOT NULL CHECK (status IN ('Open', 'Won', 'Lost', 'Archived')),
    value_cents INTEGER NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL,
    probability_bp INTEGER NOT NULL DEFAULT 0,
    expected_close_date TEXT,
    owner_user_id TEXT REFERENCES users(id),
    lost_reason TEXT,
    next_step TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_opportunities_company ON opportunities(company_id);
CREATE INDEX idx_opportunities_stage ON opportunities(stage);
CREATE INDEX idx_opportunities_status ON opportunities(status);

CREATE TABLE opportunity_products (
    id TEXT PRIMARY KEY,
    opportunity_id TEXT NOT NULL REFERENCES opportunities(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES products(id),
    quantity_milli INTEGER NOT NULL DEFAULT 1000,
    unit_price_cents INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_opportunity_products_opportunity ON opportunity_products(opportunity_id);

CREATE TABLE quotes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    quote_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    contact_id TEXT REFERENCES contacts(id),
    opportunity_id TEXT REFERENCES opportunities(id),
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Sent', 'Viewed', 'Accepted', 'Rejected', 'Expired', 'Cancelled')),
    currency_code TEXT NOT NULL,
    subtotal_cents INTEGER NOT NULL DEFAULT 0,
    discount_cents INTEGER NOT NULL DEFAULT 0,
    tax_cents INTEGER NOT NULL DEFAULT 0,
    total_cents INTEGER NOT NULL DEFAULT 0,
    issue_date TEXT,
    expiry_date TEXT,
    notes TEXT,
    terms TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_quotes_company ON quotes(company_id);
CREATE INDEX idx_quotes_opportunity ON quotes(opportunity_id);
CREATE INDEX idx_quotes_status ON quotes(status);

CREATE TABLE quote_lines (
    id TEXT PRIMARY KEY,
    quote_id TEXT NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id),
    description TEXT NOT NULL,
    quantity_milli INTEGER NOT NULL DEFAULT 1000,
    unit_price_cents INTEGER NOT NULL DEFAULT 0,
    discount_bp INTEGER NOT NULL DEFAULT 0,
    tax_rate_bp INTEGER NOT NULL DEFAULT 0,
    line_total_cents INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_quote_lines_quote ON quote_lines(quote_id);

CREATE TABLE orders (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    order_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    contact_id TEXT REFERENCES contacts(id),
    source_quote_id TEXT REFERENCES quotes(id),
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Confirmed', 'Processing', 'Partially Fulfilled', 'Fulfilled', 'Cancelled')),
    currency_code TEXT NOT NULL,
    subtotal_cents INTEGER NOT NULL DEFAULT 0,
    discount_cents INTEGER NOT NULL DEFAULT 0,
    tax_cents INTEGER NOT NULL DEFAULT 0,
    total_cents INTEGER NOT NULL DEFAULT 0,
    order_date TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_orders_company ON orders(company_id);
CREATE INDEX idx_orders_source_quote ON orders(source_quote_id);
CREATE INDEX idx_orders_status ON orders(status);

CREATE TABLE order_lines (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id),
    description TEXT NOT NULL,
    quantity_milli INTEGER NOT NULL DEFAULT 1000,
    unit_price_cents INTEGER NOT NULL DEFAULT 0,
    discount_bp INTEGER NOT NULL DEFAULT 0,
    tax_rate_bp INTEGER NOT NULL DEFAULT 0,
    line_total_cents INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_order_lines_order ON order_lines(order_id);

CREATE TABLE invoices (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    invoice_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    contact_id TEXT REFERENCES contacts(id),
    source_order_id TEXT REFERENCES orders(id),
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Issued', 'Partially Paid', 'Paid', 'Overdue', 'Void', 'Cancelled')),
    currency_code TEXT NOT NULL,
    subtotal_cents INTEGER NOT NULL DEFAULT 0,
    discount_cents INTEGER NOT NULL DEFAULT 0,
    tax_cents INTEGER NOT NULL DEFAULT 0,
    total_cents INTEGER NOT NULL DEFAULT 0,
    amount_paid_cents INTEGER NOT NULL DEFAULT 0,
    balance_cents INTEGER NOT NULL DEFAULT 0,
    issue_date TEXT,
    due_date TEXT,
    payment_terms TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_invoices_company ON invoices(company_id);
CREATE INDEX idx_invoices_source_order ON invoices(source_order_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);

CREATE TABLE invoice_lines (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    product_id TEXT REFERENCES products(id),
    description TEXT NOT NULL,
    quantity_milli INTEGER NOT NULL DEFAULT 1000,
    unit_price_cents INTEGER NOT NULL DEFAULT 0,
    discount_bp INTEGER NOT NULL DEFAULT 0,
    tax_rate_bp INTEGER NOT NULL DEFAULT 0,
    line_total_cents INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_invoice_lines_invoice ON invoice_lines(invoice_id);

CREATE TABLE payments (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL REFERENCES invoices(id),
    amount_cents INTEGER NOT NULL,
    paid_at TEXT NOT NULL,
    method TEXT,
    reference TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_payments_invoice ON payments(invoice_id);

-- Contracts: schema present for architectural completeness; no service/UI
-- layer yet (deferred to a later phase). Deliberately has no opportunity_id
-- column - FR-CTR-03 / FR-OPP-06 / BR-009 prohibit that relationship.
CREATE TABLE contracts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    contract_number TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL REFERENCES companies(id),
    contact_id TEXT REFERENCES contacts(id),
    source_quote_id TEXT REFERENCES quotes(id),
    title TEXT NOT NULL,
    type TEXT,
    value_cents INTEGER NOT NULL DEFAULT 0,
    currency_code TEXT NOT NULL,
    owner_user_id TEXT REFERENCES users(id),
    start_date TEXT,
    end_date TEXT,
    renewal_date TEXT,
    notice_period_days INTEGER,
    status TEXT NOT NULL CHECK (status IN ('Draft', 'Under Review', 'Active', 'Expiring', 'Renewed', 'Expired', 'Terminated')),
    notes TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_contracts_company ON contracts(company_id);

-- Tasks: schema present for architectural completeness; no service/UI layer
-- yet (deferred to a later phase).
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    task_number TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    owner_user_id TEXT REFERENCES users(id),
    priority TEXT NOT NULL CHECK (priority IN ('Low', 'Normal', 'High', 'Urgent')),
    status TEXT NOT NULL CHECK (status IN ('Not Started', 'In Progress', 'Waiting', 'Completed', 'Cancelled')),
    due_date TEXT,
    reminder_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    updated_at TEXT NOT NULL,
    updated_by TEXT REFERENCES users(id),
    archived_at TEXT
);
CREATE INDEX idx_tasks_owner ON tasks(owner_user_id);

CREATE TABLE task_links (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    related_type TEXT CHECK (related_type IN ('Company', 'Contact', 'Opportunity', 'Quote', 'Order', 'Invoice', 'Contract')),
    related_id TEXT
);
CREATE INDEX idx_task_links_task ON task_links(task_id);
CREATE INDEX idx_task_links_related ON task_links(related_type, related_id);

CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    related_type TEXT,
    related_id TEXT,
    activity_type TEXT NOT NULL,
    subject TEXT,
    notes TEXT,
    occurred_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL
);
CREATE INDEX idx_activities_related ON activities(related_type, related_id);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER,
    checksum TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT REFERENCES users(id)
);
CREATE INDEX idx_attachments_entity ON attachments(entity_type, entity_id);

CREATE TABLE audit_events (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    occurred_at TEXT NOT NULL,
    user_id TEXT REFERENCES users(id),
    event_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    summary TEXT NOT NULL,
    details_json TEXT
);
CREATE INDEX idx_audit_events_entity ON audit_events(entity_type, entity_id);
CREATE INDEX idx_audit_events_occurred ON audit_events(occurred_at);

CREATE TABLE settings (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    key TEXT NOT NULL,
    value TEXT,
    UNIQUE (workspace_id, key)
);
