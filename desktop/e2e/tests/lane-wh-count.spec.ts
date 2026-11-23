import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The Count lane on the warehouse demo (WP-0710 AC-5).
 * Covers: REQ-LANE-017 (WP-0710 AC-1, AC-2, AC-3, AC-5).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4178 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Ports in use by the other demo specs: 4173 (warehouse-lane,
 * assistant-draft-lane), 4174 (warehouse-lane-blueprint), 4175
 * (lane-publish-dryrun), 4176 (lane-wh-child), 4177 (lane-builder-demo,
 * warehouse-lane-mobile).
 * Run locally 2026-11-18: 4/4 pass.
 */

const STATIC_PORT = 4178;
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

async function startCountLane(page, ccId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-start="lane-wh-count:${ccId}"]`);
  await expect(page.locator('.lane-step-card h3')).toBeVisible();
}

async function openCountRecord(page, ccId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('#sideNav [data-nav="cycle_count"]');
  await page.click(`[data-open-record="cycle_count:${ccId}"]`);
}

async function completeStep(page) {
  await page.click('#laneCompleteBtn');
}

async function completeItem(page, lineId: string) {
  await page.click(`[data-lane-complete-item="${lineId}"]`);
}

async function expectStep(page, name: string) {
  await expect(page.locator('.lane-step-card h3')).toHaveText(name);
}

test('AC-1: a repeating form step completes one item at a time, each with its assignee', async ({ page }) => {
  await startCountLane(page, 'cc-2026-001');
  await expect(page.locator('.view-head h2')).toContainText('CC-2026-001 · Count · stage 1 of 5');
  await expectStep(page, 'Count the lines');

  // Every line shows who it is assigned to (Q-33: counted_by per line).
  const groups = page.locator('.lane-line-group');
  await expect(groups).toHaveCount(5);
  for (let i = 0; i < 5; i++) {
    await expect(groups.nth(i)).toContainText('Assigned to: Emily Zhao');
  }
  // The counted quantities are prefilled from the seed.
  await expect(page.locator('[data-lane-field="ccl-001-1:counted_qty"]')).toHaveValue('46');
  await expect(page.locator('[data-lane-field="ccl-001-2:counted_qty"]')).toHaveValue('22');

  // One item done: the step stays put, the other items are untouched.
  await completeItem(page, 'ccl-001-1');
  await expectStep(page, 'Count the lines');
  await expect(page.locator('.lane-line-done')).toHaveCount(1);
  await expect(groups.nth(0)).toContainText('✓ Done');
  await expect(page.locator('[data-lane-field="ccl-001-2:counted_qty"]')).toHaveValue('22');
  await expect(groups.nth(1)).toContainText('Mark done');
  // The audit trail records the item as it is done.
  await expect(page.locator('.lane-audit')).toContainText('HG-1001 (expected 48, counted 46)');

  // The remaining four items complete the step. Both checks run on their
  // own: verify passes, the variances fail, and the run waits on the manager.
  await completeItem(page, 'ccl-001-2');
  await completeItem(page, 'ccl-001-3');
  await completeItem(page, 'ccl-001-4');
  await completeItem(page, 'ccl-001-5');
  await expectStep(page, 'Review the variances');
});

test('AC-5: approving the variances posts only the nonzero adjustments', async ({ page }) => {
  await startCountLane(page, 'cc-2026-001');
  await completeStep(page); // Count the lines, all five at once
  await expectStep(page, 'Review the variances');

  // The approval shows expected vs counted vs variance per line.
  const card = page.locator('.lane-step-card');
  await expect(card).toContainText('HG-1001');
  await expect(card).toContainText('-2');
  await expect(card).toContainText('HG-1003');
  await expect(card).toContainText('+1');

  await page.click('#laneApproveBtn');
  await expectStep(page, 'Post count adjustments');
  await completeStep(page);

  // The two zero-variance lines posted nothing; the audit trail says so.
  await expect(page.locator('.lane-audit')).toContainText('Skipped 2 lines with nothing to post');
  const writes = page.locator('.lane-writes');
  await expect(writes).toContainText('COUNT_ADJUSTMENT HG-1001 × -2');
  await expect(writes).toContainText('COUNT_ADJUSTMENT HG-1003 × 1');
  await expect(writes).toContainText('COUNT_ADJUSTMENT HG-1005 × -2');
  await expect(writes).not.toContainText('HG-1002');
  await expect(writes).not.toContainText('HG-1013');

  await expectStep(page, 'Close the count');
  await completeStep(page);
  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');
  await expect(page.locator('.lane-side')).toContainText('APPROVED');
  // Stock on hand moved by the variances: HG-1001 48 -> 46, HG-1003 14 -> 15.
  const hg1001 = page.locator('.lane-po-lines tbody tr').filter({ hasText: 'HG-1001' });
  await expect(hg1001.locator('td').last()).toHaveText('46');
  const hg1003 = page.locator('.lane-po-lines tbody tr').filter({ hasText: 'HG-1003' });
  await expect(hg1003.locator('td').last()).toHaveText('15');
});

test('AC-2: a count with no variance skips Post adjustments and closes', async ({ page }) => {
  // CC-2026-011 counts two lines with no variance. Start it from its record.
  await openCountRecord(page, 'cc-2026-011');
  await page.click('[data-lane-start="lane-wh-count:cc-2026-011"]');
  await expect(page.locator('.view-head h2')).toContainText('CC-2026-011 · Count · stage 1 of 5');
  await expectStep(page, 'Count the lines');

  await completeItem(page, 'ccl-011-1');
  await completeItem(page, 'ccl-011-2');
  // Both checks pass, and the run enters Post adjustments — whose skip
  // condition holds (the count is still COUNTING, so there is nothing to
  // post) — so the stage is skipped and the run lands on Close the count.
  await expectStep(page, 'Close the count');
  await expect(page.locator('.lane-swim.skipped')).toContainText('Post adjustments');
  await expect(page.locator('.lane-audit')).toContainText('stage skipped ($record.status equals "COUNTING")');

  await completeStep(page);
  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');
  await expect(page.locator('.lane-side')).toContainText('APPROVED');
  await expect(page.locator('.lane-writes')).not.toContainText('COUNT_ADJUSTMENT');
});

test('AC-3: a count that is not COUNTING cannot start, and the record says why', async ({ page }) => {
  // CC-2026-002 is APPROVED, so the Count lane's start condition fails.
  await openCountRecord(page, 'cc-2026-002');
  const card = page.locator('.card', { hasText: 'Process Lane — Count' });
  await expect(card).toContainText('This lane cannot start on CC-2026-002.');
  await expect(card).toContainText('Its start condition is not met');
  await expect(card).toContainText('$record.status equals "COUNTING"');
  await expect(card).toContainText('this record is APPROVED');
  await expect(page.locator('[data-lane-start="lane-wh-count:cc-2026-002"]')).toHaveCount(0);
});
