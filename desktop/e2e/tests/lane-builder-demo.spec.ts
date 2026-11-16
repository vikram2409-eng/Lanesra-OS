import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The visual lane builder on the warehouse demo (WP-0718).
 * Covers: REQ-LANE-024 (WP-0718 AC-5, demo half of AC-1).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4177 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Ports in use by the other demo specs: 4173 (warehouse-lane), 4174
 * (warehouse-lane-blueprint), 4175 (lane-publish-dryrun), 4176
 * (lane-wh-child).
 */

const STATIC_PORT = 4177;
const BASE = `http://localhost:${STATIC_PORT}`;
const STORE_KEY = 'lanesra-os-warehouse-demo-v1';
let server: ChildProcess | null = null;

test.beforeAll(async () => {
  const candidates = [
    path.resolve('e2e/static-site-server.mjs'),
    path.resolve('desktop/e2e/static-site-server.mjs'),
  ];
  const serverPath = candidates.find((p) => fs.existsSync(p));
  if (!serverPath) throw new Error('static-site-server.mjs not found (run from desktop/ or the repo root)');
  server = spawn(process.execPath, [serverPath, String(STATIC_PORT)], { stdio: 'pipe' });
  const deadline = Date.now() + 15000;
  for (;;) {
    try {
      const res = await fetch(`${BASE}/demo/warehouse`);
      if (res.ok) break;
    } catch {
      // not up yet
    }
    if (Date.now() > deadline) throw new Error('static site server did not start');
    await new Promise((r) => setTimeout(r, 250));
  }
});

test.afterAll(async () => {
  server?.kill();
});

async function openBuilder(page, laneId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-builder="${laneId}"]`);
  await expect(page.locator('#laneBuilder')).toBeVisible();
}

// Canonical JSON: sorted keys, so the round-trip compares modulo key order.
function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value as Record<string, unknown>)
      .sort()
      .map((k) => `${JSON.stringify(k)}:${canonical((value as Record<string, unknown>)[k])}`)
      .join(',')}}`;
  }
  return JSON.stringify(value) ?? 'null';
}

async function exportDef(page): Promise<any> {
  const raw = await page.locator('#laneBuilderExport').inputValue();
  return JSON.parse(raw);
}

test('AC-5: the builder loads Receiving with 13 steps and the child step', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  await expect(page.locator('#view h2')).toHaveText('Receive Shipment');
  // Five stage chevrons, thirteen step cards.
  await expect(page.locator('.lb-stage-chevron')).toHaveCount(5);
  await expect(page.locator('.lb-step-card')).toHaveCount(13);
  for (const stage of ['Verify', 'Record receipt', 'Hold & review', 'Stage & put away', 'Confirm']) {
    await expect(page.locator('#laneBuilderCanvas')).toContainText(stage);
  }
  // Order numbers are definition order (1.1, 1.2, 1.3, ... 5.2).
  await expect(page.locator('.lb-step-card').first()).toContainText('1.1');
  await expect(page.locator('.lb-step-card').last()).toContainText('5.2');
  // Step 1.3 is the child step: it names the child lane and its anchor.
  const child = page.locator('.lb-step-card[data-lb-step="verify/verify-supplier"]');
  await expect(child).toContainText('Verify the supplier');
  await expect(child).toContainText('lane-wh-supplier-check');
  await expect(child).toContainText('$record.supplier_id');
  // Branches are drawn arrows: the discrepancy check's pass/fail arrows
  // and the hold end marker are in the SVG branch map.
  await expect(page.locator('#laneBuilderBranches svg')).toContainText('pass');
  await expect(page.locator('#laneBuilderBranches svg')).toContainText('fail');
  await expect(page.locator('#laneBuilderBranches svg')).toContainText('held');
});

test('AC-1: round-trip holds — export equals the embedded definition modulo key order', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  const exported = await exportDef(page);
  // app.js loads as a module, so its top-level LANE_WH_RECEIVING const
  // is not reachable from evaluate. The comparison source is the shared
  // example file itself: the embedded const is verified against this
  // same file by program/tools/check_lane_schema.py (and was verified
  // identical by hand before this test was written).
  const res = await page.request.get(`${BASE}/program/spec/schemas/examples/lane-wh-receiving.json`);
  expect(res.ok()).toBe(true);
  const embedded = await res.json();
  expect(canonical(exported)).toBe(canonical(embedded));
});

test('AC-1: renaming a step is reflected in the export and the child step survives', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  await page.click('.lb-step-card[data-lb-step="verify/confirm-arrival"]');
  await expect(page.locator('#laneBuilderPanel')).toContainText('Confirm shipment arrived');
  await page.fill('[data-lb-field="name"]', 'Confirm the truck arrived');
  await page.keyboard.press('Tab');
  const exported = await exportDef(page);
  const first = exported.lane.stages[0].steps[0];
  expect(first.name).toBe('Confirm the truck arrived');
  expect(first.id).toBe('confirm-arrival');
  // The child step is untouched by the rename.
  const child = exported.lane.stages[0].steps[2];
  expect(child.id).toBe('verify-supplier');
  expect(child.child_lane).toEqual({ lane_id: 'lane-wh-supplier-check', anchor_record: '$record.supplier_id' });
  // The whole lane is still 13 steps.
  const count = exported.lane.stages.reduce((n: number, s: any) => n + s.steps.length, 0);
  expect(count).toBe(13);
});

test('AC-5: publish checks run live and publish is disabled naming the first failing check', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  // The SAME checks as the blueprint page: every P-01..P-16 listed.
  for (const id of ['P-01', 'P-02', 'P-03', 'P-04', 'P-05', 'P-06', 'P-07', 'P-08', 'P-09', 'P-10', 'P-11', 'P-12', 'P-13', 'P-14', 'P-15', 'P-16']) {
    await expect(page.locator('#lanePublishChecks')).toContainText(id);
  }
  await expect(page.locator('#lanePublishChecks')).toContainText('Cannot publish');
  // The first refusing check is P-12 (provisional Action Types) and the
  // publish button names it.
  const publish = page.locator('#laneBuilderPublish');
  await expect(publish).toBeDisabled();
  await expect(publish).toContainText('P-12');
  await expect(page.locator('#laneBuilder')).toContainText('provisional Action Type');
});

test('AC-5: the check-last warning appears and the one-click fix moves the check to the end', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  // Discrepancy check starts last in its stage, so there is no warning.
  await expect(page.locator('.lb-check-last-warn')).toHaveCount(0);
  // Move it up one: now a step follows it on the pass path.
  await page.click('.lb-step-card[data-lb-step="record-receipt/discrepancy-check"] [data-lb-move="record-receipt/discrepancy-check:up"]');
  const warn = page.locator('.lb-check-last-warn');
  await expect(warn).toHaveCount(1);
  await expect(warn).toContainText('discrepancy-check');
  await expect(warn).toContainText('Enter received quantities');
  // The P-10 warning shows in the live checks too.
  await expect(page.locator('#lanePublishChecks')).toContainText('P-10');
  // One click moves the check to the end of its stage.
  await page.click('[data-lb-fix-check-last="record-receipt/discrepancy-check"]');
  await expect(page.locator('.lb-check-last-warn')).toHaveCount(0);
  const exported = await exportDef(page);
  const stage = exported.lane.stages.find((s: any) => s.id === 'record-receipt');
  expect(stage.steps.map((s: any) => s.id)).toEqual(['enter-quantities', 'discrepancy-check']);
});

test('AC-5: the dry-run preview walks Receiving on PO-2026-001 and writes nothing', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  const before = await page.evaluate((key) => localStorage.getItem(key) || '', STORE_KEY);
  expect(before).toContain('PO-2026-001');

  await expect(page.locator('[data-dryrun-anchor]')).toHaveValue('po-2026-001');
  await page.click('[data-dryrun-run]');
  const result = page.locator('#laneDryRunResult');
  await expect(result).toContainText('Dry-run result');
  await expect(result).toContainText('ends with outcome done');
  await expect(result).toContainText('matches the expected outcome');
  await expect(result).toContainText('Discrepancy check');
  await expect(result).toContainText('passed');
  await expect(result).toContainText('Verify the supplier');
  await expect(result).toContainText('would start lane-wh-supplier-check on sup-01');
  await expect(result).toContainText('warehouse.post_stock_move');
  await expect(result).toContainText('RECEIPT');
  await expect(result).toContainText('PUTAWAY');
  await expect(result).toContainText('Nothing was written');

  const after = await page.evaluate((key) => localStorage.getItem(key) || '', STORE_KEY);
  expect(after).toBe(before);
});

test('AC-5: saving stores the edited lane as a draft and never touches the embedded definition', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  await page.click('.lb-step-card[data-lb-step="verify/confirm-arrival"]');
  await page.fill('[data-lb-field="name"]', 'Confirm the truck arrived');
  await page.keyboard.press('Tab');
  await page.click('#laneBuilderSave');
  await expect(page.locator('#laneBuilderSaveNote')).toContainText('Saved as a draft');
  const store = await page.evaluate((key) => JSON.parse(localStorage.getItem(key) || '{}'), STORE_KEY);
  const draft = (store.draftLanes as any[]).find((d) => d.id === 'lane-wh-receiving');
  expect(draft).toBeTruthy();
  expect(draft.source).toBe('builder');
  expect(draft.definition.lane.stages[0].steps[0].name).toBe('Confirm the truck arrived');
  // The embedded definition is untouched by the save: the blueprint
  // reads the seeded copy of the embedded const, so it still shows the
  // original step name, not the renamed one. (Scoped to the Process
  // Lanes card: the saved draft has its own View blueprint button with
  // the same lane id.)
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('section:has(h3:text-is("Process Lanes")) [data-lane-blueprint="lane-wh-receiving"]');
  await expect(page.locator('#view h2')).toHaveText('Receive Shipment');
  await expect(page.locator('#view')).toContainText('Confirm shipment arrived');
});

test('AC-5: Pick & Pack shows the nested condition builder and the per-line StockMove preview', async ({ page }) => {
  await openBuilder(page, 'lane-wh-pickpack');
  await expect(page.locator('.lb-step-card')).toHaveCount(13);
  await page.click('.lb-step-card[data-lb-step="pick/pick-check"]');
  const panel = page.locator('#laneBuilderPanel');
  // The damage check is a two-level condition: any of (all of (...),
  // all of (...)) — three groups in the tree.
  await expect(panel).toContainText('Pick check');
  await expect(panel.locator('[data-lb-cond-group]')).toHaveCount(3);
  await expect(panel).toContainText('damage_approved');
  // The operator list comes from the schema.
  const operators = await panel.locator('[data-lb-cond-operator] option').allTextContents();
  expect(operators).toContain('equals');
  expect(operators).toContain('is_not_empty');
  expect(operators).toContain('greater_than');
  // "What does this step post?" — the PICK move per line, with the
  // dry-run fixture quantities (picked = reserved).
  const perLane = page.locator('#laneBuilderPerLane');
  await expect(perLane).toContainText('what does this step post');
  await expect(perLane).toContainText('PICK');
  await expect(perLane).toContainText('HG-1011');
  await expect(perLane).toContainText('× 5');
  await expect(perLane).toContainText('HG-1002');
  await expect(perLane).toContainText('× 2');
});

test('AC-5: Count shows expected/counted/variance per line, the whole-count approval, and the no-filter warning', async ({ page }) => {
  await openBuilder(page, 'lane-wh-count');
  await expect(page.locator('.lb-step-card')).toHaveCount(7);
  const perLane = page.locator('#laneBuilderPerLane');
  // Per-line table: expected vs counted vs variance, variances highlighted.
  await expect(perLane).toContainText('48');
  await expect(perLane).toContainText('46');
  await expect(perLane).toContainText('-2');
  await expect(perLane).toContainText('+1');
  // The approval sees the whole count at once, not one line at a time.
  await expect(perLane).toContainText('whole count');
  // The repeating record step has no filter (Q-34 case): it posts for
  // every line, including the zero-variance ones.
  await expect(perLane).toContainText('has no filter');
  await expect(perLane).toContainText('zero variance');
});

test('AC-5: Replenish previews the draft PO and states the reject behaviour', async ({ page }) => {
  await openBuilder(page, 'lane-wh-replenish');
  await expect(page.locator('.lb-step-card')).toHaveCount(7);
  const perLane = page.locator('#laneBuilderPerLane');
  await expect(perLane).toContainText('Kiln & Clay Co.');
  await expect(perLane).toContainText('Stoneware Dinner Set, 16-pc');
  await expect(perLane).toContainText('40');
  await expect(perLane).toContainText('1,640');
  // On reject the draft stays DRAFT: nothing is submitted.
  await expect(perLane).toContainText('stays DRAFT');
  await expect(perLane).toContainText('nothing is submitted');
});

test('AC-5: New lane starts blank and its publish button is disabled until it has content', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('[data-lane-builder-new]');
  await expect(page.locator('#laneBuilder')).toBeVisible();
  await expect(page.locator('#view h2')).toHaveText('New lane');
  await expect(page.locator('.lb-step-card')).toHaveCount(1);
  // A blank lane passes every REFUSE check, but Publish is disabled —
  // enabled-then-no-op was the old behaviour (gap 13 cosmetic fix).
  const publish = page.locator('#laneBuilderPublish');
  await expect(publish).toBeDisabled();
  await expect(publish).toContainText('blank lane');
  // Adding a field makes the lane non-blank: Publish enables, and the
  // click still says the demo cannot publish.
  await page.click('.lb-step-card[data-lb-step="work/first-step"]');
  await page.click('[data-lb-ff-add="work/first-step"]');
  await expect(publish).toBeEnabled();
  await page.click('#laneBuilderPublish');
  await expect(page.locator('#laneBuilderPublishNote')).toContainText('demo cannot publish');
});

test('AC-5: the blueprint page opens the same lane in the builder', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('[data-lane-blueprint="lane-wh-receiving"]');
  await expect(page.locator('#view h2')).toHaveText('Receive Shipment');
  await page.click('[data-lane-builder="lane-wh-receiving"]');
  await expect(page.locator('#laneBuilder')).toBeVisible();
  await expect(page.locator('.lb-step-card')).toHaveCount(13);
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): Add step / Add stage grow a new lane', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('[data-lane-builder-new]');
  await expect(page.locator('.lb-step-card')).toHaveCount(1);
  await page.click('[data-lb-step-add="work"]');
  await expect(page.locator('.lb-step-card')).toHaveCount(2);
  await page.click('[data-lb-stage-add]');
  await expect(page.locator('.lb-step-card')).toHaveCount(3);
  await expect(page.locator('.lb-stage-chevron')).toHaveCount(2);
  const exported = await exportDef(page);
  expect(exported.lane.stages).toHaveLength(2);
  expect(exported.lane.stages[0].steps).toHaveLength(2);
  expect(exported.lane.stages[1].steps).toHaveLength(1);
  expect(exported.lane.stages[1].steps[0].id).toBe('new-step-2');
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): deleting a step removes the branches that pointed at it and says so', async ({ page }) => {
  page.on('dialog', (d) => d.accept());
  await openBuilder(page, 'lane-wh-receiving');
  await expect(page.locator('.lb-step-card')).toHaveCount(13);
  // assign-staging-bin is the target of two branches (discrepancy-check
  // pass, review-discrepancy approved).
  await page.click('[data-lb-step-delete="stage-put-away/assign-staging-bin"]');
  await expect(page.locator('.lb-step-card')).toHaveCount(12);
  await expect(page.locator('.lb-delete-note')).toContainText('Also removed 2 branch(es)');
  const exported = await exportDef(page);
  const allBranches = exported.lane.stages.flatMap((s: any) => s.steps.flatMap((st: any) => st.branches || []));
  expect(allBranches.some((b: any) => b.target && b.target.step === 'assign-staging-bin')).toBe(false);
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): deleting a stage removes it and its steps', async ({ page }) => {
  page.on('dialog', (d) => d.accept());
  await openBuilder(page, 'lane-wh-count');
  await expect(page.locator('.lb-step-card')).toHaveCount(7);
  await page.click('[data-lb-stage-delete="hold-review"]');
  await expect(page.locator('.lb-step-card')).toHaveCount(5);
  await expect(page.locator('.lb-delete-note')).toContainText("Removed stage 'Hold & review' and its 2 steps");
  const exported = await exportDef(page);
  expect(exported.lane.stages.map((s: any) => s.id)).not.toContain('hold-review');
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): the run-data editor declares a field and refuses a duplicate key', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('[data-lane-builder-new]');
  await expect(page.locator('#laneBuilder')).toContainText('No run-data fields declared yet');
  await page.fill('#lbRunDataKey', 'carrier');
  await page.selectOption('#lbRunDataType', 'string');
  await page.check('#lbRunDataRequired');
  await page.click('[data-lb-rundata-add]');
  await expect(page.locator('.lb-plumbing')).toContainText('carrier');
  let exported = await exportDef(page);
  expect(exported.lane.run_data.fields).toEqual([{ key: 'carrier', type: 'string', required: true }]);
  // A duplicate key is refused inline and adds nothing.
  await page.fill('#lbRunDataKey', 'carrier');
  await page.click('[data-lb-rundata-add]');
  await expect(page.locator('#laneBuilder')).toContainText('already declared');
  exported = await exportDef(page);
  expect(exported.lane.run_data.fields).toHaveLength(1);
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): Receiving shows no wall of missing-object refusals and no "not evaluated" flood', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  const checks = page.locator('#lanePublishChecks');
  // P-11 and P-13 pass: the warehouse objects and workflows exist in the seed.
  await expect(checks.locator('tbody tr', { hasText: 'P-11' })).toContainText('Pass');
  await expect(checks.locator('tbody tr', { hasText: 'P-13' })).toContainText('Pass');
  await expect(checks).not.toContainText('not evaluated');
  // P-09 reasons are labelled by half.
  await expect(checks).toContainText('reachability + outcome coverage');
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): a branch target list excludes its own step', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  await page.click('.lb-step-card[data-lb-step="record-receipt/discrepancy-check"]');
  const panel = page.locator('#laneBuilderPanel');
  await expect(panel.locator('option[value="record-receipt/discrepancy-check"]')).toHaveCount(0);
  // End outcomes now include done and rejected as well as held.
  await expect(panel.locator('option[value="end:done"]')).toHaveCount(2);
  await expect(panel.locator('option[value="end:rejected"]')).toHaveCount(2);
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): Add missing branch defaults to the next step, else ends held', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('[data-lane-builder-new]');
  await page.click('[data-lb-step-add="work"]');
  // The first step becomes a check; its pass branch targets the next step.
  await page.click('.lb-step-card[data-lb-step="work/first-step"]');
  await page.selectOption('[data-lb-field="kind"]', 'check');
  await page.click('[data-lb-branch-add="work/first-step"]');
  let exported = await exportDef(page);
  expect(exported.lane.stages[0].steps[0].branches[0]).toEqual({ on: 'pass', target: { stage: 'work', step: 'new-step' } });
  // The last step has no next step: its branch ends the run held.
  await page.click('.lb-step-card[data-lb-step="work/new-step"]');
  await page.selectOption('[data-lb-field="kind"]', 'check');
  await page.click('[data-lb-branch-add="work/new-step"]');
  exported = await exportDef(page);
  expect(exported.lane.stages[0].steps[1].branches[0]).toEqual({ on: 'pass', end: { outcome: 'held' } });
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): Count assignment and skip filter write through; the per-line table stays a read-only preview', async ({ page }) => {
  await openBuilder(page, 'lane-wh-count');
  const perLane = page.locator('#laneBuilderPerLane');
  // The per-line table has no inputs: counted values are dry-run fixtures.
  await expect(perLane.locator('table input')).toHaveCount(0);
  await expect(perLane).toContainText('Read-only preview');
  await expect(perLane).toContainText('dry-run fixtures');
  // Assignment: a child field on the repeating form step.
  await perLane.locator('[data-lb-count-assign-by]').selectOption('child_field');
  await perLane.locator('[data-lb-count-assign-field]').fill('counted_by');
  await page.keyboard.press('Tab');
  // Skip: zero-variance lines on the repeating record step.
  await page.click('[data-lb-count-skip-add]');
  await perLane.locator('[data-lb-count-skip-field]').fill('$item.variance');
  await page.keyboard.press('Tab');
  await perLane.locator('[data-lb-count-skip-operator]').selectOption('equals');
  await perLane.locator('[data-lb-count-skip-value]').fill('0');
  await page.keyboard.press('Tab');
  const exported = await exportDef(page);
  const steps = exported.lane.stages.flatMap((s: any) => s.steps);
  expect(steps.find((st: any) => st.id === 'count-lines').repeat_over.assign).toEqual({ by: 'child_field', field: 'counted_by' });
  expect(steps.find((st: any) => st.id === 'post-count-adjustments').skip_when).toEqual({ field: '$item.variance', operator: 'equals', value: 0 });
  await expect(perLane).toContainText('Skips lines where $item.variance equals 0');
  await expect(perLane).not.toContainText('has no filter');
  await expect(perLane).toContainText('WP-0710');
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): the Replenish supplier select lists suppliers, not reorder rules', async ({ page }) => {
  await openBuilder(page, 'lane-wh-replenish');
  const picker = page.locator('[data-lb-supplier-pick]');
  await expect(picker).toHaveValue('sup-01');
  const options = await picker.locator('option').allTextContents();
  expect(options).toContain('Kiln & Clay Co. (sup-01)');
  expect(options).toContain('Northern Linen Mills (sup-02)');
  // The dry-run anchor is a different control: it lists reorder rules.
  await expect(page.locator('[data-dryrun-anchor]')).toHaveValue('rr-01');
  await picker.selectOption('sup-02');
  await expect(page.locator('#laneBuilderPerLane')).toContainText('Northern Linen Mills');
  // The pick also sets the supplier dry-run fixture.
  await expect(page.locator('[data-dryrun-fixture="supplier_id"]')).toHaveValue('sup-02');
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): the replaces editor writes through to the export', async ({ page }) => {
  await openBuilder(page, 'lane-wh-replenish');
  await page.click('.lb-step-card[data-lb-step="approve/approve-draft-po"]');
  const panel = page.locator('#laneBuilderPanel');
  await expect(panel).toContainText('Replaces');
  await expect(panel.locator('[data-lb-replaces-wf]')).toHaveValue('wf-wh-02');
  await panel.locator('[data-lb-replaces-wf]').selectOption('wf-wh-03');
  let exported = await exportDef(page);
  let step = exported.lane.stages.flatMap((s: any) => s.steps).find((st: any) => st.id === 'approve-draft-po');
  expect(step.replaces).toEqual([{ workflow_id: 'wf-wh-03', action_index: 0 }]);
  // Add a replaces entry to a step that has none.
  await page.click('.lb-step-card[data-lb-step="review/confirm-rule"]');
  await page.click('[data-lb-replaces-add="review/confirm-rule"]');
  exported = await exportDef(page);
  step = exported.lane.stages.flatMap((s: any) => s.steps).find((st: any) => st.id === 'confirm-rule');
  expect(step.replaces).toEqual([{ workflow_id: 'wf-wh-01', action_index: 0 }]);
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): the resolve control writes a latest-record lookup onto the supplier field', async ({ page }) => {
  await openBuilder(page, 'lane-wh-replenish');
  await page.click('.lb-step-card[data-lb-step="review/confirm-rule"]');
  const panel = page.locator('#laneBuilderPanel');
  await page.click('[data-lb-resolve-add="review/confirm-rule|1"]');
  await expect(panel).toContainText('Resolve from latest record');
  await expect(panel).toContainText('Runtime support for resolve is a follow-up');
  const exported = await exportDef(page);
  const step = exported.lane.stages.flatMap((s: any) => s.steps).find((st: any) => st.id === 'confirm-rule');
  expect(step.fields[1].resolve).toEqual({
    kind: 'latest_record',
    object: 'purchase_order',
    where: { product_id: '$record.product_id' },
    order_by: 'order_date',
    direction: 'desc',
    take: 1,
    field: 'supplier_id',
  });
});

test('Covers: REQ-LANE-024 (WP-0718 AC-5): the step role is a picker over the six warehouse roles and saves through', async ({ page }) => {
  await openBuilder(page, 'lane-wh-receiving');
  // Step 1.1 starts as the receiver.
  await page.click('.lb-step-card[data-lb-step="verify/confirm-arrival"]');
  const panel = page.locator('#laneBuilderPanel');
  const role = panel.locator('[data-lb-field="role"]');
  await expect(role).toHaveValue('role_wh_receiver');
  const options = await role.locator('option').allTextContents();
  expect(options).toContain('Warehouse Manager — role_wh_warehouse_manager');
  expect(options).toContain('Receiver — role_wh_receiver');
  expect(options).toContain('Picker — role_wh_picker');
  expect(options).toContain('Packer — role_wh_packer');
  expect(options).toContain('Admin — role_wh_admin');
  expect(options).toContain('Viewer — role_wh_viewer');
  await expect(role.locator('option')).toHaveCount(6);
  // Change the role, save, and check the stored draft carries it.
  await role.selectOption('role_wh_warehouse_manager');
  await page.click('#laneBuilderSave');
  await expect(page.locator('#laneBuilderSaveNote')).toContainText('Saved as a draft');
  const store = await page.evaluate((key) => JSON.parse(localStorage.getItem(key) || '{}'), STORE_KEY);
  const draft = (store.draftLanes as any[]).find((d) => d.id === 'lane-wh-receiving');
  expect(draft).toBeTruthy();
  expect(draft.definition.lane.stages[0].steps[0].role).toBe('role_wh_warehouse_manager');
  // The export agrees with the stored draft.
  const exported = await exportDef(page);
  expect(exported.lane.stages[0].steps[0].role).toBe('role_wh_warehouse_manager');
});
