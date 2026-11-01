# ADR-0019: Desktop and Team Workspace are released separately

- Status: Accepted (owner, 2026-11-01: "We need to release desktop and team workspace separately to avoid the confusion. That will also give us two separate links."; own version line chosen over a shared number)
- Date: 2026-11-01
- Work package: WP-0200
- Requirements: REQ-PRE-024 (new), REQ-PRE-021 (reworded)
- Amends: ADR-0013 §3 (the "same release" part only)

## Context

Since WP-0020 one `desktop-v*` tag built the Windows installer and, from a second workflow, the Team Workspace server image. Both shared one version number and one GitHub Release page, titled "Lanesra OS Desktop desktop-vX.Y.Z". The owner found this confusing: a person looking for Team Workspace lands on a release page that says "desktop", and there is no separate link to give them. The server crate already has its own version (`desktop/server/Cargo.toml`, reported to MCP clients), so the shared number was a release convention, not a code constraint.

## Decision

1. **Two release lines, two tags.** Desktop: `desktop-vX.Y.Z`. Team Workspace: `server-vX.Y.Z`. Each has its own GitHub Release page, notes and link. The Docker image `ghcr.io/vikram2409-eng/lanesra-os-server:X.Y.Z` uses the server release's number.
2. **Own version line.** The two numbers are independent. The server line continues from the last shared release so numbers never go backwards for anyone who pulled `0.16.x`: the first Team Workspace release is `0.17.0`. Releases up to 0.16.3 stay as they are.
3. **No coupling in the pipelines.** A `desktop-v*` tag builds only the installer. A `server-v*` tag builds only the image and drafts the server release. `publish-release.yml` publishes desktop drafts and never touches the image. `publish-server-release.yml` publishes server drafts and moves `:latest`, only to the newest published server release. A program tool (`check_release_workflows.py`) fails CI if the two are tangled again.
4. **The server's own version is checked.** `server-image-release.yml` refuses a release whose number differs from `desktop/server/Cargo.toml` (the same guard the installer has for `tauri.conf.json`). The Cargo version moves to `0.17.0` now, ready for the first server release.
5. **Requirement change.** ADR-0013 §3 said a feature ships in desktop and Team Workspace in the same release. Replaced by: a feature is built for both editions from the same code, and the online demo shows it only once both have released it (REQ-PRE-021, reworded). Separate releases (REQ-PRE-024) are new.

## Alternatives considered

- **Same number, two tags.** Keeps one version in your head, but couples the two schedules and leaves the version number meaning two things. Rejected by the owner in favour of independent lines.
- **One release page with two sections.** Cheaper, but is the confusion the owner wants gone and gives one link, not two.

## Consequences

- Each release is cut with its own workflow run: "Desktop Release" then "Publish desktop release"; "Release Team Workspace server" then "Publish Team Workspace server release". Server notes live in `desktop/server-release-notes/X.Y.Z.md`.
- The feature-flag rule (demo follows `ga`) is unchanged in the tool; a flag is switched to `ga` once the feature is in a release of each edition.
- Existing links to `desktop-v0.16.x` and `:0.16.x` keep working.
- The first run of the new workflows is the real test; it is part of WP-0200's acceptance.
