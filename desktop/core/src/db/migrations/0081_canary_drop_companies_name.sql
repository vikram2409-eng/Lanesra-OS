-- Migration 0081: CANARY - drops companies.name (WP-0007 AC-2 demonstration)
-- Work package: WP-0007
-- Change: destructive (canary for AC-2 test demonstration - DO NOT MERGE)
-- Data touched: companies.name column (populated in fixtures)
-- Re-run safe: no (canary - will be removed before merge)

-- This is a canary migration to prove WP-0007 AC-2: the upgrade test must fail
-- when a migration drops a populated column. This file will be removed before merge.
ALTER TABLE companies DROP COLUMN name;
