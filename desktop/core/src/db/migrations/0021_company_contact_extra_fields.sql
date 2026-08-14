-- Record-detail-page round: a handful more built-in fields on Company and
-- Contact - the two entities that were genuinely missing common CRM fields
-- (Quote/Order/Invoice already carry discount/tax/terms/payment_terms,
-- Opportunity already has next_step, Product already has description,
-- Contract already has owner_user_id/type/renewal_date/notice_period_days
-- - see their existing models). All nullable/optional, so every existing
-- row just reads back NULL until edited.
ALTER TABLE companies ADD COLUMN phone TEXT;
ALTER TABLE companies ADD COLUMN email TEXT;
ALTER TABLE companies ADD COLUMN website TEXT;
ALTER TABLE companies ADD COLUMN annual_revenue_cents INTEGER;
ALTER TABLE companies ADD COLUMN employee_count INTEGER;
ALTER TABLE companies ADD COLUMN preferred_contact_method TEXT CHECK (preferred_contact_method IS NULL OR preferred_contact_method IN ('Email', 'Phone', 'Text'));

ALTER TABLE contacts ADD COLUMN department TEXT;
ALTER TABLE contacts ADD COLUMN preferred_contact_method TEXT CHECK (preferred_contact_method IS NULL OR preferred_contact_method IN ('Email', 'Phone', 'Text'));
ALTER TABLE contacts ADD COLUMN linkedin_url TEXT;
