-- Admin flexibility: lets an Administrator override the prefix and
-- zero-padded digit width used for a given entity's auto-generated
-- numbers (Appendix B), e.g. changing Company from "CUS-000001" to
-- "ACC-000001" or "ACC-ab0001" (the letters are just part of the chosen
-- prefix text - there is no separate alpha-segment mini-language). One row
-- per (workspace, entity_type); entity types with no row keep the
-- hardcoded default from domain::numbering. Changing the prefix does not
-- reset or renumber already-issued numbers - the sequence in
-- number_sequences (keyed by entity_type + period) is untouched, so the
-- next number picks up from wherever it left off, just formatted with the
-- new prefix/width going forward.

CREATE TABLE numbering_configs (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    entity_type TEXT NOT NULL,
    prefix TEXT NOT NULL,
    digits INTEGER NOT NULL DEFAULT 6,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (workspace_id, entity_type)
);
