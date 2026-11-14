# Build status

> **Generated file - do not edit by hand.** Run `python3 program/tools/programctl.py status` after changing any work package. If this file conflicts in a merge, take either side and re-run the command.

## Phase summary

| Phase | Work packages | Done | In progress / review | Ready | Blocked | Proposed | Points done / total | Progress |
|---|---|---|---|---|---|---|---|---|
| 0 · Precursors | 20 | 19 | 0 | 0 | 0 | 1 | 34 / 36 | `█████████░` 94% |
| 1 · Foundations | 62 | 35 | 0 | 4 | 0 | 23 | 82 / 162 | `█████░░░░░` 51% |
| 2 · Playground | 24 | 0 | 0 | 0 | 0 | 24 | 0 / 90 | `░░░░░░░░░░` 0% |
| 3 · Teams and modes | 31 | 0 | 0 | 0 | 0 | 31 | 0 / 120 | `░░░░░░░░░░` 0% |
| 4 · Enterprise scale | 22 | 0 | 0 | 0 | 0 | 22 | 0 / 93 | `░░░░░░░░░░` 0% |
| 5 · Platform and ecosystem | 42 | 0 | 0 | 0 | 0 | 42 | 0 / 140 | `░░░░░░░░░░` 0% |
| 6 · Admin console and Experience Studio | 31 | 0 | 0 | 0 | 0 | 31 | 0 / 96 | `░░░░░░░░░░` 0% |
| 7 · Process Lanes | 21 | 2 | 3 | 2 | 0 | 14 | 6 / 61 | `█░░░░░░░░░` 10% |
| v2 · Open interop | 1 | 0 | 0 | 0 | 0 | 1 | 0 / 4 | `░░░░░░░░░░` 0% |

## Edition coverage

Every capability ships in each edition it applies to (REQ-PRE-015). A work package is done only when it is delivered in all of its editions.

| Edition | Work packages targeting it | Delivered | Points delivered / total |
|---|---|---|---|
| Desktop | 195 | 31 | 79 / 638 |
| Team Workspace | 222 | 37 | 89 / 732 |
| Online demo | 150 | 17 | 42 / 500 |

Progress is weighted by estimate points (S=1, M=2, L=4, XL=8). The timeline is driven by how many AI sessions and tokens are available, so forecasts use points delivered per session (see ROADMAP.md, Forecasting), not calendar weeks.

## Active now

| WP | Title | Status | Owner | Branch | PR |
|---|---|---|---|---|---|
| [WP-0702](work/P7/WP-0702-online-demo-the-receive-shipment-lane-on-the-warehouse-demo.md) | Online demo: the Receive Shipment lane on the warehouse demo | in_review | muse | `wp/WP-0702-receive-shipment-lane-demo-r2` | https://github.com/vikram2409-eng/Lanesra-OS/pull/758 |
| [WP-0712](work/P7/WP-0712-blueprint-view-and-export-of-a-lane.md) | Blueprint view and export of a lane | in_review | muse | `wp/WP-0712-blueprint-view-and-export-r2` | https://github.com/vikram2409-eng/Lanesra-OS/pull/761 |
| [WP-0719](work/P7/WP-0719-count-and-replenish-lanes-as-definitions-with-an-authoring-log.md) | Count and Replenish lanes as definitions, with an authoring log | in_progress | muse | `wp/WP-0719-count-and-replenish-lanes` | - |

## Ready to start (all dependencies done)

Whichever AI you are using takes the first one that is not claimed (a `wp/WP-XXXX-*` branch or a draft PR means claimed). Hand it over with `programctl brief WP-XXXX`.

| WP | Title | Phase | Estimate |
|---|---|---|---|
| [WP-0114](work/P1/WP-0114-adr-external-agent-identity-a2a-trust-and-trace-ingest.md) | ADR: external agent identity, A2A trust and trace ingest | 1 | M |
| [WP-0141](work/P1/WP-0141-idempotency-keys-and-side-effect-safe-resume.md) | Idempotency keys and side-effect-safe resume | 1 | L |
| [WP-0147](work/P1/WP-0147-adr-delegated-credentials-token-vault-and-mcp-gateway.md) | ADR: delegated credentials, token vault and MCP gateway | 1 | M |
| [WP-0160](work/P1/WP-0160-agent-editor-in-eight-settings-groups.md) | Agent editor in eight settings groups | 1 | L |
| [WP-0704](work/P7/WP-0704-human-steps-and-run-data.md) | Human steps and run data | 7 | L |

## Planned workload (ADR-0021)

A planned lane is the default AI for a work package, not a lock: any agent may take it when the planned one has no usage left.

| Planned for | Open work packages | Points | Work packages |
|---|---|---|---|
| claude | 23 | 72 | WP-0141, WP-0520, WP-0521, WP-0522, WP-0525, WP-0599, WP-0601, WP-0602, WP-0605, WP-0610, WP-0611, WP-0613, WP-0643, WP-0699, WP-0704, WP-0705, WP-0706, WP-0707, WP-0708, WP-0709, WP-0710, WP-0720, WP-0799 |
| codex | 45 | 158 | WP-0015, WP-0102, WP-0103, WP-0104, WP-0105, WP-0146, WP-0194, WP-0195, WP-0196, WP-0197, WP-0501, WP-0502, WP-0503, WP-0504, WP-0505, WP-0510, WP-0511, WP-0512, WP-0513, WP-0530, WP-0533, WP-0540, WP-0550, WP-0551, WP-0552, WP-0553, WP-0560, WP-0572, WP-0574, WP-0580, WP-0581, WP-0582, WP-0583, WP-0585, WP-0612, WP-0614, WP-0615, WP-0620, WP-0622, WP-0623, WP-0624, WP-0640, WP-0641, WP-0642, WP-0650 |
| muse | 38 | 101 | WP-0114, WP-0147, WP-0422, WP-0523, WP-0524, WP-0531, WP-0532, WP-0541, WP-0561, WP-0570, WP-0571, WP-0573, WP-0584, WP-0590, WP-0591, WP-0592, WP-0603, WP-0604, WP-0606, WP-0607, WP-0608, WP-0621, WP-0625, WP-0630, WP-0631, WP-0632, WP-0651, WP-0652, WP-0702, WP-0711, WP-0712, WP-0713, WP-0714, WP-0715, WP-0716, WP-0717, WP-0718, WP-0719 |

## Requirement coverage

- Spec requirements tracked: **352**
- Mapped to a work package: **352**
- Delivered (every mapped work package done): **58**

## Phase 0 · Precursors

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0001](work/P0/WP-0001-program-control-plane-and-multi-ai-working-protocol.md) | Program control plane and multi-AI working protocol | E0.1 Program and engineering readiness | ✅ done | M | — | - |
| [WP-0002](work/P0/WP-0002-repair-dependency-lockfiles-and-pin-toolchains.md) | Repair dependency lockfiles and pin toolchains | E0.1 Program and engineering readiness | ✅ done | S | — | - |
| [WP-0003](work/P0/WP-0003-pull-request-ci-pipeline-and-branch-protection.md) | Pull request CI pipeline and branch protection | E0.1 Program and engineering readiness | ✅ done | M | — | WP-0002, WP-0004, WP-0016 |
| [WP-0004](work/P0/WP-0004-rust-format-and-lint-baseline.md) | Rust format and lint baseline | E0.1 Program and engineering readiness | ✅ done | S | — | - |
| [WP-0005](work/P0/WP-0005-deterministic-scripted-model-provider-for-tests.md) | Deterministic scripted model provider for tests | E0.2 Test infrastructure | ✅ done | M | — | - |
| [WP-0006](work/P0/WP-0006-browser-end-to-end-test-harness.md) | Browser end-to-end test harness | E0.2 Test infrastructure | ✅ done | M | ✓Desktop | WP-0003 |
| [WP-0007](work/P0/WP-0007-database-upgrade-migration-test-harness.md) | Database upgrade (migration) test harness | E0.2 Test infrastructure | ✅ done | M | — | WP-0003 |
| [WP-0008](work/P0/WP-0008-verify-the-spec-s-assumptions-against-the-code-gap-audit.md) | Verify the spec's assumptions against the code (gap audit) | E0.3 Architecture decisions | ✅ done | M | — | - |
| [WP-0009](work/P0/WP-0009-foundational-adrs-run-state-machine-trace-schema-errors-migr.md) | Foundational ADRs: run state machine, trace schema, errors, migrations, prompt catalog | E0.3 Architecture decisions | ✅ done | L | — | WP-0008 |
| [WP-0010](work/P0/WP-0010-workspace-feature-flags.md) | Workspace feature flags | E0.1 Program and engineering readiness | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0009 |
| [WP-0011](work/P0/WP-0011-security-baseline-threat-model-and-scanning.md) | Security baseline: threat model and scanning | E0.4 Security and compliance baseline | ✅ done | M | — | WP-0003 |
| [WP-0012](work/P0/WP-0012-clean-room-controls.md) | Clean-room controls | E0.4 Security and compliance baseline | ✅ done | S | — | - |
| [WP-0013](work/P0/WP-0013-online-demo-architecture-decision-for-the-playground.md) | Online demo architecture decision for the Playground | E0.3 Architecture decisions | ✅ done | M | — | - |
| [WP-0014](work/P0/WP-0014-design-token-inventory-and-accessibility-tooling.md) | Design token inventory and accessibility tooling | E0.2 Test infrastructure | ✅ done | M | ✓Desktop ✓Team Workspace | - |
| [WP-0015](work/P0/WP-0015-performance-baseline.md) | Performance baseline | E0.2 Test infrastructure | ⚪ proposed | M | — | WP-0005 |
| [WP-0016](work/P0/WP-0016-fix-tests-failing-on-main.md) | Fix tests failing on main | E0.1 Program and engineering readiness | ✅ done | M | ✓Desktop ✓Team Workspace | - |
| [WP-0017](work/P0/WP-0017-mirror-work-packages-to-github-issues.md) | Mirror work packages to GitHub Issues | E0.1 Program and engineering readiness | ✅ done | S | — | WP-0001 |
| [WP-0018](work/P0/WP-0018-tracker-upkeep-and-test-feedback-for-ais-without-a-shell.md) | Tracker upkeep and test feedback for AIs without a shell | E0.1 Program and engineering readiness | ✅ done | S | — | WP-0003 |
| [WP-0019](work/P0/WP-0019-always-current-program-dashboard.md) | Always-current program dashboard | E0.1 Program and engineering readiness | ✅ done | S | — | - |
| [WP-0020](work/P0/WP-0020-team-workspace-server-image-builds-and-ships.md) | Team Workspace server image builds, is pinned, and ships | E0.1 Program and engineering readiness | ✅ done | M | ✓Team Workspace | WP-0003 |

## Phase 1 · Foundations

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0100](work/P1/WP-0100-principal-and-agent-identity-foundation.md) | Principal model, Agent Identities and agent credentials | E1.0 Identity foundation | ~~dropped~~ | XL | ○Desktop ○Team Workspace ○Online demo | WP-0009 |
| [WP-0101](work/P1/WP-0101-identity-and-sign-in-architecture-adr.md) | Identity and sign-in architecture ADR | E1.1 Enterprise sign-in | ✅ done | M | — | WP-0009, WP-0107, WP-0108 |
| [WP-0102](work/P1/WP-0102-oidc-sign-in-for-team-workspace.md) | OIDC sign-in for Team Workspace | E1.1 Enterprise sign-in | ⚪ proposed | L | ○Team Workspace | WP-0101, WP-0010, WP-0106 |
| [WP-0103](work/P1/WP-0103-saml-2-0-sign-in-for-team-workspace.md) | SAML 2.0 sign-in for Team Workspace | E1.1 Enterprise sign-in | ⚪ proposed | L | ○Team Workspace | WP-0101, WP-0010, WP-0106 |
| [WP-0104](work/P1/WP-0104-directory-mapping-groups-to-roles-approvers-and-provisioning.md) | Directory mapping: groups to roles, approvers and provisioning | E1.1 Enterprise sign-in | ⚪ proposed | M | ○Team Workspace | WP-0102, WP-0103 |
| [WP-0105](work/P1/WP-0105-enterprise-sign-in-on-the-desktop-edition.md) | Enterprise sign-in on the desktop edition | E1.1 Enterprise sign-in | ⚪ proposed | L | ○Desktop | WP-0102, WP-0103 |
| [WP-0106](work/P1/WP-0106-team-workspace-first-run-and-sign-in-hardening.md) | Team Workspace first-run and sign-in hardening | E1.1 Enterprise sign-in | ✅ done | M | ✓Team Workspace ✓Desktop | WP-0107, WP-0108, WP-0020 |
| [WP-0107](work/P1/WP-0107-principal-model-and-effective-identity.md) | Principal model and effective-identity rules | E1.0 Identity foundation | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0009 |
| [WP-0108](work/P1/WP-0108-agent-identities-and-credentials.md) | Agent Identities and credentials | E1.0 Identity foundation | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0107 |
| [WP-0109](work/P1/WP-0109-trigger-migration-and-ui.md) | Trigger migration and Agent Identity UI | E1.0 Identity foundation | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0107, WP-0108 |
| [WP-0110](work/P1/WP-0110-run-and-step-schema-extensions.md) | Run and step schema extensions | E1.2 Run model and trace | ✅ done | M | ✓Desktop ✓Team Workspace | WP-0009, WP-0007, WP-0107 |
| [WP-0111](work/P1/WP-0111-trace-span-model-and-persistence.md) | Trace span model and persistence | E1.2 Run model and trace | ✅ done | L | ✓Desktop ✓Team Workspace | WP-0110 |
| [WP-0112](work/P1/WP-0112-run-events-over-sse-with-stable-schemas.md) | Run events over SSE with stable schemas | E1.2 Run model and trace | ✅ done | L | ✓Desktop ✓Team Workspace | WP-0111 |
| [WP-0113](work/P1/WP-0113-full-trace-conformance-suite-phase-1-gate-test.md) | Full-trace conformance suite (Phase 1 gate test) | E1.2 Run model and trace | ✅ done | M | — | WP-0112, WP-0122, WP-0150, WP-0107, WP-0108, WP-0109 |
| [WP-0114](work/P1/WP-0114-adr-external-agent-identity-a2a-trust-and-trace-ingest.md) | ADR: external agent identity, A2A trust and trace ingest | E1.0 Identity foundation | 🟢 ready | M | — | WP-0108, WP-0111 |
| [WP-0120](work/P1/WP-0120-agent-persona-limits-and-reasoning-modes.md) | Agent persona, limits and reasoning modes | E1.3 Agent loop | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0110, WP-0130 |
| [WP-0121](work/P1/WP-0121-step-limits-and-partial-results.md) | Step limits and partial results | E1.3 Agent loop | ✅ done | L | ✓Desktop ✓Team Workspace ✓Online demo | WP-0110, WP-0005 |
| [WP-0122](work/P1/WP-0122-native-function-calling-and-strict-json-tool-format.md) | Native function calling and strict JSON tool format | E1.3 Agent loop | ✅ done | L | ✓Desktop ✓Team Workspace | WP-0005 |
| [WP-0123](work/P1/WP-0123-per-step-model-routing-and-data-boundaries.md) | Per-step model routing and data boundaries | E1.3 Agent loop | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0121 |
| [WP-0124](work/P1/WP-0124-retry-with-backoff-and-error-taxonomy.md) | Retry with backoff and error taxonomy | E1.3 Agent loop | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0009, WP-0005 |
| [WP-0125](work/P1/WP-0125-context-window-handling-and-compaction-policies.md) | Context window handling and compaction policies | E1.3 Agent loop | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0121 |
| [WP-0126](work/P1/WP-0126-budget-awareness-in-agent-turns.md) | Budget awareness in agent turns | E1.3 Agent loop | ✅ done | S | ✓Desktop ✓Team Workspace ✓Online demo | WP-0121 |
| [WP-0127](work/P1/WP-0127-do-not-persist-state-fields.md) | Do-not-persist state fields | E1.3 Agent loop | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0111 |
| [WP-0128](work/P1/WP-0128-policy-hook-points-around-step-model-call-and-tool-call.md) | Policy hook points around step, model call and tool call | E1.3 Agent loop | ✅ done | L | ✓Desktop ✓Team Workspace ✓Online demo | WP-0122, WP-0133 |
| [WP-0130](work/P1/WP-0130-prompt-catalog-written-fresh.md) | Prompt catalog, written fresh | E1.4 Task contracts and guardrails | ✅ done | L | ✓Desktop ✓Team Workspace | WP-0009 |
| [WP-0131](work/P1/WP-0131-task-contracts-and-context-bindings.md) | Task contracts and context bindings | E1.4 Task contracts and guardrails | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0110 |
| [WP-0132](work/P1/WP-0132-output-validation-guardrail-retries-and-on-fail.md) | Output validation, guardrail retries and on_fail | E1.4 Task contracts and guardrails | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0131, WP-0133 |
| [WP-0133](work/P1/WP-0133-guardrails-as-versioned-policy-objects.md) | Guardrails as versioned policy objects | E1.4 Task contracts and guardrails | ✅ done | L | ✓Desktop ✓Team Workspace ✓Online demo | WP-0009 |
| [WP-0140](work/P1/WP-0140-per-tool-caps-and-rate-limits.md) | Per-tool caps and rate limits | E1.5 Tools and firewall | ✅ done | M | ✓Desktop ✓Team Workspace ✓Online demo | WP-0122, WP-0111 |
| [WP-0141](work/P1/WP-0141-idempotency-keys-and-side-effect-safe-resume.md) | Idempotency keys and side-effect-safe resume | E1.5 Tools and firewall | 🟢 ready | L | ○Desktop ○Team Workspace | WP-0110 |
| [WP-0142](work/P1/WP-0142-gate-on-server-initiated-mcp-requests.md) | Gate on server-initiated MCP requests | E1.5 Tools and firewall | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0150 |
| [WP-0143](work/P1/WP-0143-tool-catalog-credentials-and-delegation-guard-audit.md) | Tool catalog, credentials and delegation guard audit | E1.5 Tools and firewall | ✅ done | S | — | WP-0008 |
| [WP-0144](work/P1/WP-0144-test-button-on-every-model-tool-connector-agent-and-team.md) | Test button on every model, tool, connector, agent and team | E1.5 Tools and firewall | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0010 |
| [WP-0145](work/P1/WP-0145-minimal-untrusted-content-defence.md) | Minimal untrusted-content defence for tool results | E1.5 Tools and firewall | ✅ done | L | ✓Desktop ✓Team Workspace ✓Online demo | WP-0122, WP-0128, WP-0011, WP-0130 |
| [WP-0146](work/P1/WP-0146-bring-your-own-model-provider-coverage.md) | Bring-your-own-model provider coverage and model catalog | E1.3 Agent loop | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0005, WP-0122 |
| [WP-0147](work/P1/WP-0147-adr-delegated-credentials-token-vault-and-mcp-gateway.md) | ADR: delegated credentials, token vault and MCP gateway | E1.5 Tools and firewall | 🟢 ready | M | — | WP-0143, WP-0108 |
| [WP-0150](work/P1/WP-0150-move-every-pause-for-human-onto-ai-approvals.md) | Move every pause-for-human onto ai_approvals | E1.6 Approvals | ✅ done | L | ✓Desktop ✓Team Workspace ✓Online demo | WP-0110, WP-0007 |
| [WP-0151](work/P1/WP-0151-four-review-responses.md) | Four review responses | E1.6 Approvals | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0150 |
| [WP-0152](work/P1/WP-0152-approvals-bound-to-exact-arguments.md) | Approvals bound to exact arguments | E1.6 Approvals | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0150 |
| [WP-0153](work/P1/WP-0153-standing-approvals.md) | Standing approvals | E1.6 Approvals | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0152 |
| [WP-0160](work/P1/WP-0160-agent-editor-in-eight-settings-groups.md) | Agent editor in eight settings groups | E1.7 Agent setup experience | 🟢 ready | L | ○Desktop ○Team Workspace ○Online demo | WP-0120, WP-0014 |
| [WP-0170](work/P1/WP-0170-run-the-published-agent-version.md) | Run the published agent version | E1.3 Agent loop | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0110 |
| [WP-0180](work/P1/WP-0180-team-workspace-operations-backups-upgrades-health.md) | Team Workspace operations: backups, key recovery, upgrades and health | E1.9 Team Workspace platform | ⚪ proposed | L | ○Team Workspace ○Desktop | WP-0020, WP-0007 |
| [WP-0181](work/P1/WP-0181-several-workspaces-on-one-team-workspace-server.md) | Several workspaces on one Team Workspace server | E1.9 Team Workspace platform | ⚪ proposed | XL | ○Team Workspace | WP-0107, WP-0108, WP-0109, WP-0020, WP-0007 |
| [WP-0182](work/P1/WP-0182-deliver-outbound-webhooks-for-real-record-events.md) | Deliver outbound webhooks for real record events | E1.9 Team Workspace platform | ✅ done | S | ✓Team Workspace ✓Desktop | - |
| [WP-0183](work/P1/WP-0183-webhook-signing-secret-show-once-and-rotate.md) | Webhook signing secret: show once and rotate | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace ✓Desktop ✓Online demo | WP-0182 |
| [WP-0184](work/P1/WP-0184-api-client-network-restriction-and-scope-enforcement.md) | API clients: enforce the allowed network with a trusted-proxy rule | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace | - |
| [WP-0185](work/P1/WP-0185-emit-field-changed-and-workflow-webhook-events.md) | Emit the field.changed, workflow.completed and workflow.failed webhook events | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace ✓Desktop | WP-0182 |
| [WP-0186](work/P1/WP-0186-api-record-search.md) | API: record search behind the search.read scope | E1.9 Team Workspace platform | ✅ done | S | ✓Team Workspace | WP-0184 |
| [WP-0187](work/P1/WP-0187-api-bulk-export-and-bulk-write.md) | API: bulk export and bulk write behind bulk.read and bulk.write | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace | WP-0184 |
| [WP-0188](work/P1/WP-0188-api-webhook-management.md) | API: webhook management behind the webhooks.manage scope | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace | WP-0183, WP-0184 |
| [WP-0189](work/P1/WP-0189-api-integration-administration-and-scope-vocabulary-test.md) | API: integration administration and the scope-vocabulary test | E1.9 Team Workspace platform | ✅ done | M | ✓Team Workspace | WP-0186, WP-0187, WP-0188 |
| [WP-0190](work/P1/WP-0190-release-the-agent-identities-screen.md) | Release the Agent Identities screen and trigger picker ahead of WP-0160 | E1.0 Identity foundation | ✅ done | S | ✓Desktop ✓Team Workspace ✓Online demo | WP-0109 |
| [WP-0191](work/P1/WP-0191-admin-visible-outbound-refusals.md) | Show outbound-guard refusals to admins with the fix | E1.9 Team Workspace platform | ✅ done | S | ✓Team Workspace | WP-0143 |
| [WP-0192](work/P1/WP-0192-two-level-outbound-allow-list.md) | Two-level outbound allow-list: operator bound, workspace narrows | E1.9 Team Workspace platform | ⚪ proposed | M | ○Team Workspace | WP-0181, WP-0143 |
| [WP-0193](work/P1/WP-0193-upgrade-rusqlite.md) | Upgrade rusqlite 0.31 to 0.40 | E0.1 Program and engineering readiness | ✅ done | M | ✓Desktop ✓Team Workspace | - |
| [WP-0194](work/P1/WP-0194-upgrade-axum-and-tower-http.md) | Upgrade axum 0.8 and tower-http 0.6 | E0.1 Program and engineering readiness | ⚪ proposed | M | ○Desktop ○Team Workspace | - |
| [WP-0195](work/P1/WP-0195-upgrade-react-19.md) | Upgrade the desktop frontend to React 19 | E0.1 Program and engineering readiness | ⚪ proposed | M | ○Desktop ○Team Workspace | - |
| [WP-0196](work/P1/WP-0196-upgrade-vite-and-vitest.md) | Upgrade vite 8, vitest 5 and plugin-react 6 | E0.1 Program and engineering readiness | ⚪ proposed | S | ○Desktop ○Team Workspace | - |
| [WP-0197](work/P1/WP-0197-upgrade-tauri-action.md) | Upgrade tauri-action to 1.0 in the desktop release workflow | E0.1 Program and engineering readiness | ⚪ proposed | S | ○Desktop ○Team Workspace | - |
| [WP-0198](work/P1/WP-0198-per-team-trace-bodies-override.md) | Per-Team override of trace bodies | E1.2 Run model and trace | ⚪ proposed | S | ○Desktop ○Team Workspace | WP-0111 |
| [WP-0199](work/P1/WP-0199-dollar-budget-in-agent-turns.md) | Dollars remaining in agent turns | E1.3 Agent loop | ⚪ proposed | S | ○Desktop ○Team Workspace ○Online demo | WP-0126, WP-0146 |
| [WP-0200](work/P1/WP-0200-release-desktop-and-team-workspace-separately.md) | Release desktop and Team Workspace separately | E1.9 Team Workspace platform | ✅ done | M | ✓Desktop ✓Team Workspace | WP-0020 |

## Phase 2 · Playground

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0201](work/P2/WP-0201-roles-viewer-builder-approver-ai-admin.md) | Roles: Viewer, Builder, Approver, AI Admin | E2.1 Playground core | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0104 |
| [WP-0202](work/P2/WP-0202-run-rest-endpoints-get-trace-cancel-pause-resume.md) | Run REST endpoints: get, trace, cancel, pause, resume | E2.2 Run control | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0112 |
| [WP-0203](work/P2/WP-0203-playground-shell-build-conversation-and-trace-panes.md) | Playground shell: Build, Conversation and Trace panes | E2.1 Playground core | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0202, WP-0160, WP-0201 |
| [WP-0204](work/P2/WP-0204-trace-pane-stream-levels-and-resumable-streams.md) | Trace pane, stream levels and resumable streams | E2.1 Playground core | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0203 |
| [WP-0205](work/P2/WP-0205-step-through.md) | Step-through | E2.2 Run control | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0204 |
| [WP-0206](work/P2/WP-0206-simulate-writes-with-shadow-transactions-and-diffs.md) | Simulate writes with shadow transactions and diffs | E2.2 Run control | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0141, WP-0203 |
| [WP-0207](work/P2/WP-0207-replay-from-step-and-fork.md) | Replay from step and fork | E2.2 Run control | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0111, WP-0203 |
| [WP-0208](work/P2/WP-0208-edit-state-and-resume-breakpoints-on-live-runs.md) | Edit state and resume; breakpoints on live runs | E2.2 Run control | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0207, WP-0201 |
| [WP-0209](work/P2/WP-0209-compare-variants-side-by-side.md) | Compare variants side by side | E2.1 Playground core | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0203 |
| [WP-0210](work/P2/WP-0210-recorded-runs-and-deterministic-replay.md) | Recorded runs and deterministic replay | E2.3 Evaluation and publish | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0005, WP-0111 |
| [WP-0211](work/P2/WP-0211-save-as-test-case-golden-cases-and-graders.md) | Save as test case, golden cases and graders | E2.3 Evaluation and publish | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0210 |
| [WP-0212](work/P2/WP-0212-deeper-evaluation-and-unified-test-run-feed.md) | Deeper evaluation and Unified Test Run feed | E2.3 Evaluation and publish | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0211 |
| [WP-0213](work/P2/WP-0213-publish-gate-ready-to-publish-page-and-impact-analysis.md) | Publish gate, Ready to publish page and impact analysis | E2.3 Evaluation and publish | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0212 |
| [WP-0214](work/P2/WP-0214-explain-this-tool-cards-and-inline-citations.md) | Explain this, tool cards and inline citations | E2.1 Playground core | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0204 |
| [WP-0215](work/P2/WP-0215-interactive-output-cards.md) | Interactive output cards | E2.1 Playground core | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0203 |
| [WP-0216](work/P2/WP-0216-record-lineage-on-activity-timelines.md) | Record lineage on activity timelines | E2.2 Run control | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0111 |
| [WP-0217](work/P2/WP-0217-run-detail-screen.md) | Run detail screen | E2.4 Setup and screens | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0207 |
| [WP-0218](work/P2/WP-0218-approvals-inbox-ui-and-phone-view.md) | Approvals inbox UI and phone view | E2.4 Setup and screens | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0151, WP-0153 |
| [WP-0219](work/P2/WP-0219-agent-studio-home-and-editor-polish.md) | Agent Studio home and editor polish | E2.4 Setup and screens | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0160 |
| [WP-0220](work/P2/WP-0220-guided-setup-for-agents-and-pipelines.md) | Guided Setup for agents and pipelines | E2.4 Setup and screens | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0219, WP-0144 |
| [WP-0221](work/P2/WP-0221-three-industry-templates-with-evaluation-suites.md) | Three industry templates with evaluation suites | E2.5 Templates and demo | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0213, WP-0224 |
| [WP-0222](work/P2/WP-0222-accessibility-pass-for-phase-2-screens.md) | Accessibility pass for Phase 2 screens | E2.4 Setup and screens | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0203, WP-0217, WP-0218 |
| [WP-0223](work/P2/WP-0223-playground-in-the-online-demo.md) | Playground in the online demo | E2.5 Templates and demo | ⚪ proposed | XL | ○Online demo | WP-0013, WP-0203 |
| [WP-0224](work/P2/WP-0224-provider-verification-without-program-owned-api-keys.md) | Provider verification without program-owned API keys | E2.3 Evaluation and publish | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0210, WP-0211, WP-0146 |

## Phase 3 · Teams and modes

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0301](work/P3/WP-0301-team-seat-and-version-model-with-lifecycle.md) | Team, seat and version model with lifecycle | E3.1 Teams and compiler | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0131 |
| [WP-0302](work/P3/WP-0302-playbook-model-and-state-schema.md) | Playbook model and state schema | E3.1 Teams and compiler | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0301 |
| [WP-0303](work/P3/WP-0303-mode-compiler-with-sequential-consensus-and-peer-review.md) | Mode compiler with Sequential, Consensus and Peer review | E3.1 Teams and compiler | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0302 |
| [WP-0304](work/P3/WP-0304-run-controller-and-budget-lease.md) | Run controller and budget lease | E3.1 Teams and compiler | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0303 |
| [WP-0305](work/P3/WP-0305-graph-runtime-new-node-types-superstep-commits-merge-rules.md) | Graph runtime: new node types, superstep commits, merge rules | E3.3 Graph and flow control | ⚪ proposed | XL | ○Desktop ○Team Workspace | WP-0303 |
| [WP-0306](work/P3/WP-0306-convert-pipelines-to-teams-with-compatibility-aliases.md) | Convert Pipelines to Teams with compatibility aliases | E3.1 Teams and compiler | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0307](work/P3/WP-0307-manager-led-mode-with-plan-approval-and-progress-card.md) | Manager-led mode with plan approval and progress card | E3.2 Modes | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0305, WP-0319 |
| [WP-0308](work/P3/WP-0308-round-table-mode-speaker-rules-and-seat-visibility.md) | Round Table mode, speaker rules and seat visibility | E3.2 Modes | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0305 |
| [WP-0309](work/P3/WP-0309-handoff-mode-and-team-tools.md) | Handoff mode and team tools | E3.2 Modes | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0305 |
| [WP-0310](work/P3/WP-0310-debate-mode.md) | Debate mode | E3.2 Modes | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0305 |
| [WP-0311](work/P3/WP-0311-graph-mode-contracts-type-checking-waits-choices-for-each-ru.md) | Graph mode: contracts, type-checking, waits, choices, for-each, run-at-end | E3.3 Graph and flow control | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0305, WP-0312 |
| [WP-0312](work/P3/WP-0312-safe-expression-language.md) | Safe expression language | E3.3 Graph and flow control | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0302 |
| [WP-0313](work/P3/WP-0313-event-playbooks-triggers-and-busy-run-policy.md) | Event Playbooks, triggers and busy-run policy | E3.3 Graph and flow control | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0314](work/P3/WP-0314-composable-stop-rules.md) | Composable stop rules | E3.3 Graph and flow control | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0304, WP-0312 |
| [WP-0315](work/P3/WP-0315-nested-teams-and-their-returns.md) | Nested teams and their returns | E3.2 Modes | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0316](work/P3/WP-0316-plan-and-execute-agent-modes.md) | Plan and Execute agent modes | E3.2 Modes | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0317](work/P3/WP-0317-durability-per-team-and-team-variants.md) | Durability per Team and Team variants | E3.1 Teams and compiler | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0318](work/P3/WP-0318-team-level-identity-roles.md) | Team-level identity: Team Owner and Auditor roles | E3.4 Identity, budget and memory | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0107, WP-0108, WP-0109, WP-0104, WP-0304 |
| [WP-0319](work/P3/WP-0319-budgets-at-four-levels-and-cost-forecast.md) | Budgets at four levels and cost forecast | E3.4 Identity, budget and memory | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0304 |
| [WP-0320](work/P3/WP-0320-composite-memory-recall-team-and-run-scopes-auto-extraction.md) | Composite memory recall, team and run scopes, auto-extraction | E3.4 Identity, budget and memory | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0301 |
| [WP-0321](work/P3/WP-0321-team-drive.md) | Team drive | E3.4 Identity, budget and memory | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0301 |
| [WP-0322](work/P3/WP-0322-progressive-skills.md) | Progressive skills | E3.4 Identity, budget and memory | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0130 |
| [WP-0323](work/P3/WP-0323-organisation-aware-approval-routing-sla-and-escalation.md) | Organisation-aware approval routing, SLA and escalation | E3.4 Identity, budget and memory | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0151, WP-0104 |
| [WP-0324](work/P3/WP-0324-policy-hooks-on-agent-to-agent-messages.md) | Policy hooks on agent-to-agent messages | E3.4 Identity, budget and memory | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0128, WP-0305 |
| [WP-0325](work/P3/WP-0325-team-and-run-apis-over-rest-mcp-cli-and-webhooks.md) | Team and run APIs over REST, MCP, CLI and webhooks | E3.5 Team experience | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0304 |
| [WP-0326](work/P3/WP-0326-team-editor-settings-groups-mode-chooser-team-setup-screen.md) | Team editor: settings groups, mode chooser, team setup screen | E3.5 Team experience | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0301, WP-0219 |
| [WP-0327](work/P3/WP-0327-work-steps-canvas.md) | Work steps canvas | E3.5 Team experience | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0311, WP-0326 |
| [WP-0328](work/P3/WP-0328-guided-setup-for-teams-modes-and-the-canvas.md) | Guided Setup for Teams, modes and the canvas | E3.5 Team experience | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0220, WP-0327 |
| [WP-0329](work/P3/WP-0329-team-level-evaluation-metrics.md) | Team-level evaluation metrics | E3.5 Team experience | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0212, WP-0304 |
| [WP-0330](work/P3/WP-0330-workspace-gallery-and-further-templates.md) | Workspace gallery and further templates | E3.5 Team experience | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0221, WP-0301 |
| [WP-0331](work/P3/WP-0331-phase-3-security-review-gate.md) | Phase 3 security review (gate) | E3.6 Security gate | ⚪ proposed | L | — | WP-0307, WP-0308, WP-0309, WP-0318, WP-0324 |

## Phase 4 · Enterprise scale

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0401](work/P4/WP-0401-worker-pool-on-team-workspace.md) | Worker pool on Team Workspace | E4.1 Scale | ⚪ proposed | XL | ○Team Workspace | WP-0305 |
| [WP-0402](work/P4/WP-0402-hash-chained-audit-log-and-evidence-export.md) | Hash-chained audit log and evidence export | E4.2 Compliance and governance | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0111 |
| [WP-0403](work/P4/WP-0403-retention-redaction-and-erasure-cascade.md) | Retention, redaction and erasure cascade | E4.2 Compliance and governance | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0402, WP-0320 |
| [WP-0404](work/P4/WP-0404-otlp-export-orchestration-dashboard-and-cost-reports.md) | OTLP export, orchestration dashboard and cost reports | E4.2 Compliance and governance | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0111 |
| [WP-0405](work/P4/WP-0405-kill-switch-and-four-eyes-rule.md) | Kill switch and four-eyes rule | E4.2 Compliance and governance | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0304, WP-0151 |
| [WP-0406](work/P4/WP-0406-content-labels-and-untrusted-input-defence.md) | Content labels and untrusted-input defence | E4.2 Compliance and governance | ⚪ proposed | XL | ○Desktop ○Team Workspace ○Online demo | WP-0128, WP-0305 |
| [WP-0407](work/P4/WP-0407-missions.md) | Missions | E4.3 Advanced capabilities | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0305 |
| [WP-0408](work/P4/WP-0408-online-sampling-a-b-and-shadow-mode.md) | Online sampling, A/B and shadow mode | E4.3 Advanced capabilities | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0213 |
| [WP-0409](work/P4/WP-0409-background-model-calls.md) | Background model calls | E4.3 Advanced capabilities | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0304 |
| [WP-0410](work/P4/WP-0410-memory-consolidation-and-lessons-from-corrections.md) | Memory consolidation and lessons from corrections | E4.3 Advanced capabilities | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0320, WP-0151 |
| [WP-0411](work/P4/WP-0411-team-as-an-endpoint.md) | Team as an endpoint | E4.3 Advanced capabilities | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0325 |
| [WP-0412](work/P4/WP-0412-outcome-metrics.md) | Outcome metrics | E4.3 Advanced capabilities | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0404 |
| [WP-0413](work/P4/WP-0413-delta-checkpoints-and-identity-aware-step-cache.md) | Delta checkpoints and identity-aware step cache | E4.1 Scale | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0305, WP-0318 |
| [WP-0414](work/P4/WP-0414-team-events.md) | Team events | E4.3 Advanced capabilities | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0313 |
| [WP-0415](work/P4/WP-0415-remote-workers.md) | Remote workers | E4.1 Scale | ⚪ proposed | XL | ○Team Workspace | WP-0401 |
| [WP-0416](work/P4/WP-0416-sandboxed-code-tool-and-notebook-sessions.md) | Sandboxed code tool and notebook sessions | E4.3 Advanced capabilities | ⚪ proposed | XL | ○Desktop ○Team Workspace | WP-0331 |
| [WP-0417](work/P4/WP-0417-deployment-residency-and-provider-allow-lists.md) | Deployment residency and provider allow-lists | E4.2 Compliance and governance | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0123 |
| [WP-0418](work/P4/WP-0418-remove-pipeline-aliases.md) | Remove Pipeline aliases | E4.1 Scale | ⚪ proposed | S | ○Desktop ○Team Workspace ○Online demo | WP-0306 |
| [WP-0419](work/P4/WP-0419-ga-readiness-success-metrics.md) | GA readiness: success metrics | E4.3 Advanced capabilities | ⚪ proposed | M | — | WP-0328, WP-0221 |
| [WP-0420](work/P4/WP-0420-storage-layer-for-sqlite-and-postgresql.md) | Storage layer that can run on SQLite or PostgreSQL | E4.1 Scale | ⚪ proposed | XL | ○Team Workspace ○Desktop | WP-0181 |
| [WP-0421](work/P4/WP-0421-postgresql-for-team-workspace.md) | PostgreSQL for Team Workspace | E4.1 Scale | ⚪ proposed | XL | ○Team Workspace | WP-0420, WP-0180 |
| [WP-0422](work/P4/WP-0422-saas-edition-architecture-plan.md) | SaaS edition architecture plan | E4.1 Scale | ⚪ proposed | M | — | WP-0181, WP-0420 |

## Phase 5 · Platform and ecosystem

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0501](work/P5/WP-0501-external-agent-registry-and-approval.md) | External agent registry and approval | E5.1 Agent control plane | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0114, WP-0301 |
| [WP-0502](work/P5/WP-0502-published-teams-callable-over-a2a.md) | Published Teams callable over A2A | E5.1 Agent control plane | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0114, WP-0325, WP-0411 |
| [WP-0503](work/P5/WP-0503-external-agents-as-seats-and-tools.md) | External agents as seats and tools | E5.1 Agent control plane | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0501, WP-0309, WP-0315 |
| [WP-0504](work/P5/WP-0504-trace-ingest-for-external-agent-runs.md) | Trace ingest for external agent runs | E5.1 Agent control plane | ⚪ proposed | L | ○Team Workspace | WP-0501, WP-0404 |
| [WP-0505](work/P5/WP-0505-fleet-governance-and-fleet-view.md) | Fleet governance and fleet view | E5.1 Agent control plane | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0503, WP-0504, WP-0405, WP-0319 |
| [WP-0510](work/P5/WP-0510-token-vault-and-per-user-oauth-for-connectors.md) | Token vault and per-user OAuth for connectors | E5.2 Delegated credentials and MCP gateway | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0147, WP-0102 |
| [WP-0511](work/P5/WP-0511-delegated-grants-in-runs.md) | Delegated grants in runs | E5.2 Delegated credentials and MCP gateway | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0510 |
| [WP-0512](work/P5/WP-0512-mcp-server-registry-oauth-and-definition-pinning.md) | MCP server registry, OAuth and definition pinning | E5.2 Delegated credentials and MCP gateway | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0147, WP-0142 |
| [WP-0513](work/P5/WP-0513-mcp-gateway-endpoint.md) | MCP gateway endpoint | E5.2 Delegated credentials and MCP gateway | ⚪ proposed | M | ○Team Workspace | WP-0512 |
| [WP-0520](work/P5/WP-0520-channel-framework-and-conversation-sessions.md) | Channel framework and conversation sessions | E5.3 Channels | ⚪ proposed | L | ○Team Workspace ○Online demo | WP-0411, WP-0325 |
| [WP-0521](work/P5/WP-0521-slack-channel.md) | Slack channel | E5.3 Channels | ⚪ proposed | L | ○Team Workspace | WP-0520 |
| [WP-0522](work/P5/WP-0522-microsoft-teams-channel.md) | Microsoft Teams channel | E5.3 Channels | ⚪ proposed | L | ○Team Workspace | WP-0520 |
| [WP-0523](work/P5/WP-0523-email-channel.md) | Email channel | E5.3 Channels | ⚪ proposed | M | ○Team Workspace | WP-0520 |
| [WP-0524](work/P5/WP-0524-embeddable-web-chat-widget.md) | Embeddable web chat widget | E5.3 Channels | ⚪ proposed | L | ○Team Workspace ○Online demo | WP-0520 |
| [WP-0525](work/P5/WP-0525-approvals-in-slack-and-microsoft-teams.md) | Approvals in Slack and Microsoft Teams | E5.3 Channels | ⚪ proposed | M | ○Team Workspace | WP-0521, WP-0522, WP-0218, WP-0152 |
| [WP-0530](work/P5/WP-0530-simulated-users-for-multi-turn-evaluation.md) | Simulated users for multi-turn evaluation | E5.4 Evaluation depth | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0212 |
| [WP-0531](work/P5/WP-0531-synthetic-test-case-generation.md) | Synthetic test-case generation | E5.4 Evaluation depth | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0211 |
| [WP-0532](work/P5/WP-0532-annotation-queues-and-end-user-feedback.md) | Annotation queues and end-user feedback | E5.4 Evaluation depth | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0212, WP-0408 |
| [WP-0533](work/P5/WP-0533-red-team-suite-in-the-publish-gate.md) | Red-team suite in the publish gate | E5.4 Evaluation depth | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0213, WP-0406 |
| [WP-0540](work/P5/WP-0540-agent-optimizer-proposals.md) | Agent optimizer proposals | E5.5 Agent optimizer | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0212, WP-0130 |
| [WP-0541](work/P5/WP-0541-model-right-sizing-report.md) | Model right-sizing report | E5.5 Agent optimizer | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0540, WP-0146 |
| [WP-0550](work/P5/WP-0550-hybrid-retrieval-with-reranking.md) | Hybrid retrieval with reranking | E5.6 Ontology-grounded knowledge | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0320 |
| [WP-0551](work/P5/WP-0551-graph-retrieval-over-the-ontology.md) | Graph retrieval over the Ontology | E5.6 Ontology-grounded knowledge | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0550 |
| [WP-0552](work/P5/WP-0552-permission-aware-source-sync.md) | Permission-aware source sync | E5.6 Ontology-grounded knowledge | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0510, WP-0550 |
| [WP-0553](work/P5/WP-0553-ask-the-data-over-glossary-metrics.md) | Ask the data over Glossary metrics | E5.6 Ontology-grounded knowledge | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0412 |
| [WP-0560](work/P5/WP-0560-human-seats-and-the-work-queue.md) | Human seats and the work queue | E5.7 Humans as seats | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0301, WP-0323, WP-0704 |
| [WP-0561](work/P5/WP-0561-sla-and-metrics-for-human-seats.md) | SLA and metrics for human seats | E5.7 Humans as seats | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0560, WP-0329 |
| [WP-0570](work/P5/WP-0570-teams-as-code-declarative-export-and-import.md) | Teams as code: declarative export and import | E5.8 Developer surface | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0325 |
| [WP-0571](work/P5/WP-0571-promotion-between-environments.md) | Promotion between environments | E5.8 Developer surface | ⚪ proposed | M | ○Team Workspace | WP-0570, WP-0181 |
| [WP-0572](work/P5/WP-0572-python-sdk.md) | Python SDK | E5.8 Developer surface | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0325 |
| [WP-0573](work/P5/WP-0573-typescript-sdk.md) | TypeScript SDK | E5.8 Developer surface | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0572 |
| [WP-0574](work/P5/WP-0574-signed-custom-tools-in-the-sandbox.md) | Signed custom tools in the sandbox | E5.8 Developer surface | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0416 |
| [WP-0580](work/P5/WP-0580-scim-2-0-provisioning.md) | SCIM 2.0 provisioning | E5.9 Enterprise operations | ⚪ proposed | L | ○Team Workspace | WP-0104 |
| [WP-0581](work/P5/WP-0581-external-key-management-and-secrets-managers.md) | External key management and secrets managers | E5.9 Enterprise operations | ⚪ proposed | L | ○Team Workspace | WP-0180, WP-0181 |
| [WP-0582](work/P5/WP-0582-siem-export.md) | SIEM export | E5.9 Enterprise operations | ⚪ proposed | M | ○Team Workspace | WP-0402 |
| [WP-0583](work/P5/WP-0583-alerts-and-service-level-objectives.md) | Alerts and service-level objectives | E5.9 Enterprise operations | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0404 |
| [WP-0584](work/P5/WP-0584-helm-chart-and-reference-deployment.md) | Helm chart and reference deployment | E5.9 Enterprise operations | ⚪ proposed | M | ○Team Workspace | WP-0421 |
| [WP-0585](work/P5/WP-0585-high-availability-and-disaster-recovery.md) | High availability and disaster recovery | E5.9 Enterprise operations | ⚪ proposed | L | ○Team Workspace | WP-0421, WP-0401 |
| [WP-0590](work/P5/WP-0590-ai-inventory-and-agent-cards.md) | AI inventory and agent cards | E5.10 AI compliance | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0501, WP-0402 |
| [WP-0591](work/P5/WP-0591-risk-classification-and-control-mappings.md) | Risk classification and control mappings | E5.10 AI compliance | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0590 |
| [WP-0592](work/P5/WP-0592-compliance-report.md) | Compliance report | E5.10 AI compliance | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0591 |
| [WP-0599](work/P5/WP-0599-phase-5-gate-platform-readiness.md) | Phase 5 gate: platform readiness | E5.11 Platform gate | ⚪ proposed | M | — | WP-0505, WP-0513, WP-0525, WP-0533, WP-0585, WP-0592 |

## Phase 6 · Admin console and Experience Studio

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0601](work/P6/WP-0601-adr-and-menu-registry-for-the-admin-console.md) | ADR and menu registry for the Admin console | E6.1 Admin console | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | - |
| [WP-0602](work/P6/WP-0602-admin-console-shell-side-menu-sub-menus-top-bar-and-addresse.md) | Admin console shell: side menu, sub-menus, top bar and addresses | E6.1 Admin console | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0601 |
| [WP-0603](work/P6/WP-0603-admin-home-with-attention-counts.md) | Admin Home with attention counts | E6.1 Admin console | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0602 |
| [WP-0604](work/P6/WP-0604-admin-assistant-and-help-as-side-panels.md) | Admin Assistant and Help as side panels | E6.1 Admin console | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0602 |
| [WP-0605](work/P6/WP-0605-one-pattern-for-admin-tools-list-and-detail-scaffold.md) | One pattern for admin tools: list and detail scaffold | E6.1 Admin console | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0602 |
| [WP-0606](work/P6/WP-0606-apply-the-tool-pattern-to-the-run-and-govern-areas.md) | Apply the tool pattern to the Run and Govern areas | E6.1 Admin console | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0605 |
| [WP-0607](work/P6/WP-0607-findability-descriptions-tour-previous-location-map-and-test.md) | Findability: descriptions, tour, previous-location map and test | E6.1 Admin console | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0603 |
| [WP-0608](work/P6/WP-0608-website-help-and-pages-for-the-new-menu.md) | Website help and pages for the new menu | E6.1 Admin console | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0602 |
| [WP-0610](work/P6/WP-0610-adr-one-page-model-and-how-existing-layouts-are-carried-over.md) | ADR: one page model and how existing layouts are carried over | E6.2 One page model | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | - |
| [WP-0611](work/P6/WP-0611-page-store-and-renderer-for-every-page-type-and-every-object.md) | Page store and renderer for every page type and every object | E6.2 One page model | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0610 |
| [WP-0612](work/P6/WP-0612-finish-the-24-existing-components.md) | Finish the 24 existing components | E6.2 One page model | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0610 |
| [WP-0613](work/P6/WP-0613-carry-screen-layouts-forms-into-the-page-model.md) | Carry Screen layouts (forms) into the page model | E6.2 One page model | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0614](work/P6/WP-0614-carry-dashboards-into-the-page-model.md) | Carry Dashboards into the page model | E6.2 One page model | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611, WP-0620 |
| [WP-0615](work/P6/WP-0615-custom-screens-with-data-sources-filters-and-parameters.md) | Custom screens with data sources, filters and parameters | E6.2 One page model | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0620](work/P6/WP-0620-data-components-record-list-board-timeline-metric-tile-filte.md) | Data components: record list, board, timeline, metric tile, filter bar | E6.3 Components and blocks | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0615 |
| [WP-0621](work/P6/WP-0621-form-and-content-components.md) | Form and content components | E6.3 Components and blocks | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0622](work/P6/WP-0622-work-and-ai-components-agent-chat-approvals-run-activity.md) | Work and AI components: agent chat, approvals, run activity | E6.3 Components and blocks | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0615, WP-0218, WP-0202 |
| [WP-0623](work/P6/WP-0623-work-queue-component.md) | Work queue component | E6.3 Components and blocks | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0622, WP-0560 |
| [WP-0624](work/P6/WP-0624-blocks-reusable-components.md) | Blocks: reusable components | E6.3 Components and blocks | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0625](work/P6/WP-0625-six-starter-blocks.md) | Six starter blocks | E6.3 Components and blocks | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0624, WP-0620 |
| [WP-0630](work/P6/WP-0630-five-page-layouts-and-templates-for-every-page-type.md) | Five page layouts and templates for every page type | E6.4 Layouts and templates | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0615 |
| [WP-0631](work/P6/WP-0631-screen-templates-list-and-detail-analytics-board-guided-inta.md) | Screen templates: List and Detail, Analytics Board, Guided Intake | E6.4 Layouts and templates | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0630, WP-0620, WP-0621 |
| [WP-0632](work/P6/WP-0632-screen-templates-workspace-home-operations-console-agent-wor.md) | Screen templates: Workspace Home, Operations Console, Agent Workbench | E6.4 Layouts and templates | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0631, WP-0622 |
| [WP-0640](work/P6/WP-0640-builder-undo-redo-copy-paste-and-keyboard.md) | Builder: undo, redo, copy, paste and keyboard | E6.5 Builder experience | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0641](work/P6/WP-0641-visibility-rules-per-component.md) | Visibility rules per component | E6.5 Builder experience | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0642](work/P6/WP-0642-preview-as-a-role-version-compare-and-pre-publish-checks.md) | Preview as a role, version compare and pre-publish checks | E6.5 Builder experience | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0643](work/P6/WP-0643-describe-a-page-with-the-admin-assistant.md) | Describe a page with the Admin Assistant | E6.5 Builder experience | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0615, WP-0624 |
| [WP-0650](work/P6/WP-0650-app-menu-editor-and-home-page-per-role.md) | App menu editor and home page per role | E6.6 Navigation and theme | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0615 |
| [WP-0651](work/P6/WP-0651-themes-per-app-light-and-dark-pairs-component-styles.md) | Themes per app, light and dark pairs, component styles | E6.6 Navigation and theme | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0611 |
| [WP-0652](work/P6/WP-0652-theme-on-print-pdf-and-system-email.md) | Theme on print, PDF and system email | E6.6 Navigation and theme | ⚪ proposed | M | ○Desktop ○Team Workspace | WP-0651 |
| [WP-0699](work/P6/WP-0699-phase-6-gate-findability-parity-and-help-complete.md) | Phase 6 gate: findability, parity and help complete | E6.7 Track gate | ⚪ proposed | M | — | WP-0606, WP-0607, WP-0608, WP-0613, WP-0614, WP-0632, WP-0650, WP-0651 |

## Phase 7 · Process Lanes

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0701](work/P7/WP-0701-adr-process-lanes-on-the-run-engine-and-the-lane-definition.md) | ADR: Process Lanes on the run engine, and the lane definition schema | E7.1 Lane foundation | ✅ done | M | — | - |
| [WP-0702](work/P7/WP-0702-online-demo-the-receive-shipment-lane-on-the-warehouse-demo.md) | Online demo: the Receive Shipment lane on the warehouse demo | E7.6 Shipped lanes | 👀 in review | L | ○Online demo | WP-0701 |
| [WP-0703](work/P7/WP-0703-lane-definitions-in-the-core-store-lifecycle-and-compile-to.md) | Lane definitions in the core: store, lifecycle and compile to a graph | E7.1 Lane foundation | ✅ done | L | ✓Desktop ✓Team Workspace | WP-0701 |
| [WP-0704](work/P7/WP-0704-human-steps-and-run-data.md) | Human steps and run data | E7.1 Lane foundation | 🟢 ready | L | ○Desktop ○Team Workspace | WP-0703 |
| [WP-0705](work/P7/WP-0705-record-and-check-steps-real-atomic-repeat-safe-writes.md) | Record and check steps: real, atomic, repeat-safe writes | E7.2 Steps and rules | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0703, WP-0141 |
| [WP-0706](work/P7/WP-0706-approvals-branches-with-exits-pause-handoff-and-cancel.md) | Approvals, branches with exits, pause, handoff and cancel | E7.2 Steps and rules | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0704 |
| [WP-0707](work/P7/WP-0707-lane-runner-screen-entry-points-and-object-360.md) | Lane runner, first cut: the straight path on desktop and Team Workspace | E7.3 Runner and visibility | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0704, WP-0705 |
| [WP-0708](work/P7/WP-0708-agent-steps-assistance-and-autonomy-level-per-step.md) | Agent steps, assistance and autonomy level per step | E7.4 Agents in lanes | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0707 |
| [WP-0709](work/P7/WP-0709-lanes-and-workflows-declared-replacement-and-starting-a-lane.md) | Lanes and workflows: declared replacement, and starting a lane from a workflow | E7.2 Steps and rules | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0705 |
| [WP-0710](work/P7/WP-0710-repeat-over-child-records-and-stage-and-start-conditions.md) | Repeat over child records, and stage and start conditions | E7.2 Steps and rules | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0705 |
| [WP-0711](work/P7/WP-0711-notes-attachments-and-reassign-at-any-point.md) | Notes, attachments and reassign at any point | E7.3 Runner and visibility | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0707 |
| [WP-0712](work/P7/WP-0712-blueprint-view-and-export-of-a-lane.md) | Blueprint view and export of a lane | E7.5 Authoring | 👀 in review | M | ○Desktop ○Team Workspace ○Online demo | WP-0703 |
| [WP-0713](work/P7/WP-0713-board-by-stage-my-steps-and-lane-timing.md) | Board by stage, "my steps" and lane timing | E7.3 Runner and visibility | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0707 |
| [WP-0714](work/P7/WP-0714-dry-run-publish-checks-test-framework-and-solution-export.md) | Dry run, publish checks, test framework and Solution export | E7.5 Authoring | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0705, WP-0706 |
| [WP-0715](work/P7/WP-0715-describe-a-process-the-admin-assistant-drafts-a-lane.md) | Describe a process: the Admin Assistant drafts a lane | E7.5 Authoring | 🟢 ready | M | ○Desktop ○Team Workspace ○Online demo | WP-0703, WP-0712 |
| [WP-0716](work/P7/WP-0716-child-lanes.md) | Child lanes | E7.2 Steps and rules | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0706 |
| [WP-0717](work/P7/WP-0717-three-more-warehouse-lanes-and-lane-templates.md) | Three more warehouse lanes and lane templates | E7.6 Shipped lanes | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0707, WP-0710 |
| [WP-0718](work/P7/WP-0718-visual-lane-builder.md) | Visual lane builder | E7.5 Authoring | ⚪ proposed | L | ○Desktop ○Team Workspace ○Online demo | WP-0712, WP-0714 |
| [WP-0719](work/P7/WP-0719-count-and-replenish-lanes-as-definitions-with-an-authoring-log.md) | Count and Replenish lanes as definitions, with an authoring log | E7.6 Shipped lanes | 🔨 in progress | S | — | WP-0701 |
| [WP-0720](work/P7/WP-0720-lane-runner-second-cut-approvals-branches-pause-and-handoff.md) | Lane runner, second cut: approvals, branches, pause and handoff | E7.3 Runner and visibility | ⚪ proposed | M | ○Desktop ○Team Workspace ○Online demo | WP-0707, WP-0706 |
| [WP-0799](work/P7/WP-0799-phase-7-gate-process-lanes-ready.md) | Phase 7 gate: Process Lanes ready | E7.7 Track gate | ⚪ proposed | M | — | WP-0708, WP-0709, WP-0713, WP-0714, WP-0717, WP-0720 |

## Phase v2 · Open interop

| WP | Title | Epic | Status | Est | Editions (✓ delivered) | Depends on |
|---|---|---|---|---|---|---|
| [WP-0901](work/Pv2/WP-0901-open-interop-ag-ui.md) | Open interop: AG-UI | Ev2 Open interop | ⚪ proposed | L | ○Desktop ○Team Workspace | WP-0419 |
