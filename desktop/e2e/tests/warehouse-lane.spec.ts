import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The Receive Shipment lane on the warehouse demo (WP-0702).
 * Covers: REQ-NFR-008 (WP-0702 AC-1), REQ-LANE-014 (WP-0702 AC-1, AC-4),
 * REQ-LANE-025 (WP-0702 AC-1, AC-2, AC-3, AC-4).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts its own
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4173 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Untested locally — see CI.
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

async function startLane(page, poId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-start="lane-wh-receiving:${poId}"]`);
  await expect(page.locator('.lane-step-card h3')).toBeVisible();
}

async function completeStep(page) {
  await page.click('#laneCompleteBtn');
}

async function expectStep(page, name: string) {
  await expect(page.locator('.lane-step-card h3')).toHaveText(name);
}

test('AC-1: a first-time user completes PO-2026-001 on the lane screen alone', async ({ page }) => {
  await startLane(page, 'po-2026-001');
  await expect(page.locator('.view-head h2')).toContainText('PO-2026-001 · Receive Shipment · stage 1 of 5');

  // 1.1 Confirm shipment arrived — the received date must match the expected date.
  await page.fill('[data-lane-field="carrier"]', 'FastFreight');
  await page.fill('[data-lane-field="delivery_note_number"]', 'DN-7781');
  await page.fill('[data-lane-field="received_date"]', '2026-11-13');
  await completeStep(page);
  await expectStep(page, 'Match lines to the delivery note');

  // 1.2 Match lines to the delivery note — the Receiving Agent suggests each
  // line's quantity; confirm the prefilled values.
  await expect(page.locator('[data-lane-field="pol-001-1:delivery_note_qty"]')).toHaveValue('40');
  await expect(page.locator('[data-lane-field="pol-001-2:delivery_note_qty"]')).toHaveValue('30');
  await completeStep(page);
  await expectStep(page, 'Enter received quantities');

  // 2.1 Enter received quantities — the check then passes on its own.
  await page.fill('[data-lane-field="pol-001-1:qty_received"]', '40');
  await page.fill('[data-lane-field="pol-001-1:qty_rejected"]', '0');
  await page.fill('[data-lane-field="pol-001-2:qty_received"]', '30');
  await page.fill('[data-lane-field="pol-001-2:qty_rejected"]', '0');
  await completeStep(page);
  await expectStep(page, 'Stage the goods');

  // 3.1 Stage the goods — staging bin suggested, confirm it.
  await expect(page.locator('[data-lane-field="staging_bin"]')).toHaveValue('loc-rec-01');
  await completeStep(page);
  await expectStep(page, 'Post receipt moves');

  // 3.2 Post receipt moves.
  await completeStep(page);
  await expectStep(page, 'Confirm put-away');

  // 3.3 Confirm put-away — each line's preferred bin is suggested.
  await expect(page.locator('[data-lane-field="pol-001-1:putaway_bin"]')).toHaveValue('loc-a-01-03');
  await expect(page.locator('[data-lane-field="pol-001-2:putaway_bin"]')).toHaveValue('loc-a-01-03');
  await completeStep(page);
  await expectStep(page, 'Post put-away moves');

  // 3.4 Post put-away moves.
  await completeStep(page);
  await expectStep(page, 'Review summary');

  // 4.1 Review summary — no fields, just complete.
  await completeStep(page);
  await expectStep(page, 'Close the PO');

  // 4.2 Close the PO.
  await completeStep(page);
  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');
  await expect(page.locator('.lane-side')).toContainText('RECEIVED');

  // End state matches the worked example: stock 88 and 52, four moves.
  await expect(page.locator('.lane-po-lines')).toContainText('88');
  await expect(page.locator('.lane-po-lines')).toContainText('52');
  const writes = page.locator('.lane-writes');
  await expect(writes).toContainText('MV-2026-016 RECEIPT HG-1001 × 40');
  await expect(writes).toContainText('MV-2026-017 RECEIPT HG-1002 × 30');
  await expect(writes).toContainText('MV-2026-018 PUTAWAY HG-1001 × 40');
  await expect(writes).toContainText('MV-2026-019 PUTAWAY HG-1002 × 30');
});

test('AC-2: entering more than ordered blocks the step and writes nothing', async ({ page }) => {
  await startLane(page, 'po-2026-001');
  await page.fill('[data-lane-field="carrier"]', 'FastFreight');
  await page.fill('[data-lane-field="delivery_note_number"]', 'DN-7781');
  await page.fill('[data-lane-field="received_date"]', '2026-11-13');
  await completeStep(page);
  await completeStep(page); // 1.2, suggested quantities
  await expectStep(page, 'Enter received quantities');

  await page.fill('[data-lane-field="pol-001-1:qty_received"]', '41');
  await page.fill('[data-lane-field="pol-001-1:qty_rejected"]', '0');
  await page.fill('[data-lane-field="pol-001-2:qty_received"]', '30');
  await page.fill('[data-lane-field="pol-001-2:qty_rejected"]', '0');
  await completeStep(page);

  await expect(page.locator('#laneStepError')).toContainText('must not exceed ordered');
  await expectStep(page, 'Enter received quantities');
  await expect(page.locator('.lane-writes')).toHaveCount(0);
});

test('AC-3: the seeded PO-2026-005 run waits for the manager; approving puts away 18', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('#sideNav [data-nav="purchase_order"]');
  await page.click('[data-open-record="purchase_order:po-2026-005"]');
  await page.click('[data-lane-open-run]');
  await expect(page.locator('.view-head h2')).toContainText('PO-2026-005 · Receive Shipment · stage 3 of 5');
  await expectStep(page, 'Review discrepancy');

  await page.click('#laneApproveBtn');
  await expectStep(page, 'Stage the goods');

  // The receipt for the 18 good units already posted (MV-2026-015); the
  // staging suggestion is the bin it sits in, RCV-02.
  await expect(page.locator('[data-lane-field="staging_bin"]')).toHaveValue('loc-rec-02');
  await completeStep(page);
  await expectStep(page, 'Post receipt moves');
  await completeStep(page);
  await expectStep(page, 'Confirm put-away');
  await expect(page.locator('[data-lane-field="pol-005-1:putaway_bin"]')).toHaveValue('loc-b-01-02');
  await completeStep(page);
  await expectStep(page, 'Post put-away moves');
  await completeStep(page);
  await expectStep(page, 'Review summary');
  await completeStep(page);
  await expectStep(page, 'Close the PO');
  await completeStep(page);

  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');
  await expect(page.locator('.lane-side')).toContainText('PARTIALLY_RECEIVED');
  await expect(page.locator('.lane-po-lines')).toContainText('18');
  const writes = page.locator('.lane-writes');
  await expect(writes).toContainText('MV-2026-016 PUTAWAY HG-1008 × 18');
  await expect(writes).toContainText('MV-2026-015 RECEIPT HG-1008 × 18 → RCV-02 (already posted)');
  await expect(writes).not.toContainText('MV-2026-016 RECEIPT');
});

test('AC-3: rejecting the discrepancy holds the order and posts nothing', async ({ page }) => {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click('#sideNav [data-nav="purchase_order"]');
  await page.click('[data-open-record="purchase_order:po-2026-005"]');
  await page.click('[data-lane-open-run]');
  await expectStep(page, 'Review discrepancy');

  await page.click('#laneRejectBtn');
  await expectStep(page, 'Record hold');
  await page.fill('[data-lane-field="hold_reason"]', 'Supplier to collect the damaged units');
  await completeStep(page);

  await expect(page.locator('.lane-outcome')).toContainText('Run completed — held');
  await expect(page.locator('.lane-writes')).toHaveCount(0);
});

test('AC-4: leaving and reopening returns to the current step; Reset restores the seed', async ({ page }) => {
  await startLane(page, 'po-2026-001');
  await page.fill('[data-lane-field="carrier"]', 'FastFreight');
  await page.fill('[data-lane-field="delivery_note_number"]', 'DN-7781');
  await page.fill('[data-lane-field="received_date"]', '2026-11-13');
  await completeStep(page);
  await expectStep(page, 'Match lines to the delivery note');

  // Leave the runner (back to the purchase order) and open it again: the same
  // run reopens at the same step.
  await page.click('[data-lane-back-record]');
  await expect(page.locator('[data-lane-open-run]')).toBeVisible();
  await page.click('[data-lane-open-run]');
  await expectStep(page, 'Match lines to the delivery note');

  // Reset clears the run and keeps the definition: PO-2026-001 is due again,
  // and the seeded PO-2026-005 run waits for the manager again.
  await page.click('#sideResetDemo');
  await expect(page.locator('[data-lane-start="lane-wh-receiving:po-2026-001"]')).toBeVisible();
  await page.click('#sideNav [data-nav="purchase_order"]');
  await page.click('[data-open-record="purchase_order:po-2026-005"]');
  await expect(page.locator('[data-lane-open-run]')).toBeVisible();
});
