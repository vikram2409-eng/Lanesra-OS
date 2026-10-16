---
id: WP-0020
title: Team Workspace server image builds, is pinned, and ships
phase: 0
epic: E0.1 Program and engineering readiness
status: in_review
owner: claude
depends_on: [WP-0003]
requirements: [REQ-PRE-018, REQ-PRE-015]
estimate: M
editions: [workspace]
editions_reason: Packaging of the Team Workspace server only; the desktop app has its own release workflow and the online demo is static files
delivered: []
docs: [set-up-team-workspace]
branch: claude/enterprise-build-planning-jxy2kq
pr: https://github.com/vikram2409-eng/Lanesra-OS/pull/636
updated: 2026-10-16
---

# WP-0020 — Team Workspace server image builds, is pinned, and ships

## Goal

The Team Workspace server is shipped as a Docker image (`desktop/Dockerfile`, `desktop/docker-compose.yml`). A review on 2026-10-16 found that **the image no longer builds**, and that it was never part of CI:

- **Broken build (RAID I-24).** The Dockerfile copies only `core/` and `server/` and builds `server/` as its own project. Since WP-0004 added `[lints] workspace = true` to every crate, Cargo cannot parse `server/Cargo.toml` without the workspace manifest. This was confirmed by simulating the Dockerfile's layout.
- **Not reproducible (RAID I-25):**
  - `Cargo.lock` is not copied, so every build resolves the newest dependency versions.
  - `rust:1-slim` floats instead of following `rust-toolchain.toml` (1.97.0).
  - `node:20-slim` contradicts `.nvmrc` (22).
- **Not tested or shipped.** No CI job builds or runs the image, and `desktop-release.yml` builds only the Windows desktop app, so no image is published.

Every Playground feature has to ship in Team Workspace too (REQ-PRE-015), so the server must be buildable and testable from Phase 0.

## Scope

- Dockerfile:
  - Copy the workspace `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and the member manifests that `cargo` needs to read.
  - Build with `cargo build --release --locked -p lanesra-server`.
  - Base the builder on the pinned Rust version and the frontend stage on Node 22 (read from `.nvmrc`/`rust-toolchain.toml`, or kept in step by a CI check).
  - Keep the runtime image minimal: non-root user, `/data` volume.
- `.dockerignore`, so that `target/`, `node_modules/` and `dist/` are not sent to the build.
- CI job `server image` on every pull request:
  - build the image (with a layer cache);
  - start it with an empty volume;
  - check `/api/health`;
  - complete first-run setup through the API;
  - sign in;
  - restart the container and check the data is still there;
  - stop.
- Release: on a version tag, build and publish the image to GitHub Container Registry (`ghcr.io/vikram2409-eng/lanesra-os-server:<version>` and `:latest`), in a workflow that alone has `packages: write`.
- Docs: a new help article `set-up-team-workspace`, covering:
  - Docker and Compose quick start;
  - LAN versus internet exposure;
  - the reverse proxy;
  - the environment variables;
  - backing up the volume **and** `secret.key`.

  Link it from `desktop/README.md`.

## Acceptance criteria

- [x] AC-1: `docker build` succeeds from a clean checkout and uses `Cargo.lock` (the build fails if the lockfile is out of date)
- [x] AC-2: The Rust and Node versions in the image match `rust-toolchain.toml` and `.nvmrc`; a CI check fails if they drift
- [x] AC-3: The `server image` CI job runs the smoke test above on every pull request, and fails when it fails
- [ ] AC-4: A version tag publishes the image to GHCR, and the published image passes the same smoke test
- [ ] AC-5: The help article `set-up-team-workspace` is live, names this work package, and is in the sitemap

## Test plan

- The CI smoke job is the test. Prove it red once with a deliberate break (as in WP-0003 AC-2), then revert.
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Must follow (WP-0009): ADR-0010 migration conventions (the image runs migrations on start). ADR-0010 applies to any migration this work package adds.
- Ask the owner to add `server image` to the required checks once it has been green for a week.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->

- 2026-10-16 claude: Created from the Team Workspace review. The broken build (I-24) was caused by WP-0004's workspace lints, so it is a regression to fix first.

- 2026-10-16 claude: Built on the branch after PR #628 merged:
  - `desktop/Dockerfile` copies the whole Cargo workspace (manifests, `Cargo.lock`, `rust-toolchain.toml`, member sources) and builds `cargo build --release --locked -p lanesra-server`, so workspace lints apply and the lockfile is honoured. The Rust (1.97.0) and Node (22) base images are pinned to the repo's pins. A `HEALTHCHECK` uses `/api/health`.
  - `program/tools/check_toolchain_pins.py` and 5 self-tests: CI fails if the Dockerfile's versions drift from `rust-toolchain.toml` or `.nvmrc`.
  - `desktop/scripts/server-image-smoke.sh`: start the image on an empty volume, run first-run setup, sign in, create a record, restart, sign in again and confirm the record is still there.
  - CI job `server image (Team Workspace)`: checks the pins, builds with buildx and a GitHub Actions layer cache, runs the smoke test, and prints the server log.
  - `.github/workflows/server-image-release.yml`: on the same `desktop-v*` tag as the desktop release (REQ-PRE-021), builds, smoke-tests, then pushes `ghcr.io/vikram2409-eng/lanesra-os-server:<version>` and `:latest`. Only this workflow has `packages: write`.
  - Help article `set-up-team-workspace` (wps WP-0020) and its sitemap entry; `llms.txt`, `llms-full.txt`, `desktop/README.md` and `docker-compose.yml` updated; PROTOCOL §13 notes the new check.
  - Verified here:
    - Cargo parses the Dockerfile's layout and `--locked` resolves `lanesra-server`. The old layout failed to parse, which reproduces RAID I-24.
    - Every HTTP step of the smoke test passed against a local `lanesra-server` binary.
    - Base image tags exist on Docker Hub.
  - Not verifiable here: there is no Docker daemon and only 2.4 GB of disk, so the image build itself is proven by the CI job.
  - Next: the CI run proves AC-1 and AC-3. Prove the job red once (AC-3), then revert. AC-4 is proven on the first release tag. Ask the owner to make `server image` a required check after a week of green runs.
  - Gotcha: `src-tauri/Cargo.lock` is a stray, tracked file that Cargo ignores, because the workspace lockfile wins. It is left alone; noted as RAID T-06.

- 2026-10-16 claude: CI on PR #636:
  - AC-1: run 38965473554, job `server image (Team Workspace)`, is green in 7 min 16 s. The image built from the lockfile, and the smoke test passed all 8 steps, including that data survives a restart.
  - AC-3: the temporary commit b26fdff made the smoke test expect a missing record. Run 38967056285 went red with "FAIL: record created before the restart is missing", after every earlier step passed. Reverted. With a warm layer cache the job takes about 1 minute.
  - State: in_review. AC-4 is proven on the first `desktop-v*` tag, and AC-5 when the help article goes live on merge.
  - Next: after merge, mark this done once AC-4 and AC-5 hold (or record that AC-4 waits on the next release); close RAID I-24 and I-25; then WP-0010.
