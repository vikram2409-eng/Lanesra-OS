-- Lets an Administrator edit the workspace profile after first-run and add
-- a logo, both reflected on the printed quote/order/invoice letterhead
-- (FR-BRD). Previously business_name/legal_name were set once at first-run
-- and never editable again, and there was no business address or logo at
-- all - only each Company's own billing address.

ALTER TABLE workspaces ADD COLUMN business_address TEXT;
ALTER TABLE workspaces ADD COLUMN logo_base64 TEXT;
ALTER TABLE workspaces ADD COLUMN logo_mime TEXT;
