-- Admin UX polish (spec §10): bounded version history for Business Rules
-- and Workflow Automation. A version is read-only, append-only history -
-- never queried structurally the way the live conditions/actions tables
-- are - so it's stored as one JSON snapshot per save rather than a second
-- normalized schema mirroring business_rule_conditions/_actions and
-- workflow_conditions/_actions. Pruned to the most recent 10 per rule at
-- write time (see business_rule_repo::insert_version /
-- workflow_repo::insert_version), so this can't grow unbounded.

CREATE TABLE business_rule_versions (
    id TEXT PRIMARY KEY,
    business_rule_id TEXT NOT NULL REFERENCES business_rules(id) ON DELETE CASCADE,
    snapshot_json TEXT NOT NULL,
    saved_at TEXT NOT NULL
);
CREATE INDEX idx_business_rule_versions_rule ON business_rule_versions(business_rule_id, saved_at);

CREATE TABLE workflow_rule_versions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    snapshot_json TEXT NOT NULL,
    saved_at TEXT NOT NULL
);
CREATE INDEX idx_workflow_rule_versions_workflow ON workflow_rule_versions(workflow_id, saved_at);
