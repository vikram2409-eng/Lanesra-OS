import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * Describe a process: the Admin Assistant drafts a lane (WP-0715).
 * Covers: REQ-LANE-022 (WP-0715 AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts its own
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4173 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * The demo drafts deterministically from the description itself (no live
 * model call in a static site); the core tests prove the real model path.
 * Run locally 2026-11-12 (Chromium, 2 passed); CI runs it on app.js changes.
 */

const STATIC_PORT = 4173;
const BASE = `http://localhost:${STATIC_PORT}`;
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

const SAMPLE =
  'Receive a purchase order at the Toronto warehouse. Verify the delivery against the order. Record what arrived. Stage the goods in a receiving bin. Put the goods away in their bins. Confirm the order is received.';

test('AC-5: describing the receiving process drafts a lane, always a draft', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  const panel = page.locator('#process-lanes');
  await expect(panel).toBeVisible();
  await expect(panel.locator('h3')).toHaveText('Process Lanes — describe a process');

  // The sample receiving description is prefilled; the lane runs on a
  // purchase order.
  await expect(page.locator('#laneDraftDescription')).toHaveValue(SAMPLE);
  await expect(page.locator('#laneDraftObject')).toHaveValue('purchase_order');
  await expect(panel.locator('#laneDraftList')).toContainText('No drafts yet');

  await page.click('#laneDraftForm button[type="submit"]');
  await expect(page.locator('#laneDraftResult')).toContainText(
    'Drafted Receive a purchase order at the Toronto warehouse — 4 stages, 5 steps. Saved as a draft.',
  );

  // The draft lists with a Draft badge, its stages and its data fields -
  // and no publish control anywhere in the panel: a draft is never
  // published from here.
  const list = page.locator('#laneDraftList');
  await expect(list).toContainText('lane-draft-receive-purchase-order-toronto');
  await expect(list).toContainText('Draft');
  await expect(list).toContainText('4 stages');
  await expect(list).toContainText('5 steps');
  await expect(list).toContainText('Verify');
  await expect(list).toContainText('Stage & put away');
  await expect(list).toContainText('Confirm');
  await expect(list).toContainText('verify_the_delivery_against_note');
  await expect(panel).not.toContainText('Publish');

  // The drafted lane opens in the blueprint view (WP-0712): the same page
  // as the seeded lane's blueprint, showing the draft's stages, steps and
  // run-data fields - and nothing else. The button is bound after the
  // in-place list update, not only at first render.
  await page.locator('#laneDraftList [data-lane-blueprint]').click();
  await expect(page.locator('.page-heading h2')).toHaveText(
    'Receive a purchase order at the Toronto warehouse',
  );
  await expect(page.locator('.page-heading')).toContainText('v1 · draft');
  await expect(page.locator('.page-heading')).toContainText('Runs on purchase_order');
  await expect(page.getByText('Stage 1 of 4 — Verify')).toBeVisible();
  await expect(page.getByText('Stage 4 of 4 — Confirm')).toBeVisible();
  await expect(page.getByText('This blueprint shows the lane definition only')).toBeVisible();
  await expect(page.getByText('Print / Save as PDF')).toBeVisible();

  // The draft survives a reload: it is stored in the demo store.
  await page.reload();
  await expect(page.locator('#laneDraftList')).toContainText(
    'lane-draft-receive-purchase-order-toronto',
  );
});

test('AC-5: an empty description is refused and drafts nothing', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.fill('#laneDraftDescription', '   ');
  await page.click('#laneDraftForm button[type="submit"]');
  await expect(page.locator('#laneDraftResult')).toContainText('Describe the process first');
  await expect(page.locator('#laneDraftList')).toContainText('No drafts yet');
});
