---
id: WP-XXXX
title: Short imperative title
phase: 1
epic: E1.x Epic name
status: proposed
owner: unassigned
depends_on: []
requirements: [REQ-AREA-NNN]
estimate: M
branch:
pr:
updated: YYYY-MM-DD
---

# WP-XXXX — Short imperative title

## Goal

One paragraph: what outcome, for whom, and why now.

## Scope

- What is in

## Out of scope

- What is explicitly not in (and which WP owns it)

## Acceptance criteria

- [ ] AC-1: Observable, testable statement
- [ ] AC-2: Include the deny/limit path, not only the happy path

## Test plan

- `desktop/core/tests/<capability>.rs`: which ACs it covers
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADRs this WP must follow; threats from the threat model it mitigates; feature flag name

## Progress log

<!-- Append dated entries, newest last:
### YYYY-MM-DD agent
Done: / State (green or RED and why): / Next: / Gotchas: / Commands: -->
