# Threat Model — LanesraOS Agent Playground (WP-0011)

**Status:** Reviewed 2026-10-17 by Claude Code, the owner's delegate for WP-0011 AC-1. Drafted by Muse.
**Owner:** the program owner. Changes go through a PR like any other tracked file.

This is the **only** threat list for the program. Every work package touching tools, approvals,
auth, identities or data access links the threats it mitigates or affects (WP-0011 AC-3, PR
template "Security" section). A threat whose mitigation is not yet built is a RAID risk until it is.

Ratings: **L**ikelihood and **I**mpact, each High / Medium / Low, for the product as it will ship
(desktop, Team Workspace exposed to a network, online demo). Status: **Open** (not built),
**Partial** (some controls exist), **Mitigated** (controls built and tested).

## Assets

1. **Business records**: companies, contacts, opportunities and custom objects in each workspace's database.
2. **Credentials held by the platform**: model-provider API keys, connector and MCP secrets, Agent Identity credentials (all encrypted with the workspace data key).
3. **The workspace data key** (`secret.key` on the Team Workspace `/data` volume; desktop app data directory).
4. **User credentials and sessions**: password hashes, session cookies, SSO assertions.
5. **Audit trail**: run history, approval decisions, tool-call traces.
6. **Model spend**: the organisation's provider budget.
7. **The build itself**: source, CI workflows, dependencies, release artifacts.

## Trust boundaries

| Boundary | Trusted side | Untrusted side |
|---|---|---|
| Browser ↔ Team Workspace server (network, behind TLS proxy) | Server | Every request, cookie and header from the browser |
| Tauri UI ↔ desktop core (IPC) | Core | Command arguments from the webview |
| Workspace ↔ workspace on one server (ADR-0014) | Each workspace's own database and key | Every other workspace |
| Core ↔ model providers | Core | Model output (may be steered by injected content) |
| Core ↔ tools, MCP servers, connectors, SFTP/HTTP endpoints | Core | Tool results, server-initiated MCP requests, remote hosts |
| Business records ↔ agent context | Agent instructions | Record content written by users or imported from outside |
| CI ↔ repository | Pinned actions, protected `main` | Pull-request code, third-party actions, dependencies |

## Threats

| ID | Threat | STRIDE / OWASP LLM | L | I | Mitigation (work packages) | Status |
|---|---|---|---|---|---|---|
| T-01 | **Direct prompt injection**: a user's prompt makes an agent ignore its instructions or policy | Tampering / LLM01 | H | M | Policy is enforced outside the model: tool-call firewall and risk tiers (existing), guardrails WP-0132/WP-0133, policy hooks WP-0128 | Partial |
| T-02 | **Indirect prompt injection**: instructions hidden in a record, email, file or tool result are followed when an agent reads them | Tampering / LLM01 | H | H | Untrusted-content defence for tool results WP-0145; content labels WP-0406; approvals bound to exact arguments WP-0152 | Open (RAID R-26) |
| T-03 | **Excessive agency**: an agent acts beyond its scope or with a human's or an Administrator's rights | Elevation of privilege / LLM06 | M | H | Principal model and effective identity WP-0107, Agent Identities WP-0108/WP-0109, an agent calls only its own tools WP-0143 (`firewall_chain.rs`), per-tool caps WP-0140, approvals WP-0150/WP-0152 | Partial (WP-0107, WP-0108, WP-0109, WP-0143 built) |
| T-04 | **Data exfiltration**: sensitive data sent to an external model, tool or URL | Information disclosure / LLM02 | M | H | Per-step routing and data boundaries (local-only classes) WP-0123; tool catalog and delegation audit WP-0143; content labels WP-0406 | Open |
| T-05 | **Approval spoofing or replay**: an approval is forged, reused or applied to different arguments | Spoofing / Repudiation | M | H | Approvals bound to exact arguments WP-0152; standing approvals scoped WP-0153; hash-chained audit WP-0402; four-eyes WP-0405 | Open |
| T-06 | **Cost exhaustion**: loops, retries or abuse run up provider spend | Denial of service / LLM10 | H | M | Step limits WP-0121; budget awareness WP-0126; run controller and budget lease WP-0304; four-level budgets WP-0319; kill switch WP-0405 | Partial (daily token budget exists) |
| T-07 | **Memory poisoning**: false content written to long-term memory steers later runs | Tampering / LLM04 | M | M | Memory recall scopes and provenance; consolidation with review WP-0410 | Open |
| T-08 | **Vulnerable dependency** in Rust or npm code | Tampering / LLM03 | M | H | `cargo deny` and `npm audit` in CI with documented, dated exceptions (WP-0011; RAID R-20 to R-25) | Mitigated for new findings |
| T-09 | **Secret committed to the repository** | Information disclosure | M | H | gitleaks in CI (WP-0011); test keys only, never real keys (BRIEFING hard rules) | Mitigated |
| T-10 | **SQL injection** | Tampering | L | H | Parameterised queries only (`rusqlite` params); SQL kept in `repositories/` (ADR-0014) | Mitigated by convention; covered by review |
| T-11 | **Account takeover on Team Workspace**: password guessing, session theft, cross-site request forgery | Spoofing | H | H | First-run and sign-in hardening (rate limit, lockout, cookie flags, CSRF) WP-0106; OIDC/SAML WP-0102/WP-0103 | Partial |
| T-12 | **Cross-workspace data access** on a server hosting several workspaces | Information disclosure / Elevation | M | H | One database and one data key per workspace (ADR-0014); isolation tests in WP-0181 | Open (single workspace today) |
| T-13 | **Server-side request forgery**: a user-configured URL (OpenAI-compatible base URL, connector, webhook, MCP endpoint) makes the server call internal addresses or cloud metadata | Information disclosure / Elevation | M | H | `services/ssrf_guard.rs` (WP-0143): every outbound HTTP client is built by the guard; metadata, link-local, unspecified and multicast addresses are always refused (also as IPv4-mapped or NAT64 IPv6); loopback and private addresses are refused on Team Workspace unless the server operator lists them in `LANESRA_OUTBOUND_ALLOW`; names are checked when resolved at connect time (DNS rebinding) and every redirect target is checked (`core/tests/ssrf_guard.rs`). SFTP and SMTP connect only to addresses the guard allows | Mitigated |
| T-14 | **Malicious or compromised MCP server**: poisoned tool descriptions, or server-initiated sampling and elicitation requests | Tampering / LLM03 | M | H | Gate on server-initiated MCP requests WP-0142; tool catalog audit WP-0143; tool results treated as untrusted WP-0145 | Open |
| T-15 | **Unsafe rendering of model output**: script or HTML in a model reply or record runs in the UI | Tampering / LLM05 | M | M | React escapes text by default; any Markdown or HTML renderer must sanitise (review rule for WP-0203 and the online demo) | Partial |
| T-16 | **SFTP host-key not verified**: `sftp_service::QuietHandler::check_server_key` accepts any server key, so a network attacker can impersonate the SFTP host and capture the credential | Spoofing | M | H | The connection must carry the server's `host_key_sha256` fingerprint; an unknown key is refused with the fingerprint shown for the Administrator to confirm, a changed key is refused (`core/tests/sftp_host_key.rs`) | Mitigated |
| T-17 | **Data key or backup stolen**: whoever has `secret.key` and the database, or a backup holding both, can decrypt every stored credential | Information disclosure | L | H | Key kept apart from backups, key recovery and backup encryption WP-0180 | Open |
| T-18 | **Audit trail altered or deleted** to hide an action | Repudiation | L | H | Hash-chained audit log and evidence export WP-0402; retention rules WP-0403 | Open |
| T-19 | **CI or supply-chain compromise**: a third-party action or pull-request code steals secrets or pushes to `main` | Tampering | L | H | Branch protection with required checks (WP-0003); every action in `.github/workflows/` pinned by commit SHA (WP-0011); agent workflows never run for forks and never push to `main` (WP-0018) | Mitigated |
| T-20 | **Leaked API key used from outside the intended network**: a stolen `client_id.secret` works from any address because the client's allowed network was stored and never checked | Spoofing / Elevation | M | H | `services/network_trust.rs` and `server/src/caller.rs` (WP-0184, ADR-0018): `authorize()` refuses a restricted client from outside its range with 403, after the key verifies; the caller is the TCP peer, `X-Forwarded-For` is read only from a peer in `LANESRA_TRUSTED_PROXIES` and walked from the right, no other header is read; refusals are logged with the resolved address (`server/tests/api_network.rs`). Clients with no allowed network are unrestricted by design, so setting one is an admin choice | Mitigated |

## Gaps tracked in RAID

- **R-26** (T-02): indirect prompt injection has no defence until WP-0145 ships.
- **R-27** (T-13): closed; outbound HTTP, SFTP and SMTP connections are checked by `ssrf_guard.rs`.
- **R-28** (T-16): closed; SFTP now requires a pinned host-key fingerprint.
- **R-20 to R-25** (T-08): dated dependency-advisory exceptions in `desktop/deny.toml`.

Review this file whenever a work package adds a new boundary (a new connector type, a new
edition, a new kind of credential) and at each phase gate.
