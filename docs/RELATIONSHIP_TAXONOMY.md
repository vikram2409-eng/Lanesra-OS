# Relationship Taxonomy — Dependency Explorer 2.0

Source of truth for "what counts as a relationship" in the Dependency Explorer
(type-level graph) and Object 360 (record-level graph).
Part of epic #359. Delivers issue #360.

Every edge is **directed** and **typed**. Direction reads as
*source —label→ target*. Custom and OOB components use the same edge labels;
OOB-vs-custom is a node flag, never an edge variant.

## Structural edges (what it's made of / shown on)

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `has_field` | Object → Field | Field belongs to the object's schema | Companies `has_field` → Industry |
| `shown_on` | Field → Screen Layout | Field is placed on a screen | Industry `shown_on` → Company Detail Screen |
| `renders` | Screen Layout → Object | Screen renders records of the object | Company Detail Screen `renders` → Companies |
| `displays` | Page Layout → Object | Page layout hosts the object | Company Record Page `displays` → Companies |
| `embeds` | Page Layout → Screen Layout | Screen is embedded in a page | Company Record Page `embeds` → Company Detail Screen |

## Relational edges (object-to-object, custom & OOB)

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `related_via` | Object → Relationship | Object participates via a relationship record | Companies `related_via` → Company–Contact link |
| `links_to` | Relationship → Object | Relationship points at the other object | Company–Contact link `links_to` → Contacts |
| `has_many` / `belongs_to` / `many_to_many` | Object → Object | Cardinality shortcut derived from the Relationship record (shown when "flatten relationships" is on) | Companies `has_many` → Contacts |

## Logic edges (rules, workflows, tests)

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `enforced_on` | Business Rule → Object | Rule fires on this object's records | "Require Industry" `enforced_on` → Companies |
| `validates` | Business Rule → Field | Rule constrains a specific field | "Require Industry" `validates` → Industry |
| `triggered_by` | Workflow → Object | Object event starts the workflow | "New company onboarding" `triggered_by` → Companies |
| `reads` / `writes` | Workflow → Field | Workflow reads/updates the field | "Company score sync" `writes` → Health Score |
| `covers` | Test Case → Component (any) | Test exercises the component | "Company dedupe test" `covers` → Companies |
| `enables` | Action Type → Workflow / AI Agent | Workflow or agent uses this action | "Send email" `enables` → "New company onboarding" |
| `acts_on` | Action Type → Object | Action operates on records of the object type | "Send Email" `acts_on` → Companies |

## Access edges (who can touch it)

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `can_read` / `can_update` / `can_delete` | AI Agent → Object / Field | Agent's granted data access, per permission | Ops Supervisor `can_update` → Companies |
| `can_invoke` | AI Agent → Workflow | Agent may execute the workflow | Ops Supervisor `can_invoke` → "Company score sync" |
| `constrained_by` | AI Agent → Business Rule | Rule limits what the agent may do | Ops Supervisor `constrained_by` → "Require Industry" |
| `delegates_to` | AI Agent → AI Agent | Agent hands off to another | Research Analyst `delegates_to` → Ops Supervisor |
| `includes` | Agent Team → AI Agent | Team membership | Sales Ops Team `includes` → Ops Supervisor |
| `manages` | Agent Team → Object | Team's scope covers the object | Sales Ops Team `manages` → Companies |

## Knowledge edges

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `computed_from` | Metric → Object / Field | Metric is derived from this data | "Pipeline by company" `computed_from` → Companies |
| `defines` | Glossary Term → Object / Field | Term gives the canonical meaning | "Account vs Company" `defines` → Companies |

## Record-level edges (Object 360 only)

Same labels where they apply, plus:

| Edge label | Source → Target | Meaning | Example |
|---|---|---|---|
| `linked_record` | Record → Record | Records connected via a relationship | BrightPath Logistics `linked_record` → Ava Martin |
| `has_business_unit` | Record → Business Unit | Record's owning business unit | BrightPath `has_business_unit` → Northstar Digital |
| `scoped_agent` | AI Agent → Object type | Agent is scoped to act on the type | Research Analyst `scoped_agent` → Companies |
| `applies_to` | Business Rule → Record | Rule's conditions match this record | "Require Industry" `applies_to` → BrightPath |
| `ran_on` | Workflow → Record | Workflow executed on this record | "Score sync" `ran_on` → BrightPath |
| `agent_can_read` / `agent_can_update` | AI Agent → Record | Agent's effective access to this record | Ops Supervisor `agent_can_update` → BrightPath |

## Legacy mapping

- `DEPENDS_ON` (old UI heading) becomes a **view**, not an edge kind: the
  detail panel groups edges under "Depends on" (outgoing) vs "Depended on by"
  (incoming).
- `DELEGATES_TO` maps 1:1 to `delegates_to`.
- No data migration — the crawler re-emits typed edges.

## Contract rule

The crawler MUST only emit labels from this taxonomy. A unit test asserts
every emitted edge label is in this set. Adding a label = updating this file
first.
