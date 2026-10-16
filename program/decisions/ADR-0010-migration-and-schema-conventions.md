# ADR-0010: Migration and schema conventions

- Status: Proposed. Accepted when the owner merges the WP-0009 pull request.
- Date: 2026-10-16
- Work package: WP-0009 (extends ADR-0003)
- Requirements: REQ-PRE-005, REQ-PRE-007
- Constrains: every work package that adds a migration (WP-0007, WP-0100, WP-0110, WP-0111, WP-0141, WP-0146, WP-0170, and many later)

## Context

ADR-0003 fixed migration **numbering**: take the next number at rebase, and CI enforces it. This ADR fixes what goes **inside** a migration.

Three facts about the code shape the rules:

1. `run_migrations` (`desktop/core/src/db/migrate.rs:177-202`) runs each file with `execute_batch` and then records its version, **without a transaction**. A migration that fails halfway leaves the schema partly changed and unrecorded.
2. 79 migrations mostly follow one style:
   - `id TEXT PRIMARY KEY` holding a UUID v4 (`domain/ids.rs`);
   - `workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE`;
   - timestamps are RFC 3339 strings, `created_at TEXT NOT NULL` (105 tables) and `updated_at TEXT NOT NULL` (75);
   - `created_by TEXT`, sometimes `REFERENCES users(id)`;
   - JSON in `TEXT` columns, usually suffixed `_json`, with defaults `'{}'` or `'[]'`;
   - 73 `CHECK (... IN (...))` enums;
   - 188 indexes named `idx_…`.

   It is not uniform, though: `trigger_input` holds JSON but has no `_json` suffix, and `created_by` sometimes has no foreign key.
3. WP-0100 introduces principals (humans, Agent Identities and API clients), so a `created_by` that can reference only `users(id)` will not fit agent-made rows.

## Decision

### 1. Every migration file

- **Name:** `NNNN_snake_case_summary.sql`. The number is set by ADR-0003.
- **Header:** the file starts with this comment block. `check_migrations.py` will check it for migrations numbered above 0079 (added in WP-0007):

```sql
-- Migration 0081: Run lease and stop reason (ADR-0007)
-- Work package: WP-0110
-- Change: additive | data | destructive (destructive needs an ADR named here)
-- Data touched: none | <tables and how many rows, at most>
-- Re-run safe: yes | no (data migrations must be yes)
```

- **One concern per file**, and one file per pull request where possible (ADR-0003).
- **Never edit a migration after it has merged.** Fix it with a new one.
- **Data migrations** are idempotent: running them twice leaves the same data. They are written as `UPDATE ... WHERE <not yet migrated>`.
- **Destructive changes** (dropping or renaming a column or table, narrowing a type) need an ADR. That ADR must name the data-preserving steps: copy the data, verify it, then drop it in a later release.

### 2. The runner (changed in WP-0007)

Each migration and its `schema_migrations` row run inside **one transaction** (`BEGIN IMMEDIATE ... COMMIT`), so a failed migration leaves no trace and the version is not recorded. Migrations that SQLite cannot run inside a transaction, such as `VACUUM`, are not allowed.

### 3. Table conventions for new tables

| Column | Rule |
|---|---|
| `id` | `TEXT PRIMARY KEY`, a UUID from `domain::ids::new_uuid()` |
| `workspace_id` | `TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE`, on every workspace-owned table. Global tables say why in the header |
| `created_at`, `updated_at` | `TEXT NOT NULL`, RFC 3339 UTC from `domain::ids::now_iso()` |
| `created_by_principal_id` | `TEXT`, nullable only for system-made rows. After WP-0100 it references `principals(id)`. New tables use this name, not `created_by` |
| Status and kind columns | `TEXT NOT NULL` with a `CHECK (col IN (...))` matching the Rust constant, and a unit test that the two lists match. Run and step status columns use exactly the ADR-0007 lists |
| JSON | `TEXT NOT NULL DEFAULT '{}'` or `'[]'`, named `<thing>_json`, parsed into a typed Rust struct (never handled as a raw `Value` outside the repository). No `json_extract` in a hot path without an index on a generated column |
| Money | `cost_usd REAL` for display, plus `cost_micros INTEGER` (millionths of a dollar) for sums, so totals never drift |
| Foreign keys | Always declared. `ON DELETE CASCADE` only for child rows that have no meaning alone; otherwise `SET NULL` or restrict |
| Indexes | `idx_<table>_<columns>`. Add one for every foreign key used in a lookup, and for every `(workspace_id, status)` or `(workspace_id, created_at)` filter a screen uses |
| Secrets | Never stored in plain text. Use `integration_secrets` (AES-256-GCM) and reference it by id |

### 4. Tests every migration brings

- The upgrade harness (WP-0007) applies it to a database from the last release that holds fixture data, and checks that no rows were lost.
- If the migration rewrites data, a test asserts the before and after rows.
- The `migrate.rs` unit test's expected version is bumped. Missing this already fails CI today.

## Example

```sql
-- Migration 0081: Run lease and stop reason (ADR-0007)
-- Work package: WP-0110
-- Change: additive + data
-- Data touched: ai_runs rows whose status is 'waiting_tool' or 'waiting_agent' (expected 0)
-- Re-run safe: yes
ALTER TABLE ai_runs ADD COLUMN stop_reason TEXT;
ALTER TABLE ai_runs ADD COLUMN lease_owner TEXT;
ALTER TABLE ai_runs ADD COLUMN lease_expires_at TEXT;
UPDATE ai_runs SET status = 'paused', stop_reason = 'interrupted'
 WHERE status IN ('waiting_tool', 'waiting_agent');
CREATE INDEX idx_ai_runs_status_lease ON ai_runs (status, lease_expires_at);
```

## Consequences

- A failed upgrade can be retried safely, because it leaves nothing half-applied.
- Reviewers have a checklist. CI checks the header, numbering and listing; people check the rest.
- Older tables keep their names (`created_by`, `trigger_input`). They are renamed only when a work package touches them for another reason, with a data-preserving migration.
