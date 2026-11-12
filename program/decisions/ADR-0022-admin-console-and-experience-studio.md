# ADR-0022: Admin console and Experience Studio — Phase 6, a parallel track

- Status: Accepted (program owner decisions, 2026-11-11)
- Date: 2026-11-11
- Requirements: REQ-ADM-001..009, REQ-EXP-001..020, REQ-NFR-007
- Design: [`../design/SOLUTION-DESIGN.md`](../design/SOLUTION-DESIGN.md) sections 15 and 16
- Detailed ADRs to follow: WP-0601 (menu registry), WP-0610 (page model and carry-over)

## Context

A review of the admin screens on 2026-11-11 (source and README; the app was not run) found:

- **Admin panel** (`desktop/src/features/settings/Settings.tsx`): a landing page of 15 category
  cards and 40 tools. Opening a tool hides the menu. Four cards hold one tool, Dependency Explorer
  is listed twice, and the AI Agent Foundry card (7 tools) would hold about 25 after Phases 2 to 5.
- **Three screen builders:** Screen layouts (create and edit forms), Page Builder (record pages on
  a 12-column grid with 24 components and 4 templates) and Dashboards, each with its own editor
  and table. `page_layout_service.rs` itself calls the Page Builder "a second system".
- **Page Builder gaps:** every page belongs to one object; Chart, every action component, Agent
  action and two KPI sources render a placeholder; on desktop a published page takes over only
  Company and Product; only a whole page can be saved for reuse.
- **Theme Studio:** one workspace theme, four presets, draft and publish, a contrast gate.

## Decision

The owner decided three things on 2026-11-11:

1. **Merge the three builders into one** page builder with page types. Existing forms and
   dashboards are carried over automatically and must look the same afterwards; the old tools,
   APIs and Solution exports stay as aliases until removed after a deprecation notice.
2. **Regroup Admin into ten areas in three bands**, with AI split across AI Studio, AI operations
   and Governance and compliance, **provided users can understand the grouping easily**. That
   condition is a requirement (REQ-ADM-008) with a measured bar (REQ-NFR-007: 80% first-attempt
   success on ten tasks, one in each AI area).
3. **Add the work to the build program**, with every work package shipping in the desktop
   edition, Team Workspace and the online demo, and updating the website help with features,
   setup and navigation (REQ-ADM-009).

Further decisions made here:

4. **Phase 6 · Admin console and Experience Studio** holds the work: `WP-06xx` in `work/P6/`,
   closed by gate G6. It is a **parallel track**: a Phase 6 work package may be refined and started
   as soon as its own `depends_on` are done. It waits for no other gate, because most of it does
   not touch agent orchestration. Four packages depend on AI features and come later
   (WP-0622, WP-0623, WP-0632, WP-0643).
5. **The menu is data.** One registry (area, tool, description, role, flag, previous names) feeds
   desktop, Team Workspace and the online demo; CI fails when a tool has no home or two homes, or
   when a help article names a path that is not in the registry. Later phases add a screen by
   adding a row (notes added to WP-0160, WP-0203, WP-0218, WP-0219).
6. **Two design packages come first** and need nothing else: WP-0601 and WP-0610.
7. **At least three templates do not depend on AI features** (List and Detail, Analytics Board,
   Guided Intake), so the owner's "at least three ready-made screens" ships without waiting for
   Phase 2 or Phase 5.
8. **Editions rule restated.** REQ-PRE-015, REQ-PRE-016 and REQ-PRE-021 already require every
   edition and the website help. For this track each work package also carries them as explicit
   acceptance criteria, and WP-0608 rewrites every existing help path (the help names an
   "Admin →" path 138 times today).

## The ten areas

| Band | Area | Purpose label |
|---|---|---|
| Build | Data and ontology | What the business records and how it connects |
| Build | Experience Studio | How it looks and what people see |
| Build | Automation | Rules and work that run themselves |
| Build | AI Studio | Design and test agents and teams |
| Run | AI operations | Watch agents work and respond |
| Run | Integrations and channels | Connections to the outside |
| Run | Apps and deployment | Package, install and promote |
| Govern | People and access | Who can sign in and what they can do |
| Govern | Governance and compliance | The rules agents work under, and the proof |
| Govern | System | Keeping the workspace healthy |

## Alternatives considered

- **Keep three builders and add a fourth for custom pages.** Rejected by the owner: it deepens the split.
- **One large AI area.** Rejected by the owner: about 25 tools in one area is the problem being fixed.
- **Put the work in Phase 2 or Phase 5.** Rejected: it would move the G2 gate, or wait for G3 for no reason.

## Consequences

- The plan grows by 31 work packages and 96 points (234 packages in total).
- Planned split (ADR-0021): Claude Code 8 packages (the two ADRs, the shell, the page store,
  forms carry-over, the gate), Codex 11 (components, custom screens, blocks, builder experience,
  navigation), Muse AI 12 (home, side panels, findability, help refresh, form and content
  components, starter blocks, layouts, templates, themes).
- `Settings.tsx` becomes a hotspot while WP-0602 is in flight; other packages that add admin
  screens register a row in the menu registry instead of editing the shell.
- The mock-ups shown to the owner are at https://claude.ai/artifact/EC5GQwxn691RnSpAh4MHYe (a
  private page; the design document is the source of truth).
