---
id: WP-0602
title: Admin console shell: side menu, sub-menus, top bar and addresses
phase: 6
epic: E6.1 Admin console
status: proposed
owner: unassigned
planned_for: claude
depends_on: [WP-0601]
requirements: [REQ-ADM-002, REQ-ADM-003, REQ-ADM-004]
estimate: L
editions: [desktop, workspace, demo]
delivered: []
docs: [admin-console, workspace-essentials]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0602 — Admin console shell: side menu, sub-menus, top bar and addresses

## Goal

Replace the landing-page-then-tool flow with a console whose menu never disappears, so an admin always knows where they are and can move in one click.

## Scope

- Persistent side menu in three bands with a sub-menu per area; collapses to a drawer on narrow screens
- An address for every admin screen; command palette and existing deep links (`openAdminTab`, help links) keep working
- Top bar: search including previous names, environment label, unpublished-changes tray
- Role-aware and flag-aware menu, with the server refusing a tool the role may not use
- All three editions: desktop, Team Workspace and the online demo; website help with features, setup steps and menu location (REQ-ADM-009)

## Acceptance criteria

- [ ] AC-1: Opening any tool keeps the menu visible and marks the current area and tool
- [ ] AC-2: Every current deep link and command-palette entry still opens its tool
- [ ] AC-3: A role without rights does not see a tool and gets a refusal from the server if it calls the tool directly
- [ ] AC-4: The tray lists drafts with unpublished changes across pages, themes, agents, rules and workflows
- [ ] AC-5: Desktop and Team Workspace behave the same from the same code; tests run in both modes
- [ ] AC-6: Online demo (`app.js`) shows the same screens on sample data with writes simulated, released once both editions have shipped it (REQ-PRE-021); demo E2E covers it
- [ ] AC-7: Website help for every slug in `docs:` explains what the feature does, how to set it up step by step and where it is in the Admin menu; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

## Test plan

- vitest for the shell; Playwright E2E in both modes; axe checks; `desktop/server/tests/admin_tool_access.rs`
- Online demo: `node --check app.js` and the demo E2E for the new screens
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR from WP-0601. Feature flag `admin_console`; the old landing page stays reachable while the flag is off.
- ADR-0022; design in SOLUTION-DESIGN.md sections 15 (Admin console) and 16 (Experience Studio). Refine acceptance criteria before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
