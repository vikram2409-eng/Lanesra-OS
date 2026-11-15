import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * Publish checks and dry run on the warehouse demo (WP-0714).
 * Covers: REQ-LANE-021 (WP-0714 AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4175 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Untested locally — see CI.
 */

const STATIC_PORT = 4175;
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

async function openBlueprint(page, laneId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-blueprint="${laneId}"]`);
  await expect(page.locator('#lanePublishChecks')).toBeVisible();
}

async function runDryRun(page) {
  await page.click('[data-dryrun-run]');
  await expect(page.locator('#laneDryRunResult')).toContainText('Dry-run result');
}

async function storeSnapshot(page): Promise<string> {
  return page.evaluate((key) => localStorage.getItem(key) || '', STORE_KEY);
}

test('AC-5: publish checks render on the Receiving blueprint with named P-12 refusals', async ({ page }) => {
  await openBlueprint(page, 'lane-wh-receiving');
  await expect(page.locator('#view h2')).toHaveText('Receive Shipment');
  // Every check P-01..P-16 is listed with its step and its reason.
  for (const id of ['P-01', 'P-02', 'P-03', 'P-04', 'P-05', 'P-06', 'P-07', 'P-08', 'P-09', 'P-10', 'P-11', 'P-12', 'P-13', 'P-14', 'P-15', 'P-16']) {
    await expect(page.locator('#lanePublishChecks')).toContainText(id);
  }
  // P-12: the demo has no adopted Action Type catalog, so every warehouse
  // Action Type is provisional and the lane cannot publish. The refusal
  // names the step and the type.
  await expect(page.locator('#lanePublishChecks')).toContainText('Cannot publish');
  await expect(page.locator('#lanePublishChecks')).toContainText('provisional Action Type');
  await expect(page.locator('#lanePublishChecks')).toContainText('purchase_order.close_receipt');
  await expect(page.locator('#lanePublishChecks')).toContainText('warehouse.post_stock_move');
  // The draft panel is not the publish surface: no publish control there.
  await expect(page.locator('#lanePublishChecks')).toContainText('no adopted Action Type catalog');
});

test('AC-5: a dry run walks Receiving on PO-2026-001 and writes nothing', async ({ page }) => {
  await openBlueprint(page, 'lane-wh-receiving');
  const before = await storeSnapshot(page);
  expect(before).toContain('PO-2026-001');

  await expect(page.locator('[data-dryrun-anchor]')).toHaveValue('po-2026-001');
  await runDryRun(page);

  const result = page.locator('#laneDryRunResult');
  await expect(result).toContainText('ends with outcome done');
  await expect(result).toContainText('matches the expected outcome');
  await expect(result).toContainText('Discrepancy check');
  await expect(result).toContainText('passed');
  await expect(result).toContainText('warehouse.post_stock_move');
  await expect(result).toContainText('RECEIPT');
  await expect(result).toContainText('PUTAWAY');
  await expect(result).toContainText('purchase_order.close_receipt');
  await expect(result).toContainText('Nothing was written');

  // The seed is unchanged: the whole demo store is byte-identical, so no
  // StockMove, no status change and no run was created by the dry run.
  const after = await storeSnapshot(page);
  expect(after).toBe(before);
  const store = JSON.parse(after);
  const po = store.purchase_order.find((r: any) => r.id === 'po-2026-001');
  expect(po.status).toBe('OPEN');
  expect(po.lines[0].qty_received).toBe(0);
  const stock1001 = store.stock.find((r: any) => r.product_id === 'prod-hg-1001');
  const stock1002 = store.stock.find((r: any) => r.product_id === 'prod-hg-1002');
  expect(stock1001.on_hand).toBe(48);
  expect(stock1002.on_hand).toBe(22);
});

test('AC-5: a Count dry run on CC-2026-001 flags the 2 zero-variance lines', async ({ page }) => {
  await openBlueprint(page, 'lane-wh-count');
  await expect(page.locator('#view h2')).toHaveText('Count');
  await expect(page.locator('[data-dryrun-anchor]')).toHaveValue('cc-2026-001');
  await runDryRun(page);

  const result = page.locator('#laneDryRunResult');
  // CC-2026-001 has variances -2, 0, +1, -2, 0: the check fails, the
  // manager approves, and the two zero-variance lines post nothing.
  await expect(result).toContainText('Check the variances');
  await expect(result).toContainText('failed');
  await expect(result).toContainText('would skip 2 children with nothing to post');
  await expect(result).toContainText('warehouse.post_stock_move');
  await expect(result).toContainText('COUNT_ADJUSTMENT');
  await expect(result).toContainText('ends with outcome done');
  await expect(result).toContainText('Nothing was written');
});

test('AC-5: a check with a missing fixture reports unevaluated', async ({ page }) => {
  await openBlueprint(page, 'lane-wh-receiving');
  // Clear the received quantity the discrepancy check reads: with no
  // fixture value the check cannot evaluate, and the dry run says so
  // instead of guessing.
  await page.fill('[data-dryrun-fixture="qty_received:pol-001-1"]', '');
  await runDryRun(page);

  const result = page.locator('#laneDryRunResult');
  await expect(result).toContainText('unevaluated (needs fixture)');
  await expect(result).toContainText('qty_received');
  await expect(result).toContainText('Nothing was written');
});

test('AC-5: the Replenish blueprint resolves by fetch and its dry run refuses to start', async ({ page }) => {
  // Replenish is not embedded in app.js: the blueprint page fetches its
  // definition from /program/spec/schemas/examples/lane-wh-replenish.json.
  await openBlueprint(page, 'lane-wh-replenish');
  await expect(page.locator('#view h2')).toHaveText('Replenish');
  await expect(page.locator('#lanePublishChecks')).toContainText('P-12');
  await expect(page.locator('[data-dryrun-anchor]')).toHaveValue('rr-01');
  await runDryRun(page);

  // Every seeded reorder rule has triggered: false, so the start
  // condition fails and the dry run refuses to start instead of walking.
  const result = page.locator('#laneDryRunResult');
  await expect(result).toContainText('would refuse to start');
  await expect(result).toContainText('Nothing was written');
});
