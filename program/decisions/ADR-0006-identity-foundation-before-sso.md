# ADR-0006: Build the identity foundation, including Agent Identities, first in Phase 1

- Status: Accepted (program owner, 2026-10-16)
- Changes the spec's decision "OIDC and SAML built first, in phase 1, before Agent Identities ship"
- Requirements: REQ-GOV-002 and REQ-GOV-003 move from Phase 3 to Phase 1; REQ-GOV-029 added
- Work packages: new WP-0100; WP-0318 reduced to team-level roles; WP-0101, WP-0110, WP-0113 now depend on WP-0100

## Context

The owner asked whether agents need credentials like humans, used to grant access, and whether
that should come before SAML. In the code today agents have no identity: an agent acts as the
human who started it, and an unattended run acts as the owner of the API client that triggered it
(`ai_orchestration_service.rs:15-19`), so an admin-scoped automation runs with a human
Administrator's rights and is audited in that person's name. Almost every Phase 1 work package
(run model, spans, audit, approvals, budgets) records who acted; building them on human-only
identity and adding agents in Phase 3 would mean reworking them.

## Decision

1. WP-0100 (principal model, Agent Identities, agent credentials) is the first Phase 1 build
   package, right after the foundational ADRs (WP-0009).
2. Agents get the same *access model* as humans — a principal with an Access Role and Record
   Scope, granted and revoked by admins, audited in its own name — but not the same *login*:
   Agent Identities never use passwords, OIDC or SAML; they use Lanesra-issued, short-lived, scoped
   credentials that the platform holds and never passes to a model.
3. OIDC and SAML (WP-0101 to WP-0105) follow and map directory users onto the same principal model.
4. Team-level identity features stay in Phase 3 (WP-0318).

## Consequences

- Phase 1 grows by an XL package (split at refinement); the critical path now starts
  WP-0008 → WP-0009 → WP-0100 → WP-0110.
- Unattended automations are migrated to Agent Identities with the narrowest role that keeps them
  working; admins see what changed.
