# ADR-0018: API clients are restricted by network with an explicit trusted-proxy rule, and every grantable scope is built

- Status: Accepted (owner, 2026-10-21: "whatever is the enterprise-grade solution" for Q-19; "built if an enterprise would need them" for Q-20)
- Date: 2026-10-21
- Work package: WP-0184, WP-0185, WP-0186, WP-0187, WP-0188, WP-0189
- Requirements: REQ-INT-004, REQ-INT-005, REQ-INT-006, REQ-INT-003

## Context

An API client (`api_client_service.rs`) is created with a name, a set of scopes and an optional `allowed_cidr`. The Help article says the CIDR restricts where the client may call from, but `allowed_cidr` is stored and never read (`api_client_service.rs:76`, `server/src/api_v1.rs:85-129`): a leaked key works from anywhere (RAID I-31).

Eleven scopes can be granted. Only `metadata.read`, `objects.read`, `objects.write`, `agents.trigger` and `events.read` are checked by a route. `search.read`, `bulk.read`, `bulk.write`, `webhooks.manage`, `admin.integration.read` and `admin.integration.manage` can be granted and do nothing. Three webhook event types (`field.changed`, `workflow.completed`, `workflow.failed`) can be subscribed to and nothing emits them (RAID I-29).

The Team Workspace server is usually run behind a reverse proxy (README recipes for Caddy and nginx). Behind one, the connection address of every request is the proxy's, so a restriction that reads only the connection address either blocks everyone or, if it is widened to admit the proxy, restricts no one. Trusting `X-Forwarded-For` from any peer is the opposite failure: a caller can claim any address.

## Decision

### 1. Which address counts as the caller

- The caller address is the **TCP peer address** of the connection.
- The operator may set `LANESRA_TRUSTED_PROXIES`: a comma-separated list of IP addresses and CIDR blocks, default empty. This is independent of `LANESRA_TRUST_PROXY_HTTPS` (which only controls the Secure cookie flag and HSTS) and does not change it.
- Only when the peer address is inside `LANESRA_TRUSTED_PROXIES` is `X-Forwarded-For` read. The caller address is then found by walking the header's addresses **from right to left**, skipping addresses that are themselves trusted proxies; the first address that is not a trusted proxy is the caller. If the header is missing, unparsable, or every address is a trusted proxy, the peer address is used.
- `Forwarded`, `X-Real-IP` and any other header are never read. A request from a peer that is not a trusted proxy has its forwarding headers ignored entirely.
- IPv4-mapped IPv6 addresses (`::ffff:a.b.c.d`) are normalised to IPv4 before matching. A malformed `LANESRA_TRUSTED_PROXIES` entry stops the server at startup with the entry named, the same rule the outbound allow-list follows (WP-0143).

This is the same model as nginx `real_ip` with `set_real_ip_from`, Express `trust proxy` with a list, and AWS ALB guidance: trust nothing by default, trust named proxies explicitly, take the right-most untrusted hop.

### 2. The allowed network on an API client

- `allowed_cidr` holds a comma- or whitespace-separated list of CIDR blocks and single addresses (`10.0.0.0/8, 203.0.113.7`). Empty means unrestricted, which is today's behavior, so existing clients are unaffected. No migration: the column is already text.
- A malformed value is rejected when the client is created or edited.
- Enforcement is in the one `authorize()` function every `/api/v1` route and `/mcp` already pass through, after the key is verified and before the scope check, so REST, MCP, the agent trigger routes and the event stream are all covered. A request from outside the range returns **403** with `{"ok":false,"error":"This API client is not allowed from this network address"}`; the refusal is written to the integration log with the resolved caller address.
- The refusal is returned only after the key verifies, so the response does not reveal whether an unauthenticated guess was a valid client id.

### 3. Every grantable scope is built

The owner's rule is that a scope or event type an enterprise needs is built, not hidden. The target vocabulary and the endpoint behind each scope:

| Scope | Endpoints (all under `/api/v1`) |
|---|---|
| `metadata.read`, `objects.read`, `objects.write`, `agents.trigger`, `events.read` | exist today |
| `search.read` | `GET /search?q=&object=&page=&page_size=`: search across the objects the client may read |
| `bulk.read` | `GET /objects/:key/export?format=ndjson` (or `csv`): stream every record of an object |
| `bulk.write` | `POST /objects/:key/bulk`: up to 1000 operations (`create`, `update`, `archive`) per call, each through the normal validated write path, per-item results, never all-or-nothing by default |
| `webhooks.manage` | `GET/POST /webhooks`, `GET/PATCH/DELETE /webhooks/:id`, `POST /webhooks/:id/pause`, `reactivate`, `rotate-secret`, `test`, `GET /webhooks/:id/deliveries` |
| `admin.integration.read` | `GET /integration/connections`, `connectors`, `jobs`, `api-clients`: metadata only, **never** a secret or ciphertext |
| `admin.integration.manage` | pause and reactivate a connection or job, run a job now. It cannot create a connection, read a secret or change a scope |

Webhook events: `field.changed` is emitted once per changed field on a record update (`data` carries `field`, `previous`, `current`); `workflow.completed` and `workflow.failed` are emitted when a workflow run ends (`data` carries the workflow and run ids). Events are queued only when a webhook subscribes, as today.

Rules that apply to all of them: each endpoint checks its scope and the client's network; writes go through the same validated path as the UI (validation and business rules apply); every call is logged; each scope has an allow test and a deny test; a test fails if a scope exists in the vocabulary that no route enforces (REQ-INT-005).

## Alternatives considered

- **Connection address only.** Simple, but unusable behind the proxy almost every deployment uses.
- **Trust `X-Forwarded-For` whenever `LANESRA_TRUST_PROXY_HTTPS` is set.** Reuses a flag, but that flag is about cookies; it says nothing about who the proxies are, and any caller reaching the server directly could spoof an address.
- **Left-most `X-Forwarded-For` entry.** Client-controlled, so spoofable even behind a trusted proxy.
- **Remove the unenforced scopes and event types.** Cheaper, but the owner decided an enterprise needs them; removal would be a step backwards for integrations that already request them.
- **All-or-nothing bulk writes.** Safer for atomicity, but a thousand-row load that fails on row 900 is unusable; per-item results match how the CSV importer already reports.

## Consequences

- WP-0184 enforces section 1 and 2. WP-0185 builds the three event types. WP-0186 to WP-0189 build the endpoints in section 3, one scope family each.
- Access Control for API-client record writes (`actor = None`, RAID I-20) is **not** decided here; it stays with WP-0100 / WP-0109. Until then the API's authorization is scope plus network, and the documentation says so.
- New operator setting `LANESRA_TRUSTED_PROXIES` joins the Team Workspace settings table in the website and `desktop/README.md`.
- The MCP server gains the network check for free through `authorize()`. Search and bulk may later get MCP tools; that is not part of this decision.
