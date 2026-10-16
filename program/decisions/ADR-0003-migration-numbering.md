# ADR-0003: Migration numbers are taken at rebase time and must exceed main's maximum

- Status: Proposed (the full migration conventions ADR comes in WP-0009 and builds on this)
- Date: 2026-10-16
- Work package: WP-0001; enforced by WP-0003 and WP-0007
- Requirements: REQ-PRE-005, REQ-PRE-007

## Context

`desktop/core/src/db/migrate.rs` applies every listed migration whose version is greater than
`MAX(version)` in `schema_migrations`. If migration 0081 merges before 0080, any database that
applied 0081 will never apply 0080 — silently. With several agents adding migrations in parallel,
numbers reserved in advance would merge out of order.

## Decision

1. Never reserve migration numbers in advance.
2. When rebasing a branch on `main` before merge, a new migration takes the next number above the
   highest on `main` and is appended to `MIGRATIONS` in `migrate.rs`.
3. One migration file per pull request where possible; additive changes only, unless an ADR
   approves a destructive one with a data-preserving plan.
4. CI (WP-0003) fails a PR whose new migration number is not greater than `main`'s maximum, or whose
   numbers are not contiguous, or whose file is not listed in `MIGRATIONS`.
5. The upgrade harness (WP-0007) applies all migrations to databases from the last release.

## Consequences

- Agents may need to renumber on rebase; this is a mechanical, expected step (PROTOCOL §11).
- Out-of-order loss becomes impossible on `main`.
