-- Admin flexibility: a business phone number alongside the existing
-- address/logo branding fields, and an admin-editable ordered list of
-- which Dashboard KPI tiles to show (dashboard_kpi_prefs stores a JSON
-- array of KPI keys, e.g. '["open_pipeline","won_revenue"]'; NULL means
-- "show all, in the default order" so existing workspaces are unaffected).

ALTER TABLE workspaces ADD COLUMN phone TEXT;
ALTER TABLE workspaces ADD COLUMN dashboard_kpi_prefs TEXT;
