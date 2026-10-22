---
id: WP-0191
title: "Show outbound-guard refusals to admins with the fix"
phase: 1
epic: E1.9 Team Workspace platform
status: in_review
owner: claude
depends_on: [WP-0143]
requirements: [REQ-PRE-019]
threats: [T-13]
estimate: S
editions: [workspace]
editions_reason: Only Team Workspace refuses internal addresses; desktop allows them and the demo makes no outbound calls
delivered: [workspace]
docs: [set-up-team-workspace]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/705
updated: 2026-10-22
---

# WP-0191 — Show outbound-guard refusals to admins with the fix

## Goal

When the Team Workspace outbound guard refuses an internal address, the admin should see it where they configure the URL, with the address and what the operator must add to `LANESRA_OUTBOUND_ALLOW`, instead of finding it later in a failed run (RAID Q-22).

## Scope

- Saving or testing a connection, model provider base URL, MCP server or trace exporter whose host the guard refuses returns a clear validation error naming the address and the setting
- The admin screens show that message where the URL was entered

## Out of scope

- Hostname entries in the allow-list (later, pinned to resolution)
- Counting refusals in the admin health strip
- Per-workspace allow-lists (WP-0192)

## Acceptance criteria

- [x] AC-1: Saving an internal-address connection, provider, MCP server or exporter on Team Workspace fails with `policy.network_blocked` and a message naming the address and `LANESRA_OUTBOUND_ALLOW`; an allow-listed address saves
- [x] AC-2: The same save succeeds on desktop
- [x] AC-3: The admin screen shows the message where the URL was entered (frontend test)

## Test plan

- `desktop/core/tests/ssrf_guard.rs`
- `desktop/src/__tests__/`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-10-22 claude: Written after the owner approved the recommendations in RAID Q-22.
- 2026-10-22 claude: Built. `ssrf_guard::check_configured_url` applies the connect-time rules when a URL is saved and, unlike `check_url`, resolves host names, so `http://localhost:11434` is refused at save on Team Workspace. The lookup runs on a helper thread with a 2-second limit; a name that does not resolve in time, or at all, is accepted and checked again on use. Wired into HTTP connection create and update (rest, webhook, OData, MCP; MCP servers save through connections), named AI providers, the workspace AI settings and the OTLP exporter. Existing test `a_model_provider_on_an_internal_address_is_refused` now also asserts the save is refused, and keeps its call-time assertions by writing a pre-upgrade URL straight to the table. Scope trimmed: counting refusals in the health strip is left out. Tests: `admins_see_a_refused_address_when_they_save_it` (core), a unit test for desktop and Team Workspace policies, `outboundRefusal.test.tsx` (frontend).
- 2026-10-22 claude: In review in #705.
