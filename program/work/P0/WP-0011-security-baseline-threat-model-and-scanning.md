---
id: WP-0011
title: Security baseline: threat model and scanning
phase: 0
epic: E0.4 Security and compliance baseline
status: in_review
owner: unassigned
depends_on: [WP-0003]
requirements: [REQ-PRE-009]
estimate: M
editions: []
delivered: []
branch: 
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/653
updated: 2026-10-17
---

# WP-0011 — Security baseline: threat model and scanning

## Goal

Enterprise agents act on business records with real tools. Write the threat model before building, so every work package can cite the threats it must handle, and add automated scanning to CI.

## Scope

- `program/security/THREAT-MODEL.md`: assets, trust boundaries (UI, server, core, model providers, MCP servers, connectors), STRIDE plus LLM-specific threats (prompt injection direct and indirect, excessive agency, data exfiltration via tools, approval spoofing, cost exhaustion, memory poisoning)
- Each threat names mitigations and the requirement IDs that deliver them; gaps become RAID risks
- CI: `cargo deny` (or `cargo audit`) and `npm audit --omit=dev` with a documented exception list
- Secret scanning (GitHub secret scanning or gitleaks) on PRs
- A security review checklist added to the PR template for work packages tagged security

## Acceptance criteria

- [x] AC-1: Threat model reviewed by the owner
- [x] AC-2: Scanners run in CI and fail on new high-severity findings
- [ ] AC-3: Every Phase 1 work package touching tools, approvals or auth links to its threats

## Test plan

- Scanner jobs proven by a known-vulnerable test dependency in a throwaway branch
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-17 muse: Building WP-0011. Created .github/workflows/security.yml (cargo-deny, npm audit --audit-level=high, gitleaks; all actions pinned by SHA), desktop/deny.toml, program/security/THREAT-MODEL.md (10 threats, STRIDE+LLM). AC-2 will be proven by CI. AC-1 needs owner review. Untested locally — see CI.
- 2026-10-17 claude: Dependency fixes with a toolchain (Muse handoff). Restored main's Cargo.lock (the hand-edited h2/rustls checksums did not match crates.io) and ran `cargo update`: h2 0.4.15 -> 0.4.20, rustls 0.23.43 -> 0.23.46 (+ rustls-webpki 0.103.15), yanked chacha20 0.10.1 -> 0.10.2, so `yanked` is back to "deny". russh 0.44 -> 0.60.3 fixes RUSTSEC-2026-0153/0154; sftp_service.rs updated to the 0.60 API (native async trait, `AuthResult::success()`, `russh::keys::PublicKey`) and the now-unused async-trait dependency dropped. Allowed two permissive licences (0BSD, CDLA-Permissive-2.0). Exceptions left, each with RAID entry and review date: zip R-20, rsa R-21, unsafe-libyaml R-22, and six unmaintained Tauri/GTK transitive crates R-25; R-23 (h2) and R-24 (rustls) closed. `cargo deny check`: advisories, bans, licenses, sources all ok locally (cargo-deny 0.20.2).
- 2026-10-17 claude: Restored the cargo-update lockfile and deny.toml from de3f14f. The revert in 40d58b1 put back the old Cargo.lock while core/Cargo.toml asks for russh 0.60, so every `--locked` build failed ("cannot update the lock file"). The lockfile in de3f14f was produced by cargo and its checksums are valid; do not hand-edit or revert Cargo.lock: if it looks wrong, ask Claude. `cargo deny check`: all four checks ok; h2 0.4.20, rustls 0.23.46, russh 0.60.3, chacha20 0.10.2.
- 2026-10-17 claude: AC-1 review (owner delegated the review to Claude). Corrected program/security/THREAT-MODEL.md: fixed wrong "Covered by" links (WP-0111/0112 are trace and SSE, not budgets; WP-0100 is now WP-0107/0108/0109; WP-0005 does not mitigate injection) and replaced "future governance WPs" with the real work packages; removed the REQ-GOV-XXX placeholder; added likelihood, impact and status to every threat; added the browser-to-server, workspace-to-workspace and CI boundaries; added T-11 to T-19 (sign-in and session attacks, cross-workspace access, SSRF, malicious MCP servers, unsafe output rendering, SFTP host-key acceptance found in sftp_service.rs, data-key and backup theft, audit tampering, CI supply chain). New RAID gaps R-26 (indirect injection), R-27 (SSRF), R-28 (SFTP host key). AC-1 ticked.
- 2026-10-17 claude: Fixed two threat-model gaps in code. T-16/R-28: SFTP connections now require the server's host-key fingerprint (`host_key_sha256` in the connection's config JSON, OpenSSH `SHA256:...` form); an unknown key is refused and the error shows the presented fingerprint for the Administrator to confirm, a changed key is refused. Existing SFTP connections fail their next test until the fingerprint is added (the error says how). Tests: core/tests/sftp_host_key.rs (3, in-process russh server). T-19: every action in .github/workflows/ is pinned by commit SHA (15 tag references in 8 workflows).
- 2026-10-17 claude: AC-2 ticked. The `security` workflow (cargo-deny, npm audit, gitleaks) runs on every pull request and fails on unexempted findings: run https://github.com/vikram2409-eng/Lanesra-OS/actions/runs/39067899844 failed while advisories without an exception remained; run https://github.com/vikram2409-eng/Lanesra-OS/actions/runs/39071595073 is green on 31d1031 after the fixes. Status in_review; AC-3 (threat links in Phase 1 work packages) is left for Muse's planning PR.
