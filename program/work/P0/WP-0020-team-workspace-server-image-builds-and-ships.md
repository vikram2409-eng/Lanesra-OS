---
id: WP-0020
title: Team Workspace server image builds, is pinned, and ships
phase: 0
epic: E0.1 Program and engineering readiness
status: ready
owner: unassigned
depends_on: [WP-0003]
requirements: [REQ-PRE-018, REQ-PRE-015]
estimate: M
editions: [workspace]
editions_reason: Packaging of the Team Workspace server only; the desktop app has its own release workflow and the online demo is static files
delivered: []
docs: [set-up-team-workspace]
branch: 
pr: 
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

- [ ] AC-1: `docker build` succeeds from a clean checkout and uses `Cargo.lock` (the build fails if the lockfile is out of date)
- [ ] AC-2: The Rust and Node versions in the image match `rust-toolchain.toml` and `.nvmrc`; a CI check fails if they drift
- [ ] AC-3: The `server image` CI job runs the smoke test above on every pull request, and fails when it fails
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
