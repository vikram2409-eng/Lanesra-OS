# ADR-0026: Delegated credentials, the token vault and the MCP gateway

- Status: Accepted (program owner, 2026-11-14: "I accept. Move forward")
- Date: 2026-11-14
- Requirements: REQ-CRED-001
- Work packages: WP-0147 (this ADR); WP-0510, WP-0511, WP-0512, WP-0513 build on it
- Design: SOLUTION-DESIGN.md section 6

## Context

Today a connector holds one shared credential (WP-0143 audit). Every agent
that uses the connector acts with that one credential, so an agent cannot
respect per-user permissions in the systems it reaches: a run started by one
person can read or write what another person may, and an unattended run acts
with a credential no person granted to it. Phase 3 adds team tools on top of
the connector layer, which would spread the shared credential further. The
design must exist before that.

This follows ADR-0006 (agents are principals with an Access Role and Record
Scope, audited in their own name; Agent Identities use Lanesra-issued,
short-lived, scoped credentials the platform holds and never passes to a
model), ADR-0014 (the workspace data key encrypts workspace data) and
ADR-0018 (the API surface and network trust).

## Decision

### 1. The grant model

A **grant** is the unit of delegated access. It is (principal, connector,
scopes, expiry):

| Field | Meaning |
|---|---|
| `principal` | A user, or an Agent Identity. The grant belongs to exactly one principal. |
| `connector` | The connector the grant is for (one outside system). |
| `scopes` | The permissions granted in that system (the connector's own scope names). A grant never widens what the principal may do in Lanesra. |
| `expiry` | When the grant ends. A grant is short-lived; the vault refreshes it before expiry. |
| `created_by` | Who connected the account, and when. Audited. |

A user or an Agent Identity connects an account with OAuth 2.1
(authorisation code with PKCE). The user goes through the outside system's
own consent screen; Lanesra never sees the user's password there. An Agent
Identity's grant is connected by an admin on its behalf, and is recorded as
the identity's own grant — never as the admin's.

### 2. The token vault

Tokens live in the **token vault**, encrypted under the workspace data key
(ADR-0014). The vault:

- refreshes a token before it expires, rotates it, and revokes it on demand;
- never shows a token again after it is stored — not to the user, not to an
  admin, not in any screen or API response;
- never lets a token enter a model's context, a run trace, a checkpoint, or
  an event body;
- hands a token only to the connector layer, server-side, at the moment of
  the call. A tool never sees a credential (SOLUTION-DESIGN.md section 6:
  tools never see credentials).

Users see their own grants under My connections. Admins see that grants
exist, who they belong to and when they expire, but cannot read or use them.
Deactivating a user (SCIM) revokes their grants.

### 3. How the effective identity of a run selects a grant

The connector layer picks the grant from the run's **effective identity** —
the principal the run acts as (ADR-0006), set once when the run starts and
audited. The selection rule is:

| Run kind | Connector mode | Credential used |
|---|---|---|
| Interactive | Delegated | The starting user's grant; refused with "connect your account" if there is none |
| Unattended | Delegated | The Agent Identity's own grant; refused if there is none |
| Any | Shared | The connector's credential, as today |
| Any | Another user's grant | Never, including through delegation, handoff or nested Teams |

"Another user's grant: never" is the rule that stops the confused deputy:
a run started by one person, a run handed to another agent, and a run nested
inside a Team all use the effective identity's grant only. There is no
borrowing, and no fallback to a different principal's grant.

### 4. Refusal rules for unattended runs

An unattended run whose Agent Identity has no grant for the connector it
needs is **refused, not queued and not retried with another identity's
grant**. The refusal is audited with the reason, and the run ends as failed
with a readable error ("no grant for this Agent Identity on this
connector"). The run is never silently downgraded to the shared credential:
a connector in Delegated mode has no shared fallback.

### 5. The MCP registry

Only approved MCP servers are callable. Each tool definition is **pinned by
hash** at approval. A changed definition is quarantined until an admin
approves the diff — the tool cannot be called while quarantined. Tool
descriptions are scanned for instruction-like content at registration and on
change; a finding blocks approval by default (an admin may override it, and
the override is audited).

### 6. The MCP gateway

One Lanesra endpoint offers approved tools from several servers to outside
MCP clients, with namespaced names, scoped keys, the Tool-Call Firewall,
rate limits and audit. A gateway key is scoped to the tools it may offer and
to the workspace; it is not a user credential and it never grants access to
the token vault.

## Threat model

Three new threats, with their mitigations.

**Token theft.** A stolen token lets the thief act as the user in the
outside system. Mitigations: tokens are encrypted under the workspace data
key; they are never shown again after storage; they never enter a model's
context, a trace, a checkpoint or an event body; they are short-lived and
refreshed before expiry; revocation on user deactivation (SCIM) is
immediate; and the vault hands a token only to the connector layer,
server-side, at the moment of the call — never to a model, a tool or an API
response.

**Confused deputy.** A tool call tricks the connector into using the wrong
principal's grant — an agent acting for user A calls with user B's grant, or
a handoff silently changes whose credential is used. Mitigations: the grant
is selected from the run's effective identity only (decision 3); another
user's grant is never used, including through delegation, handoff or nested
Teams; the effective identity is set once at run start and audited; and an
unattended run with no grant of its own is refused rather than given
someone else's.

**Tool-definition change after approval.** An MCP server changes a tool's
definition after an admin approved it — for example, its description now
carries instructions that steer the model. Mitigations: definitions are
pinned by hash; any change quarantines the tool until an admin approves the
diff; descriptions are scanned for instruction-like content at registration
and on change, and a finding blocks approval by default.

## Consequences

- WP-0510 (token vault and per-user OAuth), WP-0511 (delegated grants in
  runs), WP-0512 (MCP registry and definition pinning) and WP-0513 (MCP
  gateway endpoint) build on this ADR. The grant model is the basis for
  per-user permissions in Phase 3 team tools.
- The shared-credential mode stays for connectors whose outside system has
  no per-user OAuth, and for runs whose connector is in Shared mode. It is
  a named mode, not a silent fallback: a connector in Delegated mode never
  falls back to it.
- The vault, the registry and the gateway are all new stores. Their
  migrations follow ADR-0010, and their contents are workspace data under
  the workspace data key.
