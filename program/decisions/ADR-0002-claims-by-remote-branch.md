# ADR-0002: A pushed `wp/WP-XXXX-*` branch is the claim on a work package

- Status: Proposed
- Date: 2026-10-16
- Work package: WP-0001

## Context

Two agents must not start the same work package. A status change in the WP file is only visible to
others after it merges to `main`, which can take days. Assignment through GitHub Issues needs API
access some agents lack.

## Decision

The claim is the existence of a remote branch whose name starts with `wp/WP-XXXX`. `git push` of a
new branch name is atomic: if it already exists, the push is rejected and the second agent picks
other work. `programctl next` lists ready work packages and hides those with a remote `wp/` branch.
Agents whose tooling forces another branch name also push a claim marker
`wp/WP-XXXX-claim-<agent>` pointing at their branch. A draft PR titled `[WP-XXXX]` gives people
the same visibility in the GitHub UI.

## Consequences

- Claims are visible within seconds to every agent with `git` access.
- Stale claims (abandoned branches) must be cleaned up; the gate review lists `wp/` branches with no
  commits in 7 days.
