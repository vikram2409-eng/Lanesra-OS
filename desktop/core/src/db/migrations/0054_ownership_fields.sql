-- Enterprise Access Foundation, Phase 1 continued: the mandatory system
-- fields (spec §2.2) for every User/Team-owned object - companies,
-- contacts, opportunities, products, quotes, orders, invoices, contracts,
-- tasks, and the shared custom_records table (covering every Custom
-- Object at once, since they all live in that one physical table).
--
-- record_owner_type/record_owner_id are the new canonical owner pointer,
-- generalizing "owner is always a user" to "owner is a User OR a Work
-- Team". The five tables that already have owner_user_id (companies,
-- opportunities, contracts, tasks, custom_records) keep that column
-- unchanged and untouched - it is NOT dropped or renamed (SQLite can't do
-- that safely without a full table rebuild) - and this migration backfills
-- record_owner_type='USER'/record_owner_id=owner_user_id from it below.
-- owner_user_id becomes a read-only legacy shadow of record_owner_id from
-- here on; nothing new should write to it.
--
-- owning_org_unit_id is the record's organizational security home,
-- independent of who currently owns it. Every existing record backfills to
-- its own workspace's root Organization Unit, so the column is never NULL
-- for a pre-existing record even though the schema itself allows NULL
-- (matches this table's own existing nullable-FK convention elsewhere).
--
-- ownership_version counts how many times a record's owner has actually
-- been set, not how many rows exist - it defaults to 0 (never assigned),
-- and ownership_repo::set_owner's `+ 1` on every call is what makes the
-- very first assignment (the default-owner-on-create call every
-- *_service::create makes) land on 1, with each later explicit reassignment
-- incrementing from there. A DEFAULT of 1 here would have made every
-- brand-new, still-unowned record look like it had already been
-- transferred once.
ALTER TABLE companies ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE companies ADD COLUMN record_owner_id TEXT;
ALTER TABLE companies ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE companies ADD COLUMN assigned_at TEXT;
ALTER TABLE companies ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE contacts ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE contacts ADD COLUMN record_owner_id TEXT;
ALTER TABLE contacts ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE contacts ADD COLUMN assigned_at TEXT;
ALTER TABLE contacts ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE opportunities ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE opportunities ADD COLUMN record_owner_id TEXT;
ALTER TABLE opportunities ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE opportunities ADD COLUMN assigned_at TEXT;
ALTER TABLE opportunities ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE products ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE products ADD COLUMN record_owner_id TEXT;
ALTER TABLE products ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE products ADD COLUMN assigned_at TEXT;
ALTER TABLE products ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE quotes ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE quotes ADD COLUMN record_owner_id TEXT;
ALTER TABLE quotes ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE quotes ADD COLUMN assigned_at TEXT;
ALTER TABLE quotes ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE orders ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE orders ADD COLUMN record_owner_id TEXT;
ALTER TABLE orders ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE orders ADD COLUMN assigned_at TEXT;
ALTER TABLE orders ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE invoices ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE invoices ADD COLUMN record_owner_id TEXT;
ALTER TABLE invoices ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE invoices ADD COLUMN assigned_at TEXT;
ALTER TABLE invoices ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE contracts ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE contracts ADD COLUMN record_owner_id TEXT;
ALTER TABLE contracts ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE contracts ADD COLUMN assigned_at TEXT;
ALTER TABLE contracts ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE tasks ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE tasks ADD COLUMN record_owner_id TEXT;
ALTER TABLE tasks ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE tasks ADD COLUMN assigned_at TEXT;
ALTER TABLE tasks ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

ALTER TABLE custom_records ADD COLUMN record_owner_type TEXT CHECK (record_owner_type IN ('USER', 'TEAM'));
ALTER TABLE custom_records ADD COLUMN record_owner_id TEXT;
ALTER TABLE custom_records ADD COLUMN owning_org_unit_id TEXT REFERENCES org_units(id);
ALTER TABLE custom_records ADD COLUMN assigned_at TEXT;
ALTER TABLE custom_records ADD COLUMN ownership_version INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_companies_owning_org_unit ON companies(owning_org_unit_id);
CREATE INDEX idx_companies_record_owner ON companies(record_owner_type, record_owner_id);
CREATE INDEX idx_contacts_owning_org_unit ON contacts(owning_org_unit_id);
CREATE INDEX idx_contacts_record_owner ON contacts(record_owner_type, record_owner_id);
CREATE INDEX idx_opportunities_owning_org_unit ON opportunities(owning_org_unit_id);
CREATE INDEX idx_opportunities_record_owner ON opportunities(record_owner_type, record_owner_id);
CREATE INDEX idx_products_owning_org_unit ON products(owning_org_unit_id);
CREATE INDEX idx_products_record_owner ON products(record_owner_type, record_owner_id);
CREATE INDEX idx_quotes_owning_org_unit ON quotes(owning_org_unit_id);
CREATE INDEX idx_quotes_record_owner ON quotes(record_owner_type, record_owner_id);
CREATE INDEX idx_orders_owning_org_unit ON orders(owning_org_unit_id);
CREATE INDEX idx_orders_record_owner ON orders(record_owner_type, record_owner_id);
CREATE INDEX idx_invoices_owning_org_unit ON invoices(owning_org_unit_id);
CREATE INDEX idx_invoices_record_owner ON invoices(record_owner_type, record_owner_id);
CREATE INDEX idx_contracts_owning_org_unit ON contracts(owning_org_unit_id);
CREATE INDEX idx_contracts_record_owner ON contracts(record_owner_type, record_owner_id);
CREATE INDEX idx_tasks_owning_org_unit ON tasks(owning_org_unit_id);
CREATE INDEX idx_tasks_record_owner ON tasks(record_owner_type, record_owner_id);
CREATE INDEX idx_custom_records_owning_org_unit ON custom_records(owning_org_unit_id);
CREATE INDEX idx_custom_records_record_owner ON custom_records(record_owner_type, record_owner_id);

-- Backfill record_owner_type/id from the legacy owner_user_id column on
-- the five tables that already had one - each counts as one real
-- assignment, so ownership_version becomes 1 rather than staying at the
-- unassigned default of 0.
UPDATE companies SET record_owner_type = 'USER', record_owner_id = owner_user_id, assigned_at = created_at, ownership_version = 1 WHERE owner_user_id IS NOT NULL;
UPDATE opportunities SET record_owner_type = 'USER', record_owner_id = owner_user_id, assigned_at = created_at, ownership_version = 1 WHERE owner_user_id IS NOT NULL;
UPDATE contracts SET record_owner_type = 'USER', record_owner_id = owner_user_id, assigned_at = created_at, ownership_version = 1 WHERE owner_user_id IS NOT NULL;
UPDATE tasks SET record_owner_type = 'USER', record_owner_id = owner_user_id, assigned_at = created_at, ownership_version = 1 WHERE owner_user_id IS NOT NULL;
UPDATE custom_records SET record_owner_type = 'USER', record_owner_id = owner_user_id, assigned_at = created_at, ownership_version = 1 WHERE owner_user_id IS NOT NULL;

-- Backfill owning_org_unit_id on every existing record, all 10 tables, to
-- its own workspace's root Organization Unit (guaranteed to exist as of
-- 0051_organization_and_org_units.sql).
UPDATE companies SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = companies.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE contacts SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = contacts.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE opportunities SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = opportunities.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE products SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = products.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE quotes SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = quotes.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE orders SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = orders.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE invoices SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = invoices.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE contracts SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = contracts.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE tasks SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = tasks.workspace_id) WHERE owning_org_unit_id IS NULL;
UPDATE custom_records SET owning_org_unit_id = (SELECT w.root_org_unit_id FROM workspaces w WHERE w.id = custom_records.workspace_id) WHERE owning_org_unit_id IS NULL;
