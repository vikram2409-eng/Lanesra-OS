# ADR-0027: External agent identity, A2A trust and trace ingest

- Status: Accepted (program owner, 2026-11-14: "I accept. Move forward")
- Date: 2026-11-14
- Requirements: REQ-FLEET-001
- Work packages: WP-0114 (this ADR); WP-0501 to WP-0505 build on it
- Design: SOLUTION-DESIGN.md section 8

## Context

The aim is one inventory, one policy set and one run history for every agent
an organisation runs, whatever built it. Phase 3 builds team tools and
Phase 5 builds the control plane. Without this decision, Phase 3 handoff and
team tools would assume every seat is a Lanesra agent: an external agent
would have no identity to audit, no way to be trusted or refused, and no
trace to show for its work.

This follows ADR-0006 (agents are principals with an Access Role and Record
Scope, audited in their own name; Agent Identities use Lanesra-issued,
short-lived, scoped credentials the platform holds and never passes to a
model), ADR-0008 (the trace and event schema: run → step → round → model
and tool calls) and ADR-0016 (identity and sign-in architecture).

## Decision

### 1. An external agent is a principal kind, bound to an Agent Identity

An agent built outside Lanesra is a **principal kind** — external agent —
bound to an Agent Identity (ADR-0006, WP-0108). The Agent Identity is the
Lanesra principal: it carries the Access Role and Record Scope, it is
audited in its own name, and it is what policies, budgets and the kill
switch act on. The external agent — its framework, its endpoint, its
protocol — is what fills that identity. Nothing outside Lanesra ever holds
a Lanesra credential; the platform holds the identity and speaks to the
agent on its behalf.

**Fields a Team seat needs to hold an external agent:**

| Field | Meaning |
|---|---|
| `external_agent_id` | The registry entry (WP-0501): owner, framework, endpoint, protocol, data classes, risk tier. |
| `framework` | What built it (for example, the framework named in the registry). Recorded, not trusted: it changes what the fleet view shows, not what the agent may do. |
| `endpoint` | Where Lanesra calls it. Must be registered; an unregistered endpoint is refused. |
| `protocol` | A2A. The seat speaks A2A to the agent and nothing else. |
| `data_classes` | The classes of data the agent may touch. A reply carrying a class outside this set is fenced. |
| `risk_tier` | The tier the firewall and the approval service use for its calls. |
| `approved` | Whether an AI Admin has approved this agent for this seat. Unapproved means uncallable. |
| `trust_level` | Called, observed, or called and observed (decision 4). |

**Fields a run step needs when it calls an external agent:**

| Field | Meaning |
|---|---|
| `external_agent_id` | Which agent the step called. |
| `correlation_id` | Joins the call to the ingested trace (decision 5). Carried in the A2A task and in the ingested spans. |
| `trust_level` | What Lanesra can and cannot do with this agent (decision 4). |
| `budget_consumed` | What the call has cost against the run's and the agent's budget, so the deny path in decision 6 can fire. |

### 2. Authentication to and from A2A peers

Each published Team has an **agent card** and an **A2A endpoint**. A caller
discovers the card, then sends a task; the task maps to a run. Approvals
surface to the caller as input required: the run waits for a person, and the
caller is told to wait rather than given a made-up answer.

- **Inbound (a peer calls Lanesra):** the caller authenticates with a
  scoped key for that Team's endpoint. The key names the Team and the
  scopes it may use; it never grants access to the token vault (ADR-0026)
  or to another Team's runs.
- **Outbound (Lanesra calls an external agent):** Lanesra authenticates
  with the seat's credential, held by the platform. It never sends a
  user's grant (ADR-0026) and never sends a Lanesra user password.

### 3. An external reply is labelled untrusted

A reply from an external agent is fenced as untrusted (WP-0145). It cannot
change instructions or the tool list, it cannot widen what the run may do,
and it is labelled untrusted in the run trace, so a reader can tell what
came from outside Lanesra. A tool call proposed by an external reply goes
through the same pipeline as any other — schema, permission, policy,
approval, execute, audit — and the firewall judges it on its own merits.

### 4. Trust levels

An external agent is one of three kinds, and the fleet view shows which:

| Kind | Lanesra can | Lanesra cannot |
|---|---|---|
| Called (A2A client) | Refuse, budget, pause, audit every call | See inside the agent's own steps unless it also sends traces |
| Observed (trace ingest) | Show runs and cost, raise findings and alerts | Stop the agent; numbers are self-reported |
| Called and observed | Both of the above, joined by a correlation id | — |

A called agent that also sends traces is the full case: Lanesra can refuse
its calls and can see what it did. An observed-only agent is shown for what
it reports, and its numbers are labelled self-reported — never presented as
Lanesra-measured.

### 5. Trace ingest

An OTLP receiver maps incoming spans onto the ADR-0008 span tree: run →
step → round → model and tool calls. The receiver:

- joins an ingested trace to the run that called the agent by the
  **correlation id** carried in the A2A task;
- keeps the external agent's own span ids alongside Lanesra's (a
  `trace_id` of 32 lowercase hex characters and a `span_id` of 16 hex
  characters, per ADR-0008), assigning Lanesra-shaped ids on ingest when
  the sender does not supply them;
- stores ingested spans in the same `ai_run_spans` table, marked
  **externally reported**;
- keeps ingested runs **read-only**: they cannot be edited, re-run or
  used as the basis for a new run;
- runs the same redaction hook as a Lanesra trace (ADR-0008 section 4)
  before anything is stored or emitted, under the workspace's
  `trace_bodies` setting.

An ingested trace never overwrites a Lanesra trace. Where the same run has
both — the call Lanesra made and the trace the agent sent — the two are
shown joined by the correlation id, and a disagreement between them (for
example, the agent reports a cost Lanesra did not measure) is shown as a
finding, not silently resolved.

### 6. Deny paths

A call to an external agent is refused, with a readable reason and **without
failing the run**, when:

- the endpoint is not registered;
- the agent is not approved;
- the agent is paused;
- its budget is spent.

The refusal is audited, the run step records it, and the run continues from
the next step the playbook allows (or waits for a person, if the playbook
says so). A refused call never falls back to a different agent, and never
falls back to an ungoverned direct call.

## Consequences

- WP-0501 (registry and approval), WP-0502 (A2A server), WP-0503 (A2A
  client), WP-0504 (trace ingest) and WP-0505 (fleet governance) build on
  this ADR. The registry is the gate: nothing is callable until an AI
  Admin approves it.
- WP-0301 (Team seat and version model) must carry the seat fields from
  decision 1; WP-0309 (handoff mode and team tools) must refuse a handoff
  to an unapproved or paused external agent; WP-0501 must record the
  registry fields and the approval. Each cites this ADR in Notes.
- The control plane is an inventory, not a takeover: an observed-only
  agent keeps running its own way, and Lanesra can only show what it
  reports. That limit is a feature of the trust model, not a gap to be
  closed by reaching into the agent.
