# ADR-0020: Platform extension — Phase 5 and two interop items pulled forward

- Status: Accepted (program owner instruction, 2026-11-11: "add your recommendation to the build program in the right sequence and with the right dependencies")
- Date: 2026-11-11
- Requirements: REQ-FLEET-001..006, REQ-CRED-001..007, REQ-CHAN-001..007, REQ-EVAL-013..017, REQ-OPT-001..003, REQ-KNOW-001..004, REQ-HUM-001..003, REQ-DEV-001..005, REQ-OPS-001..006, REQ-COMP-001..004, REQ-NFR-006; moves REQ-API-010 and REQ-API-012
- Design: [`../design/SOLUTION-DESIGN.md`](../design/SOLUTION-DESIGN.md)

## Context

The October spec compared Lanesra with four agent *frameworks* (CrewAI, Microsoft Agent Framework,
LangGraph, AutoGen). A review on 2026-11-11 compared the plan with the agent *platforms* around
them (hosted tracing and evaluation services, cloud agent services, business-user agent builders).
The 245 requirements already cover the frameworks. Ten capability areas that buyers of a platform
expect were absent: governing agents built elsewhere, channels, per-user delegated credentials,
deeper evaluation, an optimizer, ontology-grounded retrieval, people as team members, a developer
surface, enterprise operations and a compliance pack.

The review used the reviewer's knowledge of those platforms up to mid-2026; no source code was read
(clean room, REQ-CLEAN-001). Where a work package needs protocol detail it starts with a behaviour
note in `program/research/`.

## Decision

1. **Add ten areas to the requirements register** (51 requirements) under "Platform extension". The
   October spec file stays unchanged; the design for the extension is `design/SOLUTION-DESIGN.md`.
2. **Add Phase 5 · Platform and ecosystem**, work packages `WP-05xx` in `work/P5/`, closed by gate
   G5 (`gates/G5-platform-release.md`). Build order is `0 → 1 → 2 → 3 → 4 → 5 → v2`.
3. **Phase 5 is not held behind G4.** A Phase 5 work package may be refined and started once every
   work package in its `depends_on` is done and the G3 security gate has passed, so Phase 5 lanes
   can run beside Phase 4. Nothing in Phase 5 is on the path to G4.
4. **Two design-only work packages go into Phase 1** (WP-0114 external agent identity and A2A
   trust; WP-0147 delegated credentials and MCP gateway). They produce ADRs, no code. They are
   early because Phase 3 builds seats, handoff and team tools; those must leave room for an
   external agent in a seat and for a per-user credential behind a tool. They do not join the G1
   gate checks.
5. **Pull forward from v2:** REQ-API-010 (Teams callable over A2A) moves to Phase 5 as WP-0502;
   REQ-API-012 (declarative export) moves to Phase 5 as WP-0570. WP-0901 keeps AG-UI only.
6. **The Python SDK decided in ADR-0017 gets a work package** (WP-0572), with a TypeScript SDK
   (WP-0573). The spec's non-goal "a general-purpose Python SDK" is narrowed: the SDK is a client
   of the public API; Lanesra stays no-code-first and gains no in-process agent framework.
7. **Deferred, not planned:** multimodal and voice agents, gateway cost routing and caching,
   computer-use tools, a public hub, policy as code, event-stream triggers. Each needs its own ADR.

## Phase 5 lanes and entry dependencies

| Lane | Work packages | Enters after |
|---|---|---|
| Agent control plane | 0114 → 0501 → 0503, 0504 → 0505; 0502 | Teams (0301), handoff (0309), run APIs (0325), OTLP (0404), kill switch (0405) |
| Delegated credentials and MCP gateway | 0147 → 0510 → 0511; 0512 → 0513 | OIDC (0102), MCP request gate (0142) |
| Channels | 0520 → 0521, 0522, 0523, 0524 → 0525 | Team as an endpoint (0411), approvals inbox (0218) |
| Evaluation depth | 0530, 0531, 0532, 0533 | Deeper evaluation (0212), publish gate (0213), content labels (0406) |
| Agent optimizer | 0540 → 0541 | Deeper evaluation (0212), model catalog (0146) |
| Ontology-grounded knowledge | 0550 → 0551, 0552; 0553 | Composite recall (0320), outcome metrics (0412) |
| Humans as seats | 0560 → 0561 | Teams (0301), approval routing (0323) |
| Developer surface | 0570 → 0571; 0572 → 0573; 0574 | Run APIs (0325), sandbox (0416) |
| Enterprise operations | 0580, 0581, 0582, 0583, 0584, 0585 | Directory mapping (0104), audit (0402), PostgreSQL (0421), worker pool (0401) |
| AI compliance | 0590 → 0591 → 0592 | Registry (0501), audit (0402) |
| Gate | 0599 | The last package of each security-relevant lane |

## Alternatives considered

- **Fold the new work into Phases 1–4.** Rejected: 143 of 158 work packages in those phases were
  still open; widening them would move every gate.
- **Leave A2A in v2.** Rejected: governing external agents is the lead differentiator and needs A2A
  in both directions.
- **A new spec HTML file.** Rejected: a Markdown design document is readable by an AI that works
  through GitHub and renders on the dashboard from one source.

## Consequences

- The plan grows by 44 work packages and 144 points (203 packages in total).
- `programctl` knows phase `5`; the dashboard shows Phase 5, gate G5 and a Design tab.
- WP-0301, WP-0309 and WP-0142 must read the ADRs from WP-0114 and WP-0147 before they start.
- The threat model (WP-0011) gains the new surfaces when WP-0114 and WP-0147 are written.
