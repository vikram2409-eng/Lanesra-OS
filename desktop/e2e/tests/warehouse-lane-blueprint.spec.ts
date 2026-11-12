import { test, expect } from '@playwright/test';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The lane blueprint on the warehouse demo (WP-0712).
 * Covers: REQ-LANE-020 (WP-0712 AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts its own
 * zero-dependency static server on port 4174 and drives /demo/warehouse
 * through it. Each test gets a fresh browser context, so localStorage
 * (the demo store) starts from the seed. The server is inline here
 * rather than shared with WP-0702's static-site-server.mjs: this WP's
 * branch stacks on WP-0703, which does not have that file, and the spec
 * must stay green on its own branch. Untested locally — see CI.
 */

const STATIC_PORT = 4174;
const BASE = `http://localhost:${STATIC_PORT}`;
let server: http.Server | null = null;

test.beforeAll(async () => {
  const candidates = [path.resolve('..', '..'), process.cwd()];
  const root =
    candidates.find((p) => fs.existsSync(path.join(p, 'app.js'))) ?? candidates[0];
  const types: Record<string, string> = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.txt': 'text/plain',
    '.xml': 'application/xml',
  };
  server = http.createServer((req, res) => {
    const url = decodeURIComponent((req.url || '/').split('?')[0]);
    let file = path.join(root, url);
    if (!file.startsWith(root) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
      file = path.join(root, 'index.html');
    }
    fs.readFile(file, (err, body) => {
      if (err) {
        res.writeHead(404);
        res.end('not found');
        return;
      }
      res.writeHead(200, { 'content-type': types[path.extname(file)] || 'application/octet-stream' });
      res.end(body);
    });
  });
  await new Promise<void>((resolve) => server?.listen(STATIC_PORT, resolve));
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
  await new Promise<void>((resolve) => server?.close(() => resolve()));
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
