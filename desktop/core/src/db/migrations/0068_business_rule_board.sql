-- UX/UI Modernization, Business Rule Board 2.0 (issue #194): purely additive
-- columns so the board can visually chain an IF rule with its ELSE IF/ELSE
-- siblings as one lane set. `branch_group_id` is NULL for every rule created
-- before this migration (and for any ordinary unbranched rule after it) -
-- `business_rule_service::conditions_match` and friends never read either
-- column, so no existing rule's evaluation changes.
ALTER TABLE business_rules ADD COLUMN branch_group_id TEXT;
ALTER TABLE business_rules ADD COLUMN branch_role TEXT NOT NULL DEFAULT 'if';

CREATE INDEX idx_business_rules_branch_group ON business_rules(branch_group_id);
