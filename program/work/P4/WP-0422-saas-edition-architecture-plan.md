---
id: WP-0422
title: SaaS edition architecture plan
phase: 4
epic: E4.1 Scale
status: proposed
owner: unassigned
depends_on: [WP-0181, WP-0420]
requirements: [REQ-PRE-022]
estimate: M
editions: []
editions_reason: A plan, not a feature; the SaaS edition itself would be a later program
delivered: []
docs_reason: Planning document only; there is nothing user-facing yet
branch: 
pr: 
updated: 2026-10-16
---

# WP-0422 — SaaS edition architecture plan

## Goal

The owner plans a SaaS version of Lanesra OS (ADR-0013 #4). This program does not build it, but its foundations (ADR-0014) are designed to keep it possible. This package writes the plan that a later program would follow, based on what has actually been built by then.

## Scope

- **Tenancy model:**
  - database-per-tenant, as in Team Workspace, or a shared PostgreSQL database with row-level security on `workspace_id`;
  - weighed on cost, isolation, noisy neighbours, backup and residency.
- **Control plane:**
  - sign-up, billing hooks, plans and limits (separate from feature flags, ADR-0012 §4);
  - tenant lifecycle: create, suspend, export, delete.
- **Keys:**
  - per-tenant data keys in a KMS;
  - bring-your-own-key for enterprise tenants.
- **Operations:**
  - multi-region and data residency (builds on WP-0417);
  - worker pools (WP-0401);
  - observability per tenant;
  - incident and abuse handling.
- **Compliance path:** SOC 2 and ISO 27001 controls mapped to what exists (audit, retention, four-eyes).
- **Gap list:** everything in the code that still assumes one organisation per server, each turned into a proposed work package.

## Acceptance criteria

- [ ] AC-1: An ADR proposes the SaaS tenancy model, with the trade-offs measured on the real code
- [ ] AC-2: The gap list is captured as proposed work packages
- [ ] AC-3: The owner reviews it and decides whether, and when, to start a SaaS program

## Test plan

- Not a code change; reviewed by the owner
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0014. ADR-0010 applies to any migration this work package adds.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
