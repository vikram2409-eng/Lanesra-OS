---
id: WP-0200
title: "Release desktop and Team Workspace separately"
phase: 1
epic: E1.9 Team Workspace platform
status: done
owner: claude
depends_on: [WP-0020]
requirements: [REQ-PRE-024, REQ-PRE-021, REQ-PRE-018]
threats: [T-19]
estimate: M
editions: [desktop, workspace]
editions_reason: Release tooling and documentation only; the online demo has no releases of its own
delivered: [desktop, workspace]
docs: [set-up-team-workspace]
branch: chore/separate-server-release
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/731
updated: 2026-11-03
---

# WP-0200 — Release desktop and Team Workspace separately

## Goal

Every GitHub Release page is titled "desktop" and carries the Team Workspace image as a side note, so people looking for Team Workspace have no link of their own. Give Team Workspace its own tag, release page, version line and link, as the owner decided (ADR-0019).

## Scope

- `server-image-release.yml` starts on `team-workspace-server-v*` (tag or manual run), checks the version against `desktop/server/Cargo.toml`, smoke-tests, pushes `:X.Y.Z` and drafts a `team-workspace-server-vX.Y.Z` prerelease with notes
- New `publish-server-release.yml` publishes the draft and moves `:latest` to the newest published server release
- `publish-release.yml` loses everything about the image and the `release` trigger
- `desktop/server/Cargo.toml` and `desktop/Cargo.lock` move to `0.17.0`, the first server-line version
- `program/tools/check_release_workflows.py` (+ self-test) fails CI if the two lines are tangled; wired into the `program` CI job
- ADR-0019; REQ-PRE-024; REQ-PRE-021 and BRIEFING reworded
- README, `desktop/README.md`, `docker-compose.yml`, the Team Workspace help article, the Releases page and the desktop feature-flag screen say so

## Out of scope

- Re-tagging or editing the existing `desktop-v0.16.x` releases and `:0.16.x` images
- Code-signing, Windows/macOS builds, SaaS release channels
- Changing how the demo follows feature flags

## Acceptance criteria

- [x] AC-1: A `desktop-v*` tag starts only the installer build, and a `team-workspace-server-v*` tag starts only the server build; a self-test fails if either workflow listens to the other's tag
- [x] AC-2: The desktop publisher no longer references the server image, its registry or `:latest`; the server publisher moves `:latest` only for the newest published `team-workspace-server-v*` release
- [x] AC-3: The server release refuses a version that differs from `desktop/server/Cargo.toml`, and that file and `Cargo.lock` agree at `0.17.0`
- [x] AC-4: README, `desktop/README.md`, the help article and the Releases page describe two releases and two links
- [x] AC-5: The first real run proves it: "Release Team Workspace Server" with `0.17.0` builds and pushes `:0.17.0` and drafts `team-workspace-server-v0.17.0`; "Publish Team Workspace Server release" publishes it and moves `:latest`; the desktop workflows do not start (owner runs these; needs the Actions tab)

## Test plan

- `program/tools/test_check_release_workflows.py`: 7 tests, tagged `Covers: REQ-PRE-024 (WP-0200 AC-1, AC-2)`; seen failing against the previous workflows (server build on the desktop tag, desktop publisher touching the image, missing publisher)
- `cargo check --locked -p lanesra-server` with the new version (lockfile agrees)
- Workflow YAML parses; first real run is AC-5

## Notes

- Workflows cannot be run from this session, so the real behaviour (gh release create of a draft, `:latest` move) is unproven until AC-5. The draft/publish steps copy the shape of the desktop pair that already works.
- Security: the new workflows read the version from an environment variable, not from a `${{ }}` expression inside the shell script, so a crafted tag or input cannot inject a command (T-19). Both are pinned by commit SHA like the others.
- The README and Releases wording for 0.16.3 stays as it was: it was true when written.

## Progress log

<!-- Append dated entries, newest last. -->
- 2026-11-01 claude: Owner asked for separate releases; chose an own version line. Built ADR-0019, the two server workflows, the desktop publisher cleanup, the check tool and docs. AC-1 to AC-4 proven locally; AC-5 needs a real run.
- 2026-11-02 claude: AC-5 proven (status stays in_review: dependency WP-0020 is in_review, awaiting the owner's confirmation that /help/set-up-team-workspace is live). Ran "Release Team Workspace Server" with 0.17.0 (run 41384617265): it built, smoke-tested, pushed `ghcr.io/vikram2409-eng/lanesra-os-server:0.17.0` and drafted `team-workspace-server-v0.17.0`, and started no desktop workflow. Then "Publish Team Workspace Server release" (run 41384737043): the release is published (prerelease) and `:latest` now resolves to the 0.17.0 image (verified against the registry). Server release notes added in `desktop/server-release-notes/0.17.0.md`; the website download card, Releases callout, roadmap status, README and llms.txt now name team-workspace-server-v0.17.0. Done.
- 2026-11-03 claude: Dependency WP-0020 is now done (owner confirmed the help article is live). All ACs were already proven and PR #731 merged, so this moves in_review -> done, delivered to desktop and Team Workspace.
