# ADR-0016: Identity and sign-in architecture (OIDC/SAML)

- Status: Proposed. Drafted by Muse (PR #670), revised by Claude after review (2026-10-18). Awaiting owner acceptance before WP-0107, WP-0108 or WP-0109 build on it.
- Date: 2026-10-17, revised 2026-10-18
- Work package: WP-0101
- Requirements: REQ-GOV-001, REQ-GOV-028, REQ-GOV-029
- Follows: ADR-0006 (identity foundation, principals), ADR-0014 (control database, one database and data key per workspace)
- Threats: T-03 (excessive agency), T-11 (account takeover), T-12 (cross-workspace data access), T-18 (audit tampering)
- Editions: Desktop, Team Workspace; a future Cloud SaaS edition builds on the same model (ADR-0014 §7)

## Context

The owner requires enterprise sign-in on both Desktop and Team Workspace (ADR-0004, REQ-GOV-001, REQ-GOV-028). Desktop is a single-user Tauri app. Team Workspace is a self-hosted server that will host several workspaces (ADR-0014). A future Cloud SaaS edition needs the same foundation.

ADR-0006: humans, Agent Identities and API clients are all principals with an Access Role and a Record Scope. Agents never use passwords, OIDC or SAML; they use Lanesra-issued, short-lived, scoped credentials.

ADR-0014: a Team Workspace server keeps a **control database** holding accounts, memberships, sessions and the workspace registry. Each workspace has its own database and data key. One account can belong to several workspaces with a different role in each.

## Decision

### 1. Accounts, principals and where each lives

| Thing | Lives in | Notes |
|---|---|---|
| **Account** (a person who can sign in: email, password hash, SSO links, MFA later) | Control database (Team Workspace); the local database (Desktop) | One account, many workspace memberships (ADR-0014 §3) |
| **Membership** (account × workspace → Access Role, Record Scope, status) | Control database | Removing a membership ends access to that workspace at once |
| **Human principal** inside a workspace | Workspace database, referring to the account by id | All audit rows name this principal |
| **Agent Identity and API client principals** | Workspace database only | Belong to exactly one workspace; never cross it; never sign in with a password, OIDC or SAML (ADR-0006) |
| **Session** | Control database | Bound to one account and one workspace at a time |
| **Identity-provider configuration** (OIDC or SAML) | Workspace database, with secrets under the workspace data key | Admin-only; each workspace chooses which provider it trusts |

SSO is an authentication method for an account, not a new principal type.

### 2. Sign-in flows

**Local password.** Argon2id hashes. Limits on failed attempts apply per account and per source address together: 5 failures in 15 minutes on one account slows that account; many failures from one address block that address. Neither lets an outsider lock a real user out for long.

**OIDC (Team Workspace).** Authorization-code flow with **PKCE** (RFC 7636), a one-time `state` value and a `nonce`.
- Validate the ID token's signature against the provider's JWKS. Only RS256, PS256 and ES256 are accepted; `none` and symmetric algorithms are refused.
- Refetch the JWKS when a key id is unknown, and cache it otherwise.
- Check issuer, audience, expiry, issued-at and nonce, allowing 60 seconds of clock skew.

**SAML 2.0 (Team Workspace).** SP-initiated, using a maintained library only (RAID R-10). Not hand-written XML signature code.
- Require a signed assertion, not just a signed response.
- Reject documents with more than one assertion, or with an assertion outside the signed element (signature-wrapping).
- Check `InResponseTo`, audience, recipient, `NotBefore` and `NotOnOrAfter`.
- Keep a replay cache of assertion ids until they expire.

**Desktop (REQ-GOV-028).** OIDC through the system browser with a loopback redirect (RFC 8252) and PKCE.
- The refresh token is stored in the OS keychain.
- ID and access tokens are held only in memory, never written to SQLite.
- SAML on desktop goes through the same system-browser hand-off; WP-0105 decides whether it uses a broker.

### 3. Linking a sign-in to an account

- An SSO identity is the pair **(issuer, subject)**, stored as an account link. Email is never the key, because some providers let users choose or change it.
- **First SSO sign-in for an account that already exists** (same verified email): the link is created only after the person proves they own the account, by signing in with its password or by an admin approving the link. An unverified email (`email_verified` false or missing) never links.
- **Just-in-time account creation is off by default.** An admin can turn it on per workspace, restricted to listed email domains. New accounts get the workspace's lowest-privilege role (Viewer once REQ-GOV-004 lands), never an admin role.

### 4. Directory groups to roles

- An admin maps named directory groups to Access Roles. Directory membership never grants an admin or AI Admin role implicitly.
- Roles from groups are recomputed at every sign-in. A user removed from a group loses the matching role at the next sign-in, and their open sessions in that workspace are ended.
- If the provider omits groups because there are too many (Entra ID "group overage"), sign-in keeps the user's current roles and records a warning. It never upgrades.

### 5. Sessions (Team Workspace)

- Server-side sessions in the control database. The cookie is an opaque random id: `HttpOnly`, `Secure`, `SameSite=Lax`, `__Host-` prefix.
- A new session id is issued at sign-in and whenever privileges change (no session fixation).
- CSRF protection on every state-changing request.
- Idle timeout 30 minutes, absolute timeout 8 hours; both configurable per workspace.
- All of an account's sessions end on password change, membership removal, role change from the directory, or admin revocation. Users can end their own sessions.
- A session names exactly one workspace. Switching workspace issues a new session, and a request whose session names another workspace is refused before any workspace database is opened (T-12).

### 6. Break-glass access

Each workspace keeps at least one local Administrator who can sign in with a password when the identity provider is down. SSO configuration cannot disable or delete it. Every break-glass sign-in is audited and shown to the other Administrators. MFA for it comes with MFA generally.

### 7. Audit (T-18)

These events are written to the workspace audit log under the principal's own name:
- sign-in success and failure, and the method used;
- SSO link created or removed;
- just-in-time account creation;
- roles granted or removed from directory groups;
- session revocation;
- break-glass use;
- changes to identity-provider configuration.

### 8. Threat coverage

- **T-11 (account takeover):** §2 attempt limits and protocol checks, §3 linking by (issuer, subject) with proof of ownership, §5 session rules.
- **T-12 (cross-workspace access):** accounts and sessions in the control database, data in per-workspace databases (ADR-0014); a session is bound to one workspace and checked before that workspace's database is opened.
- **T-03 (excessive agency):** SSO authenticates human accounts only. Agent Identities and API clients never derive credentials or roles from SSO tokens, and an agent never sees more than its user (REQ-GOV-002).
- **T-18 (audit tampering):** §7 events, made tamper-evident later by WP-0402.

## Consequences

- WP-0107 (principal model) builds the account, membership and principal split in §1. WP-0181 (several workspaces) moves accounts and sessions into the control database.
- WP-0102 (OIDC) and WP-0103 (SAML) implement §2 to §4, each with negative tests for every check in §2. WP-0106 implements §2 attempt limits and §5. WP-0105 implements the desktop flow.
- The online demo shows the sign-in settings screens with simulated providers, in the same release as desktop and Team Workspace (ADR-0012).
- No identity code in this ADR (WP-0101 scope).

## Open questions

- Cloud SaaS: central account directory or per-region control databases? Deferred to WP-0422.
- SAML single logout: deferred. Local session revocation is enough for the first release.
- MFA for local accounts, including break-glass: separate work package, not yet planned. To be added to RAID.

## Review

- 2026-10-18 Claude: reviewed. Fixed a conflict with ADR-0014: the draft put accounts and sessions in each workspace database. Replaced email-based account linking with (issuer, subject) plus proof of ownership. Made just-in-time creation opt-in and domain-limited. Added PKCE, `state`, the algorithm allow-list and JWKS rotation, the SAML signature-wrapping and replay checks, session-id renewal and revocation rules, audit events and API-client principals. Status stays **Proposed** until the owner accepts it.
