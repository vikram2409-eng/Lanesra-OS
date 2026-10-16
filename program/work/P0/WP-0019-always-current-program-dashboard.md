---
id: WP-0019
title: Always-current program dashboard
phase: 0
epic: E0.1 Program and engineering readiness
status: in_review
owner: claude
depends_on: []
requirements: [REQ-PRE-017]
estimate: S
editions: []
delivered: []
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/622
updated: 2026-10-16
---

# WP-0019 — Always-current program dashboard

## Goal

Claude Code and Muse AI work in sequence, never in parallel (owner, 2026-10-16), so the program
dashboard is how the owner and each incoming AI see what was built, by whom, and what is next. Today
only Claude Code can republish it (https://claude.ai/artifact/TANp9Vzmur4A37bsgHEtKg), so it goes stale
during a Muse week. Make a copy that updates itself on every merge, and make freshness visible.

## Scope

- `.github/workflows/program-dashboard.yml`: on every push to `main` (and on demand) run `programctl export`, and publish `program/dashboard/index.html` plus the generated `data.json` to GitHub Pages
- The page shows a freshness line: data commit, generated time, and "N merges behind main" when that can be computed (Pages copy is always current; the claude.ai copy states its own age)
- `program/dashboard/README.md`, PROTOCOL, BRIEFING and the finish checklist: the Pages URL is the always-current view; Claude Code refreshes the claude.ai copy at the start and end of every session; Muse's handoff note asks for a refresh
- Owner action: enable GitHub Pages with source "GitHub Actions" (Settings → Pages), one time

## Acceptance criteria

- [ ] AC-1: A merge to `main` updates the Pages dashboard within one workflow run, with the merge's commit shown on the page
- [x] AC-2: The workflow has least privilege (`pages: write`, `id-token: write`, `contents: read`) and runs only on `main`
- [x] AC-3: The page renders the same on Pages as on claude.ai (same `index.html`, same data format)
- [x] AC-4: PROTOCOL and BRIEFING name the Pages URL and the refresh rule

## Test plan

- First run after merge: Pages URL shows the merge commit (link the run in the Progress log)
- `programctl` self-tests cover the export used by the workflow

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

### 2026-10-16 claude
Owner enabled Pages (source: GitHub Actions). Shipped in the same PR as WP-0004 at the owner's request
for speed (sequential agents; tracker tooling only).
Done: `.github/workflows/program-dashboard.yml` (validate, export, wrap, deploy; permissions contents:read,
pages:write, id-token:write; main and manual only) - AC-2. `program/tools/dashboard_page.py` adds the
HTML skeleton, with a self-test on the real page - AC-3 (rendered locally in Chromium: no errors,
correct UTF-8). PROTOCOL, BRIEFING, FINISH-CHECKLIST and dashboard README name the Pages URL - AC-4.
Next: AC-1 after merge - check the Pages URL shows the merge commit; link the Actions run here.
