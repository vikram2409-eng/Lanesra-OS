---
id: WP-0717
title: Three more warehouse lanes and lane templates
phase: 7
epic: E7.6 Shipped lanes
status: ready
owner: muse
planned_for: muse
depends_on: [WP-0707, WP-0710]
requirements: [REQ-LANE-025]
estimate: L
editions: [demo]
editions_reason: The warehouse is a demo showcase that proves the platform scales to any vertical app, not a shipped product app. The reusable building blocks it is built on - Process Lanes, Blueprint, the runner, authoring/visual builder and the template mechanism - already ship in desktop and Team Workspace (WP-0703..0718). The warehouse-specific lanes themselves are demo-only by design (owner decision 2026-11-27).
delivered: []
docs: [process-lanes]
branch: 
pr: 
updated: 2026-11-27
---

# WP-0717 — Three more warehouse lanes and lane templates

## Goal

Show the feature is general, and give admins lanes to start from.

## Scope

- Build the remaining warehouse lanes as a **showcase on the online demo** (`app.js` / the warehouse demo): Pick and Pack (sales order), Count (cycle count) and Replenish (reorder rule), alongside Receive Shipment (already shipped in the demo, WP-0702). Each is presented as an installable/editable lane in the demo so a viewer sees the platform's own template + authoring blocks being used to assemble a real vertical app.
- The warehouse is a **demo only** (see `editions_reason`). The reusable building blocks (Process Lanes, Blueprint, runner, authoring/visual builder, template mechanism) are the product and already ship in desktop + Team Workspace; do **not** add warehouse-specific lanes or objects to the product editions.
- Reuse existing definitions: Count/Replenish from WP-0719 (`program/spec/schemas/examples/lane-wh-count.json`, `lane-wh-replenish.json`); Receive Shipment from the demo definition. Only Pick and Pack is genuinely new.
- **Position the warehouse as the reference _industry accelerator_**: package the assembled warehouse (its lanes, templates, objects and sample data) as an installable **solution pack** on top of the Solution export building block (WP-0714), so the story is "this whole vertical app was built on the platform and can be installed in one step." The warehouse is the first of a family of accelerators — property management, recruitment, etc. are siblings built the same way — so keep the packaging generic (nothing warehouse-specific in the mechanism; the mechanism is the product, the accelerator is content).

## Acceptance criteria

- [ ] AC-1: Each lane completes end to end in the demo with the (simulated) records it should change
- [ ] AC-2: Each is offered as a lane template a demo admin can install and edit, exercising the product's template block
- [ ] AC-5: Online demo (`app.js`) shows the lanes on the warehouse sample data; demo E2E covers them
- [ ] AC-6: Website help (`/help/process-lanes` and any other slug in `docs:`) explains the warehouse demo built on Process Lanes and where to find it; `sitemap.xml`, `llms.txt`, `llms-full.txt`, the Releases page and the Roadmap page are updated

> Scope note (owner decision 2026-11-27): the former AC-3 ("warehouse lanes run in desktop, Team Workspace and the online demo") and AC-4 ("desktop and Team Workspace behave the same from the same code") are **removed**. The warehouse is a demo showcase, not a product feature; edition parity for the *building blocks* is already satisfied by WP-0703..0718. AC numbering is kept (1, 2, 5, 6) so it stays aligned with the assignment already handed to muse.

## Test plan

- Demo E2E for the new lanes (no `process_lane_warehouse.rs` product suite — the warehouse is demo-only)
- Tag every new test with `Covers: REQ-LANE-025` (and this WP id) so `programctl trace` finds it
- Full regression suite green; test count not lower than before (see testing/REGRESSION.md)

## Notes

- ADR-0023; design in SOLUTION-DESIGN.md section 17 (Process Lanes). ADR-0025 (demo lane / freeze lineage).
- REQ-LANE-025 is already demo-framed ("Receive Shipment on the warehouse demo ... Pick and Pack, Count and Replenish, each also offered as a lane template"), so this scope matches the requirement as written; no requirement change needed.
- Industry accelerators (owner direction 2026-11-27): the warehouse accelerator here is the reference exemplar of a product pattern — installable vertical solution packs (property, recruitment, …) assembled from the platform's own building blocks and shipped via Solution export (WP-0714). Building out further accelerators as a tracked line of work is a separate backlog item (to be filed); this WP only has to prove the pattern with the warehouse.

## Progress log

<!-- Append dated entries: YYYY-MM-DD agent: what changed, what is next. Newest last. This is the handoff point for the next agent. -->
- 2026-11-27 claude: Scope decided with the owner and WP set to ready for muse. Intent clarified: the warehouse is a demo showcase that proves the Lanesra OS platform scales to any vertical app — it is NOT a desktop/Team Workspace product feature. The reusable building blocks (Process Lanes, Blueprint, runner, authoring/visual builder, template mechanism) already ship in the product (WP-0703..0718); the warehouse-specific lanes are demo-only. Changed editions [desktop, workspace, demo] → [demo] with editions_reason; removed the product-delivery ACs (former AC-3/AC-4); re-scoped AC-1/AC-2/AC-5/AC-6 to the demo; reuse WP-0719 Count/Replenish definitions + the WP-0702 Receive Shipment demo definition, Pick and Pack is the only new lane. REQ-LANE-025 is already demo-framed, so no requirement change. owner unassigned → muse, status proposed → ready. Next (muse): build the three demo lanes + templates + demo E2E + docs; finalize (tick ACs, delivered [demo], status done, re-run programctl status) in the same PR.
