import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The anytime actions on the warehouse demo runner (WP-0711 AC-5).
 * Covers: REQ-LANE-018 (WP-0711 AC-1, AC-2, AC-3, AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4180 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Ports in use by the other demo specs: 4173 (warehouse-lane,
 * assistant-draft-lane), 4174 (warehouse-lane-blueprint), 4175
 * (lane-publish-dryrun), 4176 (lane-wh-child), 4177 (lane-builder-demo,
 * warehouse-lane-mobile), 4178 (lane-wh-count), 4179 (foundry-demo).
 */

const STATIC_PORT = 4180;
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

async function startReceivingLane(page, poId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-start="lane-wh-receiving:${poId}"]`);
  await expect(page.locator('.lane-step-card h3')).toBeVisible();
}

async function openRunForRecord(page, objectKey: string, recordId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`#sideNav [data-nav="${objectKey}"]`);
  await page.click(`[data-open-record="${objectKey}:${recordId}"]`);
}

test('AC-1: a note and an attachment land on the run and in its audit trail', async ({ page }) => {
  await startReceivingLane(page, 'po-2026-001');
  await expect(page.locator('.view-head h2')).toContainText('PO-2026-001 · Receive Shipment · stage 1 of 5');
  const card = page.locator('[aria-label="Notes, attachments and reassign"]');
  await expect(card).toBeVisible();

  await page.fill('#laneNoteText', 'Checked the truck seal against the delivery note.');
  await page.click('#laneAddNoteBtn');
  await expect(card).toContainText('Checked the truck seal against the delivery note.');
  await expect(card).toContainText('Sarah Chen');

  await page.setInputFiles('#laneAttachFile', {
    name: 'count-sheet.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('count sheet'),
  });
  await expect(card).toContainText('count-sheet.txt');

  const trail = page.locator('.lane-side [aria-label="Audit trail"], .lane-audit');
  await expect(trail).toContainText('Added a note to the run (step Confirm shipment arrived)');
  await expect(trail).toContainText('Attached count-sheet.txt to the run');
});

test('AC-2: reassign lists the role holders and narrows the step to one person', async ({ page }) => {
  await startReceivingLane(page, 'po-2026-001');
  const select = page.locator('#laneReassignTo');
  await expect(select).toBeVisible();
  // The first step waits on role_wh_receiver: Marcus Webb and Aisha Khan
  // hold RECEIVER; Sarah Chen (the manager) does not.
  await expect(select).toContainText('Marcus Webb');
  await expect(select).toContainText('Aisha Khan');
  await expect(select).not.toContainText('Sarah Chen');

  await select.selectOption({ label: 'Marcus Webb' });
  await page.fill('#laneReassignReason', 'Marcus is on shift');
  await page.click('#laneReassignBtn');
  await expect(page.locator('.lane-step-card')).toContainText('waiting on Marcus Webb');
  await expect(page.locator('.lane-audit')).toContainText(
    'Reassigned Confirm shipment arrived to Marcus Webb — Marcus is on shift',
  );
});

test('AC-3: an attachment is visible only on its own run', async ({ page }) => {
  await startReceivingLane(page, 'po-2026-001');
  await page.setInputFiles('#laneAttachFile', {
    name: 'count-sheet.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('count sheet'),
  });
  await expect(page.locator('[aria-label="Notes, attachments and reassign"]')).toContainText('count-sheet.txt');

  // The seeded PO-2026-005 run is a different run on a different record:
  // its card carries no attachments from this one.
  await openRunForRecord(page, 'purchase_order', 'po-2026-005');
  await page.click('[data-lane-open-run]');
  await expect(page.locator('.view-head h2')).toContainText('PO-2026-005');
  const otherCard = page.locator('[aria-label="Notes, attachments and reassign"]');
  await expect(otherCard).toBeVisible();
  await expect(otherCard).not.toContainText('count-sheet.txt');
});
