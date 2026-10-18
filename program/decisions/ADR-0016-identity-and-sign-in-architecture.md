# ADR-0016: Identity and sign-in architecture (OIDC/SAML)

- Status: Proposed (awaiting owner and Claude review before WP-0107/WP-0108/WP-0109 build)
- Date: 2026-10-17
- Work package: WP-0101
- Requirements: REQ-GOV-001
- Follows: ADR-0006 (identity foundation, principals), ADR-0014 (one database and data key per workspace)
- Threats: T-03 (excessive agency), T-11 (account takeover), T-12 (cross-workspace data access)
- Editions: Desktop, Team Workspace, Cloud SaaS (future)

## Context

The owner requires SSO on both Desktop and Team Workspace (ADR-0004, Q-07). Desktop is a single-user Tauri app; Team Workspace is a multi-user self-hosted server. A future Cloud SaaS edition will need the same identity foundation.

ADR-0006 established that agents get Agent Identities (WP-0100) with the same access model as humans but different credentials. OIDC/SAML (WP-0101 to WP-0105) must map directory users onto the same principal model.

ADR-0014 established: one database and one data key per workspace. No row, key, file, event or cache entry may be shared between workspaces, except account identity.

## Decision

### 1. Principal model (from ADR-0006)

Every actor — human user or Agent Identity — is a **principal** with:
- An Access Role (capability set)
- A Record Scope (Owner, Team, Org Unit and below, Organization)
- An audit identity (actions recorded in the principal's own name)

Human principals authenticate via:
- Local password (with rate limiting, lockout per T-11)
- OIDC (Okta, Microsoft Entra ID, Google)
- SAML (Okta, Microsoft Entra ID)

Agent principals (WP-0108) authenticate via Lanesra-issued short-lived scoped credentials. They never use passwords, OIDC or SAML.

### 2. Session model

**Team Workspace / Cloud SaaS (server):**
- Sessions are server-side, stored in the workspace database
- Session cookie: `HttpOnly`, `Secure`, `SameSite=Lax` (T-11)
- CSRF protection on all state-changing endpoints (T-11)
- Session revocation: admin can revoke any session; user can revoke own sessions
- Idle timeout: 30 minutes; absolute timeout: 8 hours (configurable)

**Desktop (Tauri):**
- Single user, no server session
- OIDC flow uses system browser + loopback redirect (RFC 8252)
- Tokens stored in OS keychain (not in SQLite)
- No session cookie; the app holds tokens in memory

### 3. OIDC/SAML provider configuration

**Supported providers:**
- OIDC: Okta, Microsoft Entra ID, Google
- SAML: Okta, Microsoft Entra ID
- Test: Keycloak (local, for CI)

**Configuration (per workspace, admin-only):**
- Provider type (OIDC/SAML), issuer URL, client ID
- SAML: IdP metadata URL or XML
- Attribute mapping: directory `email` → user email, `name` → display name, `groups` → role mapping

**Account linking:**
- Existing local user with matching email is linked on first SSO sign-in (with confirmation)
- New SSO user creates a new local user with default Standard User role
- Break-glass local admin account always exists (cannot be disabled via SSO)

### 4. Directory-to-principal mapping

When a user signs in via OIDC/SAML:
1. Verify token signature against provider JWKS/metadata
2. Extract email, name, groups from claims/assertion
3. Find or create local user by email (within the workspace)
4. Map groups to Access Roles via admin-configured mapping table
5. Create session for the principal

Group-to-role mapping is explicit (admin configures which directory group maps to which Access Role). No implicit admin grant from directory membership.

### 5. Tenancy and isolation (ADR-0014, T-12)

- Each workspace has its own database and data key
- User accounts are per-workspace (no shared user table across workspaces)
- SSO provider configuration is per-workspace
- A user in Workspace A cannot access Workspace B, even with the same email
- Session tokens are scoped to the workspace database that issued them

**Exception:** Cloud SaaS (future) may have a central account directory for billing, but workspace data isolation remains absolute. This ADR does not decide the SaaS account model.

### 6. Threat mitigations

**T-11 (Account takeover):**
- Rate limiting on password attempts (5 failures → 15 min lockout)
- Session cookie flags (HttpOnly, Secure, SameSite)
- CSRF tokens on state-changing requests
- SAML: verify assertion signature, check `NotBefore`/`NotOnOrAfter`, validate audience
- OIDC: verify JWT signature, issuer, audience, expiry; use `nonce` for replay protection

**T-12 (Cross-workspace access):**
- One database per workspace (ADR-0014)
- Sessions bound to workspace; token from Workspace A rejected by Workspace B
- No shared session store

**T-03 (Excessive agency):**
- SSO maps to human principals only, never to Agent Identities
- Agent credentials are Lanesra-issued, never derived from SSO tokens
- An agent cannot escalate via SSO; its permissions come from its Access Role (WP-0108)

## Consequences

- WP-0102 (OIDC) and WP-0103 (SAML) build on this ADR
- WP-0105 (Desktop SSO) uses system browser + loopback per §2
- WP-0106 (sign-in hardening) implements T-11 mitigations
- The principal model (ADR-0006) is unchanged; SSO is an authentication method, not a new principal type
- No identity code in this ADR (per WP-0101 scope)

## Open questions

- Q: Should Cloud SaaS have a central account directory, or per-workspace accounts like Team Workspace? (Deferred to SaaS planning)
- Q: SAML single logout (SLO) — required for MVP or later? (Default: later; local session revocation suffices for MVP)

## Review

Status is **Proposed**. Requires review by the owner and Claude before WP-0107, WP-0108 or WP-0109 build on it.
