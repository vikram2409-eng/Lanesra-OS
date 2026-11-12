---
id: WP-0584
title: Helm chart and reference deployment
phase: 5
epic: E5.9 Enterprise operations
status: proposed
owner: unassigned
planned_for: muse
depends_on: [WP-0421]
requirements: [REQ-OPS-005]
estimate: M
editions: [workspace]
delivered: []
editions_reason: deployment packaging for the server
docs: [set-up-team-workspace]
branch: 
pr: 
updated: 2026-11-11
---

# WP-0584 — Helm chart and reference deployment

## Goal

A supported way to run on Kubernetes.

## Scope

- Chart for server, workers and ingress; values for PostgreSQL and KMS; probes and limits; CI lint and smoke test

## Acceptance criteria

- [ ] AC-1: `helm lint` and a kind-cluster smoke test pass in CI
- [ ] AC-2: An upgrade keeps data and runs the pre-upgrade backup

## Test plan

- CI job `helm (chart)`
- Tag every new test with `Covers: <REQ-ID>` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- Scoped from ADR-0020 and SOLUTION-DESIGN.md; refine acceptance criteria (and split if larger than L) before moving to `ready`.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
