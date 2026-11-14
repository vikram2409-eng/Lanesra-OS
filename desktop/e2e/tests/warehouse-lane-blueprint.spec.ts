import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The lane blueprint on the warehouse demo (WP-0712).
 * Covers: REQ-LANE-020 (WP-0712 AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4174 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * The server is shared with WP-0702's lane spec (port 4173 there); this
 * spec uses 4174 so the two files never clash. FIX (PR #761): the first
 * version of this spec inlined its own server and resolved the repo root
 * from process.cwd() — it only worked when run from the repo root, and CI
 * runs Playwright from desktop/, so the server never started there
 * ("static site server did not start"). The shared server resolves the
 * root from its own file location, so it works from either directory.
 */

const STATIC_PORT = 4174;
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

test('the warehouse dashboard links to the Receive Shipment blueprint', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await expect(page.locator('[data-lane-blueprint="lane-wh-receiving"]')).toBeVisible();
  await page.click('[data-lane-blueprint="lane-wh-receiving"]');
  // The blueprint page: purpose, stage chevrons, steps, run data.
  await expect(page.locator('#view h2')).toHaveText('Receive Shipment');
  await expect(page.locator('#view')).toContainText('v1 · draft');
  await expect(page.locator('#view')).toContainText('Runs on purchase_order');
  for (const stage of ['Verify', 'Record receipt', 'Hold & review', 'Stage & put away', 'Confirm']) {
    await expect(page.locator('#view')).toContainText(stage);
  }
  for (const step of ['Confirm shipment arrived', 'Discrepancy check', 'Review discrepancy', 'Close the PO']) {
    await expect(page.locator('#view')).toContainText(step);
  }
  // Branches name their targets; run data and mapped record fields are listed.
  await expect(page.locator('#view')).toContainText('pass → Stage & put away · Stage the goods');
  await expect(page.locator('#view')).toContainText('fail → Hold & review · Review discrepancy');
  await expect(page.locator('#view')).toContainText('carrier');
  await expect(page.locator('#view')).toContainText('record.lines[].qty_received');
  // The print button is present (the export itself is the browser print
  // dialog, which Playwright cannot drive - the button is the contract).
  await expect(page.locator('[data-lane-blueprint-print]')).toBeVisible();
});
